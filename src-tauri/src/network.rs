use std::io;
use std::net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
use std::time::Duration;

const CONNECT_TIMEOUT: Duration = Duration::from_millis(1000);

pub(crate) fn normalize_host(host: &str) -> &str {
    let trimmed = host.trim();
    trimmed
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .unwrap_or(trimmed)
}

pub(crate) fn is_loopback_host(host: &str) -> bool {
    let host = normalize_host(host);
    if host.eq_ignore_ascii_case("localhost") {
        return true;
    }
    host.parse::<std::net::IpAddr>()
        .is_ok_and(|ip| ip.is_loopback())
}

fn illegal_bind_char(ch: char) -> bool {
    ch.is_control()
        || ch.is_whitespace()
        || matches!(
            ch,
            '/' | '\\' | '@' | '?' | '#' | '%' | '"' | '\'' | '<' | '>' | '|' | '&' | ';' | '*'
        )
}

/// Accepts a host that can be placed in `--address` and in an `http://` URL.
///
/// Returns the rejected host (or `"empty host"`) so this file can be compiled
/// on its own by `rustc --test`. The launcher maps that string to `Error::InvalidHost`.
pub(crate) fn validate_bind_host(host: &str) -> Result<&str, String> {
    let host = normalize_host(host);
    if host.is_empty() {
        return Err("empty host".to_string());
    }
    if host.len() > 253 || host.chars().any(illegal_bind_char) {
        return Err(host.to_string());
    }
    Ok(host)
}

pub(crate) fn format_bind_address(host: &str, port: u16) -> String {
    let host = normalize_host(host);
    if host.contains(':') {
        format!("[{host}]:{port}")
    } else {
        format!("{host}:{port}")
    }
}

fn resolve_socket_addrs(host: &str, port: u16) -> io::Result<Vec<SocketAddr>> {
    (normalize_host(host), port)
        .to_socket_addrs()
        .map(Iterator::collect)
}

fn any_tcp_online(addrs: impl IntoIterator<Item = SocketAddr>, timeout: Duration) -> bool {
    addrs
        .into_iter()
        .any(|addr| TcpStream::connect_timeout(&addr, timeout).is_ok())
}

/// Address used to see whether a bind address is accepting connections.
/// `0.0.0.0` and `::` listen on every interface, but connecting to them fails.
pub(crate) fn probe_host(host: &str) -> &str {
    match normalize_host(host) {
        "0.0.0.0" | "*" => "127.0.0.1",
        "::" => "::1",
        host => host,
    }
}

pub(crate) fn tcp_online(host: &str, port: u16) -> bool {
    resolve_socket_addrs(probe_host(host), port)
        .map(|addrs| any_tcp_online(addrs, CONNECT_TIMEOUT))
        .unwrap_or(false)
}

pub(crate) fn is_port_available(host: &str, port: u16) -> bool {
    resolve_socket_addrs(host, port)
        .and_then(|addrs| TcpListener::bind(addrs.as_slice()))
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_and_formats_ipv6_hosts() {
        assert_eq!(normalize_host("::1"), "::1");
        assert_eq!(normalize_host("[::1]"), "::1");
        assert_eq!(normalize_host(" 127.0.0.1 "), "127.0.0.1");
        assert_eq!(format_bind_address("127.0.0.1", 9000), "127.0.0.1:9000");
        assert_eq!(format_bind_address("::1", 9000), "[::1]:9000");
        assert_eq!(format_bind_address("[::1]", 9001), "[::1]:9001");
    }

    #[test]
    fn tcp_probe_tries_every_resolved_address() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let online = listener.local_addr().unwrap();
        let closed_listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let offline = closed_listener.local_addr().unwrap();
        drop(closed_listener);

        assert!(any_tcp_online(
            [offline, online],
            Duration::from_millis(100)
        ));
    }

    #[test]
    fn bracketed_ipv6_uses_the_same_socket_as_unbracketed_ipv6() {
        if !is_port_available("[::1]", 0) {
            return;
        }
        let Ok(listener) = TcpListener::bind(("::1", 0)) else {
            return;
        };
        let port = listener.local_addr().unwrap().port();

        assert!(tcp_online("::1", port));
        assert!(tcp_online("[::1]", port));
        assert!(!is_port_available("[::1]", port));
    }

    #[test]
    fn invalid_hosts_are_offline_and_unavailable() {
        assert!(!tcp_online("[invalid", 9));
        assert!(!is_port_available("[invalid", 9));
    }

    #[test]
    fn unspecified_binds_are_probed_through_loopback() {
        assert_eq!(probe_host("0.0.0.0"), "127.0.0.1");
        assert_eq!(probe_host(" * "), "127.0.0.1");
        assert_eq!(probe_host("::"), "::1");
        assert_eq!(probe_host("[::]"), "::1");
        assert_eq!(probe_host("192.168.1.9"), "192.168.1.9");
        assert_eq!(probe_host("[::1]"), "::1");

        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        assert!(
            tcp_online("0.0.0.0", port),
            "a wildcard bind must be checked via loopback, not by connecting to 0.0.0.0"
        );
    }

    #[test]
    fn loopback_hosts_are_recognized_after_normalization() {
        assert!(is_loopback_host("127.0.0.1"));
        assert!(is_loopback_host("  localhost "));
        assert!(is_loopback_host("[::1]"));
        assert!(is_loopback_host("127.1.2.3"));
        assert!(!is_loopback_host("0.0.0.0"));
        assert!(!is_loopback_host("192.168.1.9"));
        assert!(!is_loopback_host("*"));
    }

    #[test]
    fn bind_host_rejects_url_and_argument_characters() {
        assert_eq!(validate_bind_host(" 127.0.0.1 ").unwrap(), "127.0.0.1");
        assert_eq!(validate_bind_host("[::1]").unwrap(), "::1");
        assert_eq!(
            validate_bind_host("127.0.0.1/admin").unwrap_err(),
            "127.0.0.1/admin"
        );
        assert_eq!(
            validate_bind_host("user@127.0.0.1").unwrap_err(),
            "user@127.0.0.1"
        );
        assert_eq!(
            validate_bind_host("127.0.0.1\n--help").unwrap_err(),
            "127.0.0.1\n--help"
        );
        assert_eq!(validate_bind_host("*").unwrap_err(), "*");
        assert_eq!(validate_bind_host("   ").unwrap_err(), "empty host");
        assert!(validate_bind_host(&"a".repeat(254)).is_err());
    }
}
