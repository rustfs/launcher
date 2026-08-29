use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("Data path is required")]
    DataPathRequired,

    #[error("Data path does not exist: {0}")]
    DataPathNotExist(String),

    #[error("Data path is not a directory: {0}")]
    DataPathNotDirectory(String),

    #[error("Invalid data path: {0}")]
    InvalidDataPath(String),

    #[error("API port and console port must be different")]
    PortConflict,

    #[error("Port {0} is already in use")]
    PortInUse(u16),

    #[error("RustFS is already running")]
    AlreadyRunning,

    #[error("RustFS binary not found at {0}")]
    BinaryNotFound(String),

    #[error("Failed to read metadata for {0}: {1}")]
    Metadata(String, std::io::Error),

    #[error("Failed to execute RustFS binary: {0}")]
    BinaryExecution(std::io::Error),

    #[error("RustFS binary failed with exit code: {0}")]
    BinaryFailed(String),

    #[error("Invalid service URL")]
    InvalidUrl,

    #[error("Update failed: {0}")]
    Update(String),

    #[error("RustFS Launcher supports Windows and macOS only (detected {0})")]
    UnsupportedPlatform(String),

    #[error("Network request failed: {0}")]
    Network(String),

    #[error("Release asset not found: {0}")]
    AssetNotFound(String),

    #[error("Checksum mismatch for {asset}: expected {expected}, got {actual}")]
    ChecksumMismatch {
        asset: String,
        expected: String,
        actual: String,
    },

    #[error("RustFS update failed: {0}")]
    RustFsUpdate(String),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::Error;

    #[test]
    fn serializes_as_the_display_string_for_the_frontend() {
        let error = Error::PortInUse(9000);
        assert_eq!(
            serde_json::to_string(&error).unwrap(),
            "\"Port 9000 is already in use\""
        );
    }

    #[test]
    fn unsupported_platform_names_the_supported_targets() {
        let message = Error::UnsupportedPlatform("linux-x86_64".to_string()).to_string();
        assert!(message.contains("Windows"));
        assert!(message.contains("macOS"));
        assert!(message.contains("linux-x86_64"));
    }

    #[test]
    fn checksum_mismatch_reports_both_digests() {
        let message = Error::ChecksumMismatch {
            asset: "rustfs-windows-x86_64-v1.0.0.zip".to_string(),
            expected: "aaa".to_string(),
            actual: "bbb".to_string(),
        }
        .to_string();
        assert!(message.contains("rustfs-windows-x86_64-v1.0.0.zip"));
        assert!(message.contains("aaa"));
        assert!(message.contains("bbb"));
    }
}
