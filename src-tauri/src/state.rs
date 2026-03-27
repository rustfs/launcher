use lazy_static::lazy_static;
use regex::Regex;
use std::collections::VecDeque;
use std::process::Child;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager};

lazy_static! {
    pub static ref APP_LOGS: Arc<Mutex<VecDeque<String>>> = Arc::new(Mutex::new(VecDeque::new()));
    pub static ref RUSTFS_LOGS: Arc<Mutex<VecDeque<String>>> =
        Arc::new(Mutex::new(VecDeque::new()));
    pub static ref APP_HANDLE: Arc<Mutex<Option<AppHandle>>> = Arc::new(Mutex::new(None));
    pub static ref RUSTFS_PROCESS: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));
}

lazy_static! {
    static ref ANSI_REGEX: Regex = Regex::new(r"\x1B\[[0-9;]*m").unwrap();
    static ref SECRET_PATTERNS: Vec<Regex> = vec![
        Regex::new(r#"(--access-key\s+)(\S+)"#).unwrap(),
        Regex::new(r#"(--secret-key\s+)(\S+)"#).unwrap(),
        Regex::new(r#"(access_key\s*=\s*)([^,\s]+)"#).unwrap(),
        Regex::new(r#"(secret_key\s*=\s*)([^,\s]+)"#).unwrap(),
        Regex::new(r#"(access_key:\s*Some\(")([^"]+)("\))"#).unwrap(),
        Regex::new(r#"(secret_key:\s*Some\(")([^"]+)("\))"#).unwrap(),
        Regex::new(r#"("access_key"\s*:\s*")([^"]+)(")"#).unwrap(),
        Regex::new(r#"("secret_key"\s*:\s*")([^"]+)(")"#).unwrap(),
    ];
}

fn clean_ansi_codes(s: &str) -> String {
    ANSI_REGEX.replace_all(s, "").to_string()
}

fn redact_secrets(s: &str) -> String {
    SECRET_PATTERNS.iter().fold(s.to_string(), |acc, regex| {
        regex.replace_all(&acc, "${1}[REDACTED]${3}").to_string()
    })
}

fn clean_log_message(s: &str) -> String {
    redact_secrets(&clean_ansi_codes(s))
}

fn buffer_log(logs: &Arc<Mutex<VecDeque<String>>>, message: String, capacity: usize) -> String {
    let cleaned_message = clean_log_message(&message);
    let log_entry = format!(
        "[{}] {}",
        chrono::Local::now().format("%H:%M:%S"),
        cleaned_message
    );

    {
        let mut logs = logs.lock().unwrap();
        logs.push_back(log_entry.clone());
        if logs.len() > capacity {
            logs.pop_front();
        }
    }

    log_entry
}

fn emit_log(event_name: &str, log_entry: String) {
    if let Some(handle) = APP_HANDLE.lock().unwrap().as_ref() {
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
    *APP_HANDLE.lock().unwrap() = Some(handle);
}

pub fn get_app_logs() -> Vec<String> {
    APP_LOGS.lock().unwrap().iter().cloned().collect()
}

pub fn get_rustfs_logs() -> Vec<String> {
    RUSTFS_LOGS.lock().unwrap().iter().cloned().collect()
}

pub fn is_rustfs_process_running() -> bool {
    let mut process_guard = RUSTFS_PROCESS.lock().unwrap();

    match process_guard.as_mut() {
        Some(child) => match child.try_wait() {
            Ok(None) => true,
            Ok(Some(_)) => {
                *process_guard = None;
                false
            }
            Err(e) => {
                add_app_log(format!("Failed to inspect RustFS process status: {}", e));
                *process_guard = None;
                false
            }
        },
        None => false,
    }
}

pub fn set_rustfs_process(process: Child) {
    let pid = process.id();
    *RUSTFS_PROCESS.lock().unwrap() = Some(process);
    add_app_log(format!("RustFS process registered with PID: {}", pid));

    // Spawn a monitor thread
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));

            let mut should_break = false;
            {
                let mut process_guard = RUSTFS_PROCESS.lock().unwrap();
                if let Some(child) = process_guard.as_mut() {
                    if child.id() == pid {
                        match child.try_wait() {
                            Ok(Some(status)) => {
                                add_app_log(format!(
                                    "RustFS process exited with status: {}",
                                    status
                                ));

                                // Emit exit event
                                if let Some(handle) = APP_HANDLE.lock().unwrap().as_ref() {
                                    let _ = handle.emit("rustfs-exit", format!("{}", status));
                                }

                                // Clear the process from state since it exited
                                *process_guard = None;
                                should_break = true;
                            }
                            Ok(None) => {
                                // Still running
                            }
                            Err(e) => {
                                add_app_log(format!("Error monitoring process: {}", e));
                                should_break = true;
                            }
                        }
                    } else {
                        // PID mismatch, likely a new process was started
                        should_break = true;
                    }
                } else {
                    // No process in state
                    should_break = true;
                }
            }

            if should_break {
                break;
            }
        }
    });
}

pub fn terminate_rustfs_process() {
    let mut process_guard = RUSTFS_PROCESS.lock().unwrap();
    if let Some(mut process) = process_guard.take() {
        let pid = process.id();
        add_app_log(format!("Terminating RustFS process with PID: {}", pid));

        match process.kill() {
            Ok(_) => {
                add_app_log("RustFS process terminated successfully".to_string());
                // Wait for the process to actually exit
                let _ = process.wait();
            }
            Err(e) => {
                add_app_log(format!("Failed to terminate RustFS process: {}", e));
            }
        }
    } else {
        add_app_log("No RustFS process to terminate".to_string());
    }
}
