use crate::types::DEFAULT_HOST;

/// Host shown on the API/Console cards. Wildcards become a loopback address
/// the user's browser can actually open.
pub fn display_host(host: Option<&str>) -> String {
    match host.map(str::trim).filter(|value| !value.is_empty()) {
        Some("0.0.0.0") | Some("*") | None => DEFAULT_HOST.to_string(),
        Some("::") | Some("[::]") => "[::1]".to_string(),
        Some(host) if host.contains(':') && !host.starts_with('[') => format!("[{host}]"),
        Some(host) => host.to_string(),
    }
}

pub fn rustfs_source_label(managed: bool) -> &'static str {
    if managed {
        "installed"
    } else {
        "bundled"
    }
}

pub fn rustfs_idle_message(notes: Option<&str>) -> String {
    notes
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("RustFS is up to date.")
        .to_string()
}

/// Percent for the update progress bar. Accepts both the camelCase fields
/// emitted by serde and the snake_case names the older listener used.
pub fn progress_percent(downloaded: f64, content_length: Option<f64>) -> Option<u32> {
    let total = content_length.filter(|total| *total > 0.0)?;
    if !downloaded.is_finite() || downloaded < 0.0 {
        return None;
    }
    Some(((downloaded / total) * 100.0).min(99.0) as u32)
}

#[cfg(test)]
mod tests {
    use super::{display_host, progress_percent, rustfs_idle_message, rustfs_source_label};

    #[test]
    fn display_host_rewrites_wildcards_and_wraps_ipv6() {
        assert_eq!(display_host(None), "127.0.0.1");
        assert_eq!(display_host(Some("")), "127.0.0.1");
        assert_eq!(display_host(Some("0.0.0.0")), "127.0.0.1");
        assert_eq!(display_host(Some("*")), "127.0.0.1");
        assert_eq!(display_host(Some("::")), "[::1]");
        assert_eq!(display_host(Some("[::]")), "[::1]");
        assert_eq!(display_host(Some("2001:db8::1")), "[2001:db8::1]");
        assert_eq!(display_host(Some("[::1]")), "[::1]");
        assert_eq!(display_host(Some("192.168.1.9")), "192.168.1.9");
    }

    #[test]
    fn rustfs_labels_describe_the_binary_in_use() {
        assert_eq!(rustfs_source_label(true), "installed");
        assert_eq!(rustfs_source_label(false), "bundled");
        assert_eq!(
            rustfs_idle_message(Some("RustFS 1.0.0-rc.4 has no macOS (Intel) build.")),
            "RustFS 1.0.0-rc.4 has no macOS (Intel) build."
        );
        assert_eq!(rustfs_idle_message(Some("  ")), "RustFS is up to date.");
        assert_eq!(rustfs_idle_message(None), "RustFS is up to date.");
    }

    #[test]
    fn progress_percent_caps_below_one_hundred_until_finished() {
        assert_eq!(progress_percent(0.0, Some(100.0)), Some(0));
        assert_eq!(progress_percent(50.0, Some(100.0)), Some(50));
        assert_eq!(progress_percent(100.0, Some(100.0)), Some(99));
        assert_eq!(progress_percent(10.0, Some(0.0)), None);
        assert_eq!(progress_percent(10.0, None), None);
        assert_eq!(progress_percent(f64::NAN, Some(100.0)), None);
    }
}
