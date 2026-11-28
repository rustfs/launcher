use crate::i18n::{ActionLabel, Strings};
use crate::types::{
    DEFAULT_ACCESS_KEY, DEFAULT_API_PORT, DEFAULT_CONSOLE_PORT, DEFAULT_SECRET_KEY,
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

/// A saved empty host is the previous loopback fallback. A fresh install sets
/// `0.0.0.0` explicitly, so this does not change that default.
pub fn normalize_saved_host(host: Option<String>) -> Option<String> {
    match host {
        Some(value) if !value.trim().is_empty() => Some(value),
        _ => Some(BROWSER_LOOPBACK.to_string()),
    }
}

fn bind_host_label(host: Option<&str>) -> String {
    let host = host
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(BROWSER_LOOPBACK);
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
        .unwrap_or(BROWSER_LOOPBACK);
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

pub fn reachability_note(host: Option<&str>, t: &Strings) -> &'static str {
    if endpoint_is_on_lan(host) {
        t.reachability_lan
    } else {
        t.reachability_local
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
    #[allow(dead_code)]
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

/// Open the console once after a launch that succeeded with the console on.
/// Status polls and settings edits must not call this.
pub fn should_open_console_after_launch(success: bool, console_enabled: bool) -> bool {
    success && console_enabled
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrimaryAction {
    pub label: ActionLabel,
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
            ActionLabel::Launching
        } else if can_stop {
            ActionLabel::Stop
        } else {
            ActionLabel::Launch
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

#[allow(dead_code)]
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

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FirstScreenState {
    pub phase: ScreenPhase,
    pub show_path: bool,
    pub launch_visible: bool,
    pub launch_disabled: bool,
    pub launch_label: ActionLabel,
    pub stop_visible: bool,
    pub stop_disabled: bool,
    /// Stop is the only filled action on the post-start screen.
    pub stop_is_only_primary: bool,
    pub show_network_on_first_screen: bool,
    pub show_credentials_on_first_screen: bool,
    pub show_listening_address: bool,
    pub show_current_credentials: bool,
    pub show_secondary_actions: bool,
    pub show_open_console: bool,
    pub show_logs_toggle: bool,
    pub show_change_admin: bool,
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
    launch_failed: bool,
) -> FirstScreenState {
    let action = primary_action(can_stop, busy, data_path_empty);
    let post_start = can_stop;
    // View logs stays available after a failure, and while the panel is open,
    // so Escape still has a button to return to. It stays off a fresh screen.
    let show_logs_toggle = post_start || logs_requested || launch_failed;
    let show_change_admin = post_start || launch_failed;
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
            ActionLabel::Launch
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
        show_secondary_actions: show_logs_toggle || show_change_admin,
        show_open_console: post_start,
        show_logs_toggle,
        show_change_admin,
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

pub fn rustfs_idle_message(notes: Option<&str>, up_to_date: &str) -> String {
    notes
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(up_to_date)
        .to_string()
}

/// A status poll is stale once launch, stop, or process exit moves the
/// generation forward. Applying the old result would unlock the form while
/// RustFS is still starting, or mark it running after it has already exited.
pub fn status_poll_is_current(captured: u64, generation: u64) -> bool {
    captured == generation
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
///
/// `raw` is matched in English (backend errors and the launcher's own failure
/// notes are English); reasons and fixes the launcher writes itself come from `t`.
/// A sentence taken from a backend error is shown as received.
pub fn operation_failure(kind: OperationKind, raw: &str, t: &Strings) -> OperationFailure {
    let message = peel_operation_message(raw);
    if message.is_empty() {
        return OperationFailure {
            reason: default_reason(kind, t),
            fix: default_fix(kind, t),
        };
    }
    if let Some(failure) = match_known_failure(kind, &message, t) {
        return failure;
    }
    if let Some((reason, fix)) = split_instruction(&message) {
        return OperationFailure { reason, fix };
    }
    OperationFailure {
        reason: ensure_sentence(&message),
        fix: default_fix(kind, t),
    }
}

fn default_reason(kind: OperationKind, t: &Strings) -> String {
    match kind {
        OperationKind::Launch => t.reason_launch_default,
        OperationKind::Stop => t.reason_stop_default,
        OperationKind::Upgrade => t.reason_upgrade_default,
        OperationKind::Check => t.reason_check_default,
    }
    .to_string()
}

fn default_fix(kind: OperationKind, t: &Strings) -> String {
    match kind {
        OperationKind::Launch => t.fix_launch_default,
        OperationKind::Stop => t.fix_stop_default,
        OperationKind::Upgrade => t.fix_upgrade_default,
        OperationKind::Check => t.fix_check_default,
    }
    .to_string()
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

fn match_known_failure(
    kind: OperationKind,
    message: &str,
    t: &Strings,
) -> Option<OperationFailure> {
    let lower = message.to_lowercase();
    let failure = |reason: String, fix: &str| OperationFailure {
        reason,
        fix: fix.to_string(),
    };

    if lower.contains("data path is required") {
        return Some(failure(
            t.data_path_required.into(),
            t.fix_choose_data_folder,
        ));
    }
    if lower.contains("data path does not exist") {
        return Some(failure(first_sentence(message), t.fix_create_folder));
    }
    if lower.contains("not a directory") {
        return Some(failure(first_sentence(message), t.fix_folder_not_file));
    }
    if lower.contains("invalid data path") {
        return Some(failure(first_sentence(message), t.fix_pick_local_folder));
    }
    if lower.contains("log directory") {
        return Some(failure(first_sentence(message), t.fix_writable_parent));
    }
    if lower.contains("already in use") {
        return Some(failure(first_sentence(message), t.fix_port_in_use));
    }
    if lower.contains("must be different") {
        return Some(failure(t.port_conflict.into(), t.fix_different_ports));
    }
    if lower.contains("must be a number between 1 and 65535") {
        return Some(failure(first_sentence(message), t.fix_port_number));
    }
    if lower.contains("invalid bind host") || lower.contains("invalid host") {
        return Some(failure(first_sentence(message), t.fix_bind_host));
    }
    if lower.contains("rustfsadmin") || (lower.contains("credential") && lower.contains("default"))
    {
        return Some(failure(first_sentence(message), t.fix_public_credentials));
    }
    if lower.contains("already running") {
        return Some(failure(
            t.reason_already_running.into(),
            t.fix_already_running,
        ));
    }
    if lower.contains("binary not found") {
        return Some(failure(ensure_sentence(message), t.fix_binary_not_found));
    }
    if lower.contains("failed to execute") {
        return Some(failure(ensure_sentence(message), t.fix_binary_exec));
    }
    if lower.contains("exit code") || lower.contains("exited with code") {
        // A RustFS update smoke-test (`--version`) can also exit non-zero. That
        // check does not use the data folder or the service port.
        let fix = match kind {
            OperationKind::Launch => t.fix_launch_exit,
            OperationKind::Upgrade | OperationKind::Check => t.fix_upgrade_default,
            OperationKind::Stop => {
                return Some(failure(ensure_sentence(message), &default_fix(kind, t)))
            }
        };
        return Some(failure(ensure_sentence(message), fix));
    }
    if lower.contains("checksum mismatch") {
        return Some(failure(ensure_sentence(message), t.fix_retry_rustfs_update));
    }
    if lower.contains("network request failed") || lower.contains("timed out") {
        let fix = if matches!(kind, OperationKind::Launch | OperationKind::Stop) {
            default_fix(kind, t)
        } else {
            t.fix_network_update.to_string()
        };
        return Some(OperationFailure {
            reason: ensure_sentence(message),
            fix,
        });
    }
    if lower.contains("no update is available") {
        return Some(failure(t.reason_no_update.into(), t.fix_no_update));
    }
    if lower.contains("asset not found") || lower.contains("release asset") {
        return Some(failure(ensure_sentence(message), t.fix_asset_missing));
    }
    if lower.contains("not runnable")
        || lower.contains("archive did not contain")
        || lower.contains("could not open the archive")
        || lower.contains("could not read the archive")
    {
        return Some(failure(ensure_sentence(message), t.fix_retry_rustfs_update));
    }
    if lower.contains("not running in tauri") {
        return Some(failure(
            t.reason_not_connected.into(),
            t.fix_open_from_shell,
        ));
    }
    if lower.contains("launch command failed")
        || lower.contains("parsing failed")
        || lower.contains("failed to parse")
        || lower.contains("could not read")
    {
        return Some(failure(
            t.reason_unreadable_result.into(),
            &default_fix(kind, t),
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
        first_screen_state, listening_address, logs_open_by_default, normalize_saved_host,
        operation_failure, primary_action, progress_percent, reachability_note,
        rustfs_idle_message, settings_control_class, should_open_console_after_launch,
        status_poll_is_current, ControlWeight, OperationKind, ScreenPhase,
    };
    use crate::i18n::{ActionLabel, Lang, EN};
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
    fn rustfs_idle_message_prefers_backend_notes() {
        assert_eq!(
            rustfs_idle_message(
                Some("RustFS 1.0.0-rc.4 has no macOS (Intel) build."),
                EN.rustfs_up_to_date
            ),
            "RustFS 1.0.0-rc.4 has no macOS (Intel) build."
        );
        assert_eq!(
            rustfs_idle_message(Some("  "), EN.rustfs_up_to_date),
            "RustFS is up to date."
        );
        assert_eq!(
            rustfs_idle_message(None, EN.rustfs_up_to_date),
            "RustFS is up to date."
        );
    }

    #[test]
    fn a_second_click_cannot_stop_a_launch_that_has_not_finished() {
        let launching = primary_action(false, true, false);
        assert_eq!(launching.label, ActionLabel::Launching);
        assert!(launching.disabled);
        assert!(launching.lock_form);
        assert!(!launching.stops_service);

        let stopping = primary_action(true, true, false);
        assert_eq!(stopping.label, ActionLabel::Stop);
        assert!(stopping.disabled);
        assert!(!stopping.stops_service);

        let running = primary_action(true, false, false);
        assert_eq!(running.label, ActionLabel::Stop);
        assert!(!running.disabled);
        assert!(running.stops_service);

        let idle = primary_action(false, false, true);
        assert_eq!(idle.label, ActionLabel::Launch);
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
        let state = first_screen_state(false, false, true, logs_open_by_default(), false);
        assert_eq!(state.phase, ScreenPhase::PreStart);
        assert!(state.show_path);
        assert!(state.launch_visible);
        assert!(state.launch_disabled);
        assert_eq!(state.launch_label, ActionLabel::Launch);
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
        let state = first_screen_state(false, false, false, false, false);
        assert!(state.launch_visible);
        assert!(!state.launch_disabled);
        assert!(!state.logs_visible);
        assert!(!state.show_logs_toggle);
        assert!(!state.show_change_admin);

        let launching = first_screen_state(false, true, false, false, false);
        assert_eq!(launching.phase, ScreenPhase::PreStart);
        assert_eq!(launching.launch_label, ActionLabel::Launching);
        assert!(launching.launch_disabled);
        assert!(launching.show_path);
        assert!(!launching.logs_visible);
    }

    #[test]
    fn post_start_replaces_launch_with_stop_and_shows_the_endpoint() {
        let state = first_screen_state(true, false, false, false, false);
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

        let stopping = first_screen_state(true, true, false, false, false);
        assert!(stopping.stop_visible);
        assert!(stopping.stop_disabled);
        assert!(!stopping.launch_visible);
    }

    /// The shell is a client-side Leptos view. Both screens stay on the
    /// two-state model, and none of the sources that view is built from
    /// still contain the desktop production warning.
    #[test]
    fn neither_screen_renders_a_production_warning_banner() {
        let pre = first_screen_state(false, false, true, logs_open_by_default(), false);
        let post = first_screen_state(true, false, false, false, false);
        assert_eq!(pre.phase, ScreenPhase::PreStart);
        assert_eq!(post.phase, ScreenPhase::PostStart);
        assert!(pre.launch_visible && !pre.stop_visible);
        assert!(post.stop_visible && !post.launch_visible);

        let sources = [
            ("src/app.rs", include_str!("app.rs")),
            ("src/i18n.rs", include_str!("i18n.rs")),
            ("src/main.rs", include_str!("main.rs")),
            ("src/types.rs", include_str!("types.rs")),
            ("src/logs.css", include_str!("logs.css")),
            (
                "src/components/config_form.rs",
                include_str!("components/config_form.rs"),
            ),
            (
                "src/components/log_viewer.rs",
                include_str!("components/log_viewer.rs"),
            ),
            (
                "src/components/toast.rs",
                include_str!("components/toast.rs"),
            ),
            ("src/components/mod.rs", include_str!("components/mod.rs")),
            ("styles.css", include_str!("../styles.css")),
            ("index.html", include_str!("../index.html")),
        ];

        const MARKERS: &[&str] = &[
            "tips-banner",
            "banner_text",
            "banner_close",
            "show_tips",
            "not recommended for production",
            "不建议在生产使用",
            "recommandée en production",
            "本番環境での使用を推奨しません",
            "Dismiss notice",
            "关闭提示",
            "Fermer l'avertissement",
        ];

        for (name, source) in sources {
            for marker in MARKERS {
                assert!(
                    !source.contains(marker),
                    "{name} still contains the production warning banner ({marker})"
                );
            }
        }
    }

    #[test]
    fn view_logs_is_the_only_way_the_panel_opens() {
        assert!(!logs_open_by_default());
        assert!(!first_screen_state(true, false, false, false, false).logs_visible);
        assert!(first_screen_state(true, false, false, true, false).logs_visible);
        let failure = operation_failure(OperationKind::Launch, "See App Logs for details.", &EN);
        assert!(!failure.reason.to_lowercase().contains("see app logs"));
        assert!(!first_screen_state(false, false, false, false, false).logs_visible);

        let failed = first_screen_state(false, false, false, false, true);
        assert!(!failed.logs_visible);
        assert!(failed.show_logs_toggle);
        assert!(failed.show_change_admin);
        assert!(!failed.show_open_console);
        assert!(first_screen_state(false, false, false, true, false).show_logs_toggle);
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
        assert!(reachability_note(Some("0.0.0.0"), &EN).contains("network"));

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
        assert_eq!(
            endpoint.console_url,
            "http://127.0.0.1:9001/rustfs/console/"
        );
        assert_ne!(endpoint.console_url, endpoint.listening_address);
        assert!(should_open_console_after_launch(true, true));
        assert!(!should_open_console_after_launch(true, false));
        assert!(!should_open_console_after_launch(false, true));
        assert_eq!(EN.open_in_browser, "Open in browser");
        assert_eq!(credential_in_use(None, "rustfsadmin"), "rustfsadmin");
        assert_eq!(
            credential_in_use(Some("  custom  "), "rustfsadmin"),
            "custom"
        );
        assert_eq!(normalize_saved_host(None).as_deref(), Some("127.0.0.1"));
        assert_eq!(
            normalize_saved_host(Some("  ".into())).as_deref(),
            Some("127.0.0.1")
        );
        assert_eq!(
            normalize_saved_host(Some("0.0.0.0".into())).as_deref(),
            Some("0.0.0.0")
        );
        assert!(!endpoint_is_on_lan(None));
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
        let failure = operation_failure(kind, raw, &EN);
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
            &EN,
        );
        assert!(missing.reason.contains("D:\\RustFS\\data"));
        assert!(missing.fix.to_lowercase().contains("create this folder"));
        rejects_logs_only_copy(OperationKind::Launch, &missing.reason);

        let port = operation_failure(
            OperationKind::Launch,
            "Port 9000 is already in use. Choose a different port, or stop the program that is listening on it",
            &EN,
        );
        assert!(port.reason.contains("9000"));
        assert!(port.fix.to_lowercase().contains("different port"));

        let required = operation_failure(OperationKind::Launch, "Data path is required", &EN);
        assert!(required.reason.contains("Data path is required"));
        assert!(required.fix.to_lowercase().contains("browse"));
    }

    #[test]
    fn upgrade_failures_keep_the_installed_build_and_name_the_fix() {
        let network = operation_failure(
            OperationKind::Upgrade,
            "RustFS update failed: Network request failed: timed out",
            &EN,
        );
        assert!(network.reason.to_lowercase().contains("timed out"));
        assert!(network.fix.to_lowercase().contains("network"));
        assert!(network.fix.to_lowercase().contains("unchanged"));

        let checksum = operation_failure(
            OperationKind::Upgrade,
            "Checksum mismatch for rustfs.zip: expected aaa, got bbb",
            &EN,
        );
        assert!(checksum.reason.contains("aaa"));
        assert!(checksum.reason.contains("bbb"));
        assert!(checksum.fix.to_lowercase().contains("again"));
        assert!(checksum.fix.to_lowercase().contains("unchanged"));

        let exit = operation_failure(OperationKind::Launch, "RustFS exited with code: 1", &EN);
        assert!(exit.reason.contains("1"));
        assert!(exit.fix.to_lowercase().contains("logs folder"));
        rejects_logs_only_copy(OperationKind::Launch, "RustFS exited with code: 1");

        let smoke = operation_failure(
            OperationKind::Upgrade,
            "RustFS update failed: Downloaded RustFS binary exited with exit code: 1 when asked for its version",
            &EN,
        );
        assert!(smoke.reason.to_lowercase().contains("exit code"));
        assert!(smoke.fix.to_lowercase().contains("unchanged"));
        assert!(!smoke.fix.to_lowercase().contains("data folder"));
        assert!(!smoke.fix.to_lowercase().contains("port is free"));
    }

    #[test]
    fn failure_copy_follows_the_language_but_backend_text_is_kept() {
        let zh = Lang::Zh.strings();
        let missing = operation_failure(
            OperationKind::Launch,
            "Data path does not exist: D:\\RustFS\\data. Create this folder, then launch again",
            zh,
        );
        assert!(missing.reason.contains("D:\\RustFS\\data"));
        assert_eq!(missing.fix, zh.fix_create_folder);

        let required = operation_failure(OperationKind::Launch, "Data path is required", zh);
        assert_eq!(required.reason, zh.data_path_required);

        for lang in Lang::ALL {
            let t = lang.strings();
            let failure = operation_failure(OperationKind::Upgrade, "See App Logs for details.", t);
            assert_eq!(failure.reason, t.reason_upgrade_default);
            assert_eq!(failure.fix, t.fix_upgrade_default);
        }
    }
}
