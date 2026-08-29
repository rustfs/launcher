use serde::{Deserialize, Serialize};

/// Defaults mirrored from `src-tauri/src/config.rs` so the form shows the same
/// values the backend falls back to.
pub const DEFAULT_API_PORT: u16 = 9000;
pub const DEFAULT_CONSOLE_PORT: u16 = 9001;
pub const DEFAULT_HOST: &str = "127.0.0.1";
pub const DEFAULT_ACCESS_KEY: &str = "rustfsadmin";
pub const DEFAULT_SECRET_KEY: &str = "rustfsadmin";

/// Log buffer sizes, matching the backend ring buffers in `state.rs`.
pub const APP_LOG_CAPACITY: usize = 100;
pub const RUSTFS_LOG_CAPACITY: usize = 1000;

#[derive(Debug, Clone, PartialEq)]
pub struct LogEntry {
    pub id: u64,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct RustFsConfig {
    pub data_path: String,
    pub port: Option<u16>,
    pub console_port: Option<u16>,
    pub host: Option<String>,
    pub access_key: Option<String>,
    pub secret_key: Option<String>,
    pub console_enable: bool,
}

impl Default for RustFsConfig {
    fn default() -> Self {
        Self {
            data_path: String::new(),
            port: Some(DEFAULT_API_PORT),
            console_port: Some(DEFAULT_CONSOLE_PORT),
            host: Some(DEFAULT_HOST.to_string()),
            access_key: Some(DEFAULT_ACCESS_KEY.to_string()),
            secret_key: Some(DEFAULT_SECRET_KEY.to_string()),
            console_enable: false,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum LogType {
    App,
    RustFS,
}

#[derive(Debug, Deserialize)]
pub struct CommandResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct AppVersionInfo {
    pub launcher_version: String,
    /// Version of the RustFS binary the launcher will actually start.
    pub rustfs_version: String,
    /// Version shipped inside the installer, when the build recorded one.
    pub bundled_rustfs_version: Option<String>,
    /// True when a user-installed RustFS binary takes precedence over the bundle.
    pub rustfs_managed: bool,
    pub platform: String,
}

/// Result of a launcher self-update check (tauri-plugin-updater).
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct UpdateInfo {
    pub available: bool,
    pub current_version: String,
    pub version: Option<String>,
    pub notes: Option<String>,
    pub date: Option<String>,
    pub rustfs_running: bool,
}

/// Result of a RustFS binary update check against the upstream release feed.
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct RustFsUpdateInfo {
    pub available: bool,
    pub current_version: String,
    pub current_managed: bool,
    pub latest_version: Option<String>,
    pub release_date: Option<String>,
    pub release_type: Option<String>,
    pub release_url: Option<String>,
    pub asset_name: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    pub platform: String,
    pub rustfs_running: bool,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct RuntimeStatus {
    pub managed_running: bool,
    pub service_online: bool,
}
