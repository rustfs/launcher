use crate::types::{DEFAULT_ACCESS_KEY, DEFAULT_HOST, DEFAULT_SECRET_KEY};
use std::net::IpAddr;

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

pub fn is_loopback_bind_host(host: Option<&str>) -> bool {
    let host = host
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_HOST);
    let host = host
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .unwrap_or(host);
    if host.eq_ignore_ascii_case("localhost") {
        return true;
    }
    host.parse::<IpAddr>().is_ok_and(|ip| ip.is_loopback())
}

fn credential_is_published_default(value: Option<&str>, default: &str) -> bool {
    match value.map(str::trim).filter(|item| !item.is_empty()) {
        None => true,
        Some(value) => value == default,
    }
}

/// True when a non-loopback bind would start RustFS with the published
/// `rustfsadmin` / `rustfsadmin` pair. Empty fields count: RustFS fills those
/// in itself.
pub fn refuses_public_default_credentials(
    host: Option<&str>,
    access_key: Option<&str>,
    secret_key: Option<&str>,
) -> bool {
    !is_loopback_bind_host(host)
        && (credential_is_published_default(access_key, DEFAULT_ACCESS_KEY)
            || credential_is_published_default(secret_key, DEFAULT_SECRET_KEY))
}

pub fn public_default_credentials_message(host: &str) -> String {
    format!(
        "Refusing to expose the default rustfsadmin credentials on {host}. Bind to 127.0.0.1, or set a unique access key and secret key. Empty credentials also fall back to rustfsadmin"
    )
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
    use super::{
        display_host, progress_percent, public_default_credentials_message,
        refuses_public_default_credentials, rustfs_idle_message, rustfs_source_label,
    };

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

    #[test]
    fn public_default_credentials_are_refused_off_loopback() {
        assert!(!refuses_public_default_credentials(
            Some("127.0.0.1"),
            Some("rustfsadmin"),
            Some("rustfsadmin"),
        ));
        assert!(!refuses_public_default_credentials(
            Some("localhost"),
            None,
            None,
        ));
        assert!(refuses_public_default_credentials(
            Some("0.0.0.0"),
            Some("rustfsadmin"),
            Some("sk-unique"),
        ));
        assert!(refuses_public_default_credentials(
            Some("192.168.1.9"),
            None,
            Some("rustfsadmin"),
        ));
        assert!(!refuses_public_default_credentials(
            Some("0.0.0.0"),
            Some("ak-unique"),
            Some("sk-unique"),
        ));
        let message = public_default_credentials_message("0.0.0.0");
        assert!(message.contains("0.0.0.0"));
        assert!(message.contains("unique access key"));
    }
}
