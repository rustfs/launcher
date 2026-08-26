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

impl RustFsConfig {
    pub fn bind_host(&self) -> &str {
        self.host
            .as_deref()
            .filter(|host| !host.is_empty())
            .unwrap_or("127.0.0.1")
    }

    pub fn api_port(&self) -> u16 {
        self.port.unwrap_or(9000)
    }

    pub fn console_port(&self) -> u16 {
        self.console_port.unwrap_or(9001)
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
