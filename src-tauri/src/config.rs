use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct RustFsConfig {
    pub binary_path: Option<String>,
    pub data_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub console_port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret_key: Option<String>,
    pub console_enable: bool,
}

impl Default for RustFsConfig {
    fn default() -> Self {
        Self {
            binary_path: None,
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
