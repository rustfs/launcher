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

pub(crate) fn tcp_online(host: &str, port: u16) -> bool {
    resolve_socket_addrs(host, port)
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
}
