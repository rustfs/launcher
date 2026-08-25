use crate::config::RustFsConfig;
use crate::error::{Error, Result};
use crate::state::{
    acquire, add_app_log, add_rustfs_log, is_rustfs_process_running, set_rustfs_process, APP_HANDLE,
};
use std::io::{BufRead, BufReader};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};
use tauri::Manager;

const EARLY_EXIT_WAIT: Duration = Duration::from_millis(400);

pub(crate) fn inferred_binary_name_for(os: &str, arch: &str) -> &'static str {
    match (os, arch) {
        ("macos", "aarch64") => "rustfs-macos-aarch64",
        ("macos", "x86_64") => "rustfs-macos-x86_64",
        ("windows", "x86_64") | ("windows", "aarch64") => "rustfs-windows-x86_64.exe",
        ("linux", "x86_64") => "rustfs-linux-x86_64",
        ("linux", "aarch64") => "rustfs-linux-aarch64",
        _ => {
            if os == "windows" {
                "rustfs-windows-x86_64.exe"
            } else {
                "rustfs"
            }
        }
    }
}

fn inferred_binary_name() -> &'static str {
    inferred_binary_name_for(std::env::consts::OS, std::env::consts::ARCH)
}

pub(crate) fn format_bind_address(host: &str, port: u16) -> String {
    if host.contains(':') && !host.starts_with('[') {
        format!("[{host}]:{port}")
    } else {
        format!("{host}:{port}")
    }
}

pub(crate) fn validate_data_path(path: &str) -> Result<&str> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err(Error::DataPathRequired);
    }
    if trimmed.starts_with('-') {
        return Err(Error::InvalidDataPath(
            "data path must not start with '-'".to_string(),
        ));
    }
    if trimmed.contains('\0') {
        return Err(Error::InvalidDataPath(
            "data path contains invalid characters".to_string(),
        ));
    }
    Ok(trimmed)
}

pub(crate) fn resolve_data_path(path: &str) -> Result<PathBuf> {
    let trimmed = validate_data_path(path)?;
    let candidate = PathBuf::from(trimmed);
    if !candidate.exists() {
        return Err(Error::DataPathNotExist(trimmed.to_string()));
    }
    if candidate.is_file() {
        return candidate
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| Error::DataPathNotDirectory(trimmed.to_string()));
    }
    if !candidate.is_dir() {
        return Err(Error::DataPathNotDirectory(trimmed.to_string()));
    }
    Ok(candidate)
}

pub(crate) fn logs_dir_for_data_path(data_path: &Path) -> PathBuf {
    match data_path.parent() {
        Some(parent) if is_filesystem_root(parent) || parent.as_os_str().is_empty() => {
            data_path.join("logs")
        }
        Some(parent) => parent.join("logs"),
        None => data_path.join("logs"),
    }
}

fn is_filesystem_root(path: &Path) -> bool {
    let mut components = path.components();
    match components.next() {
        Some(std::path::Component::RootDir) => components.next().is_none(),
        Some(std::path::Component::Prefix(_)) => match components.next() {
            None | Some(std::path::Component::RootDir) => components.next().is_none(),
            _ => false,
        },
        _ => false,
    }
}

pub(crate) fn binary_candidates(
    exe_dir: &Path,
    cwd: Option<&Path>,
    env_dir: Option<&Path>,
    resource_dir: Option<&Path>,
    binary_name: &str,
) -> Vec<PathBuf> {
    let mut candidates = Vec::<PathBuf>::new();
    let mut push_candidate = |path: PathBuf| {
        if !candidates.iter().any(|existing| existing == &path) {
            candidates.push(path);
        }
    };

    push_candidate(exe_dir.join("binaries").join(binary_name));
    push_candidate(exe_dir.join("resources").join("binaries").join(binary_name));
    push_candidate(exe_dir.join("../Resources/binaries").join(binary_name));
    push_candidate(exe_dir.join(binary_name));

    if let Some(resource_dir) = resource_dir {
        push_candidate(resource_dir.join("binaries").join(binary_name));
        push_candidate(resource_dir.join(binary_name));
    }

    if let Some(env_dir) = env_dir {
        push_candidate(env_dir.join(binary_name));
    }

    if let Some(cwd) = cwd {
        push_candidate(cwd.join("src-tauri/binaries").join(binary_name));
        push_candidate(cwd.join("binaries").join(binary_name));
    }

    push_candidate(PathBuf::from("src-tauri/binaries").join(binary_name));
    candidates
}

fn resource_dir_from_app() -> Option<PathBuf> {
    let handle = acquire(&APP_HANDLE);
    handle
        .as_ref()
        .and_then(|app| app.path().resource_dir().ok())
}

fn get_binary_path() -> Result<PathBuf> {
    let current_exe = std::env::current_exe().map_err(Error::Io)?;
    let exe_dir = current_exe.parent().ok_or_else(|| {
        Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Parent directory of executable not found",
        ))
    })?;
    let binary_name = inferred_binary_name();
    let env_dir = std::env::var_os("RUSTFS_BINARY_DIR").map(PathBuf::from);
    let cwd = std::env::current_dir().ok();
    let resource_dir = resource_dir_from_app();

    let candidates = binary_candidates(
        exe_dir,
        cwd.as_deref(),
        env_dir.as_deref(),
        resource_dir.as_deref(),
        binary_name,
    );

    for candidate in &candidates {
        add_app_log(format!(
            "Checking RustFS binary candidate: {}",
            candidate.display()
        ));
        if candidate.exists() {
            add_app_log(format!(
                "Using RustFS binary for {}-{} at {}",
                std::env::consts::OS,
                std::env::consts::ARCH,
                candidate.display()
            ));
            return Ok(candidate.clone());
        }
    }

    Err(Error::BinaryNotFound(
        candidates
            .first()
            .map(|path| path.to_string_lossy().to_string())
            .unwrap_or_else(|| "<unknown>".to_string()),
    ))
}

fn apply_no_window(cmd: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    let _ = cmd;
}

#[cfg(unix)]
fn ensure_executable(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let metadata = std::fs::metadata(path)
        .map_err(|err| Error::Metadata(path.to_string_lossy().to_string(), err))?;
    let mut permissions = metadata.permissions();
    add_app_log(format!(
        "File permissions for {}: {:o}",
        path.display(),
        permissions.mode()
    ));

    if permissions.mode() & 0o111 == 0 {
        add_app_log(format!(
            "Binary is not executable, attempting to set +x: {}",
            path.display()
        ));
        permissions.set_mode(permissions.mode() | 0o755);
        if let Err(err) = std::fs::set_permissions(path, permissions) {
            add_app_log(format!("WARNING: Failed to make binary executable: {err}"));
        }
    }
    Ok(())
}

#[cfg(not(unix))]
fn ensure_executable(path: &Path) -> Result<()> {
    let metadata = std::fs::metadata(path)
        .map_err(|err| Error::Metadata(path.to_string_lossy().to_string(), err))?;

    add_app_log(format!("File size: {} bytes", metadata.len()));

    if !metadata.is_file() {
        return Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Path is not a regular file",
        )));
    }

    match path.extension().and_then(|ext| ext.to_str()) {
        Some(ext) if ext.eq_ignore_ascii_case("exe") || ext.eq_ignore_ascii_case("cmd") => {}
        Some(ext) => add_app_log(format!(
            "WARNING: File does not have a Windows executable extension: {ext}"
        )),
        None => add_app_log("WARNING: File has no extension".to_string()),
    }

    Ok(())
}

pub(crate) fn is_port_available(host: &str, port: u16) -> bool {
    TcpListener::bind((host, port)).is_ok()
}

pub fn diagnose_binary() -> Result<String> {
    add_app_log("Starting RustFS binary diagnosis...".to_string());
    let binary_path = get_binary_path()?;
    ensure_executable(&binary_path)?;

    add_app_log(format!(
        "Testing binary with --help: {}",
        binary_path.display()
    ));
    let mut cmd = Command::new(&binary_path);
    cmd.arg("--help");
    apply_no_window(&mut cmd);
    let output = cmd.output().map_err(Error::BinaryExecution)?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    add_app_log(format!(
        "Binary --help stdout (first 200 chars): {}",
        stdout.chars().take(200).collect::<String>()
    ));

    if output.status.success() {
        Ok("RustFS binary appears to be working".to_string())
    } else {
        Err(Error::BinaryFailed(output.status.to_string()))
    }
}

pub(crate) struct LaunchPlan {
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
}

pub(crate) fn build_launch_plan(config: &RustFsConfig, logs_dir: &Path) -> LaunchPlan {
    let mut args = Vec::new();
    let mut env = vec![(
        "RUSTFS_OBS_LOG_DIRECTORY".to_string(),
        logs_dir.to_string_lossy().into_owned(),
    )];

    let address = format_bind_address(config.bind_host(), config.api_port());
    args.push("--address".to_string());
    args.push(address);

    if config.console_enable {
        args.push("--console-enable".to_string());
        args.push("--console-address".to_string());
        args.push(format_bind_address(
            config.bind_host(),
            config.console_port(),
        ));
    }

    if let Some(access_key) = config
        .access_key
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        env.push(("RUSTFS_ACCESS_KEY".to_string(), access_key.to_string()));
    }
    if let Some(secret_key) = config
        .secret_key
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        env.push(("RUSTFS_SECRET_KEY".to_string(), secret_key.to_string()));
    }

    args.push("--".to_string());
    args.push(config.data_path.clone());

    LaunchPlan { args, env }
}

pub fn launch(config: RustFsConfig) -> Result<String> {
    add_app_log("Launch command received".to_string());
    add_app_log(format!(
        "Config: data_path={}, port={:?}, host={:?}",
        config.data_path, config.port, config.host
    ));

    let data_path = resolve_data_path(&config.data_path)?;
    let mut config = config;
    config.data_path = data_path.to_string_lossy().into_owned();

    if is_rustfs_process_running() {
        return Err(Error::AlreadyRunning);
    }

    let api_port = config.api_port();
    let console_port = config.console_port();

    if config.console_enable && api_port == console_port {
        return Err(Error::PortConflict);
    }

    if !is_port_available(config.bind_host(), api_port) {
        return Err(Error::PortInUse(api_port));
    }
    if config.console_enable && !is_port_available(config.bind_host(), console_port) {
        return Err(Error::PortInUse(console_port));
    }

    let binary_path = match &config.binary_path {
        Some(path) => PathBuf::from(path),
        None => get_binary_path()?,
    };
    ensure_executable(&binary_path)?;

    let logs_dir = logs_dir_for_data_path(&data_path);
    add_app_log(format!(
        "Creating logs directory at: {}",
        logs_dir.display()
    ));
    std::fs::create_dir_all(&logs_dir).map_err(Error::Io)?;

    let plan = build_launch_plan(&config, &logs_dir);
    let mut cmd = Command::new(&binary_path);
    for (key, value) in &plan.env {
        cmd.env(key, value);
    }
    cmd.args(&plan.args);
    apply_no_window(&mut cmd);

    add_app_log(format!(
        "Spawning RustFS: binary={}, args={}, env_keys={}, access_key_set={}, secret_key_set={}",
        binary_path.display(),
        plan.args.join(" "),
        plan.env
            .iter()
            .map(|(key, _)| key.as_str())
            .collect::<Vec<_>>()
            .join(","),
        config.access_key.is_some(),
        config.secret_key.is_some()
    ));

    let mut child = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(Error::BinaryExecution)?;

    let pid = child.id();
    add_app_log(format!("RustFS launched successfully with PID: {pid}"));
    add_rustfs_log("RustFS process started, capturing output...".to_string());

    if let Some(stdout) = child.stdout.take() {
        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines().map_while(|line| line.ok()) {
                if line.is_empty() {
                    continue;
                }
                add_rustfs_log(format!("[STDOUT] {line}"));
            }
        });
    }

    if let Some(stderr) = child.stderr.take() {
        thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines().map_while(|line| line.ok()) {
                if line.is_empty() {
                    continue;
                }
                add_rustfs_log(format!("[STDERR] {line}"));
            }
        });
    }

    let deadline = Instant::now() + EARLY_EXIT_WAIT;
    while Instant::now() < deadline {
        match child.try_wait() {
            Ok(Some(status)) => {
                return Err(Error::BinaryFailed(status.to_string()));
            }
            Ok(None) => thread::sleep(Duration::from_millis(50)),
            Err(err) => return Err(Error::BinaryExecution(err)),
        }
    }

    set_rustfs_process(child);
    Ok(format!("RustFS launched with PID: {pid}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RustFsConfig;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn infers_platform_binary_names() {
        assert_eq!(
            inferred_binary_name_for("macos", "aarch64"),
            "rustfs-macos-aarch64"
        );
        assert_eq!(
            inferred_binary_name_for("macos", "x86_64"),
            "rustfs-macos-x86_64"
        );
        assert_eq!(
            inferred_binary_name_for("windows", "x86_64"),
            "rustfs-windows-x86_64.exe"
        );
        assert_eq!(
            inferred_binary_name_for("windows", "aarch64"),
            "rustfs-windows-x86_64.exe"
        );
    }

    #[test]
    fn wraps_ipv6_bind_addresses() {
        assert_eq!(format_bind_address("127.0.0.1", 9000), "127.0.0.1:9000");
        assert_eq!(format_bind_address("::1", 9000), "[::1]:9000");
        assert_eq!(format_bind_address("[::1]", 9001), "[::1]:9001");
    }

    #[test]
    fn rejects_flag_like_data_paths() {
        assert!(matches!(
            validate_data_path(""),
            Err(Error::DataPathRequired)
        ));
        assert!(matches!(
            validate_data_path("--help"),
            Err(Error::InvalidDataPath(_))
        ));
        assert!(validate_data_path("/tmp/rustfs-data").is_ok());
    }

    #[test]
    fn places_logs_inside_volume_when_parent_is_root() {
        let unix_root_child = PathBuf::from("/data");
        assert_eq!(
            logs_dir_for_data_path(&unix_root_child),
            PathBuf::from("/data/logs")
        );

        let nested = PathBuf::from("/home/user/rustfs/data");
        assert_eq!(
            logs_dir_for_data_path(&nested),
            PathBuf::from("/home/user/rustfs/logs")
        );
    }

    #[test]
    fn includes_windows_and_macos_resource_candidates() {
        let exe_dir = PathBuf::from("/Applications/RustFS Launcher.app/Contents/MacOS");
        let resource_dir = PathBuf::from("/Applications/RustFS Launcher.app/Contents/Resources");
        let candidates = binary_candidates(
            &exe_dir,
            None,
            None,
            Some(&resource_dir),
            "rustfs-macos-aarch64",
        );

        assert!(candidates
            .iter()
            .any(|path| path.ends_with("Contents/Resources/binaries/rustfs-macos-aarch64")));

        let win_exe = PathBuf::from(r"C:\Program Files\RustFS Launcher");
        let win_resource = win_exe.join("resources");
        let win_candidates = binary_candidates(
            &win_exe,
            None,
            None,
            Some(&win_resource),
            "rustfs-windows-x86_64.exe",
        );
        assert!(win_candidates.iter().any(|path| path
            .to_string_lossy()
            .contains(r"resources\binaries\rustfs-windows-x86_64.exe")
            || path
                .to_string_lossy()
                .contains("resources/binaries/rustfs-windows-x86_64.exe")));
    }

    #[test]
    fn credentials_go_to_env_not_argv() {
        let config = RustFsConfig {
            data_path: "/tmp/data".into(),
            access_key: Some("ak-secret".into()),
            secret_key: Some("sk-secret".into()),
            console_enable: true,
            ..RustFsConfig::default()
        };
        let plan = build_launch_plan(&config, Path::new("/tmp/logs"));

        assert!(plan.args.iter().any(|arg| arg == "--"));
        assert!(plan.args.contains(&"/tmp/data".to_string()));
        assert!(!plan.args.iter().any(|arg| arg.contains("ak-secret")));
        assert!(!plan.args.iter().any(|arg| arg.contains("sk-secret")));
        assert!(plan
            .env
            .iter()
            .any(|(key, value)| key == "RUSTFS_ACCESS_KEY" && value == "ak-secret"));
        assert!(plan
            .env
            .iter()
            .any(|(key, value)| key == "RUSTFS_SECRET_KEY" && value == "sk-secret"));
        assert!(plan.args.contains(&"--console-enable".to_string()));
    }

    #[test]
    fn launch_rejects_empty_and_conflicting_ports() {
        let empty = RustFsConfig {
            data_path: String::new(),
            ..RustFsConfig::default()
        };
        assert!(matches!(launch(empty), Err(Error::DataPathRequired)));

        let dir = tempfile::tempdir().unwrap();
        let conflict = RustFsConfig {
            data_path: dir.path().to_string_lossy().into_owned(),
            port: Some(9000),
            console_port: Some(9000),
            console_enable: true,
            ..RustFsConfig::default()
        };
        assert!(matches!(launch(conflict), Err(Error::PortConflict)));
    }

    fn create_sleep_stub(dir: &Path) -> PathBuf {
        #[cfg(windows)]
        {
            let path = dir.join("stub.cmd");
            fs::write(&path, "@echo off\r\nping -n 20 127.0.0.1 >nul\r\n").unwrap();
            path
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let path = dir.join("stub.sh");
            fs::write(&path, "#!/bin/sh\necho stub started\nexec sleep 20\n").unwrap();
            let mut permissions = fs::metadata(&path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&path, permissions).unwrap();
            path
        }
    }

    #[test]
    fn launch_and_terminate_stub_binary() {
        crate::state::terminate_rustfs_process();

        let dir = tempfile::tempdir().unwrap();
        let data_dir = dir.path().join("data");
        fs::create_dir(&data_dir).unwrap();
        let stub = create_sleep_stub(dir.path());
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);

        let config = RustFsConfig {
            binary_path: Some(stub.to_string_lossy().into_owned()),
            data_path: data_dir.to_string_lossy().into_owned(),
            port: Some(port),
            console_enable: false,
            host: Some("127.0.0.1".into()),
            ..RustFsConfig::default()
        };

        launch(config).expect("stub process should launch");
        assert!(is_rustfs_process_running());
        crate::state::terminate_rustfs_process();
        assert!(!is_rustfs_process_running());
    }
}
