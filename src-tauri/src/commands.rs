use crate::config::RustFsConfig;
use crate::error::{Error, Result};
use crate::process;
use crate::state;
use serde::Serialize;
use std::io::{Error as IoError, ErrorKind};
use tauri::async_runtime;

#[derive(Debug, Serialize)]
pub struct CommandResponse {
    pub success: bool,
    pub message: String,
}

#[tauri::command]
pub async fn launch_rustfs(config: RustFsConfig) -> Result<CommandResponse> {
    let handle = async_runtime::spawn_blocking(move || process::launch(config));
    let message = handle.await.map_err(|err| {
        let io_error = IoError::other(err.to_string());
        Error::Io(io_error)
    })??;

    Ok(CommandResponse {
        success: true,
        message,
    })
}

#[tauri::command]
pub async fn stop_rustfs() -> Result<CommandResponse> {
    state::terminate_rustfs_process();
    Ok(CommandResponse {
        success: true,
        message: "RustFS process terminated".to_string(),
    })
}

#[tauri::command]
pub async fn validate_config(config: RustFsConfig) -> Result<bool> {
    if config.data_path.is_empty() {
        return Err(Error::DataPathRequired);
    }
    if !std::path::Path::new(&config.data_path).exists() {
        return Err(Error::DataPathNotExist(config.data_path));
    }
    Ok(true)
}

#[tauri::command]
pub async fn diagnose_rustfs_binary() -> Result<CommandResponse> {
    let handle = async_runtime::spawn_blocking(process::diagnose_binary);
    let message = handle.await.map_err(|err| {
        let io_error = IoError::other(err.to_string());
        Error::Io(io_error)
    })??;

    Ok(CommandResponse {
        success: true,
        message,
    })
}

#[tauri::command]
pub async fn get_app_logs() -> Result<Vec<String>> {
    Ok(state::get_app_logs())
}

#[tauri::command]
pub async fn get_rustfs_logs() -> Result<Vec<String>> {
    Ok(state::get_rustfs_logs())
}

#[tauri::command]
pub async fn check_tcp_connection(host: String, port: u16) -> Result<bool> {
    let address = format!("{}:{}", host, port);
    let socket_addr = address
        .parse()
        .map_err(|_| Error::Io(IoError::new(ErrorKind::InvalidInput, "Invalid address")))?;

    // Use spawn_blocking for network IO to avoid blocking async runtime
    let result = async_runtime::spawn_blocking(move || {
        use std::net::TcpStream;
        use std::time::Duration;

        // Use connect_timeout to avoid long hangs
        TcpStream::connect_timeout(&socket_addr, Duration::from_millis(1000)).is_ok()
    })
    .await
    .unwrap_or(false);

    Ok(result)
}

#[tauri::command]
pub async fn is_rustfs_process_running() -> Result<bool> {
    Ok(state::is_rustfs_process_running())
}
