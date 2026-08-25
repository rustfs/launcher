use crate::components::config_form::ConfigForm;
use crate::components::log_viewer::LogViewer;
use crate::components::toast::{Toast, ToastMessage, ToastType};
use crate::types::{
    AppVersionInfo, CommandResponse, LogEntry, LogType, RuntimeStatus, RustFsConfig, UpdateInfo,
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

fn display_host(host: Option<&str>) -> String {
    match host.map(str::trim).filter(|value| !value.is_empty()) {
        Some("0.0.0.0") | Some("*") | None => "127.0.0.1".to_string(),
        Some("::") | Some("[::]") => "[::1]".to_string(),
        Some(host) if host.contains(':') && !host.starts_with('[') => format!("[{host}]"),
        Some(host) => host.to_string(),
    }
}

// Helper function to check if we're in Tauri environment
fn is_tauri() -> bool {
    web_sys::window()
        .and_then(|w| js_sys::Reflect::get(&w, &"__TAURI__".into()).ok())
        .map(|v| !v.is_undefined())
        .unwrap_or(false)
}

#[derive(serde::Serialize, serde::Deserialize)]
struct PersistedConfig {
    data_path: String,
    port: Option<u16>,
    console_port: Option<u16>,
    host: Option<String>,
    console_enable: bool,
}

impl From<&RustFsConfig> for PersistedConfig {
    fn from(config: &RustFsConfig) -> Self {
        Self {
            data_path: config.data_path.clone(),
            port: config.port,
            console_port: config.console_port,
            host: config.host.clone(),
            console_enable: config.console_enable,
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
            ..RustFsConfig::default()
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

const APP_LOG_CAPACITY: usize = 100;
const RUSTFS_LOG_CAPACITY: usize = 1000;
const DEFAULT_RUSTFS_PORT: u16 = 9000;
const DEFAULT_CONSOLE_PORT: u16 = 9001;
static NEXT_LOG_ID: AtomicU64 = AtomicU64::new(1);
static HEALTH_CHECK_STARTED: AtomicBool = AtomicBool::new(false);

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
    spawn_local(async move {
        if !is_tauri() {
            set_is_running.set(false);
            set_can_stop.set(false);
            set_service_status.set(false);
            return;
        }

        let current = config.get_untracked();
        let host = current.host.unwrap_or_else(|| "127.0.0.1".to_string());
        let port = current.port.unwrap_or(DEFAULT_RUSTFS_PORT);

        let status_args = js_sys::Object::new();
        js_sys::Reflect::set(&status_args, &"host".into(), &host.into()).unwrap();
        js_sys::Reflect::set(&status_args, &"port".into(), &port.into()).unwrap();

        let (managed_running, service_online) =
            match tauri_invoke("get_runtime_status", status_args.into()).await {
                Ok(value) => serde_wasm_bindgen::from_value::<RuntimeStatus>(value)
                    .map(|status| (status.managed_running, status.service_online))
                    .unwrap_or((false, false)),
                Err(_) => (false, false),
            };

        set_service_status.set(service_online);
        set_can_stop.set(managed_running);
        set_is_running.set(managed_running || service_online);
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
    let (version_info, set_version_info) = signal(None::<AppVersionInfo>);
    let (update_info, set_update_info) = signal(None::<UpdateInfo>);
    let (is_checking_update, set_is_checking_update) = signal(false);
    let (is_installing_update, set_is_installing_update) = signal(false);
    let (update_progress, set_update_progress) = signal(None::<u32>);

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

        // Auto remove after 3 seconds
        let _ = web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                Closure::once_into_js(move || {
                    remove_toast.run(id);
                })
                .unchecked_ref(),
                3000,
            );
    };

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
                    show_toast(
                        format!("RustFS exited with code: {}", exit_code),
                        ToastType::Error,
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
                let event_name = js_sys::Reflect::get(&payload, &"event".into())
                    .ok()
                    .and_then(|value| value.as_string());

                match event_name.as_deref() {
                    Some("started") => set_update_progress.set(Some(0)),
                    Some("progress") => {
                        let downloaded = js_sys::Reflect::get(&payload, &"downloaded".into())
                            .ok()
                            .and_then(|value| value.as_f64());
                        if let Some(downloaded) = downloaded {
                            let percent = js_sys::Reflect::get(&payload, &"content_length".into())
                                .ok()
                                .and_then(|value| value.as_f64())
                                .filter(|total| *total > 0.0)
                                .map(|total| ((downloaded / total) * 100.0).min(100.0) as u32);
                            if percent.is_some() {
                                set_update_progress.set(percent);
                            }
                        }
                    }
                    Some("finished") => set_update_progress.set(Some(100)),
                    _ => {}
                }
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
                    }
                }
            }

            app_listener.forget();
            rustfs_listener.forget();
            exit_listener.forget();
            update_listener.forget();
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

    let launch_rustfs = move |ev: SubmitEvent| {
        ev.prevent_default();
        set_is_running.set(true);
        set_can_stop.set(true);
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
                show_toast(
                    "Error: Not running in Tauri environment".to_string(),
                    ToastType::Error,
                );
                push_log(
                    set_app_logs,
                    "[ERROR] Not running in Tauri environment".to_string(),
                    APP_LOG_CAPACITY,
                );
                set_is_running.set(false);
                set_can_stop.set(false);
                return;
            }

            let current_config = config.get_untracked();

            // 添加详细日志
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
            let config_js = serde_wasm_bindgen::to_value(&current_config).unwrap();
            js_sys::Reflect::set(&args, &"config".into(), &config_js).unwrap();

            let validate_args = js_sys::Object::new();
            js_sys::Reflect::set(&validate_args, &"config".into(), &config_js).unwrap();
            if let Err(err) = tauri_invoke("validate_config", validate_args.into()).await {
                let message = js_error_message(err);
                show_toast(message.clone(), ToastType::Error);
                push_log(
                    set_app_logs,
                    format!("[ERROR] Config validation failed: {message}"),
                    APP_LOG_CAPACITY,
                );
                set_is_running.set(false);
                set_can_stop.set(false);
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
                                show_toast(format!("Launch failed: {}", message), ToastType::Error);
                                let now = js_sys::Date::new_0().to_locale_time_string("en-US");
                                push_log(
                                    set_app_logs,
                                    format!("[{}] Launch result: {}", now, message),
                                    APP_LOG_CAPACITY,
                                );
                                set_is_running.set(false);
                                set_can_stop.set(false);
                            }
                        }
                        Err(_) => {
                            show_toast(
                                "RustFS launch command failed".to_string(),
                                ToastType::Error,
                            );
                            let now = js_sys::Date::new_0().to_locale_time_string("en-US");
                            push_log(
                                set_app_logs,
                                format!("[{}] Launch completed but response parsing failed", now),
                                APP_LOG_CAPACITY,
                            );
                            set_is_running.set(false);
                            set_can_stop.set(false);
                        }
                    }
                }
                Err(err) => {
                    let message = js_error_message(err);
                    show_toast(format!("Launch failed: {message}"), ToastType::Error);
                    push_log(
                        set_app_logs,
                        format!("[ERROR] Launch failed: {message}"),
                        APP_LOG_CAPACITY,
                    );
                    set_is_running.set(false);
                    set_can_stop.set(false);
                }
            }
        });
    };

    let stop_rustfs = move |_| {
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
                                show_toast("RustFS stopped".to_string(), ToastType::Success);
                                push_log(
                                    set_app_logs,
                                    "RustFS stopped successfully".to_string(),
                                    APP_LOG_CAPACITY,
                                );
                            } else {
                                show_toast(
                                    format!("Failed to stop: {}", res.message),
                                    ToastType::Error,
                                );
                                push_log(
                                    set_app_logs,
                                    format!("Failed to stop: {}", res.message),
                                    APP_LOG_CAPACITY,
                                );
                            }
                        }
                        Err(_) => {
                            show_toast(
                                "Failed to parse stop response".to_string(),
                                ToastType::Error,
                            );
                        }
                    }
                }
                Err(err) => {
                    show_toast(js_error_message(err), ToastType::Error);
                }
            }
        });
    };

    let check_update = move |_| {
        if !is_tauri() || is_checking_update.get_untracked() || is_installing_update.get_untracked()
        {
            return;
        }

        set_is_checking_update.set(true);
        set_update_info.set(None);
        spawn_local(async move {
            match tauri_invoke("check_for_update", js_sys::Object::new().into()).await {
                Ok(result) => match serde_wasm_bindgen::from_value::<UpdateInfo>(result) {
                    Ok(info) => {
                        if info.available {
                            show_toast(
                                format!(
                                    "发现新版本 {}",
                                    info.version.as_deref().unwrap_or("unknown")
                                ),
                                ToastType::Success,
                            );
                        } else {
                            show_toast("当前已是最新版本".to_string(), ToastType::Success);
                        }
                        set_update_info.set(Some(info));
                    }
                    Err(_) => {
                        show_toast("检查更新失败，请查看应用日志".to_string(), ToastType::Error);
                    }
                },
                Err(err) => {
                    show_toast(
                        format!("检查更新失败: {}", js_error_message(err)),
                        ToastType::Error,
                    );
                }
            }
            set_is_checking_update.set(false);
        });
    };

    let install_update = move |_| {
        if is_installing_update.get_untracked() {
            return;
        }

        let rustfs_running = update_info
            .get_untracked()
            .map(|info| info.rustfs_running)
            .unwrap_or(false);
        if rustfs_running {
            let confirmed = web_sys::window()
                .and_then(|window| {
                    window
                        .confirm_with_message(
                            "更新会停止当前由 Launcher 管理的 RustFS 服务，并在安装后重启 Launcher。是否继续？",
                        )
                        .ok()
                })
                .unwrap_or(false);
            if !confirmed {
                return;
            }
        }

        set_is_installing_update.set(true);
        set_update_progress.set(Some(0));
        show_toast("正在下载并安装更新…".to_string(), ToastType::Info);
        spawn_local(async move {
            match tauri_invoke("install_update", js_sys::Object::new().into()).await {
                Ok(result) => {
                    if serde_wasm_bindgen::from_value::<CommandResponse>(result).is_err() {
                        set_is_installing_update.set(false);
                        set_update_progress.set(None);
                        show_toast("更新安装失败，请查看应用日志".to_string(), ToastType::Error);
                    }
                }
                Err(err) => {
                    set_is_installing_update.set(false);
                    set_update_progress.set(None);
                    show_toast(
                        format!("更新安装失败: {}", js_error_message(err)),
                        ToastType::Error,
                    );
                }
            }
        });
    };

    let open_service_url = move |url: String| {
        if !is_tauri() {
            return;
        }

        spawn_local(async move {
            let args = js_sys::Object::new();
            js_sys::Reflect::set(&args, &"url".into(), &JsValue::from_str(&url)).unwrap();
            if let Err(err) = tauri_invoke("open_service_url", args.into()).await {
                show_toast(js_error_message(err), ToastType::Error);
            }
        });
    };

    let open_api = move |_| {
        let current = config.get_untracked();
        let host = display_host(current.host.as_deref());
        let url = format!(
            "http://{}:{}",
            host,
            current.port.unwrap_or(DEFAULT_RUSTFS_PORT)
        );
        open_service_url(url);
    };

    let open_console = move |_| {
        let current = config.get_untracked();
        let host = display_host(current.host.as_deref());
        let url = format!(
            "http://{}:{}",
            host,
            current.console_port.unwrap_or(DEFAULT_CONSOLE_PORT)
        );
        open_service_url(url);
    };

    view! {
        <style>{LOGS_CSS}</style>
        <main class="container">
            <Toast toasts=toasts remove_toast=remove_toast />

            <div class="sidebar">
                <div class="header hero-card">
                    <span class="eyebrow">"Control Plane"</span>
                    <h1>"RustFS Launcher"</h1>

                    <div class="status-cluster">
                        <div class="service-indicator" class:online=move || service_status.get()>
                            <span class="status-dot"></span>
                            <span class="status-text">
                                {move || if service_status.get() { "Service Online" } else { "Service Offline" }}
                            </span>
                        </div>
                        <div class="launcher-indicator" class:managed=move || can_stop.get()>
                            {move || if can_stop.get() {
                                "Managed by Launcher"
                            } else if is_running.get() {
                                "Detected Externally"
                            } else {
                                "Ready to Launch"
                            }}
                        </div>
                    </div>
                </div>

                <div class="summary-grid">
                    <button
                        type="button"
                        class="summary-card"
                        class:clickable=move || service_status.get()
                        disabled=move || !service_status.get()
                        title="Open API endpoint"
                        on:click=open_api
                    >
                        <span class="summary-label">"API"</span>
                        <strong>{move || format!(":{}", config.get().port.unwrap_or(DEFAULT_RUSTFS_PORT))}</strong>
                    </button>
                    <button
                        type="button"
                        class="summary-card"
                        class:clickable=move || service_status.get() && config.get().console_enable
                        disabled=move || !service_status.get() || !config.get().console_enable
                        title="Open Console"
                        on:click=open_console
                    >
                        <span class="summary-label">"Console"</span>
                        <strong>{move || {
                            let current = config.get();
                            if current.console_enable {
                                format!(":{}", current.console_port.unwrap_or(DEFAULT_CONSOLE_PORT))
                            } else {
                                "Disabled".to_string()
                            }
                        }}</strong>
                    </button>
                    <div class="summary-card">
                        <span class="summary-label">"Mode"</span>
                        <strong>{move || if is_running.get() { "Locked" } else { "Editable" }}</strong>
                    </div>
                </div>

                <section class="update-card">
                    <div class="update-heading">
                        <div>
                            <span class="summary-label">"Software Update"</span>
                            <strong>"版本与更新"</strong>
                        </div>
                        <button
                            type="button"
                            class="update-check-btn"
                            disabled=move || is_checking_update.get() || is_installing_update.get()
                            on:click=check_update
                        >
                            {move || if is_checking_update.get() { "检查中…" } else { "检查更新" }}
                        </button>
                    </div>

                    <div class="version-row">
                        <span>{move || format!(
                            "Launcher {}",
                            version_info.get().map(|info| info.launcher_version).unwrap_or_else(|| "—".to_string())
                        )}</span>
                        <span>{move || format!(
                            "RustFS {}",
                            version_info.get().map(|info| info.rustfs_version).unwrap_or_else(|| "—".to_string())
                        )}</span>
                    </div>

                    {move || update_info.get().map(|info| {
                        if info.available {
                            let version = info.version.unwrap_or_else(|| "unknown".to_string());
                            let notes = info.notes.unwrap_or_else(|| "此版本没有更新说明。".to_string());
                            view! {
                                <div class="update-available">
                                    <div>
                                        <strong>{format!("可更新至 {}", version)}</strong>
                                        <p>{notes}</p>
                                    </div>
                                    <button
                                        type="button"
                                        class="update-install-btn"
                                        disabled=move || is_installing_update.get()
                                        on:click=install_update
                                    >
                                        {move || if is_installing_update.get() { "安装中…" } else { "立即更新" }}
                                    </button>
                                </div>
                            }.into_any()
                        } else {
                            view! { <p class="update-current">"当前已是最新版本。"</p> }.into_any()
                        }
                    })}

                    {move || is_installing_update.get().then(|| view! {
                        <div class="update-progress">
                            <div
                                class="update-progress-bar"
                                style:width=move || format!("{}%", update_progress.get().unwrap_or(8))
                            ></div>
                        </div>
                    })}
                </section>

                <ConfigForm
                    config=config
                    set_config=set_config
                    is_running=is_running
                    can_stop=can_stop
                    on_launch=Callback::new(launch_rustfs)
                    on_stop=Callback::new(stop_rustfs)
                />
            </div>

            <div class="logs-section">
                <LogViewer
                    app_logs=app_logs
                    set_app_logs=set_app_logs
                    rustfs_logs=rustfs_logs
                    set_rustfs_logs=set_rustfs_logs
                    current_log_type=current_log_type
                    set_current_log_type=set_current_log_type
                />
            </div>
        </main>
    }
}
