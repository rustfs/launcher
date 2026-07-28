use serde::{Deserialize, Serialize};

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
            port: Some(9000),
            console_port: Some(9001),
            host: Some("127.0.0.1".to_string()),
            access_key: Some("rustfsadmin".to_string()),
            secret_key: Some("rustfsadmin".to_string()),
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
    pub rustfs_version: String,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct UpdateInfo {
    pub available: bool,
    pub current_version: String,
    pub version: Option<String>,
    pub notes: Option<String>,
    pub date: Option<String>,
    pub rustfs_running: bool,
}
