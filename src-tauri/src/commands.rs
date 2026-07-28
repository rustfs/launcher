use crate::config::RustFsConfig;
use crate::error::{Error, Result};
use crate::process;
use crate::state::{self, add_app_log};
use serde::Serialize;
use std::io::{Error as IoError, ErrorKind};
use tauri::async_runtime;
use tauri::{AppHandle, Emitter};
use tauri_plugin_updater::UpdaterExt;

#[derive(Debug, Serialize)]
pub struct CommandResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct AppVersionInfo {
    pub launcher_version: String,
    pub rustfs_version: String,
}

#[derive(Debug, Serialize)]
pub struct UpdateInfo {
    pub available: bool,
    pub current_version: String,
    pub version: Option<String>,
    pub notes: Option<String>,
    pub date: Option<String>,
    pub rustfs_running: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", rename_all = "camelCase")]
enum UpdateProgress {
    Started {
        content_length: Option<u64>,
    },
    Progress {
        chunk_length: usize,
        downloaded: u64,
        content_length: Option<u64>,
    },
    Finished,
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

#[tauri::command]
pub fn get_app_version_info(app: AppHandle) -> AppVersionInfo {
    AppVersionInfo {
        launcher_version: app.package_info().version.to_string(),
        rustfs_version: option_env!("RUSTFS_VERSION")
            .unwrap_or("unknown")
            .to_string(),
    }
}

#[tauri::command]
pub async fn check_for_update(app: AppHandle) -> Result<UpdateInfo> {
    let current_version = app.package_info().version.to_string();
    let update = app
        .updater()
        .map_err(|error| Error::Update(error.to_string()))?
        .check()
        .await
        .map_err(|error| Error::Update(error.to_string()))?;

    Ok(match update {
        Some(update) => UpdateInfo {
            available: true,
            current_version,
            version: Some(update.version),
            notes: update.body,
            date: update.date.map(|date| date.to_string()),
            rustfs_running: state::is_rustfs_process_running(),
        },
        None => UpdateInfo {
            available: false,
            current_version,
            version: None,
            notes: None,
            date: None,
            rustfs_running: state::is_rustfs_process_running(),
        },
    })
}

#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<CommandResponse> {
    let update = app
        .updater()
        .map_err(|error| Error::Update(error.to_string()))?
        .check()
        .await
        .map_err(|error| Error::Update(error.to_string()))?
        .ok_or_else(|| Error::Update("No update is available".to_string()))?;

    if state::is_rustfs_process_running() {
        add_app_log("Stopping RustFS before installing update".to_string());
        state::terminate_rustfs_process();
    }

    let progress_app = app.clone();
    let finished_app = app.clone();
    let mut downloaded = 0_u64;

    update
        .download_and_install(
            move |chunk_length, content_length| {
                if downloaded == 0 {
                    let _ = progress_app.emit(
                        "update-progress",
                        UpdateProgress::Started { content_length },
                    );
                }
                downloaded += chunk_length as u64;
                let _ = progress_app.emit(
                    "update-progress",
                    UpdateProgress::Progress {
                        chunk_length,
                        downloaded,
                        content_length,
                    },
                );
            },
            move || {
                let _ = finished_app.emit("update-progress", UpdateProgress::Finished);
            },
        )
        .await
        .map_err(|error| Error::Update(error.to_string()))?;

    add_app_log("Update installed; restarting launcher".to_string());
    app.restart();
}
