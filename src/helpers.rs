use crate::types::{
    DEFAULT_ACCESS_KEY, DEFAULT_API_PORT, DEFAULT_CONSOLE_PORT, DEFAULT_HOST, DEFAULT_SECRET_KEY,
};
use std::net::IpAddr;

/// Loopback address a browser can open. `0.0.0.0` and `::` are bind addresses,
/// not destinations.
const BROWSER_LOOPBACK: &str = "127.0.0.1";

/// Web console path. The console port root answers with an S3 error document.
pub const CONSOLE_UI_PATH: &str = "/rustfs/console/";

/// Host a browser can open. Wildcards become loopback; the listening address
/// the user copies keeps the real bind (see `listening_address`).
pub fn display_host(host: Option<&str>) -> String {
    match host.map(str::trim).filter(|value| !value.is_empty()) {
        Some("0.0.0.0") | Some("*") | None => BROWSER_LOOPBACK.to_string(),
        Some("::") | Some("[::]") => "[::1]".to_string(),
        Some(host) if host.contains(':') && !host.starts_with('[') => format!("[{host}]"),
        Some(host) => host.to_string(),
    }
}

fn bind_host_label(host: Option<&str>) -> String {
    let host = host
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_HOST);
    if host.contains(':') && !host.starts_with('[') {
        format!("[{host}]")
    } else {
        host.to_string()
    }
}

/// Address RustFS is bound to, including a wildcard such as `0.0.0.0`.
pub fn listening_address(host: Option<&str>, port: u16) -> String {
    format!("http://{}:{port}", bind_host_label(host))
}

/// Credential RustFS is actually using. Empty fields fall back to the
/// published `rustfsadmin` pair; this launcher does not invent a unique key.
pub fn credential_in_use(value: Option<&str>, default: &str) -> String {
    value
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .unwrap_or(default)
        .to_string()
}

/// True when other devices on the network can open the bind address.
pub fn endpoint_is_on_lan(host: Option<&str>) -> bool {
    let host = host
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_HOST);
    let bare = host
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .unwrap_or(host);
    if bare.eq_ignore_ascii_case("localhost") {
        return false;
    }
    match bare.parse::<IpAddr>() {
        Ok(ip) => !ip.is_loopback(),
        Err(_) => true,
    }
}

pub fn reachability_note(host: Option<&str>) -> &'static str {
    if endpoint_is_on_lan(host) {
        "Other devices on your network can reach this endpoint. The keys below are the ones RustFS is using."
    } else {
        "Only this computer can reach this endpoint. The keys below are the ones RustFS is using."
    }
}

/// What the post-start screen copies. Captured at launch so a later edit in
/// Settings or Change administrator does not pretend the running process
/// already switched.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveEndpoint {
    pub listening_address: String,
    pub access_key: String,
    pub secret_key: String,
    pub console_enabled: bool,
    pub api_url: String,
    pub console_url: String,
    pub on_lan: bool,
}

pub fn capture_endpoint(
    host: Option<&str>,
    port: Option<u16>,
    console_port: Option<u16>,
    console_enable: bool,
    access_key: Option<&str>,
    secret_key: Option<&str>,
) -> ActiveEndpoint {
    let port = port.unwrap_or(DEFAULT_API_PORT);
    let console_port = console_port.unwrap_or(DEFAULT_CONSOLE_PORT);
    let browser_host = display_host(host);
    ActiveEndpoint {
        listening_address: listening_address(host, port),
        access_key: credential_in_use(access_key, DEFAULT_ACCESS_KEY),
        secret_key: credential_in_use(secret_key, DEFAULT_SECRET_KEY),
        console_enabled: console_enable,
        api_url: format!("http://{browser_host}:{port}"),
        console_url: format!("http://{browser_host}:{console_port}{CONSOLE_UI_PATH}"),
        on_lan: endpoint_is_on_lan(host),
    }
}

pub fn rustfs_source_label(managed: bool) -> &'static str {
    if managed {
        "installed"
    } else {
        "bundled"
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrimaryAction {
    pub label: &'static str,
    pub disabled: bool,
    pub lock_form: bool,
    pub stops_service: bool,
}

/// The launch button shares one control with Stop. While a launch or stop is
/// still in flight the form stays locked, but the click must not start the
/// other operation: a second click used to send Stop before any process
/// existed, toast "RustFS stopped", and then let the original launch succeed.
pub fn primary_action(can_stop: bool, busy: bool, data_path_empty: bool) -> PrimaryAction {
    PrimaryAction {
        label: if busy && !can_stop {
            "Launching…"
        } else if can_stop {
            "Stop RustFS"
        } else {
            "Launch RustFS"
        },
        disabled: busy || data_path_empty,
        lock_form: can_stop || busy,
        stops_service: can_stop && !busy,
    }
}

/// Logs stay closed until the user presses View logs. Startup, update toasts,
/// and failures must not pass `true`.
pub fn logs_open_by_default() -> bool {
    false
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenPhase {
    PreStart,
    PostStart,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlWeight {
    Primary,
    Destructive,
    Outline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FirstScreenState {
    pub phase: ScreenPhase,
    pub show_path: bool,
    pub launch_visible: bool,
    pub launch_disabled: bool,
    pub launch_label: &'static str,
    pub stop_visible: bool,
    pub stop_disabled: bool,
    /// Stop is the only filled action on the post-start screen.
    pub stop_is_only_primary: bool,
    pub show_network_on_first_screen: bool,
    pub show_credentials_on_first_screen: bool,
    pub show_listening_address: bool,
    pub show_current_credentials: bool,
    pub show_secondary_actions: bool,
    pub settings_is_primary: bool,
    pub settings_is_overlay: bool,
    pub logs_visible: bool,
}

/// Two screens, one primary action each. Host, port, console, and credentials
/// never sit on the pre-start screen. After start, the path field leaves and
/// the listening address plus the keys in use take its place.
pub fn first_screen_state(
    can_stop: bool,
    busy: bool,
    data_path_empty: bool,
    logs_requested: bool,
) -> FirstScreenState {
    let action = primary_action(can_stop, busy, data_path_empty);
    let post_start = can_stop;
    FirstScreenState {
        phase: if post_start {
            ScreenPhase::PostStart
        } else {
            ScreenPhase::PreStart
        },
        show_path: !post_start,
        launch_visible: !post_start,
        launch_disabled: !post_start && action.disabled,
        launch_label: if post_start {
            "Launch RustFS"
        } else {
            action.label
        },
        stop_visible: post_start,
        stop_disabled: post_start && busy,
        stop_is_only_primary: post_start,
        show_network_on_first_screen: false,
        show_credentials_on_first_screen: false,
        show_listening_address: post_start,
        show_current_credentials: post_start,
        show_secondary_actions: post_start,
        settings_is_primary: false,
        settings_is_overlay: true,
        logs_visible: logs_requested,
    }
}

pub fn control_class(weight: ControlWeight) -> &'static str {
    match weight {
        ControlWeight::Primary => "button button-primary",
        ControlWeight::Destructive => "button button-destructive",
        ControlWeight::Outline => "button button-outline",
    }
}

/// The header control is a 32×32 icon. It is never a second primary button.
pub fn settings_control_class() -> &'static str {
    "icon-button"
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
/// A status poll is stale once launch, stop, or process exit moves the
/// generation forward. Applying the old result would unlock the form while
/// RustFS is still starting, or mark it running after it has already exited.
pub fn status_poll_is_current(captured: u64, generation: u64) -> bool {
    captured == generation
}

pub fn progress_percent(downloaded: f64, content_length: Option<f64>) -> Option<u32> {
    let total = content_length.filter(|total| *total > 0.0)?;
    if !downloaded.is_finite() || downloaded < 0.0 {
        return None;
    }
    Some(((downloaded / total) * 100.0).min(99.0) as u32)
}

/// Which action failed. Copy only — this does not change how the action runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationKind {
    Launch,
    Stop,
    Upgrade,
    Check,
}

/// Reason and fix shown on the first screen. Displaying this must not open logs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationFailure {
    pub reason: String,
    pub fix: String,
}

/// Turns a start, stop, upgrade, or update-check error into first-screen copy.
/// The log panel stays closed; "See App Logs for details" is never the only text.
pub fn operation_failure(kind: OperationKind, raw: &str) -> OperationFailure {
    let message = peel_operation_message(raw);
    if message.is_empty() {
        return OperationFailure {
            reason: default_reason(kind),
            fix: default_fix(kind),
        };
    }
    if let Some(failure) = match_known_failure(kind, &message) {
        return failure;
    }
    if let Some((reason, fix)) = split_instruction(&message) {
        return OperationFailure { reason, fix };
    }
    OperationFailure {
        reason: ensure_sentence(&message),
        fix: default_fix(kind),
    }
}

fn default_reason(kind: OperationKind) -> String {
    match kind {
        OperationKind::Launch => "RustFS did not start.".to_string(),
        OperationKind::Stop => "RustFS did not stop.".to_string(),
        OperationKind::Upgrade => "The update did not finish.".to_string(),
        OperationKind::Check => "The update check did not finish.".to_string(),
    }
}

fn default_fix(kind: OperationKind) -> String {
    match kind {
        OperationKind::Launch => {
            "Check the data path and ports, then launch again.".to_string()
        }
        OperationKind::Stop => {
            "Press Stop RustFS again. If the service is still running, quit the launcher from the tray menu.".to_string()
        }
        OperationKind::Upgrade => {
            "Try the update again. A failed update leaves the installed build unchanged.".to_string()
        }
        OperationKind::Check => {
            "Check the network connection, then press Check for Updates again.".to_string()
        }
    }
}

fn peel_operation_message(raw: &str) -> String {
    let flat = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    let kept = split_sentences(&flat)
        .into_iter()
        .filter(|sentence| !is_logs_only_hint(sentence))
        .collect::<Vec<_>>()
        .join(" ");
    let mut text = kept.trim().to_string();
    let prefixes = [
        "launcher update check failed:",
        "rustfs update check failed:",
        "launcher update failed:",
        "rustfs update failed:",
        "launch failed:",
        "failed to stop:",
        "update failed:",
        "error:",
    ];
    loop {
        let lower = text.to_lowercase();
        let Some(prefix) = prefixes.iter().find(|prefix| lower.starts_with(*prefix)) else {
            break;
        };
        text = text[prefix.len()..].trim().to_string();
    }
    text.trim()
        .trim_matches(|ch| ch == '.' || ch == ' ')
        .trim()
        .to_string()
}

fn is_logs_only_hint(sentence: &str) -> bool {
    let normalized = sentence.trim().trim_end_matches('.').trim().to_lowercase();
    normalized == "see app logs for details" || normalized == "see app logs"
}

fn split_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut start = 0;
    let bytes = text.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'.' {
            let next = index + 1;
            let boundary = next == bytes.len()
                || bytes[next].is_ascii_whitespace()
                || (next + 1 < bytes.len() && bytes[next] == b'.');
            if boundary {
                let sentence = text[start..=index].trim();
                if !sentence.is_empty() {
                    sentences.push(sentence.to_string());
                }
                start = next;
                while start < bytes.len() && bytes[start].is_ascii_whitespace() {
                    start += 1;
                }
                index = start;
                continue;
            }
        }
        index += 1;
    }
    let tail = text[start..].trim();
    if !tail.is_empty() {
        sentences.push(tail.to_string());
    }
    sentences
}

fn match_known_failure(kind: OperationKind, message: &str) -> Option<OperationFailure> {
    let lower = message.to_lowercase();
    let failure = |reason: String, fix: &str| OperationFailure {
        reason,
        fix: fix.to_string(),
    };

    if lower.contains("data path is required") {
        return Some(failure(
            "Data path is required.".into(),
            "Choose a folder with Browse, or type the path, then launch again.",
        ));
    }
    if lower.contains("data path does not exist") {
        return Some(failure(
            first_sentence(message),
            "Create this folder, then launch again. The launcher does not create it for you.",
        ));
    }
    if lower.contains("not a directory") {
        return Some(failure(
            first_sentence(message),
            "Choose a folder, not a file, then launch again.",
        ));
    }
    if lower.contains("invalid data path") {
        return Some(failure(
            first_sentence(message),
            "Pick a local folder with Browse, then launch again.",
        ));
    }
    if lower.contains("log directory") {
        return Some(failure(
            first_sentence(message),
            "Choose a data folder whose parent directory is writable, then launch again.",
        ));
    }
    if lower.contains("already in use") {
        return Some(failure(
            first_sentence(message),
            "Choose a different port, or stop the program that is listening on it, then launch again.",
        ));
    }
    if lower.contains("must be different") {
        return Some(failure(
            "API port and console port must be different.".into(),
            "Set a different API port and console port, then launch again.",
        ));
    }
    if lower.contains("must be a number between 1 and 65535") {
        return Some(failure(
            first_sentence(message),
            "Enter a whole number from 1 to 65535, then launch again.",
        ));
    }
    if lower.contains("invalid bind host") || lower.contains("invalid host") {
        return Some(failure(
            first_sentence(message),
            "Use 127.0.0.1, localhost, or an IP address such as 0.0.0.0, then launch again.",
        ));
    }
    if lower.contains("rustfsadmin") || (lower.contains("credential") && lower.contains("default"))
    {
        return Some(failure(
            first_sentence(message),
            "Bind to 127.0.0.1, or set a unique access key and secret key. Empty credentials also fall back to rustfsadmin.",
        ));
    }
    if lower.contains("already running") {
        return Some(failure(
            "RustFS is already running.".into(),
            "Press Stop RustFS if this launcher started it. Otherwise change the port, or stop the other program, then launch again.",
        ));
    }
    if lower.contains("binary not found") {
        return Some(failure(
            ensure_sentence(message),
            "Reinstall RustFS Launcher, or install the RustFS build from Check for Updates, then launch again.",
        ));
    }
    if lower.contains("failed to execute") {
        return Some(failure(
            ensure_sentence(message),
            "Reinstall RustFS Launcher so the bundled binary is restored, then launch again.",
        ));
    }
    if lower.contains("exit code") || lower.contains("exited with code") {
        // A RustFS update smoke-test (`--version`) can also exit non-zero. That
        // check does not use the data folder or the service port.
        let fix = match kind {
            OperationKind::Launch => "Confirm the data folder exists and the port is free, then launch again. RustFS writes more detail in the logs folder next to the data directory.",
            OperationKind::Upgrade | OperationKind::Check => "Try the update again. A failed update leaves the installed build unchanged.",
            OperationKind::Stop => {
                return Some(failure(ensure_sentence(message), &default_fix(kind)))
            }
        };
        return Some(failure(ensure_sentence(message), fix));
    }
    if lower.contains("checksum mismatch") {
        return Some(failure(
            ensure_sentence(message),
            "Run the RustFS update again. The current install was left unchanged.",
        ));
    }
    if lower.contains("network request failed") || lower.contains("timed out") {
        let fix = if matches!(kind, OperationKind::Launch | OperationKind::Stop) {
            default_fix(kind)
        } else {
            "Check the network connection, then try again. A failed update leaves the installed build unchanged.".to_string()
        };
        return Some(OperationFailure {
            reason: ensure_sentence(message),
            fix,
        });
    }
    if lower.contains("no update is available") {
        return Some(failure(
            "No update is available.".into(),
            "This launcher is already current. Launch RustFS with the installed build.",
        ));
    }
    if lower.contains("asset not found") || lower.contains("release asset") {
        return Some(failure(
            ensure_sentence(message),
            "Try again later. The installed build is unchanged.",
        ));
    }
    if lower.contains("not runnable")
        || lower.contains("archive did not contain")
        || lower.contains("could not open the archive")
        || lower.contains("could not read the archive")
    {
        return Some(failure(
            ensure_sentence(message),
            "Run the RustFS update again. The current install was left unchanged.",
        ));
    }
    if lower.contains("not running in tauri") {
        return Some(failure(
            "The window is not connected to the desktop shell.".into(),
            "Open RustFS Launcher from the Start menu or the Applications folder, then launch again.",
        ));
    }
    if lower.contains("launch command failed")
        || lower.contains("parsing failed")
        || lower.contains("failed to parse")
        || lower.contains("could not read")
    {
        return Some(failure(
            "The launcher could not read the result of the action.".into(),
            &default_fix(kind),
        ));
    }
    None
}

fn split_instruction(message: &str) -> Option<(String, String)> {
    let (reason, fix) = message.split_once(". ")?;
    let reason = ensure_sentence(reason);
    let fix = fix.trim();
    if fix.is_empty() || is_logs_only_hint(fix) {
        return None;
    }
    let first = fix
        .split_whitespace()
        .next()?
        .trim_matches(',')
        .to_lowercase();
    const VERBS: &[&str] = &[
        "bind",
        "change",
        "check",
        "choose",
        "confirm",
        "create",
        "enter",
        "open",
        "pick",
        "press",
        "quit",
        "reinstall",
        "run",
        "set",
        "stop",
        "try",
        "use",
    ];
    if VERBS.contains(&first.as_str()) {
        Some((reason, ensure_sentence(fix)))
    } else {
        None
    }
}

fn first_sentence(message: &str) -> String {
    split_sentences(message)
        .into_iter()
        .next()
        .map(|sentence| ensure_sentence(&sentence))
        .unwrap_or_else(|| ensure_sentence(message))
}

fn ensure_sentence(text: &str) -> String {
    let trimmed = text.trim().trim_end_matches('.').trim();
    if trimmed.is_empty() {
        return String::new();
    }
    format!("{trimmed}.")
}

#[cfg(test)]
mod tests {
    use super::{
        capture_endpoint, control_class, credential_in_use, display_host, endpoint_is_on_lan,
        first_screen_state, listening_address, logs_open_by_default, operation_failure,
        primary_action, progress_percent, reachability_note, rustfs_idle_message,
        rustfs_source_label, settings_control_class, status_poll_is_current, ControlWeight,
        OperationKind, ScreenPhase,
    };
    use crate::types::RustFsConfig;

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
    fn a_second_click_cannot_stop_a_launch_that_has_not_finished() {
        let launching = primary_action(false, true, false);
        assert_eq!(launching.label, "Launching…");
        assert!(launching.disabled);
        assert!(launching.lock_form);
        assert!(!launching.stops_service);

        let stopping = primary_action(true, true, false);
        assert_eq!(stopping.label, "Stop RustFS");
        assert!(stopping.disabled);
        assert!(!stopping.stops_service);

        let running = primary_action(true, false, false);
        assert_eq!(running.label, "Stop RustFS");
        assert!(!running.disabled);
        assert!(running.stops_service);

        let idle = primary_action(false, false, true);
        assert_eq!(idle.label, "Launch RustFS");
        assert!(idle.disabled);
        assert!(!idle.lock_form);
    }

    #[test]
    fn a_newer_runtime_operation_discards_an_older_status_poll() {
        assert!(status_poll_is_current(4, 4));
        assert!(!status_poll_is_current(4, 5));
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
    fn fresh_screen_shows_only_path_and_a_disabled_launch() {
        let state = first_screen_state(false, false, true, logs_open_by_default());
        assert_eq!(state.phase, ScreenPhase::PreStart);
        assert!(state.show_path);
        assert!(state.launch_visible);
        assert!(state.launch_disabled);
        assert_eq!(state.launch_label, "Launch RustFS");
        assert!(!state.stop_visible);
        assert!(!state.show_network_on_first_screen);
        assert!(!state.show_credentials_on_first_screen);
        assert!(!state.show_listening_address);
        assert!(!state.show_current_credentials);
        assert!(!state.show_secondary_actions);
        assert!(!state.logs_visible);
        assert!(!state.settings_is_primary);
        assert!(state.settings_is_overlay);
        assert_eq!(
            control_class(ControlWeight::Primary),
            "button button-primary"
        );
        assert!(!settings_control_class().contains("button-primary"));
    }

    #[test]
    fn choosing_a_path_enables_launch_without_opening_logs() {
        let state = first_screen_state(false, false, false, false);
        assert!(state.launch_visible);
        assert!(!state.launch_disabled);
        assert!(!state.logs_visible);

        let launching = first_screen_state(false, true, false, false);
        assert_eq!(launching.phase, ScreenPhase::PreStart);
        assert_eq!(launching.launch_label, "Launching…");
        assert!(launching.launch_disabled);
        assert!(launching.show_path);
        assert!(!launching.logs_visible);
    }

    #[test]
    fn post_start_replaces_launch_with_stop_and_shows_the_endpoint() {
        let state = first_screen_state(true, false, false, false);
        assert_eq!(state.phase, ScreenPhase::PostStart);
        assert!(!state.show_path);
        assert!(!state.launch_visible);
        assert!(state.stop_visible);
        assert!(!state.stop_disabled);
        assert!(state.stop_is_only_primary);
        assert!(state.show_listening_address);
        assert!(state.show_current_credentials);
        assert!(state.show_secondary_actions);
        assert!(!state.show_network_on_first_screen);
        assert!(!state.show_credentials_on_first_screen);
        assert!(!state.logs_visible);
        assert_eq!(
            control_class(ControlWeight::Destructive),
            "button button-destructive"
        );
        assert_eq!(
            control_class(ControlWeight::Outline),
            "button button-outline"
        );

        let stopping = first_screen_state(true, true, false, false);
        assert!(stopping.stop_visible);
        assert!(stopping.stop_disabled);
        assert!(!stopping.launch_visible);
    }

    #[test]
    fn view_logs_is_the_only_way_the_panel_opens() {
        assert!(!logs_open_by_default());
        assert!(!first_screen_state(true, false, false, false).logs_visible);
        assert!(first_screen_state(true, false, false, true).logs_visible);
        let failure = operation_failure(OperationKind::Launch, "See App Logs for details.");
        assert!(!failure.reason.to_lowercase().contains("see app logs"));
        assert!(!first_screen_state(false, false, false, false).logs_visible);
    }

    #[test]
    fn listening_address_keeps_the_wildcard_and_published_keys() {
        assert_eq!(
            listening_address(Some("0.0.0.0"), 9000),
            "http://0.0.0.0:9000"
        );
        assert_eq!(display_host(Some("0.0.0.0")), "127.0.0.1");
        assert!(endpoint_is_on_lan(Some("0.0.0.0")));
        assert!(!endpoint_is_on_lan(Some("127.0.0.1")));
        assert!(reachability_note(Some("0.0.0.0")).contains("network"));

        let defaults = RustFsConfig::default();
        let endpoint = capture_endpoint(
            defaults.host.as_deref(),
            defaults.port,
            defaults.console_port,
            defaults.console_enable,
            defaults.access_key.as_deref(),
            defaults.secret_key.as_deref(),
        );
        assert_eq!(endpoint.listening_address, "http://0.0.0.0:9000");
        assert_eq!(endpoint.access_key, "rustfsadmin");
        assert_eq!(endpoint.secret_key, "rustfsadmin");
        assert!(endpoint.console_enabled);
        assert!(endpoint.on_lan);
        assert_eq!(endpoint.api_url, "http://127.0.0.1:9000");
        assert!(endpoint.console_url.contains(":9001/rustfs/console/"));
        assert_eq!(credential_in_use(None, "rustfsadmin"), "rustfsadmin");
        assert_eq!(
            credential_in_use(Some("  custom  "), "rustfsadmin"),
            "custom"
        );
    }

    #[test]
    fn controls_stay_square_and_a_disabled_launch_is_forty_percent() {
        let css = include_str!("../styles.css");
        let logs = include_str!("logs.css");
        assert!(css.contains("opacity: 0.4"));
        assert!(css.contains(".button-primary"));
        assert!(css.contains(".button-destructive"));
        assert!(css.contains(".icon-button"));
        assert!(css.contains("inset-inline-end"));
        assert!(
            !css.contains("999px"),
            "pill radii are not part of this screen"
        );
        assert!(!logs.contains("999px"));
        assert!(!css.contains("linear-gradient(135deg, var(--brand-blue)"));
    }

    fn rejects_logs_only_copy(kind: OperationKind, raw: &str) {
        let failure = operation_failure(kind, raw);
        let combined = format!("{} {}", failure.reason, failure.fix).to_lowercase();
        assert!(!failure.reason.trim().is_empty(), "{raw}");
        assert!(!failure.fix.trim().is_empty(), "{raw}");
        assert_ne!(failure.reason.to_lowercase(), "see app logs for details.");
        assert_ne!(failure.fix.to_lowercase(), "see app logs for details.");
        assert!(
            !combined.contains("see app logs"),
            "logs must not be the only feedback for {raw}: {combined}"
        );
    }

    #[test]
    fn a_logs_only_failure_still_states_a_reason_and_a_fix() {
        for kind in [
            OperationKind::Launch,
            OperationKind::Stop,
            OperationKind::Upgrade,
            OperationKind::Check,
        ] {
            rejects_logs_only_copy(kind, "See App Logs for details.");
            rejects_logs_only_copy(kind, "Launcher update failed. See App Logs for details.");
        }
    }

    #[test]
    fn start_failures_name_the_path_or_port_and_how_to_fix_them() {
        let missing = operation_failure(
            OperationKind::Launch,
            "Data path does not exist: D:\\RustFS\\data. Create this folder, then launch again",
        );
        assert!(missing.reason.contains("D:\\RustFS\\data"));
        assert!(missing.fix.to_lowercase().contains("create this folder"));
        rejects_logs_only_copy(OperationKind::Launch, &missing.reason);

        let port = operation_failure(
            OperationKind::Launch,
            "Port 9000 is already in use. Choose a different port, or stop the program that is listening on it",
        );
        assert!(port.reason.contains("9000"));
        assert!(port.fix.to_lowercase().contains("different port"));

        let required = operation_failure(OperationKind::Launch, "Data path is required");
        assert!(required.reason.contains("Data path is required"));
        assert!(required.fix.to_lowercase().contains("browse"));
    }

    #[test]
    fn upgrade_failures_keep_the_installed_build_and_name_the_fix() {
        let network = operation_failure(
            OperationKind::Upgrade,
            "RustFS update failed: Network request failed: timed out",
        );
        assert!(network.reason.to_lowercase().contains("timed out"));
        assert!(network.fix.to_lowercase().contains("network"));
        assert!(network.fix.to_lowercase().contains("unchanged"));

        let checksum = operation_failure(
            OperationKind::Upgrade,
            "Checksum mismatch for rustfs.zip: expected aaa, got bbb",
        );
        assert!(checksum.reason.contains("aaa"));
        assert!(checksum.reason.contains("bbb"));
        assert!(checksum.fix.to_lowercase().contains("again"));
        assert!(checksum.fix.to_lowercase().contains("unchanged"));

        let exit = operation_failure(OperationKind::Launch, "RustFS exited with code: 1");
        assert!(exit.reason.contains("1"));
        assert!(exit.fix.to_lowercase().contains("logs folder"));
        rejects_logs_only_copy(OperationKind::Launch, "RustFS exited with code: 1");

        let smoke = operation_failure(
            OperationKind::Upgrade,
            "RustFS update failed: Downloaded RustFS binary exited with exit code: 1 when asked for its version",
        );
        assert!(smoke.reason.to_lowercase().contains("exit code"));
        assert!(smoke.fix.to_lowercase().contains("unchanged"));
        assert!(!smoke.fix.to_lowercase().contains("data folder"));
        assert!(!smoke.fix.to_lowercase().contains("port is free"));
    }
}
