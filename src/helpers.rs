use crate::i18n::{ActionLabel, Strings};
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

pub fn rustfs_idle_message(notes: Option<&str>, up_to_date: &str) -> String {
    notes
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(up_to_date)
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
        display_host, operation_failure, primary_action, progress_percent,
        refuses_public_default_credentials, rustfs_idle_message, status_poll_is_current,
        OperationKind,
    };
    use crate::i18n::{ActionLabel, Lang, EN};

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
                "up to date"
            ),
            "RustFS 1.0.0-rc.4 has no macOS (Intel) build."
        );
        assert_eq!(rustfs_idle_message(Some("  "), "up to date"), "up to date");
        assert_eq!(rustfs_idle_message(None, "up to date"), "up to date");
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
