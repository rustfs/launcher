use regex::Regex;
use std::collections::VecDeque;
use std::process::Child;
use std::sync::atomic::Ordering;
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::Duration;
#[cfg(unix)]
use std::time::Instant;
use tauri::{AppHandle, Emitter, Manager};

pub static APP_LOGS: Mutex<VecDeque<String>> = Mutex::new(VecDeque::new());
pub static RUSTFS_LOGS: Mutex<VecDeque<String>> = Mutex::new(VecDeque::new());
pub static APP_HANDLE: Mutex<Option<AppHandle>> = Mutex::new(None);
pub static RUSTFS_PROCESS: Mutex<Option<Child>> = Mutex::new(None);

static MONITOR_GENERATION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

fn ansi_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"\x1B\[[0-9;]*m").expect("valid ANSI regex"))
}

fn secret_patterns() -> &'static [Regex] {
    static PATTERNS: OnceLock<Vec<Regex>> = OnceLock::new();
    PATTERNS
        .get_or_init(|| {
            vec![
                Regex::new(r"(--access-key\s+)(\S+)").unwrap(),
                Regex::new(r"(--secret-key\s+)(\S+)").unwrap(),
                Regex::new(r"(access_key\s*=\s*)([^,\s]+)").unwrap(),
                Regex::new(r"(secret_key\s*=\s*)([^,\s]+)").unwrap(),
                Regex::new(r#"(access_key:\s*Some\(")([^"]+)("\))"#).unwrap(),
                Regex::new(r#"(secret_key:\s*Some\(")([^"]+)("\))"#).unwrap(),
                Regex::new(r#"("access_key"\s*:\s*")([^"]+)(")"#).unwrap(),
                Regex::new(r#"("secret_key"\s*:\s*")([^"]+)(")"#).unwrap(),
                Regex::new(r"(RUSTFS_(?:ACCESS|SECRET)_KEY=)(\S+)").unwrap(),
            ]
        })
        .as_slice()
}

pub fn acquire<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn clean_ansi_codes(value: &str) -> String {
    ansi_regex().replace_all(value, "").into_owned()
}

pub(crate) fn redact_secrets(value: &str) -> String {
    secret_patterns()
        .iter()
        .fold(value.to_string(), |acc, regex| {
            regex.replace_all(&acc, "${1}[REDACTED]${3}").into_owned()
        })
}

fn clean_log_message(value: &str) -> String {
    redact_secrets(&clean_ansi_codes(value))
}

fn buffer_log(logs: &Mutex<VecDeque<String>>, message: String, capacity: usize) -> String {
    let cleaned_message = clean_log_message(&message);
    let log_entry = format!(
        "[{}] {}",
        chrono::Local::now().format("%H:%M:%S"),
        cleaned_message
    );

    {
        let mut logs = acquire(logs);
        logs.push_back(log_entry.clone());
        if logs.len() > capacity {
            logs.pop_front();
        }
    }

    log_entry
}

fn emit_log(event_name: &str, log_entry: String) {
    if let Some(handle) = acquire(&APP_HANDLE).as_ref() {
        if let Some(window) = handle.get_webview_window("main") {
            let _ = window.emit(event_name, log_entry);
        }
    }
}

const APP_LOG_EVENT: &str = "app-log";
const RUSTFS_LOG_EVENT: &str = "rustfs-log";
const APP_LOG_CAPACITY: usize = 100;
const RUSTFS_LOG_CAPACITY: usize = 1000;

pub fn add_app_log(message: String) {
    let entry = buffer_log(&APP_LOGS, message, APP_LOG_CAPACITY);
    emit_log(APP_LOG_EVENT, entry);
}

pub fn add_rustfs_log(message: String) {
    let entry = buffer_log(&RUSTFS_LOGS, message, RUSTFS_LOG_CAPACITY);
    emit_log(RUSTFS_LOG_EVENT, entry);
}

pub fn set_app_handle(handle: AppHandle) {
    *acquire(&APP_HANDLE) = Some(handle);
}

pub fn get_app_logs() -> Vec<String> {
    acquire(&APP_LOGS).iter().cloned().collect()
}

pub fn get_rustfs_logs() -> Vec<String> {
    acquire(&RUSTFS_LOGS).iter().cloned().collect()
}

pub fn is_rustfs_process_running() -> bool {
    let mut process_guard = acquire(&RUSTFS_PROCESS);

    match process_guard.as_mut() {
        Some(child) => match child.try_wait() {
            Ok(None) => true,
            Ok(Some(_)) => {
                *process_guard = None;
                false
            }
            Err(err) => {
                add_app_log(format!("Failed to inspect RustFS process status: {err}"));
                *process_guard = None;
                false
            }
        },
        None => false,
    }
}

fn kill_child(process: &mut Child) {
    let pid = process.id();
    add_app_log(format!("Terminating RustFS process with PID: {pid}"));

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        match std::process::Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/T", "/F"])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
        {
            Ok(output) if output.status.success() => {
                add_app_log("RustFS process tree terminated with taskkill".to_string());
            }
            Ok(output) => {
                add_app_log(format!(
                    "taskkill returned {}, falling back to kill()",
                    output.status
                ));
                let _ = process.kill();
            }
            Err(err) => {
                add_app_log(format!("taskkill failed ({err}), falling back to kill()"));
                let _ = process.kill();
            }
        }
        let _ = process.wait();
    }

    #[cfg(unix)]
    {
        let terminate_result = unsafe { libc::kill(pid as i32, libc::SIGTERM) };
        if terminate_result != 0 {
            add_app_log(format!(
                "SIGTERM failed (errno {}), sending SIGKILL",
                std::io::Error::last_os_error()
            ));
            let _ = process.kill();
            let _ = process.wait();
            return;
        }

        let deadline = Instant::now() + Duration::from_secs(3);
        while Instant::now() < deadline {
            match process.try_wait() {
                Ok(Some(status)) => {
                    add_app_log(format!(
                        "RustFS process exited after SIGTERM with status: {status}"
                    ));
                    return;
                }
                Ok(None) => std::thread::sleep(Duration::from_millis(100)),
                Err(err) => {
                    add_app_log(format!("Error waiting for RustFS process: {err}"));
                    break;
                }
            }
        }

        add_app_log("RustFS did not exit after SIGTERM, sending SIGKILL".to_string());
        let _ = process.kill();
        let _ = process.wait();
    }

    #[cfg(not(any(unix, windows)))]
    {
        match process.kill() {
            Ok(_) => {
                add_app_log("RustFS process terminated successfully".to_string());
                let _ = process.wait();
            }
            Err(err) => add_app_log(format!("Failed to terminate RustFS process: {err}")),
        }
    }
}

pub fn set_rustfs_process(process: Child) {
    let pid = process.id();
    let generation = MONITOR_GENERATION.fetch_add(1, Ordering::SeqCst) + 1;

    {
        let mut process_guard = acquire(&RUSTFS_PROCESS);
        if let Some(mut previous) = process_guard.take() {
            add_app_log("Replacing an existing RustFS process".to_string());
            kill_child(&mut previous);
        }
        *process_guard = Some(process);
    }

    add_app_log(format!("RustFS process registered with PID: {pid}"));

    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(1));

        if MONITOR_GENERATION.load(Ordering::SeqCst) != generation {
            break;
        }

        let mut should_break = false;
        {
            let mut process_guard = acquire(&RUSTFS_PROCESS);
            if let Some(child) = process_guard.as_mut() {
                if child.id() == pid {
                    match child.try_wait() {
                        Ok(Some(status)) => {
                            add_app_log(format!("RustFS process exited with status: {status}"));
                            if let Some(handle) = acquire(&APP_HANDLE).as_ref() {
                                let _ = handle.emit("rustfs-exit", format!("{status}"));
                            }
                            *process_guard = None;
                            should_break = true;
                        }
                        Ok(None) => {}
                        Err(err) => {
                            add_app_log(format!("Error monitoring process: {err}"));
                            should_break = true;
                        }
                    }
                } else {
                    should_break = true;
                }
            } else {
                should_break = true;
            }
        }

        if should_break {
            break;
        }
    });
}

pub fn terminate_rustfs_process() {
    MONITOR_GENERATION.fetch_add(1, Ordering::SeqCst);
    let mut process_guard = acquire(&RUSTFS_PROCESS);
    if let Some(mut process) = process_guard.take() {
        kill_child(&mut process);
    } else {
        add_app_log("No RustFS process to terminate".to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::redact_secrets;

    #[test]
    fn redacts_cli_and_env_secrets() {
        assert_eq!(
            redact_secrets("--secret-key supersecret"),
            "--secret-key [REDACTED]"
        );
        assert_eq!(
            redact_secrets("--access-key AKIAEXAMPLE"),
            "--access-key [REDACTED]"
        );

        let env_redacted = redact_secrets("RUSTFS_SECRET_KEY=supersecret extra");
        assert!(env_redacted.contains("[REDACTED]"));
        assert!(!env_redacted.contains("supersecret"));
    }
}
