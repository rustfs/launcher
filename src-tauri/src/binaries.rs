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
    let dest = dir.join(RECORD_FILE);
    let tmp = dir.join(format!("{RECORD_FILE}.tmp"));
    std::fs::write(&tmp, contents).map_err(Error::Io)?;
    std::fs::rename(&tmp, dest).map_err(Error::Io)
}

/// User-installed binary to run, when it exists and is not older than the
/// bundled copy.
pub fn installed_binary_from(
    dir: &Path,
    bundled: Option<&str>,
) -> Option<(PathBuf, InstalledRecord)> {
    let record = read_record_from(dir)?;
    if !version::prefer_installed(&record.version, bundled) {
        return None;
    }

    let path = dir.join(&record.binary);
    path.is_file().then_some((path, record))
}

pub fn installed_binary() -> Option<(PathBuf, InstalledRecord)> {
    installed_binary_from(&managed_dir()?, bundled_version())
}

/// Version of RustFS the launcher will actually start.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectiveVersion {
    pub version: String,
    pub managed: bool,
}

pub fn effective_version_from(dir: Option<&Path>, bundled: Option<&str>) -> EffectiveVersion {
    match dir.and_then(|dir| installed_binary_from(dir, bundled)) {
        Some((_, record)) => EffectiveVersion {
            version: record.version,
            managed: true,
        },
        None => EffectiveVersion {
            version: bundled.unwrap_or("unknown").to_string(),
            managed: false,
        },
    }
}

pub fn effective_version() -> EffectiveVersion {
    effective_version_from(managed_dir().as_deref(), bundled_version())
}

/// File name the platform expects, e.g. `rustfs-windows-x86_64.exe`.
pub fn binary_name() -> Result<&'static str> {
    Platform::detect().map(Platform::binary_name)
}

#[cfg(test)]
mod tests {
    use super::{
        effective_version_from, installed_binary_from, read_record_from, write_record,
        InstalledRecord,
    };

    fn sample_record(version: &str, binary: &str) -> InstalledRecord {
        InstalledRecord {
            version: version.to_string(),
            binary: binary.to_string(),
            asset: format!("{binary}-v{version}.zip"),
            sha256: "6c6a0e84".to_string(),
            installed_at: "2026-08-28T00:00:00Z".to_string(),
        }
    }

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

    #[test]
    fn installed_binary_requires_the_file_and_a_new_enough_version() {
        let dir = tempfile::tempdir().unwrap();
        let record = sample_record("1.0.0-rc.4", "rustfs-macos-aarch64");
        write_record(dir.path(), &record).unwrap();

        assert!(
            installed_binary_from(dir.path(), Some("1.0.0-rc.3")).is_none(),
            "record without the binary file must be ignored"
        );

        std::fs::write(dir.path().join(&record.binary), b"stub").unwrap();
        let (path, loaded) =
            installed_binary_from(dir.path(), Some("1.0.0-rc.3")).expect("newer install wins");
        assert_eq!(loaded.version, "1.0.0-rc.4");
        assert_eq!(path.file_name().unwrap(), "rustfs-macos-aarch64");

        assert!(
            installed_binary_from(dir.path(), Some("1.0.0-rc.5")).is_none(),
            "an older install must not shadow a newer bundle"
        );
        assert!(installed_binary_from(dir.path(), None).is_some());
    }

    #[test]
    fn effective_version_prefers_the_install_that_will_actually_run() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(
            effective_version_from(None, Some("1.0.0-rc.3")).version,
            "1.0.0-rc.3"
        );
        assert!(!effective_version_from(None, Some("1.0.0-rc.3")).managed);
        assert_eq!(effective_version_from(None, None).version, "unknown");

        let record = sample_record("1.0.0-rc.4", "rustfs-windows-x86_64.exe");
        write_record(dir.path(), &record).unwrap();
        std::fs::write(dir.path().join(&record.binary), b"stub").unwrap();

        let managed = effective_version_from(Some(dir.path()), Some("1.0.0-rc.3"));
        assert_eq!(managed.version, "1.0.0-rc.4");
        assert!(managed.managed);

        let bundled = effective_version_from(Some(dir.path()), Some("1.0.0"));
        assert_eq!(bundled.version, "1.0.0");
        assert!(!bundled.managed);
    }
}
