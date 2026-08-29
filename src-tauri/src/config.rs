use serde::{Deserialize, Serialize};

pub const DEFAULT_API_PORT: u16 = 9000;
pub const DEFAULT_CONSOLE_PORT: u16 = 9001;
pub const DEFAULT_HOST: &str = "127.0.0.1";
pub const DEFAULT_ACCESS_KEY: &str = "rustfsadmin";
pub const DEFAULT_SECRET_KEY: &str = "rustfsadmin";

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
            port: Some(DEFAULT_API_PORT),
            console_port: Some(DEFAULT_CONSOLE_PORT),
            host: Some(DEFAULT_HOST.to_string()),
            access_key: Some(DEFAULT_ACCESS_KEY.to_string()),
            secret_key: Some(DEFAULT_SECRET_KEY.to_string()),
            console_enable: false,
        }
    }
}

impl RustFsConfig {
    pub fn bind_host(&self) -> &str {
        self.host
            .as_deref()
            .filter(|host| !host.is_empty())
            .unwrap_or(DEFAULT_HOST)
    }

    pub fn api_port(&self) -> u16 {
        self.port.unwrap_or(DEFAULT_API_PORT)
    }

    pub fn console_port(&self) -> u16 {
        self.console_port.unwrap_or(DEFAULT_CONSOLE_PORT)
    }
}

#[cfg(test)]
mod tests {
    use super::RustFsConfig;

    #[test]
    fn default_bind_is_loopback() {
        let config = RustFsConfig::default();
        assert_eq!(config.bind_host(), "127.0.0.1");
        assert_eq!(config.api_port(), 9000);
        assert_eq!(config.console_port(), 9001);
        assert!(!config.console_enable);
    }
}
