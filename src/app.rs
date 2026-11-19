use crate::components::config_form::ConfigForm;
use crate::components::log_viewer::LogViewer;
use crate::components::toast::{Toast, ToastMessage, ToastType};
use crate::helpers::{
    capture_endpoint, control_class, display_host, first_screen_state, logs_open_by_default,
    operation_failure, progress_percent, reachability_note, rustfs_idle_message,
    rustfs_source_label, settings_control_class, status_poll_is_current, ActiveEndpoint,
    ControlWeight, OperationFailure, OperationKind,
};
use crate::types::{
    AppVersionInfo, CommandResponse, LogEntry, LogType, RuntimeStatus, RustFsConfig,
    RustFsUpdateInfo, UpdateInfo, APP_LOG_CAPACITY, DEFAULT_API_PORT, DEFAULT_CONSOLE_PORT,
    DEFAULT_HOST, RUSTFS_LOG_CAPACITY,
};
use leptos::ev::SubmitEvent;
use leptos::prelude::*;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

// Import CSS styles
const LOGS_CSS: &str = include_str!("logs.css");

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], js_name = invoke, catch)]
    async fn tauri_invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

fn js_error_message(err: JsValue) -> String {
    if let Some(message) = err.as_string() {
        return message;
    }

    if let Ok(message) = js_sys::Reflect::get(&err, &"message".into()) {
        if let Some(message) = message.as_string() {
            if !message.is_empty() {
                return message;
            }
        }
    }

    format!("{err:?}")
}

// Helper function to check if we're in Tauri environment
fn is_tauri() -> bool {
    web_sys::window()
        .and_then(|w| js_sys::Reflect::get(&w, &"__TAURI__".into()).ok())
        .map(|v| !v.is_undefined())
        .unwrap_or(false)
}

fn set_js_field(object: &js_sys::Object, key: &str, value: impl Into<JsValue>) {
    let _ = js_sys::Reflect::set(object, &JsValue::from_str(key), &value.into());
}

fn empty_args() -> JsValue {
    js_sys::Object::new().into()
}

fn confirm_action(message: &str) -> bool {
    web_sys::window()
        .and_then(|window| window.confirm_with_message(message).ok())
        .unwrap_or(false)
}

fn js_f64(payload: &JsValue, keys: &[&str]) -> Option<f64> {
    keys.iter().find_map(|key| {
        js_sys::Reflect::get(payload, &JsValue::from_str(key))
            .ok()
            .and_then(|value| value.as_f64())
            .filter(|value| value.is_finite())
    })
}

fn apply_progress_payload(payload: &JsValue, set_progress: WriteSignal<Option<u32>>) {
    let event_name = js_sys::Reflect::get(payload, &"event".into())
        .ok()
        .and_then(|value| value.as_string());

    match event_name.as_deref() {
        Some("started") => set_progress.set(Some(0)),
        Some("progress") => {
            let downloaded = js_f64(payload, &["downloaded"]);
            if let Some(downloaded) = downloaded {
                let percent = progress_percent(
                    downloaded,
                    js_f64(payload, &["content_length", "contentLength"]),
                );
                if percent.is_some() {
                    set_progress.set(percent);
                }
            }
        }
        Some("verifying") => set_progress.set(Some(99)),
        Some("installing") | Some("finished") => set_progress.set(Some(100)),
        _ => {}
    }
}

fn default_access_key() -> Option<String> {
    Some(crate::types::DEFAULT_ACCESS_KEY.to_string())
}

fn default_secret_key() -> Option<String> {
    Some(crate::types::DEFAULT_SECRET_KEY.to_string())
}

#[derive(serde::Serialize, serde::Deserialize)]
struct PersistedConfig {
    data_path: String,
    port: Option<u16>,
    console_port: Option<u16>,
    host: Option<String>,
    console_enable: bool,
    // Saved with the rest of the form. Omitting them made every restart fall
    // back to the published rustfsadmin pair, even after the user changed it.
    #[serde(default = "default_access_key")]
    access_key: Option<String>,
    #[serde(default = "default_secret_key")]
    secret_key: Option<String>,
}

impl From<&RustFsConfig> for PersistedConfig {
    fn from(config: &RustFsConfig) -> Self {
        Self {
            data_path: config.data_path.clone(),
            port: config.port,
            console_port: config.console_port,
            host: config.host.clone(),
            console_enable: config.console_enable,
            access_key: config.access_key.clone(),
            secret_key: config.secret_key.clone(),
        }
    }
}

impl From<PersistedConfig> for RustFsConfig {
    fn from(config: PersistedConfig) -> Self {
        RustFsConfig {
            data_path: config.data_path,
            port: config.port,
            console_port: config.console_port,
            host: config.host,
            console_enable: config.console_enable,
            access_key: config.access_key,
            secret_key: config.secret_key,
        }
    }
}

fn load_config() -> RustFsConfig {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(Some(json)) = storage.get_item("rustfs_config") {
                if let Ok(config) = serde_json::from_str::<PersistedConfig>(&json) {
                    return config.into();
                }
            }
        }
    }
    RustFsConfig::default()
}

fn save_config(config: &RustFsConfig) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(json) = serde_json::to_string(&PersistedConfig::from(config)) {
                let _ = storage.set_item("rustfs_config", &json);
            }
        }
    }
}

fn describe_config(config: &RustFsConfig) -> String {
    format!(
        "data_path={}, host={:?}, port={:?}, console_enable={}, access_key_set={}, secret_key_set={}",
        config.data_path,
        config.host,
        config.port,
        config.console_enable,
        config.access_key.is_some(),
        config.secret_key.is_some()
    )
}

const WEBSITE_URL: &str = "https://rustfs.com";
const DOCS_URL: &str = "https://docs.rustfs.com";
const GITHUB_URL: &str = "https://github.com/rustfs/rustfs";
static NEXT_LOG_ID: AtomicU64 = AtomicU64::new(1);
static HEALTH_CHECK_STARTED: AtomicBool = AtomicBool::new(false);
static UPDATE_CHECK_STARTED: AtomicBool = AtomicBool::new(false);
static STATUS_GENERATION: AtomicU64 = AtomicU64::new(0);

fn focus_later(focus: impl Fn() + 'static) {
    request_animation_frame(move || {
        request_animation_frame(move || focus());
    });
}

fn current_status_generation() -> u64 {
    STATUS_GENERATION.load(Ordering::SeqCst)
}

fn invalidate_status_polls() {
    STATUS_GENERATION.fetch_add(1, Ordering::SeqCst);
}

fn next_log_id() -> u64 {
    NEXT_LOG_ID.fetch_add(1, Ordering::Relaxed)
}

fn to_log_entry(message: String) -> LogEntry {
    LogEntry {
        id: next_log_id(),
        message,
    }
}

fn push_log(writer: WriteSignal<VecDeque<LogEntry>>, msg: String, capacity: usize) {
    writer.update(|logs| {
        logs.push_back(to_log_entry(msg));
        if logs.len() > capacity {
            logs.pop_front();
        }
    });
}

fn sync_runtime_status(
    config: ReadSignal<RustFsConfig>,
    set_is_running: WriteSignal<bool>,
    set_can_stop: WriteSignal<bool>,
    set_service_status: WriteSignal<bool>,
) {
    let generation = current_status_generation();
    spawn_local(async move {
        if !is_tauri() {
            set_is_running.set(false);
            set_can_stop.set(false);
            set_service_status.set(false);
            return;
        }

        let current = config.get_untracked();
        let host = current.host.unwrap_or_else(|| DEFAULT_HOST.to_string());
        let port = current.port.unwrap_or(DEFAULT_API_PORT);

        let status_args = js_sys::Object::new();
        set_js_field(&status_args, "host", host);
        set_js_field(&status_args, "port", port);

        let Ok(value) = tauri_invoke("get_runtime_status", status_args.into()).await else {
            return;
        };
        let Ok(status) = serde_wasm_bindgen::from_value::<RuntimeStatus>(value) else {
            return;
        };
        if !status_poll_is_current(generation, current_status_generation()) {
            return;
        }

        set_service_status.set(status.service_online);
        set_can_stop.set(status.managed_running);
        set_is_running.set(status.managed_running || status.service_online);
    });
}

#[component]
pub fn App() -> impl IntoView {
    let (config, set_config) = signal(load_config());

    Effect::new(move |_| {
        save_config(&config.get());
    });

    let (toasts, set_toasts) = signal(Vec::<ToastMessage>::new());
    let (is_running, set_is_running) = signal(false);
    let (app_logs, set_app_logs) = signal(VecDeque::<LogEntry>::new());
    let (rustfs_logs, set_rustfs_logs) = signal(VecDeque::<LogEntry>::new());
    let (current_log_type, set_current_log_type) = signal(LogType::App);
    let (service_status, set_service_status) = signal(false);
    let (can_stop, set_can_stop) = signal(false);
    let (action_busy, set_action_busy) = signal(false);
    let (version_info, set_version_info) = signal(None::<AppVersionInfo>);
    let (update_info, set_update_info) = signal(None::<UpdateInfo>);
    let (rustfs_update_info, set_rustfs_update_info) = signal(None::<RustFsUpdateInfo>);
    let (is_checking_update, set_is_checking_update) = signal(false);
    let (is_installing_update, set_is_installing_update) = signal(false);
    let (is_installing_rustfs, set_is_installing_rustfs) = signal(false);
    let (update_progress, set_update_progress) = signal(None::<u32>);
    let (rustfs_update_progress, set_rustfs_update_progress) = signal(None::<u32>);
    // Logs stay closed until the user opens them. Startup, update checks, and
    // failures must not set this.
    let (logs_open, set_logs_open) = signal(logs_open_by_default());
    let (action_failure, set_action_failure) = signal(None::<OperationFailure>);
    let (settings_open, set_settings_open) = signal(false);
    let (admin_open, set_admin_open) = signal(false);
    let (upgrade_open, set_upgrade_open) = signal(false);
    let (session, set_session) = signal(None::<ActiveEndpoint>);
    let settings_button_ref = NodeRef::<leptos::html::Button>::new();
    let admin_button_ref = NodeRef::<leptos::html::Button>::new();
    let upgrade_button_ref = NodeRef::<leptos::html::Button>::new();
    let host_input_ref = NodeRef::<leptos::html::Input>::new();
    let access_key_input_ref = NodeRef::<leptos::html::Input>::new();
    let upgrade_panel_ref = NodeRef::<leptos::html::Div>::new();
    let logs_toggle_ref = NodeRef::<leptos::html::Button>::new();
    let logs_panel_ref = NodeRef::<leptos::html::Div>::new();
    let workspace_scroll_ref = NodeRef::<leptos::html::Div>::new();

    let remove_toast = Callback::new(move |id: u64| {
        set_toasts.update(|current| {
            current.retain(|t| t.id != id);
        });
    });

    let show_toast = move |message: String, toast_type: ToastType| {
        let id = next_log_id();
        set_toasts.update(|current| {
            current.push(ToastMessage {
                message,
                toast_type,
                id,
            });
        });

        let Some(window) = web_sys::window() else {
            return;
        };
        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
            Closure::once_into_js(move || {
                remove_toast.run(id);
            })
            .unchecked_ref(),
            3000,
        );
    };

    let report_failure = move |kind: OperationKind, raw: String| {
        let failure = operation_failure(kind, &raw);
        set_action_failure.set(Some(failure.clone()));
        show_toast(
            format!("{} {}", failure.reason, failure.fix),
            ToastType::Error,
        );
        // The alert lives with Data Path and Launch. An upgrade started from
        // the card further down the page would otherwise leave that alert
        // above the current scroll position.
        request_animation_frame(move || {
            request_animation_frame(move || {
                if let Some(scroller) = workspace_scroll_ref.get_untracked() {
                    scroller.set_scroll_top(0);
                }
            });
        });
    };

    let return_focus_to_logs_toggle = move || {
        request_animation_frame(move || {
            request_animation_frame(move || {
                if let Some(button) = logs_toggle_ref.get_untracked() {
                    let _ = button.focus();
                }
            });
        });
    };

    let toggle_logs = move |_| {
        let next = !logs_open.get_untracked();
        set_logs_open.set(next);
        request_animation_frame(move || {
            request_animation_frame(move || {
                if next {
                    if let Some(panel) = logs_panel_ref.get_untracked() {
                        let _ = panel.focus();
                    }
                } else if let Some(button) = logs_toggle_ref.get_untracked() {
                    let _ = button.focus();
                }
            });
        });
    };

    let focus_button = move |node: NodeRef<leptos::html::Button>| {
        request_animation_frame(move || {
            request_animation_frame(move || {
                if let Some(button) = node.get_untracked() {
                    let _ = button.focus();
                }
            });
        });
    };

    let close_settings = move || {
        set_settings_open.set(false);
        focus_button(settings_button_ref);
    };
    let close_admin = move || {
        set_admin_open.set(false);
        focus_button(admin_button_ref);
    };
    let close_upgrade = move || {
        set_upgrade_open.set(false);
        focus_button(upgrade_button_ref);
    };

    let close_logs = move |ev: web_sys::KeyboardEvent| {
        if ev.key() != "Escape" {
            return;
        }
        if settings_open.get_untracked() {
            ev.prevent_default();
            close_settings();
            return;
        }
        if admin_open.get_untracked() {
            ev.prevent_default();
            close_admin();
            return;
        }
        if upgrade_open.get_untracked() {
            ev.prevent_default();
            close_upgrade();
            return;
        }
        if !logs_open.get_untracked() {
            return;
        }
        ev.prevent_default();
        set_logs_open.set(false);
        return_focus_to_logs_toggle();
    };

    // Escape must work even when focus never lands on <main>, for example while
    // the log panel is still taking focus.
    {
        let on_escape = close_logs;
        if let Some(window) = web_sys::window() {
            let closure = Closure::wrap(Box::new(move |ev: web_sys::KeyboardEvent| {
                on_escape(ev);
            }) as Box<dyn FnMut(web_sys::KeyboardEvent)>);
            let _ = window
                .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref());
            closure.forget();
        }
    }

    Effect::new(move |_| {
        if settings_open.get() {
            focus_later(move || {
                if let Some(input) = host_input_ref.get_untracked() {
                    let _ = input.focus();
                }
            });
        }
    });
    Effect::new(move |_| {
        if admin_open.get() {
            focus_later(move || {
                if let Some(input) = access_key_input_ref.get_untracked() {
                    let _ = input.focus();
                }
            });
        }
    });
    Effect::new(move |_| {
        if upgrade_open.get() {
            focus_later(move || {
                if let Some(panel) = upgrade_panel_ref.get_untracked() {
                    let _ = panel.focus();
                }
            });
        }
    });
    Effect::new(move |_| {
        if can_stop.get() && session.get().is_none() {
            let current = config.get();
            set_session.set(Some(capture_endpoint(
                current.host.as_deref(),
                current.port,
                current.console_port,
                current.console_enable,
                current.access_key.as_deref(),
                current.secret_key.as_deref(),
            )));
        }
    });

    Effect::new(move |_| {
        if !is_tauri() || HEALTH_CHECK_STARTED.swap(true, Ordering::SeqCst) {
            return;
        }

        sync_runtime_status(config, set_is_running, set_can_stop, set_service_status);

        let Some(window) = web_sys::window() else {
            return;
        };

        let closure = Closure::wrap(Box::new(move || {
            sync_runtime_status(config, set_is_running, set_can_stop, set_service_status);
        }) as Box<dyn FnMut()>);

        let _ = window.set_interval_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            3000,
        );
        closure.forget();
    });

    let app_log_writer = set_app_logs;
    let rustfs_log_writer = set_rustfs_logs;

    spawn_local(async move {
        if !is_tauri() {
            push_log(
                app_log_writer,
                "[WARN] Not running in Tauri environment - logs disabled".to_string(),
                APP_LOG_CAPACITY,
            );
            return;
        }

        if let Ok(version_value) =
            tauri_invoke("get_app_version_info", js_sys::Object::new().into()).await
        {
            if let Ok(info) = serde_wasm_bindgen::from_value::<AppVersionInfo>(version_value) {
                set_version_info.set(Some(info));
            }
        }

        push_log(
            app_log_writer,
            "[DEBUG] Setting up real-time log listeners...".to_string(),
            APP_LOG_CAPACITY,
        );

        const APP_LOG_EVENT: &str = "app-log";
        const RUSTFS_LOG_EVENT: &str = "rustfs-log";
        const RUSTFS_EXIT_EVENT: &str = "rustfs-exit";

        fn create_log_listener(
            logs_signal: WriteSignal<VecDeque<LogEntry>>,
            max_logs: usize,
        ) -> Closure<dyn FnMut(JsValue)> {
            Closure::wrap(Box::new(move |event: JsValue| {
                if let Ok(payload) = js_sys::Reflect::get(&event, &"payload".into()) {
                    if let Some(log) = payload.as_string() {
                        push_log(logs_signal, log, max_logs);
                    }
                }
            }) as Box<dyn FnMut(JsValue)>)
        }

        let exit_listener = Closure::wrap(Box::new(move |event: JsValue| {
            if let Ok(payload) = js_sys::Reflect::get(&event, &"payload".into()) {
                if let Some(exit_code) = payload.as_string() {
                    set_is_running.set(false);
                    set_can_stop.set(false);
                    set_service_status.set(false);
                    set_session.set(None);
                    set_admin_open.set(false);
                    invalidate_status_polls();
                    sync_runtime_status(config, set_is_running, set_can_stop, set_service_status);
                    report_failure(
                        OperationKind::Launch,
                        format!("RustFS exited with code: {exit_code}"),
                    );

                    // Log it
                    push_log(
                        app_log_writer,
                        format!("[ERROR] RustFS process exited unexpectedly: {}", exit_code),
                        APP_LOG_CAPACITY,
                    );

                    // Switch to RustFS logs so user sees why
                    set_current_log_type.set(LogType::RustFS);
                }
            }
        }) as Box<dyn FnMut(JsValue)>);

        if let Some(window) = web_sys::window() {
            let app_listener = create_log_listener(app_log_writer, APP_LOG_CAPACITY);
            let rustfs_listener = create_log_listener(rustfs_log_writer, RUSTFS_LOG_CAPACITY);
            let update_listener = Closure::wrap(Box::new(move |event: JsValue| {
                let Ok(payload) = js_sys::Reflect::get(&event, &"payload".into()) else {
                    return;
                };
                apply_progress_payload(&payload, set_update_progress);
            }) as Box<dyn FnMut(JsValue)>);
            let rustfs_update_listener = Closure::wrap(Box::new(move |event: JsValue| {
                let Ok(payload) = js_sys::Reflect::get(&event, &"payload".into()) else {
                    return;
                };
                apply_progress_payload(&payload, set_rustfs_update_progress);
            }) as Box<dyn FnMut(JsValue)>);

            if let Ok(tauri) = js_sys::Reflect::get(&window, &"__TAURI__".into()) {
                if let Ok(event) = js_sys::Reflect::get(&tauri, &"event".into()) {
                    if let Ok(listen) = js_sys::Reflect::get(&event, &"listen".into()) {
                        let listen_fn = js_sys::Function::from(listen);

                        let _ = listen_fn.call2(
                            &event,
                            &APP_LOG_EVENT.into(),
                            app_listener.as_ref().unchecked_ref(),
                        );
                        let _ = listen_fn.call2(
                            &event,
                            &RUSTFS_LOG_EVENT.into(),
                            rustfs_listener.as_ref().unchecked_ref(),
                        );
                        let _ = listen_fn.call2(
                            &event,
                            &RUSTFS_EXIT_EVENT.into(),
                            exit_listener.as_ref().unchecked_ref(),
                        );
                        let _ = listen_fn.call2(
                            &event,
                            &"update-progress".into(),
                            update_listener.as_ref().unchecked_ref(),
                        );
                        let _ = listen_fn.call2(
                            &event,
                            &"rustfs-update-progress".into(),
                            rustfs_update_listener.as_ref().unchecked_ref(),
                        );
                    }
                }
            }

            app_listener.forget();
            rustfs_listener.forget();
            exit_listener.forget();
            update_listener.forget();
            rustfs_update_listener.forget();
        }

        if let Ok(app_logs_value) = tauri_invoke("get_app_logs", js_sys::Object::new().into()).await
        {
            if let Ok(logs_vec) = serde_wasm_bindgen::from_value::<Vec<String>>(app_logs_value) {
                app_log_writer.set(logs_vec.into_iter().map(to_log_entry).collect());
            }
        }

        if let Ok(rustfs_logs_value) =
            tauri_invoke("get_rustfs_logs", js_sys::Object::new().into()).await
        {
            if let Ok(logs_vec) = serde_wasm_bindgen::from_value::<Vec<String>>(rustfs_logs_value) {
                rustfs_log_writer.set(logs_vec.into_iter().map(to_log_entry).collect());
            }
        }
    });

    let settle_runtime = move || {
        invalidate_status_polls();
        sync_runtime_status(config, set_is_running, set_can_stop, set_service_status);
    };

    let launch_rustfs = move |ev: SubmitEvent| {
        ev.prevent_default();
        if action_busy.get_untracked() {
            return;
        }
        set_action_failure.set(None);
        invalidate_status_polls();
        set_action_busy.set(true);
        show_toast("Launching RustFS...".to_string(), ToastType::Info);

        let now = js_sys::Date::new_0().to_locale_time_string("en-US");
        push_log(
            set_app_logs,
            format!("[{}] Launch button clicked", now),
            APP_LOG_CAPACITY,
        );
        push_log(
            set_app_logs,
            format!(
                "[{}] Config summary: {}",
                now,
                describe_config(&config.get())
            ),
            APP_LOG_CAPACITY,
        );

        spawn_local(async move {
            // Check if we're in Tauri environment
            if !is_tauri() {
                report_failure(
                    OperationKind::Launch,
                    "Not running in Tauri environment".to_string(),
                );
                push_log(
                    set_app_logs,
                    "[ERROR] Not running in Tauri environment".to_string(),
                    APP_LOG_CAPACITY,
                );
                set_is_running.set(false);
                set_can_stop.set(false);
                set_action_busy.set(false);
                return;
            }

            let current_config = config.get_untracked();
            let launched = capture_endpoint(
                current_config.host.as_deref(),
                current_config.port,
                current_config.console_port,
                current_config.console_enable,
                current_config.access_key.as_deref(),
                current_config.secret_key.as_deref(),
            );

            leptos::logging::log!(
                "Starting RustFS with config: data_path={}, port={:?}, host={:?}",
                current_config.data_path,
                current_config.port,
                current_config.host
            );

            let now = js_sys::Date::new_0().to_locale_time_string("en-US");
            push_log(
                set_app_logs,
                format!("[{}] Calling tauri_invoke with command: launch_rustfs", now),
                APP_LOG_CAPACITY,
            );

            // Create args object with config parameter
            let args = js_sys::Object::new();
            let config_js = serde_wasm_bindgen::to_value(&current_config).unwrap_or(JsValue::NULL);
            set_js_field(&args, "config", config_js.clone());

            let validate_args = js_sys::Object::new();
            set_js_field(&validate_args, "config", config_js);
            if let Err(err) = tauri_invoke("validate_config", validate_args.into()).await {
                let message = js_error_message(err);
                report_failure(OperationKind::Launch, message.clone());
                push_log(
                    set_app_logs,
                    format!("[ERROR] Config validation failed: {message}"),
                    APP_LOG_CAPACITY,
                );
                set_is_running.set(false);
                set_can_stop.set(false);
                set_action_busy.set(false);
                settle_runtime();
                return;
            }

            match tauri_invoke("launch_rustfs", args.into()).await {
                Ok(result_value) => {
                    let now = js_sys::Date::new_0().to_locale_time_string("en-US");
                    push_log(
                        set_app_logs,
                        format!("[{}] Invoke result: {:?}", now, result_value),
                        APP_LOG_CAPACITY,
                    );

                    match serde_wasm_bindgen::from_value::<CommandResponse>(result_value) {
                        Ok(CommandResponse { success, message }) => {
                            let now = js_sys::Date::new_0().to_locale_time_string("en-US");
                            push_log(
                                set_app_logs,
                                format!("[{}] Result message: {}", now, message),
                                APP_LOG_CAPACITY,
                            );

                            if success {
                                set_is_running.set(true);
                                set_can_stop.set(true);
                                set_session.set(Some(launched));
                                set_action_busy.set(false);
                                set_action_failure.set(None);
                                show_toast(
                                    "RustFS launched successfully!".to_string(),
                                    ToastType::Success,
                                );
                                let now = js_sys::Date::new_0().to_locale_time_string("en-US");
                                push_log(
                                    set_app_logs,
                                    format!("[{}] Launch successful!", now),
                                    APP_LOG_CAPACITY,
                                );
                            } else {
                                report_failure(
                                    OperationKind::Launch,
                                    format!("Launch failed: {message}"),
                                );
                                let now = js_sys::Date::new_0().to_locale_time_string("en-US");
                                push_log(
                                    set_app_logs,
                                    format!("[{}] Launch result: {}", now, message),
                                    APP_LOG_CAPACITY,
                                );
                                set_is_running.set(false);
                                set_can_stop.set(false);
                                set_action_busy.set(false);
                            }
                            settle_runtime();
                        }
                        Err(_) => {
                            report_failure(
                                OperationKind::Launch,
                                "RustFS launch command failed".to_string(),
                            );
                            let now = js_sys::Date::new_0().to_locale_time_string("en-US");
                            push_log(
                                set_app_logs,
                                format!("[{}] Launch completed but response parsing failed", now),
                                APP_LOG_CAPACITY,
                            );
                            set_is_running.set(false);
                            set_can_stop.set(false);
                            set_action_busy.set(false);
                            settle_runtime();
                        }
                    }
                }
                Err(err) => {
                    let message = js_error_message(err);
                    report_failure(OperationKind::Launch, format!("Launch failed: {message}"));
                    push_log(
                        set_app_logs,
                        format!("[ERROR] Launch failed: {message}"),
                        APP_LOG_CAPACITY,
                    );
                    set_is_running.set(false);
                    set_can_stop.set(false);
                    set_action_busy.set(false);
                    settle_runtime();
                }
            }
        });
    };

    let stop_rustfs = move |_| {
        if action_busy.get_untracked() {
            return;
        }
        set_action_failure.set(None);
        invalidate_status_polls();
        set_action_busy.set(true);
        show_toast("Stopping RustFS...".to_string(), ToastType::Info);
        push_log(
            set_app_logs,
            "Stopping RustFS...".to_string(),
            APP_LOG_CAPACITY,
        );

        spawn_local(async move {
            match tauri_invoke("stop_rustfs", js_sys::Object::new().into()).await {
                Ok(result_value) => {
                    match serde_wasm_bindgen::from_value::<CommandResponse>(result_value) {
                        Ok(res) => {
                            if res.success {
                                set_is_running.set(false);
                                set_can_stop.set(false);
                                set_service_status.set(false);
                                set_session.set(None);
                                set_admin_open.set(false);
                                set_action_busy.set(false);
                                set_action_failure.set(None);
                                settle_runtime();
                                show_toast("RustFS stopped".to_string(), ToastType::Success);
                                push_log(
                                    set_app_logs,
                                    "RustFS stopped successfully".to_string(),
                                    APP_LOG_CAPACITY,
                                );
                            } else {
                                set_action_busy.set(false);
                                settle_runtime();
                                report_failure(
                                    OperationKind::Stop,
                                    format!("Failed to stop: {}", res.message),
                                );
                                push_log(
                                    set_app_logs,
                                    format!("Failed to stop: {}", res.message),
                                    APP_LOG_CAPACITY,
                                );
                            }
                        }
                        Err(_) => {
                            set_action_busy.set(false);
                            settle_runtime();
                            report_failure(
                                OperationKind::Stop,
                                "Failed to parse stop response".to_string(),
                            );
                        }
                    }
                }
                Err(err) => {
                    set_action_busy.set(false);
                    settle_runtime();
                    report_failure(OperationKind::Stop, js_error_message(err));
                }
            }
        });
    };

    let updates_busy = move || {
        is_checking_update.get() || is_installing_update.get() || is_installing_rustfs.get()
    };

    let run_update_checks = move |notify: bool| {
        if !is_tauri()
            || is_checking_update.get_untracked()
            || is_installing_update.get_untracked()
            || is_installing_rustfs.get_untracked()
        {
            return;
        }

        set_is_checking_update.set(true);
        spawn_local(async move {
            let mut found = Vec::<String>::new();
            let mut failures = Vec::<String>::new();

            match tauri_invoke("check_for_update", empty_args()).await {
                Ok(result) => match serde_wasm_bindgen::from_value::<UpdateInfo>(result) {
                    Ok(info) => {
                        if info.available {
                            found.push(format!(
                                "Launcher {}",
                                info.version.as_deref().unwrap_or("unknown")
                            ));
                        }
                        set_update_info.set(Some(info));
                    }
                    Err(_) => {
                        let message = "Launcher update check failed. See App Logs for details.";
                        push_log(set_app_logs, format!("[ERROR] {message}"), APP_LOG_CAPACITY);
                        if notify {
                            failures.push(message.to_string());
                        }
                    }
                },
                Err(err) => {
                    let message =
                        format!("Launcher update check failed: {}", js_error_message(err));
                    push_log(set_app_logs, format!("[ERROR] {message}"), APP_LOG_CAPACITY);
                    if notify {
                        failures.push(message);
                    }
                }
            }

            match tauri_invoke("check_rustfs_update", empty_args()).await {
                Ok(result) => match serde_wasm_bindgen::from_value::<RustFsUpdateInfo>(result) {
                    Ok(info) => {
                        if info.available {
                            found.push(format!(
                                "RustFS {}",
                                info.latest_version.as_deref().unwrap_or("unknown")
                            ));
                        }
                        set_rustfs_update_info.set(Some(info));
                    }
                    Err(_) => {
                        let message = "RustFS update check failed. See App Logs for details.";
                        push_log(set_app_logs, format!("[ERROR] {message}"), APP_LOG_CAPACITY);
                        if notify {
                            failures.push(message.to_string());
                        }
                    }
                },
                Err(err) => {
                    let message = format!("RustFS update check failed: {}", js_error_message(err));
                    push_log(set_app_logs, format!("[ERROR] {message}"), APP_LOG_CAPACITY);
                    if notify {
                        failures.push(message);
                    }
                }
            }

            if notify && !failures.is_empty() {
                report_failure(OperationKind::Check, failures.join(" "));
            }

            if notify {
                if !found.is_empty() {
                    show_toast(
                        format!("Update available: {}", found.join(" · ")),
                        ToastType::Success,
                    );
                } else if failures.is_empty() {
                    show_toast(
                        "You are running the latest versions".to_string(),
                        ToastType::Success,
                    );
                }
            } else if !found.is_empty() {
                show_toast(
                    format!("Update available: {}", found.join(" · ")),
                    ToastType::Info,
                );
            }

            set_is_checking_update.set(false);
        });
    };

    Effect::new(move |_| {
        if !is_tauri() || UPDATE_CHECK_STARTED.swap(true, Ordering::SeqCst) {
            return;
        }
        run_update_checks(false);
    });

    let check_update = move |_| {
        set_action_failure.set(None);
        run_update_checks(true);
    };

    let install_update = move |_| {
        if is_installing_update.get_untracked() || is_installing_rustfs.get_untracked() {
            return;
        }

        if can_stop.get_untracked()
            && !confirm_action(
                "Updating the launcher stops the RustFS service it manages and restarts the app once the update is installed. Continue?",
            )
        {
            return;
        }

        set_action_failure.set(None);
        set_is_installing_update.set(true);
        set_update_progress.set(Some(0));
        show_toast(
            "Downloading and installing the launcher update…".to_string(),
            ToastType::Info,
        );
        spawn_local(async move {
            match tauri_invoke("install_update", empty_args()).await {
                Ok(result) => {
                    if serde_wasm_bindgen::from_value::<CommandResponse>(result).is_err() {
                        set_is_installing_update.set(false);
                        set_update_progress.set(None);
                        report_failure(
                            OperationKind::Upgrade,
                            "Launcher update failed. See App Logs for details.".to_string(),
                        );
                    }
                }
                Err(err) => {
                    set_is_installing_update.set(false);
                    set_update_progress.set(None);
                    let message = format!("Launcher update failed: {}", js_error_message(err));
                    push_log(set_app_logs, format!("[ERROR] {message}"), APP_LOG_CAPACITY);
                    report_failure(OperationKind::Upgrade, message);
                }
            }
        });
    };

    let install_rustfs_update = move |_| {
        if is_installing_update.get_untracked() || is_installing_rustfs.get_untracked() {
            return;
        }

        if can_stop.get_untracked()
            && !confirm_action(
                "Updating RustFS stops the service managed by this launcher. Launch it again afterwards to use the new build. Continue?",
            )
        {
            return;
        }

        set_action_failure.set(None);
        set_is_installing_rustfs.set(true);
        set_rustfs_update_progress.set(Some(0));
        show_toast(
            "Downloading and installing the RustFS update…".to_string(),
            ToastType::Info,
        );
        spawn_local(async move {
            match tauri_invoke("install_rustfs_update", empty_args()).await {
                Ok(result) => match serde_wasm_bindgen::from_value::<CommandResponse>(result) {
                    Ok(response) => {
                        set_action_failure.set(None);
                        show_toast(response.message.clone(), ToastType::Success);
                        push_log(set_app_logs, response.message, APP_LOG_CAPACITY);
                        if let Ok(version_value) =
                            tauri_invoke("get_app_version_info", empty_args()).await
                        {
                            if let Ok(info) =
                                serde_wasm_bindgen::from_value::<AppVersionInfo>(version_value)
                            {
                                set_version_info.set(Some(info));
                            }
                        }
                        if let Ok(check_value) =
                            tauri_invoke("check_rustfs_update", empty_args()).await
                        {
                            if let Ok(info) =
                                serde_wasm_bindgen::from_value::<RustFsUpdateInfo>(check_value)
                            {
                                set_rustfs_update_info.set(Some(info));
                            }
                        }
                        sync_runtime_status(
                            config,
                            set_is_running,
                            set_can_stop,
                            set_service_status,
                        );
                    }
                    Err(_) => {
                        report_failure(
                            OperationKind::Upgrade,
                            "RustFS update failed. See App Logs for details.".to_string(),
                        );
                    }
                },
                Err(err) => {
                    let message = format!("RustFS update failed: {}", js_error_message(err));
                    push_log(set_app_logs, format!("[ERROR] {message}"), APP_LOG_CAPACITY);
                    report_failure(OperationKind::Upgrade, message);
                }
            }
            set_is_installing_rustfs.set(false);
            set_rustfs_update_progress.set(None);
        });
    };

    let open_service_url = move |url: String| {
        if !is_tauri() {
            return;
        }

        spawn_local(async move {
            let args = js_sys::Object::new();
            set_js_field(&args, "url", JsValue::from_str(&url));
            if let Err(err) = tauri_invoke("open_service_url", args.into()).await {
                show_toast(js_error_message(err), ToastType::Error);
            }
        });
    };

    let open_console = move |_| {
        let url = session
            .get_untracked()
            .map(|endpoint| endpoint.console_url)
            .unwrap_or_else(|| {
                let current = config.get_untracked();
                let host = display_host(current.host.as_deref());
                format!(
                    "http://{}:{}{}",
                    host,
                    current.console_port.unwrap_or(DEFAULT_CONSOLE_PORT),
                    crate::helpers::CONSOLE_UI_PATH
                )
            });
        open_service_url(url);
    };

    let copy_text = move |text: String| {
        let Some(window) = web_sys::window() else {
            return;
        };
        let Ok(navigator) = js_sys::Reflect::get(&window, &JsValue::from_str("navigator")) else {
            return;
        };
        let Ok(clipboard) = js_sys::Reflect::get(&navigator, &JsValue::from_str("clipboard"))
        else {
            return;
        };
        let Ok(write_text) = js_sys::Reflect::get(&clipboard, &JsValue::from_str("writeText"))
        else {
            return;
        };
        let Some(write_text) = write_text.dyn_ref::<js_sys::Function>() else {
            return;
        };
        let _ = write_text.call1(&clipboard, &JsValue::from_str(&text));
        show_toast("Copied".to_string(), ToastType::Success);
    };

    let screen = move || {
        first_screen_state(
            can_stop.get(),
            action_busy.get(),
            config.get().data_path.trim().is_empty(),
            logs_open.get(),
        )
    };

    view! {
        <style>{LOGS_CSS}</style>
        <main class="container" class:logs-open=move || screen().logs_visible>
            <Toast toasts=toasts remove_toast=remove_toast />

            <header class="header hero-card">
                <h1>"RustFS Launcher"</h1>
                <p class="visually-hidden" role="status">
                    {move || if service_status.get() { "Service online" } else { "Service offline" }}
                </p>
                <div class="header-tools">
                    <button
                        type="button"
                        class=settings_control_class()
                        node_ref=settings_button_ref
                        aria-label="Settings"
                        aria-haspopup="dialog"
                        aria-expanded=move || if settings_open.get() { "true" } else { "false" }
                        on:click=move |_| {
                            let next = !settings_open.get_untracked();
                            set_admin_open.set(false);
                            set_upgrade_open.set(false);
                            set_settings_open.set(next);
                        }
                    >
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            width="24"
                            height="24"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="1.5"
                            stroke-linecap="square"
                            stroke-linejoin="miter"
                            aria-hidden="true"
                        >
                            <circle cx="12" cy="12" r="3"></circle>
                            <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09a1.65 1.65 0 0 0-1-1.51 1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09a1.65 1.65 0 0 0 1.51-1 1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9c.26.6.77 1.05 1.51 1.05H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
                        </svg>
                    </button>
                </div>
            </header>

            <div class="workspace">
                <div class="workspace-scroll" node_ref=workspace_scroll_ref>
                    <div class="stage-column">
                        <ConfigForm
                            config=config
                            set_config=set_config
                            is_running=is_running
                            can_stop=can_stop
                            action_busy=action_busy
                            on_launch=Callback::new(launch_rustfs)
                            clear_action_failure=Callback::new(move |_| set_action_failure.set(None))
                            settings_open=settings_open
                            set_settings_open=set_settings_open
                            admin_open=admin_open
                            set_admin_open=set_admin_open
                            host_input_ref=host_input_ref
                            access_key_input_ref=access_key_input_ref
                        />

                        <Show when=move || screen().show_listening_address>
                            <section class="endpoint-card" aria-label="Listening address and credentials">
                                {move || session.get().map(|endpoint| {
                                    let address = endpoint.listening_address.clone();
                                    let access = endpoint.access_key.clone();
                                    let secret = endpoint.secret_key.clone();
                                    let address_copy = address.clone();
                                    let access_copy = access.clone();
                                    let secret_copy = secret.clone();
                                    let note = if endpoint.on_lan {
                                        reachability_note(Some("0.0.0.0"))
                                    } else {
                                        reachability_note(Some("127.0.0.1"))
                                    };
                                    view! {
                                        <p class="endpoint-note">{note}</p>
                                        <div class="copy-row">
                                            <span class="copy-label">"Listening"</span>
                                            <code class="copy-value">{address}</code>
                                            <button
                                                type="button"
                                                class=control_class(ControlWeight::Outline)
                                                on:click=move |_| copy_text(address_copy.clone())
                                            >
                                                "Copy"
                                            </button>
                                        </div>
                                        <div class="copy-row">
                                            <span class="copy-label">"Access key"</span>
                                            <code class="copy-value">{access}</code>
                                            <button
                                                type="button"
                                                class=control_class(ControlWeight::Outline)
                                                on:click=move |_| copy_text(access_copy.clone())
                                            >
                                                "Copy"
                                            </button>
                                        </div>
                                        <div class="copy-row">
                                            <span class="copy-label">"Secret key"</span>
                                            <code class="copy-value">{secret}</code>
                                            <button
                                                type="button"
                                                class=control_class(ControlWeight::Outline)
                                                on:click=move |_| copy_text(secret_copy.clone())
                                            >
                                                "Copy"
                                            </button>
                                        </div>
                                    }
                                })}
                            </section>
                        </Show>

                        <Show when=move || screen().show_secondary_actions>
                            <div class="secondary-actions">
                                <button
                                    type="button"
                                    class=control_class(ControlWeight::Outline)
                                    disabled=move || {
                                        session
                                            .get()
                                            .map(|endpoint| !endpoint.console_enabled)
                                            .unwrap_or(true)
                                    }
                                    on:click=open_console
                                >
                                    "Open Console"
                                </button>
                                <button
                                    type="button"
                                    class=control_class(ControlWeight::Outline)
                                    node_ref=logs_toggle_ref
                                    aria-expanded=move || if logs_open.get() { "true" } else { "false" }
                                    aria-controls="logs-panel"
                                    on:click=toggle_logs
                                >
                                    {move || if logs_open.get() { "Hide logs" } else { "View logs" }}
                                </button>
                                <button
                                    type="button"
                                    class=control_class(ControlWeight::Outline)
                                    node_ref=admin_button_ref
                                    aria-expanded=move || if admin_open.get() { "true" } else { "false" }
                                    aria-haspopup="dialog"
                                    on:click=move |_| {
                                        let next = !admin_open.get_untracked();
                                        set_settings_open.set(false);
                                        set_upgrade_open.set(false);
                                        set_admin_open.set(next);
                                    }
                                >
                                    "Change administrator"
                                </button>
                            </div>
                        </Show>

                        <Show when=move || screen().stop_visible>
                            <button
                                type="button"
                                class=control_class(ControlWeight::Destructive)
                                disabled=move || screen().stop_disabled
                                on:click=stop_rustfs
                            >
                                "Stop RustFS"
                            </button>
                        </Show>

                        {move || action_failure.get().map(|failure| {
                            let reason = failure.reason.clone();
                            let fix = failure.fix.clone();
                            view! {
                                <div class="alert-error" role="alert">
                                    <p>{reason}</p>
                                    <p>{fix}</p>
                                </div>
                            }
                        })}
                    </div>
                </div>
            </div>

            <footer class="footer">
                <p class="footer-version">{move || match version_info.get() {
                    Some(info) if info.rustfs_version != info.launcher_version => {
                        format!(
                            "Current version: {} · RustFS {}",
                            info.launcher_version, info.rustfs_version
                        )
                    }
                    Some(info) => format!("Current version: {}", info.launcher_version),
                    None => "Current version: —".to_string(),
                }}</p>
                <nav class="footer-links" aria-label="Product links">
                    <button
                        type="button"
                        class="footer-link"
                        node_ref=upgrade_button_ref
                        aria-expanded=move || if upgrade_open.get() { "true" } else { "false" }
                        aria-haspopup="dialog"
                        on:click=move |_| {
                            let next = !upgrade_open.get_untracked();
                            set_settings_open.set(false);
                            set_admin_open.set(false);
                            set_upgrade_open.set(next);
                        }
                    >
                        "Upgrade"
                    </button>
                    <span class="footer-sep" aria-hidden="true">"|"</span>
                    <button
                        type="button"
                        class="footer-link"
                        on:click=move |_| open_service_url(WEBSITE_URL.to_string())
                    >
                        "Website"
                    </button>
                    <span class="footer-sep" aria-hidden="true">"|"</span>
                    <button
                        type="button"
                        class="footer-link"
                        on:click=move |_| open_service_url(DOCS_URL.to_string())
                    >
                        "Docs"
                    </button>
                    <span class="footer-sep" aria-hidden="true">"|"</span>
                    <button
                        type="button"
                        class="footer-link"
                        on:click=move |_| open_service_url(GITHUB_URL.to_string())
                    >
                        "GitHub"
                    </button>
                </nav>
            </footer>

            <Show when=move || upgrade_open.get()>
                <div class="overlay-layer">
                    <div class="overlay-backdrop" on:click=move |_| close_upgrade()></div>
                    <div
                        class="overlay-panel overlay-panel-upgrade"
                        role="dialog"
                        aria-modal="true"
                        aria-labelledby="upgrade-title"
                        tabindex="-1"
                        node_ref=upgrade_panel_ref
                    >
                        <div class="overlay-heading">
                            <h2 id="upgrade-title">"Upgrade"</h2>
                            <button
                                type="button"
                                class="icon-button"
                                aria-label="Close upgrade"
                                on:click=move |_| close_upgrade()
                            >
                                "×"
                            </button>
                        </div>
                        <div class="update-heading">
                            <p class="update-platform">
                                {move || version_info
                                    .get()
                                    .map(|info| info.platform)
                                    .unwrap_or_else(|| "Windows / macOS".to_string())}
                            </p>
                            <button
                                type="button"
                                class=control_class(ControlWeight::Outline)
                                disabled=updates_busy
                                on:click=check_update
                            >
                                {move || if is_checking_update.get() { "Checking…" } else { "Check for Updates" }}
                            </button>
                        </div>
                        <div class="version-row">
                            <span>{move || format!(
                                "Launcher {}",
                                version_info.get().map(|info| info.launcher_version).unwrap_or_else(|| "—".to_string())
                            )}</span>
                            <span>{move || {
                                let info = version_info.get();
                                let version = info
                                    .as_ref()
                                    .map(|info| info.rustfs_version.as_str())
                                    .unwrap_or("—");
                                let source = info
                                    .as_ref()
                                    .map(|info| rustfs_source_label(info.rustfs_managed))
                                    .unwrap_or("bundled");
                                format!("RustFS {version} ({source})")
                            }}</span>
                        </div>
                        {move || update_info.get().map(|info| {
                            if info.available {
                                let version = info.version.unwrap_or_else(|| "unknown".to_string());
                                let notes = info.notes.unwrap_or_else(|| "No release notes for this version.".to_string());
                                view! {
                                    <div class="update-available">
                                        <div>
                                            <strong>{format!("Launcher {}", version)}</strong>
                                            <p>{notes}</p>
                                        </div>
                                        <button
                                            type="button"
                                            class=control_class(ControlWeight::Outline)
                                            disabled=updates_busy
                                            on:click=install_update
                                        >
                                            {move || if is_installing_update.get() { "Installing…" } else { "Update Launcher" }}
                                        </button>
                                    </div>
                                }.into_any()
                            } else {
                                view! { <p class="update-current">"Launcher is up to date."</p> }.into_any()
                            }
                        })}
                        {move || rustfs_update_info.get().map(|info| {
                            if info.available {
                                let version = info.latest_version.unwrap_or_else(|| "unknown".to_string());
                                let detail = match (info.release_type, info.asset_name) {
                                    (Some(kind), Some(asset)) => format!("{kind} · {asset}"),
                                    (Some(kind), None) => kind,
                                    (None, Some(asset)) => asset,
                                    (None, None) => "Verified download from the RustFS release feed.".to_string(),
                                };
                                view! {
                                    <div class="update-available">
                                        <div>
                                            <strong>{format!("RustFS {}", version)}</strong>
                                            <p>{detail}</p>
                                        </div>
                                        <button
                                            type="button"
                                            class=control_class(ControlWeight::Outline)
                                            disabled=updates_busy
                                            on:click=install_rustfs_update
                                        >
                                            {move || if is_installing_rustfs.get() { "Installing…" } else { "Update RustFS" }}
                                        </button>
                                    </div>
                                }.into_any()
                            } else {
                                view! { <p class="update-current">{rustfs_idle_message(info.notes.as_deref())}</p> }.into_any()
                            }
                        })}
                        {move || is_installing_update.get().then(|| view! {
                            <div class="update-progress" aria-label="Launcher update progress">
                                <div
                                    class="update-progress-bar"
                                    style:width=move || format!("{}%", update_progress.get().unwrap_or(8))
                                ></div>
                            </div>
                        })}
                        {move || is_installing_rustfs.get().then(|| view! {
                            <div class="update-progress" aria-label="RustFS update progress">
                                <div
                                    class="update-progress-bar"
                                    style:width=move || format!("{}%", rustfs_update_progress.get().unwrap_or(8))
                                ></div>
                            </div>
                        })}
                    </div>
                </div>
            </Show>

            <Show when=move || screen().logs_visible>
                <div
                    id="logs-panel"
                    class="logs-section"
                    tabindex="-1"
                    role="region"
                    aria-label="Logs"
                    node_ref=logs_panel_ref
                >
                    <LogViewer
                        app_logs=app_logs
                        set_app_logs=set_app_logs
                        rustfs_logs=rustfs_logs
                        set_rustfs_logs=set_rustfs_logs
                        current_log_type=current_log_type
                        set_current_log_type=set_current_log_type
                    />
                </div>
            </Show>
        </main>
    }
}

#[cfg(test)]
mod tests {
    use super::{PersistedConfig, RustFsConfig};
    use crate::types::{DEFAULT_ACCESS_KEY, DEFAULT_SECRET_KEY};

    #[test]
    fn persisted_config_keeps_custom_credentials() {
        let config = RustFsConfig {
            data_path: "/data".into(),
            access_key: Some("custom-ak".into()),
            secret_key: Some("custom-sk".into()),
            ..RustFsConfig::default()
        };
        let json = serde_json::to_string(&PersistedConfig::from(&config)).unwrap();
        let loaded = RustFsConfig::from(serde_json::from_str::<PersistedConfig>(&json).unwrap());
        assert_eq!(loaded.access_key.as_deref(), Some("custom-ak"));
        assert_eq!(loaded.secret_key.as_deref(), Some("custom-sk"));
        assert_eq!(loaded.data_path, "/data");
    }

    #[test]
    fn older_saved_config_without_credentials_still_loads() {
        let legacy = r#"{"data_path":"/data","port":9000,"console_port":9001,"host":"127.0.0.1","console_enable":true}"#;
        let loaded = RustFsConfig::from(serde_json::from_str::<PersistedConfig>(legacy).unwrap());
        assert_eq!(loaded.data_path, "/data");
        assert!(loaded.console_enable);
        assert_eq!(loaded.access_key.as_deref(), Some(DEFAULT_ACCESS_KEY));
        assert_eq!(loaded.secret_key.as_deref(), Some(DEFAULT_SECRET_KEY));
    }
}
