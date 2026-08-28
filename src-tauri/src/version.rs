//! Helpers for comparing RustFS release tags.
//!
//! Upstream publishes tags such as `1.0.0-rc.4` while release assets embed a
//! `v`-prefixed variant (`rustfs-macos-aarch64-v1.0.0-rc.4.zip`), so both forms
//! have to round-trip cleanly.

/// Drops a leading `v` so `v1.2.3` and `1.2.3` compare equal.
pub fn normalize(tag: &str) -> &str {
    let trimmed = tag.trim();
    trimmed.strip_prefix('v').unwrap_or(trimmed)
}

/// Adds the leading `v` used by upstream asset file names.
pub fn asset_version(tag: &str) -> String {
    format!("v{}", normalize(tag))
}

fn parse(tag: &str) -> Option<semver::Version> {
    semver::Version::parse(normalize(tag)).ok()
}

/// True when `candidate` is a strictly newer release than `current`.
///
/// Falls back to a plain inequality check when either tag is not valid semver,
/// so an unparseable upstream tag still surfaces as an available update.
pub fn is_newer(candidate: &str, current: &str) -> bool {
    match (parse(candidate), parse(current)) {
        (Some(candidate), Some(current)) => candidate > current,
        _ => normalize(candidate) != normalize(current),
    }
}

/// Decides whether a user-installed RustFS binary should win over the one
/// bundled with the launcher. A launcher upgrade can ship a newer RustFS than
/// the locally installed copy, in which case the bundle takes precedence.
pub fn prefer_installed(installed: &str, bundled: Option<&str>) -> bool {
    let Some(bundled) = bundled.map(str::trim).filter(|value| !value.is_empty()) else {
        return true;
    };

    match (parse(installed), parse(bundled)) {
        (Some(installed), Some(bundled)) => installed >= bundled,
        // Without comparable versions, trust the explicit user action.
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::{asset_version, is_newer, normalize, prefer_installed};

    #[test]
    fn normalizes_v_prefixed_tags() {
        assert_eq!(normalize("v1.0.0-rc.4"), "1.0.0-rc.4");
        assert_eq!(normalize(" 1.0.0 "), "1.0.0");
        assert_eq!(asset_version("1.0.0-rc.4"), "v1.0.0-rc.4");
        assert_eq!(asset_version("v1.0.0-rc.4"), "v1.0.0-rc.4");
    }

    #[test]
    fn orders_prereleases_below_releases() {
        assert!(is_newer("1.0.0", "1.0.0-rc.4"));
        assert!(is_newer("1.0.0-rc.5", "1.0.0-rc.4"));
        assert!(!is_newer("1.0.0-rc.4", "1.0.0-rc.4"));
        assert!(!is_newer("v1.0.0-rc.4", "1.0.0-rc.4"));
        assert!(!is_newer("1.0.0-rc.3", "1.0.0-rc.4"));
    }

    #[test]
    fn treats_unparseable_tags_as_changed() {
        assert!(is_newer("nightly-2026-08-28", "1.0.0"));
        assert!(!is_newer("nightly", "nightly"));
    }

    #[test]
    fn bundled_binary_wins_when_it_is_newer() {
        assert!(prefer_installed("1.0.0-rc.4", Some("1.0.0-rc.3")));
        assert!(prefer_installed("1.0.0-rc.4", Some("1.0.0-rc.4")));
        assert!(!prefer_installed("1.0.0-rc.3", Some("1.0.0-rc.4")));
        assert!(prefer_installed("1.0.0-rc.3", None));
        assert!(prefer_installed("1.0.0-rc.3", Some("unknown")));
    }
}
