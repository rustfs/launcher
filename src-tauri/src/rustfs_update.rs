//! Manual RustFS binary updates.
//!
//! The launcher ships a RustFS build inside its installer, but users should not
//! have to reinstall the launcher to pick up a new RustFS release. This module
//! resolves the latest upstream release, downloads the archive for the current
//! platform, verifies it against the release `SHA256SUMS`, and installs it into
//! the writable app data directory.

use crate::binaries::{self, InstalledRecord};
use crate::error::{Error, Result};
use crate::platform::Platform;
use crate::state::{acquire, add_app_log, APP_HANDLE};
use crate::version;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tauri::{Emitter, Manager};

const LATEST_JSON_URL: &str = "https://version.rustfs.com/latest.json";
const RELEASE_DOWNLOAD_BASE: &str = "https://github.com/rustfs/rustfs/releases/download";
const CHECKSUM_FILE: &str = "SHA256SUMS";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(600);
const PROGRESS_EVENT: &str = "rustfs-update-progress";

/// Shape of <https://version.rustfs.com/latest.json>.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct LatestRelease {
    version: Option<String>,
    tag: Option<String>,
    release_date: Option<String>,
    release_type: Option<String>,
    download_url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RustFsUpdateInfo {
    pub available: bool,
    pub current_version: String,
    pub current_managed: bool,
    pub latest_version: Option<String>,
    pub release_date: Option<String>,
    pub release_type: Option<String>,
    pub release_url: Option<String>,
    pub asset_name: Option<String>,
    pub notes: Option<String>,
    pub platform: String,
    pub rustfs_running: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", rename_all = "camelCase")]
enum Stage {
    Started {
        asset: String,
        content_length: Option<u64>,
    },
    Progress {
        downloaded: u64,
        content_length: Option<u64>,
    },
    Verifying,
    Installing,
    Finished {
        version: String,
    },
}

fn emit_progress(stage: Stage) {
    if let Some(handle) = acquire(&APP_HANDLE).as_ref() {
        let _ = handle.emit(PROGRESS_EVENT, stage);
    }
}

fn client(timeout: Duration) -> Result<reqwest::Client> {
    reqwest::Client::builder()
        .timeout(timeout)
        .user_agent(concat!("rustfs-launcher/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|error| Error::Network(error.to_string()))
}

async fn fetch_text(url: &str, timeout: Duration) -> Result<String> {
    let response = client(timeout)?
        .get(url)
        .send()
        .await
        .map_err(|error| Error::Network(error.to_string()))?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(Error::AssetNotFound(url.to_string()));
    }
    let response = response
        .error_for_status()
        .map_err(|error| Error::Network(error.to_string()))?;

    response
        .text()
        .await
        .map_err(|error| Error::Network(error.to_string()))
}

/// Builds the archive name upstream publishes for a platform and tag, e.g.
/// `rustfs-macos-aarch64-v1.0.0-rc.4.zip`.
pub(crate) fn asset_name(platform: Platform, tag: &str) -> String {
    format!(
        "{}-{}.zip",
        platform.asset_slug(),
        version::asset_version(tag)
    )
}

fn asset_url(tag: &str, asset: &str) -> String {
    format!(
        "{RELEASE_DOWNLOAD_BASE}/{}/{asset}",
        version::normalize(tag)
    )
}

/// Extracts the digest for `asset` out of a `sha256sum`-style manifest.
pub(crate) fn digest_for_asset(manifest: &str, asset: &str) -> Option<String> {
    manifest.lines().find_map(|line| {
        let mut fields = line.split_whitespace();
        let digest = fields.next()?;
        let name = fields.next()?.trim_start_matches('*');
        (name == asset && digest.len() == 64).then(|| digest.to_ascii_lowercase())
    })
}

#[derive(Debug, Clone)]
pub(crate) struct FeedSnapshot {
    pub tag: String,
    pub release: LatestRelease,
}

pub(crate) fn parse_latest_feed(body: &str) -> Result<FeedSnapshot> {
    let release: LatestRelease = serde_json::from_str(body).map_err(|error| {
        Error::RustFsUpdate(format!(
            "Could not parse the upstream version feed: {error}"
        ))
    })?;

    let tag = release
        .tag
        .as_deref()
        .or(release.version.as_deref())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            Error::RustFsUpdate("Upstream version feed did not report a release tag".to_string())
        })?
        .to_string();

    Ok(FeedSnapshot { tag, release })
}

fn safe_release_url(url: Option<&str>) -> Option<String> {
    url.map(str::trim)
        .filter(|value| value.starts_with("https://") && !value.chars().any(char::is_whitespace))
        .map(str::to_string)
}

pub(crate) fn evaluate_update(
    platform: Platform,
    current: &binaries::EffectiveVersion,
    feed: &FeedSnapshot,
    rustfs_running: bool,
    manifest: Option<&str>,
) -> RustFsUpdateInfo {
    let asset = asset_name(platform, &feed.tag);
    let newer = version::is_newer(&feed.tag, &current.version);
    let asset_listed = manifest.map(|body| digest_for_asset(body, &asset).is_some());
    let (available, notes) = match (newer, asset_listed) {
        (true, Some(false)) => (
            false,
            Some(format!(
                "RustFS {} has no {} build.",
                version::normalize(&feed.tag),
                platform.label()
            )),
        ),
        (true, _) => (true, None),
        (false, _) => (false, None),
    };

    RustFsUpdateInfo {
        available,
        current_version: current.version.clone(),
        current_managed: current.managed,
        latest_version: Some(version::normalize(&feed.tag).to_string()),
        release_date: feed.release.release_date.clone(),
        release_type: feed.release.release_type.clone(),
        release_url: safe_release_url(feed.release.download_url.as_deref()),
        asset_name: Some(asset),
        notes,
        platform: platform.label().to_string(),
        rustfs_running,
    }
}

async fn resolve_latest() -> Result<FeedSnapshot> {
    let body = fetch_text(LATEST_JSON_URL, REQUEST_TIMEOUT).await?;
    parse_latest_feed(&body)
}

pub async fn check() -> Result<RustFsUpdateInfo> {
    let platform = Platform::detect()?;
    let current = binaries::effective_version();
    let feed = resolve_latest().await?;
    let manifest = fetch_text(&asset_url(&feed.tag, CHECKSUM_FILE), REQUEST_TIMEOUT)
        .await
        .ok();
    let info = evaluate_update(
        platform,
        &current,
        &feed,
        crate::state::is_rustfs_process_running(),
        manifest.as_deref(),
    );
    add_app_log(format!(
        "RustFS update check: installed={}, latest={}, available={}",
        info.current_version,
        info.latest_version.as_deref().unwrap_or("unknown"),
        info.available
    ));
    Ok(info)
}

fn cache_dir() -> Result<PathBuf> {
    let handle = acquire(&APP_HANDLE);
    let dir = handle
        .as_ref()
        .and_then(|app| app.path().app_cache_dir().ok())
        .ok_or_else(|| {
            Error::RustFsUpdate("Unable to resolve the launcher cache directory".to_string())
        })?;
    drop(handle);

    let dir = dir.join("rustfs-update");
    std::fs::create_dir_all(&dir).map_err(Error::Io)?;
    Ok(dir)
}

async fn download_and_verify(url: &str, asset: &str, expected: &str, dest: &Path) -> Result<()> {
    let mut response = client(DOWNLOAD_TIMEOUT)?
        .get(url)
        .send()
        .await
        .map_err(|error| Error::Network(error.to_string()))?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(Error::AssetNotFound(asset.to_string()));
    }
    response = response
        .error_for_status()
        .map_err(|error| Error::Network(error.to_string()))?;

    let content_length = response.content_length();
    emit_progress(Stage::Started {
        asset: asset.to_string(),
        content_length,
    });

    let mut file = std::fs::File::create(dest).map_err(Error::Io)?;
    let mut hasher = Sha256::new();
    let mut downloaded = 0_u64;
    let mut last_reported = 0_u64;

    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|error| Error::Network(error.to_string()))?
    {
        hasher.update(&chunk);
        file.write_all(&chunk).map_err(Error::Io)?;
        downloaded += chunk.len() as u64;

        // Throttle to ~1 event per 512 KiB so the webview is not flooded.
        if downloaded - last_reported >= 512 * 1024 {
            last_reported = downloaded;
            emit_progress(Stage::Progress {
                downloaded,
                content_length,
            });
        }
    }
    file.flush().map_err(Error::Io)?;
    drop(file);

    emit_progress(Stage::Progress {
        downloaded,
        content_length,
    });
    emit_progress(Stage::Verifying);

    let actual = hex_digest(&hasher.finalize());
    if let Err(error) = verify_sha256(&actual, expected, asset) {
        let _ = std::fs::remove_file(dest);
        return Err(error);
    }

    Ok(())
}

#[cfg(test)]
pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    hex_digest(&Sha256::digest(bytes))
}

pub(crate) fn verify_sha256(actual: &str, expected: &str, asset: &str) -> Result<()> {
    let actual = actual.to_ascii_lowercase();
    let expected = expected.to_ascii_lowercase();
    if actual != expected {
        return Err(Error::ChecksumMismatch {
            asset: asset.to_string(),
            expected,
            actual,
        });
    }
    Ok(())
}

pub(crate) fn hash_file(path: &Path) -> Result<String> {
    let mut hasher = Sha256::new();
    let mut file = std::fs::File::open(path).map_err(Error::Io)?;
    let mut buffer = [0_u8; 8192];
    loop {
        let read = std::io::Read::read(&mut file, &mut buffer).map_err(Error::Io)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hex_digest(&hasher.finalize()))
}

fn hex_digest(bytes: &[u8]) -> String {
    bytes
        .iter()
        .fold(String::with_capacity(64), |mut acc, byte| {
            use std::fmt::Write as _;
            let _ = write!(acc, "{byte:02x}");
            acc
        })
}

/// Pulls the `rustfs` executable out of an upstream release archive.
pub(crate) fn extract_binary(archive: &Path, dest: &Path) -> Result<()> {
    let file = std::fs::File::open(archive).map_err(Error::Io)?;
    let mut zip = zip::ZipArchive::new(file)
        .map_err(|error| Error::RustFsUpdate(format!("Could not open the archive: {error}")))?;

    let index = (0..zip.len())
        .find(|index| {
            zip.by_index(*index)
                .ok()
                .and_then(|entry| {
                    entry.enclosed_name().and_then(|path| {
                        path.file_name()
                            .map(|name| name.to_string_lossy().to_ascii_lowercase())
                    })
                })
                .is_some_and(|name| name == "rustfs" || name == "rustfs.exe")
        })
        .ok_or_else(|| {
            Error::RustFsUpdate("Archive did not contain a rustfs executable".to_string())
        })?;

    let mut entry = zip
        .by_index(index)
        .map_err(|error| Error::RustFsUpdate(format!("Could not read the archive: {error}")))?;
    let mut out = std::fs::File::create(dest).map_err(Error::Io)?;
    std::io::copy(&mut entry, &mut out).map_err(Error::Io)?;
    out.flush().map_err(Error::Io)?;
    drop(out);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(dest, std::fs::Permissions::from_mode(0o755))
            .map_err(Error::Io)?;
    }

    Ok(())
}

/// Runs `--version` on a freshly extracted binary so a truncated or
/// architecture-mismatched download is rejected before it replaces a working
/// install.
pub(crate) fn smoke_test(path: &Path) -> Result<String> {
    let mut command = std::process::Command::new(path);
    command.arg("--version");
    crate::process::apply_no_window(&mut command);

    let output = command.output().map_err(|error| {
        Error::RustFsUpdate(format!("Downloaded RustFS binary is not runnable: {error}"))
    })?;

    if !output.status.success() {
        return Err(Error::RustFsUpdate(format!(
            "Downloaded RustFS binary exited with {} when asked for its version",
            output.status
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Replaces `target` with `staged`, keeping a rollback copy while doing so.
/// Windows refuses to overwrite a file that is still mapped, so the previous
/// binary is moved aside instead of deleted in place.
fn swap_in_place(staged: &Path, target: &Path) -> Result<()> {
    if !target.exists() {
        return std::fs::rename(staged, target).map_err(Error::Io);
    }

    let backup = target.with_extension("previous");
    let _ = std::fs::remove_file(&backup);
    std::fs::rename(target, &backup).map_err(Error::Io)?;

    match std::fs::rename(staged, target) {
        Ok(()) => {
            let _ = std::fs::remove_file(&backup);
            Ok(())
        }
        Err(error) => {
            let _ = std::fs::rename(&backup, target);
            Err(Error::Io(error))
        }
    }
}

/// Extracts the archive and optionally smoke-tests it before the live binary
/// is replaced. A failed smoke test leaves the current install untouched.
pub(crate) fn stage_update(archive: &Path, staged: &Path, run_smoke: bool) -> Result<String> {
    extract_binary(archive, staged).inspect_err(|_| {
        let _ = std::fs::remove_file(staged);
    })?;

    if !run_smoke {
        return Ok(String::new());
    }

    match smoke_test(staged) {
        Ok(reported) => Ok(reported),
        Err(error) => {
            let _ = std::fs::remove_file(staged);
            Err(error)
        }
    }
}

pub(crate) fn commit_update(
    platform: Platform,
    tag: &str,
    asset: &str,
    sha256: &str,
    staged: &Path,
    managed_dir: &Path,
    was_running: bool,
) -> Result<String> {
    std::fs::create_dir_all(managed_dir).map_err(Error::Io)?;
    let binary_name = platform.binary_name();
    let target = managed_dir.join(binary_name);
    swap_in_place(staged, &target)?;

    binaries::write_record(
        managed_dir,
        &InstalledRecord {
            version: version::normalize(tag).to_string(),
            binary: binary_name.to_string(),
            asset: asset.to_string(),
            sha256: sha256.to_ascii_lowercase(),
            installed_at: chrono::Utc::now().to_rfc3339(),
        },
    )?;

    let installed = version::normalize(tag).to_string();
    Ok(if was_running {
        format!("RustFS {installed} installed. The previous instance was stopped; launch it again to use the new build.")
    } else {
        format!("RustFS {installed} installed.")
    })
}

pub async fn install() -> Result<String> {
    let platform = Platform::detect()?;
    let feed = resolve_latest().await?;
    let current = binaries::effective_version();

    if !version::is_newer(&feed.tag, &current.version) {
        return Err(Error::RustFsUpdate(format!(
            "RustFS {} is already the latest release",
            current.version
        )));
    }

    let asset = asset_name(platform, &feed.tag);
    let manifest = fetch_text(&asset_url(&feed.tag, CHECKSUM_FILE), REQUEST_TIMEOUT).await?;
    let expected = digest_for_asset(&manifest, &asset).ok_or_else(|| {
        Error::AssetNotFound(format!(
            "{asset} is not listed in {CHECKSUM_FILE} for {}",
            feed.tag
        ))
    })?;

    let cache = cache_dir()?;
    let archive = cache.join(&asset);
    add_app_log(format!("Downloading RustFS {} ({asset})", feed.tag));
    download_and_verify(&asset_url(&feed.tag, &asset), &asset, &expected, &archive).await?;
    verify_sha256(&hash_file(&archive)?, &expected, &asset)?;
    add_app_log(format!("Verified {asset} against {CHECKSUM_FILE}"));

    emit_progress(Stage::Installing);
    let managed_dir = binaries::ensure_managed_dir()?;
    let staged = managed_dir.join(format!("{}.staged", platform.binary_name()));

    let archive_for_task = archive.clone();
    let staged_for_task = staged.clone();
    let reported = tauri::async_runtime::spawn_blocking(move || {
        stage_update(&archive_for_task, &staged_for_task, true)
    })
    .await
    .map_err(|error| Error::RustFsUpdate(error.to_string()))??;
    if !reported.is_empty() {
        add_app_log(format!("Downloaded binary reports: {reported}"));
    }

    let was_running = crate::state::is_rustfs_process_running();
    if was_running {
        add_app_log("Stopping RustFS before swapping in the new binary".to_string());
        crate::state::terminate_rustfs_process();
    }

    let message = commit_update(
        platform,
        &feed.tag,
        &asset,
        &expected,
        &staged,
        &managed_dir,
        was_running,
    )?;
    let _ = std::fs::remove_file(&archive);

    add_app_log(format!(
        "RustFS {} installed at {}",
        version::normalize(&feed.tag),
        managed_dir.join(platform.binary_name()).display()
    ));
    emit_progress(Stage::Finished {
        version: version::normalize(&feed.tag).to_string(),
    });

    Ok(message)
}

#[cfg(test)]
mod tests {
    use super::{
        asset_name, asset_url, commit_update, digest_for_asset, evaluate_update, extract_binary,
        hash_file, parse_latest_feed, sha256_hex, smoke_test, stage_update, swap_in_place,
        verify_sha256, FeedSnapshot, LatestRelease, Stage,
    };
    use crate::binaries::{effective_version_from, installed_binary_from, EffectiveVersion};
    use crate::error::Error;
    use crate::platform::Platform;
    use crate::process::launch;
    use crate::version;
    use std::io::Write;
    use std::net::TcpListener;
    use std::process::Command;

    const MANIFEST: &str = "\
f3acebb751620d940c87882afc811210fc2e186f457978b83819a04df6087722  rustfs-linux-aarch64-gnu-v1.0.0-rc.4.zip
6c6a0e841466ffb3c8ffa7299624876c12ef6ce3ff5af11b8afb55c23fd9c3d7  rustfs-macos-aarch64-v1.0.0-rc.4.zip
B34390D7CE5C1797B4A127261FEE1926C2AED1D280E1EC59F149F255902221AF *rustfs-windows-x86_64-v1.0.0-rc.4.zip
";

    #[test]
    fn builds_upstream_asset_names_and_urls() {
        let macos = Platform::from_target("macos", "aarch64").unwrap();
        assert_eq!(
            asset_name(macos, "1.0.0-rc.4"),
            "rustfs-macos-aarch64-v1.0.0-rc.4.zip"
        );
        assert_eq!(
            asset_name(macos, "v1.0.0-rc.4"),
            "rustfs-macos-aarch64-v1.0.0-rc.4.zip"
        );

        let windows = Platform::from_target("windows", "aarch64").unwrap();
        assert_eq!(
            asset_name(windows, "1.0.0-rc.4"),
            "rustfs-windows-x86_64-v1.0.0-rc.4.zip"
        );

        assert_eq!(
            asset_url("v1.0.0-rc.4", "SHA256SUMS"),
            "https://github.com/rustfs/rustfs/releases/download/1.0.0-rc.4/SHA256SUMS"
        );
    }

    #[test]
    fn reads_digests_out_of_the_checksum_manifest() {
        assert_eq!(
            digest_for_asset(MANIFEST, "rustfs-macos-aarch64-v1.0.0-rc.4.zip").as_deref(),
            Some("6c6a0e841466ffb3c8ffa7299624876c12ef6ce3ff5af11b8afb55c23fd9c3d7")
        );
        assert_eq!(
            digest_for_asset(MANIFEST, "rustfs-windows-x86_64-v1.0.0-rc.4.zip").as_deref(),
            Some("b34390d7ce5c1797b4a127261fee1926c2aed1d280e1ec59f149f255902221af")
        );
        assert!(digest_for_asset(MANIFEST, "rustfs-macos-x86_64-v1.0.0-rc.4.zip").is_none());
        assert!(digest_for_asset("short  file.zip", "file.zip").is_none());
    }

    fn write_zip(path: &std::path::Path, entries: &[(&str, &[u8])]) {
        let file = std::fs::File::create(path).unwrap();
        let mut writer = zip::ZipWriter::new(file);
        let options: zip::write::FileOptions<'_, ()> = zip::write::FileOptions::default();
        for (name, contents) in entries {
            writer.start_file(*name, options).unwrap();
            writer.write_all(contents).unwrap();
        }
        writer.finish().unwrap();
    }

    #[test]
    fn extracts_the_rustfs_executable_from_a_release_archive() {
        let dir = tempfile::tempdir().unwrap();
        let archive = dir.path().join("release.zip");
        write_zip(
            &archive,
            &[("README.md", b"docs"), ("rustfs", b"binary-bytes")],
        );

        let dest = dir.path().join("rustfs-macos-aarch64");
        extract_binary(&archive, &dest).unwrap();
        assert_eq!(std::fs::read(&dest).unwrap(), b"binary-bytes");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&dest).unwrap().permissions().mode();
            assert_ne!(mode & 0o111, 0, "extracted binary should be executable");
        }
    }

    #[test]
    fn extraction_fails_when_the_archive_has_no_binary() {
        let dir = tempfile::tempdir().unwrap();
        let archive = dir.path().join("release.zip");
        write_zip(&archive, &[("README.md", b"docs")]);

        let error = extract_binary(&archive, &dir.path().join("out")).unwrap_err();
        assert!(error.to_string().contains("rustfs executable"));
    }

    #[test]
    fn swap_replaces_an_existing_binary_and_cleans_up() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("rustfs");
        let staged = dir.path().join("rustfs.staged");
        std::fs::write(&target, b"old").unwrap();
        std::fs::write(&staged, b"new").unwrap();

        swap_in_place(&staged, &target).unwrap();

        assert_eq!(std::fs::read(&target).unwrap(), b"new");
        assert!(!staged.exists());
        assert!(!target.with_extension("previous").exists());
    }

    #[test]
    fn swap_installs_a_first_time_binary() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("rustfs");
        let staged = dir.path().join("rustfs.staged");
        std::fs::write(&staged, b"new").unwrap();

        swap_in_place(&staged, &target).unwrap();
        assert_eq!(std::fs::read(&target).unwrap(), b"new");
    }

    #[test]
    fn extracts_a_windows_exe_from_a_nested_archive() {
        let dir = tempfile::tempdir().unwrap();
        let archive = dir.path().join("release.zip");
        write_zip(&archive, &[("payload/rustfs.exe", b"mz-bytes")]);

        let dest = dir.path().join("rustfs-windows-x86_64.exe");
        extract_binary(&archive, &dest).unwrap();
        assert_eq!(std::fs::read(&dest).unwrap(), b"mz-bytes");
    }

    #[test]
    fn parse_latest_feed_accepts_tag_or_version_and_rejects_junk() {
        let from_tag = parse_latest_feed(
            r#"{"tag":"1.0.0-rc.4","version":"ignored","download_url":"https://github.com/rustfs/rustfs/releases/tag/1.0.0-rc.4"}"#,
        )
        .unwrap();
        assert_eq!(from_tag.tag, "1.0.0-rc.4");

        let from_version = parse_latest_feed(r#"{"version":"  v1.2.3  "}"#).unwrap();
        assert_eq!(from_version.tag, "v1.2.3");

        assert!(parse_latest_feed("not-json").is_err());
        assert!(parse_latest_feed(r#"{"tag":"","version":""}"#).is_err());
    }

    fn sample_feed(tag: &str, url: Option<&str>) -> FeedSnapshot {
        FeedSnapshot {
            tag: tag.to_string(),
            release: LatestRelease {
                version: Some(tag.to_string()),
                tag: Some(tag.to_string()),
                release_date: Some("2026-08-28T14:24:08Z".to_string()),
                release_type: Some("prerelease".to_string()),
                download_url: url.map(str::to_string),
            },
        }
    }

    #[test]
    fn evaluate_update_requires_a_newer_release_and_a_platform_asset() {
        let current = EffectiveVersion {
            version: "1.0.0-rc.3".to_string(),
            managed: false,
        };
        let windows = Platform::from_target("windows", "x86_64").unwrap();
        let intel_mac = Platform::from_target("macos", "x86_64").unwrap();
        let feed = sample_feed(
            "1.0.0-rc.4",
            Some("https://github.com/rustfs/rustfs/releases/tag/1.0.0-rc.4"),
        );

        let ready = evaluate_update(windows, &current, &feed, false, Some(MANIFEST));
        assert!(ready.available);
        assert_eq!(ready.latest_version.as_deref(), Some("1.0.0-rc.4"));
        assert_eq!(
            ready.release_url.as_deref(),
            Some("https://github.com/rustfs/rustfs/releases/tag/1.0.0-rc.4")
        );
        assert!(ready.notes.is_none());

        let missing = evaluate_update(intel_mac, &current, &feed, true, Some(MANIFEST));
        assert!(!missing.available);
        assert!(missing.rustfs_running);
        assert!(missing.notes.as_deref().unwrap().contains("macOS (Intel)"));

        let current_same = EffectiveVersion {
            version: "1.0.0-rc.4".to_string(),
            managed: true,
        };
        let already = evaluate_update(windows, &current_same, &feed, false, Some(MANIFEST));
        assert!(!already.available);
        assert!(already.current_managed);

        let unsafe_feed = sample_feed("1.0.0-rc.5", Some("http://evil.example/update"));
        let stripped = evaluate_update(windows, &current, &unsafe_feed, false, None);
        assert!(stripped.available);
        assert!(stripped.release_url.is_none());
    }

    #[test]
    fn checksum_helpers_accept_only_an_exact_digest() {
        let digest = sha256_hex(b"rustfs");
        assert_eq!(digest.len(), 64);
        verify_sha256(&digest, &digest.to_ascii_uppercase(), "asset.zip").unwrap();

        let error = verify_sha256(&digest, &"a".repeat(64), "asset.zip").unwrap_err();
        assert!(matches!(error, Error::ChecksumMismatch { .. }));

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("blob");
        std::fs::write(&path, b"rustfs").unwrap();
        assert_eq!(hash_file(&path).unwrap(), digest);
    }

    fn compile_stub(dir: &std::path::Path, source: &str, name: &str) -> std::path::PathBuf {
        let src = dir.join(format!("{name}.rs"));
        std::fs::write(&src, source).unwrap();
        let exe = dir.join(format!("{name}{}", std::env::consts::EXE_SUFFIX));
        let status = Command::new("rustc")
            .arg(&src)
            .arg("-o")
            .arg(&exe)
            .status()
            .expect("rustc should be available");
        assert!(status.success(), "failed to compile {name}");
        exe
    }

    #[test]
    fn smoke_test_accepts_a_working_binary_and_rejects_failures() {
        let dir = tempfile::tempdir().unwrap();
        let ok = compile_stub(
            dir.path(),
            r#"fn main() { if std::env::args().any(|a| a == "--version") { println!("rustfs 1.2.3"); } }"#,
            "ok",
        );
        assert_eq!(smoke_test(&ok).unwrap(), "rustfs 1.2.3");

        let fail = compile_stub(
            dir.path(),
            r#"fn main() { std::process::exit(2); }"#,
            "fail",
        );
        let error = smoke_test(&fail).unwrap_err();
        assert!(error.to_string().contains("exited"));

        let missing = dir.path().join("missing-binary");
        assert!(smoke_test(&missing).is_err());
    }

    #[test]
    fn stage_update_does_not_leave_a_failed_binary_behind() {
        let dir = tempfile::tempdir().unwrap();
        let archive = dir.path().join("release.zip");
        write_zip(&archive, &[("rustfs", b"not-an-executable")]);
        let staged = dir.path().join("rustfs.staged");

        assert!(stage_update(&archive, &staged, true).is_err());
        assert!(!staged.exists());
    }

    #[test]
    fn offline_install_then_launch_uses_the_committed_binary() {
        crate::state::terminate_rustfs_process();

        let dir = tempfile::tempdir().unwrap();
        let stub = compile_stub(
            dir.path(),
            r#"
                fn main() {
                    if std::env::args().any(|a| a == "--version") {
                        println!("rustfs 1.0.0-rc.4");
                        return;
                    }
                    std::thread::sleep(std::time::Duration::from_secs(20));
                }
            "#,
            "rustfs-stub",
        );
        let archive = dir.path().join("release.zip");
        let binary_bytes = std::fs::read(&stub).unwrap();
        write_zip(&archive, &[("rustfs", binary_bytes.as_slice())]);

        let platform = Platform::from_target("macos", "aarch64").unwrap();
        let managed = dir.path().join("managed");
        std::fs::create_dir_all(&managed).unwrap();
        let staged = managed.join(format!("{}.staged", platform.binary_name()));
        let digest = hash_file(&archive).unwrap();

        let reported = stage_update(&archive, &staged, true).unwrap();
        assert!(reported.contains("1.0.0-rc.4"));

        let message = commit_update(
            platform,
            "v1.0.0-rc.4",
            "rustfs-macos-aarch64-v1.0.0-rc.4.zip",
            &digest,
            &staged,
            &managed,
            true,
        )
        .unwrap();
        assert!(message.contains("1.0.0-rc.4"));
        assert!(message.contains("stopped"));

        let (installed, record) = installed_binary_from(&managed, Some("1.0.0-rc.3"))
            .expect("install should be selected");
        assert_eq!(record.version, "1.0.0-rc.4");
        assert_eq!(
            effective_version_from(Some(&managed), Some("1.0.0-rc.3")).version,
            "1.0.0-rc.4"
        );
        assert!(version::prefer_installed(
            &record.version,
            Some("1.0.0-rc.3")
        ));

        let data_dir = dir.path().join("data");
        std::fs::create_dir(&data_dir).unwrap();
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);

        launch(crate::config::RustFsConfig {
            binary_path: Some(installed.to_string_lossy().into_owned()),
            data_path: data_dir.to_string_lossy().into_owned(),
            port: Some(port),
            console_enable: false,
            host: Some("127.0.0.1".into()),
            ..crate::config::RustFsConfig::default()
        })
        .expect("installed stub should launch");
        assert!(crate::state::is_rustfs_process_running());
        crate::state::terminate_rustfs_process();
        assert!(!crate::state::is_rustfs_process_running());
    }

    #[test]
    fn progress_events_advertise_a_stable_event_tag() {
        let started = serde_json::to_value(Stage::Started {
            asset: "rustfs-windows-x86_64-v1.0.0.zip".into(),
            content_length: Some(10),
        })
        .unwrap();
        assert_eq!(started["event"], "started");
        assert_eq!(started["asset"], "rustfs-windows-x86_64-v1.0.0.zip");

        let progress = serde_json::to_string(&Stage::Progress {
            downloaded: 4,
            content_length: Some(10),
        })
        .unwrap();
        assert!(progress.contains("\"event\":\"progress\""));
        assert!(progress.contains("contentLength") || progress.contains("content_length"));
    }

    #[test]
    fn real_latest_json_fixture_evaluates_windows_and_macos() {
        const FIXTURE: &str = r#"{
          "version": "1.0.0-rc.4",
          "tag": "1.0.0-rc.4",
          "release_date": "2026-08-28T14:24:08Z",
          "release_type": "prerelease",
          "download_url": "https://github.com/rustfs/rustfs/releases/tag/1.0.0-rc.4"
        }"#;
        let feed = parse_latest_feed(FIXTURE).unwrap();
        let current = EffectiveVersion {
            version: "1.0.0-rc.3".to_string(),
            managed: false,
        };

        let windows = evaluate_update(
            Platform::from_target("windows", "aarch64").unwrap(),
            &current,
            &feed,
            false,
            Some(MANIFEST),
        );
        assert!(windows.available);
        assert_eq!(
            windows.asset_name.as_deref(),
            Some("rustfs-windows-x86_64-v1.0.0-rc.4.zip")
        );

        let apple = evaluate_update(
            Platform::from_target("macos", "aarch64").unwrap(),
            &current,
            &feed,
            false,
            Some(MANIFEST),
        );
        assert!(apple.available);
    }
}
