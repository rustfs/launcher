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
