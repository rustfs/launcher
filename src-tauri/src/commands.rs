use crate::binaries;
use crate::config::RustFsConfig;
use crate::error::{Error, Result};
use crate::network;
use crate::platform;
use crate::process;
use crate::rustfs_update::{self, RustFsUpdateInfo};
use crate::state::{self, add_app_log};
use serde::Serialize;
use std::io::Error as IoError;
use tauri::async_runtime;
use tauri::{AppHandle, Emitter};
use tauri_plugin_opener::OpenerExt;
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
    pub bundled_rustfs_version: Option<String>,
    pub rustfs_managed: bool,
    pub platform: String,
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

#[derive(Debug, Serialize)]
pub struct RuntimeStatus {
    pub managed_running: bool,
    pub service_online: bool,
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
    process::resolve_data_path(&config.data_path)?;
    if config.console_enable && config.api_port() == config.console_port() {
        return Err(Error::PortConflict);
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
    let result = async_runtime::spawn_blocking(move || network::tcp_online(&host, port))
        .await
        .unwrap_or(false);
    Ok(result)
}

#[tauri::command]
pub async fn is_rustfs_process_running() -> Result<bool> {
    Ok(state::is_rustfs_process_running())
}

#[tauri::command]
pub async fn get_runtime_status(host: String, port: u16) -> Result<RuntimeStatus> {
    let managed_running = state::is_rustfs_process_running();
    let service_online = async_runtime::spawn_blocking(move || network::tcp_online(&host, port))
        .await
        .unwrap_or(false);

    Ok(RuntimeStatus {
        managed_running,
        service_online,
    })
}

#[tauri::command]
pub fn get_app_version_info(app: AppHandle) -> AppVersionInfo {
    let effective = binaries::effective_version();
    AppVersionInfo {
        launcher_version: app.package_info().version.to_string(),
        rustfs_version: effective.version,
        bundled_rustfs_version: binaries::bundled_version().map(str::to_string),
        rustfs_managed: effective.managed,
        platform: platform::current_label(),
    }
}

fn is_http_url(url: &str) -> bool {
    let scheme_ok = url.starts_with("http://") || url.starts_with("https://");
    scheme_ok && !url.chars().any(|ch| ch.is_control() || ch.is_whitespace())
}

#[tauri::command]
pub async fn open_service_url(app: AppHandle, url: String) -> Result<CommandResponse> {
    if !is_http_url(&url) {
        return Err(Error::InvalidUrl);
    }

    app.opener()
        .open_url(&url, None::<&str>)
        .map_err(|err| Error::Io(IoError::other(err.to_string())))?;

    Ok(CommandResponse {
        success: true,
        message: format!("Opened {url}"),
    })
}

#[tauri::command]
pub async fn check_for_update(app: AppHandle) -> Result<UpdateInfo> {
    let current_version = app.package_info().version.to_string();
    let update = app
        .updater()
        .map_err(|error| {
            add_app_log(format!("Launcher update check failed: {error}"));
            Error::Update(error.to_string())
        })?
        .check()
        .await
        .map_err(|error| {
            add_app_log(format!("Launcher update check failed: {error}"));
            Error::Update(error.to_string())
        })?;

    Ok(match update {
        Some(update) => {
            add_app_log(format!(
                "Launcher update available: {current_version} -> {}",
                update.version
            ));
            UpdateInfo {
                available: true,
                current_version,
                version: Some(update.version),
                notes: update.body,
                date: update.date.map(|date| date.to_string()),
                rustfs_running: state::is_rustfs_process_running(),
            }
        }
        None => {
            add_app_log(format!(
                "Launcher {current_version} is already the latest release"
            ));
            UpdateInfo {
                available: false,
                current_version,
                version: None,
                notes: None,
                date: None,
                rustfs_running: state::is_rustfs_process_running(),
            }
        }
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

/// Checks <https://version.rustfs.com/latest.json> for a newer RustFS build.
/// This is independent of the launcher's own self-update.
#[tauri::command]
pub async fn check_rustfs_update() -> Result<RustFsUpdateInfo> {
    rustfs_update::check().await
}

/// Downloads, verifies, and installs the latest RustFS build for this platform.
#[tauri::command]
pub async fn install_rustfs_update() -> Result<CommandResponse> {
    let message = rustfs_update::install().await.inspect_err(|error| {
        add_app_log(format!("RustFS update failed: {error}"));
    })?;

    Ok(CommandResponse {
        success: true,
        message,
    })
}

#[cfg(test)]
mod tests {
    use super::is_http_url;

    #[test]
    fn http_url_validation() {
        assert!(is_http_url("http://127.0.0.1:9000"));
        assert!(is_http_url("https://example.com/console"));
        assert!(!is_http_url("file:///etc/passwd"));
        assert!(!is_http_url("http://evil.example \nhttp://other"));
    }
}
