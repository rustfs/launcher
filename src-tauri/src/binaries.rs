//! Bookkeeping for the RustFS binary the launcher runs.
//!
//! Two copies can exist: the one bundled into the installer (read-only, inside
//! the app bundle or `resources/`) and one the user installed later through the
//! in-app RustFS update flow (writable, under the app data directory).

use crate::error::{Error, Result};
use crate::platform::Platform;
use crate::state::{acquire, APP_HANDLE};
use crate::version;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tauri::Manager;

const RECORD_FILE: &str = "installed.json";

/// Metadata written next to a user-installed RustFS binary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledRecord {
    pub version: String,
    pub binary: String,
    pub asset: String,
    pub sha256: String,
    pub installed_at: String,
}

/// Version compiled into the launcher by the release workflow.
pub fn bundled_version() -> Option<&'static str> {
    option_env!("RUSTFS_VERSION")
        .map(str::trim)
        .filter(|value| !value.is_empty() && *value != "unknown")
}

/// Writable directory holding user-installed RustFS binaries.
pub fn managed_dir() -> Option<PathBuf> {
    let handle = acquire(&APP_HANDLE);
    handle
        .as_ref()
        .and_then(|app| app.path().app_data_dir().ok())
        .map(|dir| dir.join("binaries"))
}

pub fn ensure_managed_dir() -> Result<PathBuf> {
    let dir = managed_dir().ok_or_else(|| {
        Error::RustFsUpdate("Unable to resolve the launcher data directory".to_string())
    })?;
    std::fs::create_dir_all(&dir).map_err(Error::Io)?;
    Ok(dir)
}

pub fn read_record_from(dir: &Path) -> Option<InstalledRecord> {
    let contents = std::fs::read_to_string(dir.join(RECORD_FILE)).ok()?;
    serde_json::from_str(&contents).ok()
}

pub fn write_record(dir: &Path, record: &InstalledRecord) -> Result<()> {
    let contents = serde_json::to_string_pretty(record)
        .map_err(|error| Error::RustFsUpdate(error.to_string()))?;
    std::fs::write(dir.join(RECORD_FILE), contents).map_err(Error::Io)
}

/// User-installed binary to run, when it exists and is not older than the
/// bundled copy.
pub fn installed_binary() -> Option<(PathBuf, InstalledRecord)> {
    let dir = managed_dir()?;
    let record = read_record_from(&dir)?;
    if !version::prefer_installed(&record.version, bundled_version()) {
        return None;
    }

    let path = dir.join(&record.binary);
    path.is_file().then_some((path, record))
}

/// Version of RustFS the launcher will actually start.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectiveVersion {
    pub version: String,
    pub managed: bool,
}

pub fn effective_version() -> EffectiveVersion {
    match installed_binary() {
        Some((_, record)) => EffectiveVersion {
            version: record.version,
            managed: true,
        },
        None => EffectiveVersion {
            version: bundled_version().unwrap_or("unknown").to_string(),
            managed: false,
        },
    }
}

/// File name the platform expects, e.g. `rustfs-windows-x86_64.exe`.
pub fn binary_name() -> Result<&'static str> {
    Platform::detect().map(Platform::binary_name)
}

#[cfg(test)]
mod tests {
    use super::{read_record_from, write_record, InstalledRecord};

    #[test]
    fn record_round_trips_through_the_managed_dir() {
        let dir = tempfile::tempdir().unwrap();
        let record = InstalledRecord {
            version: "1.0.0-rc.4".to_string(),
            binary: "rustfs-macos-aarch64".to_string(),
            asset: "rustfs-macos-aarch64-v1.0.0-rc.4.zip".to_string(),
            sha256: "6c6a0e84".to_string(),
            installed_at: "2026-08-28T00:00:00Z".to_string(),
        };

        write_record(dir.path(), &record).unwrap();
        let loaded = read_record_from(dir.path()).expect("record should be readable");

        assert_eq!(loaded.version, record.version);
        assert_eq!(loaded.binary, record.binary);
        assert_eq!(loaded.sha256, record.sha256);
    }

    #[test]
    fn missing_or_corrupt_records_are_ignored() {
        let dir = tempfile::tempdir().unwrap();
        assert!(read_record_from(dir.path()).is_none());

        std::fs::write(dir.path().join("installed.json"), "not json").unwrap();
        assert!(read_record_from(dir.path()).is_none());
    }
}
