use crate::types::{RustFsConfig, DEFAULT_API_PORT, DEFAULT_CONSOLE_PORT, DEFAULT_HOST};
use leptos::ev::SubmitEvent;
use leptos::prelude::*;
use serde_json;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

use std::sync::atomic::{AtomicBool, Ordering};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "dialog"])]
    async fn open(options: JsValue) -> JsValue;
}

const API_PORT_ERROR_MESSAGE: &str = "API port must be a number between 1 and 65535";
const CONSOLE_PORT_ERROR_MESSAGE: &str = "Console port must be a number between 1 and 65535";
const PORT_CONFLICT_ERROR_MESSAGE: &str = "API port and console port must be different";

fn extract_dropped_paths(payload: &JsValue) -> Vec<String> {
    if let Ok(paths) = js_sys::Reflect::get(payload, &"paths".into()) {
        if let Ok(parsed) = serde_wasm_bindgen::from_value::<Vec<String>>(paths) {
            return parsed;
        }
    }

    serde_wasm_bindgen::from_value::<Vec<String>>(payload.clone()).unwrap_or_default()
}

static DRAG_LISTENERS_REGISTERED: AtomicBool = AtomicBool::new(false);

fn parse_port_input(value: &str, error_message: &'static str) -> Result<Option<u16>, &'static str> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    let port = trimmed.parse::<u16>().map_err(|_| error_message)?;

    if port == 0 {
        return Err(error_message);
    }

    Ok(Some(port))
}

fn validate_port_conflict(
    config: &RustFsConfig,
    api_port: Option<u16>,
    console_port: Option<u16>,
) -> Result<(), &'static str> {
    if !config.console_enable {
        return Ok(());
    }

    let effective_api_port = api_port.unwrap_or(DEFAULT_API_PORT);
    let effective_console_port = console_port.unwrap_or(DEFAULT_CONSOLE_PORT);

    if effective_api_port == effective_console_port {
        return Err(PORT_CONFLICT_ERROR_MESSAGE);
    }

    Ok(())
}

#[component]
pub fn ConfigForm(
    #[prop(into)] config: Signal<RustFsConfig>,
    #[prop(into)] set_config: WriteSignal<RustFsConfig>,
    #[prop(into)] is_running: Signal<bool>,
    #[prop(into)] can_stop: Signal<bool>,
    #[prop(into)] on_launch: Callback<SubmitEvent>,
    #[prop(into)] on_stop: Callback<()>,
) -> impl IntoView {
    let (show_secret, set_show_secret) = signal(false);
    let (is_drag_over, set_is_drag_over) = signal(false);
    let (error_message, set_error_message) = signal(Option::<String>::None);
    let (port_input, set_port_input) = signal(
        config
            .get_untracked()
            .port
            .map(|p| p.to_string())
            .unwrap_or_default(),
    );
    let (console_port_input, set_console_port_input) = signal(
        config
            .get_untracked()
            .console_port
            .map(|p| p.to_string())
            .unwrap_or_default(),
    );

    let clear_related_error = move |messages: &[&'static str]| {
        set_error_message.update(|error| {
            if let Some(current) = error.as_deref() {
                if messages.contains(&current) {
                    *error = None;
                }
            }
        });
    };

    let validate_and_store_api_port = move |value: String| {
        set_port_input.set(value.clone());

        match parse_port_input(&value, API_PORT_ERROR_MESSAGE) {
            Ok(port) => {
                let current = config.get_untracked();
                set_config.update(|c| c.port = port);

                match validate_port_conflict(&current, port, current.console_port) {
                    Ok(()) => {
                        clear_related_error(&[API_PORT_ERROR_MESSAGE, PORT_CONFLICT_ERROR_MESSAGE])
                    }
                    Err(message) => set_error_message.set(Some(message.to_string())),
                }
            }
            Err(message) => {
                set_config.update(|c| c.port = None);
                set_error_message.set(Some(message.to_string()));
            }
        }
    };

    let validate_and_store_console_port = move |value: String| {
        set_console_port_input.set(value.clone());

        match parse_port_input(&value, CONSOLE_PORT_ERROR_MESSAGE) {
            Ok(console_port) => {
                let current = config.get_untracked();
                set_config.update(|c| c.console_port = console_port);

                match validate_port_conflict(&current, current.port, console_port) {
                    Ok(()) => clear_related_error(&[
                        CONSOLE_PORT_ERROR_MESSAGE,
                        PORT_CONFLICT_ERROR_MESSAGE,
                    ]),
                    Err(message) => set_error_message.set(Some(message.to_string())),
                }
            }
            Err(message) => {
                set_config.update(|c| c.console_port = None);
                set_error_message.set(Some(message.to_string()));
            }
        }
    };

    Effect::new(move |_| {
        if DRAG_LISTENERS_REGISTERED.swap(true, Ordering::SeqCst) {
            return;
        }

        spawn_local(async move {
            if let Some(window) = web_sys::window() {
                if let Ok(tauri) = js_sys::Reflect::get(&window, &"__TAURI__".into()) {
                    if let Ok(event) = js_sys::Reflect::get(&tauri, &"event".into()) {
                        if let Ok(listen) = js_sys::Reflect::get(&event, &"listen".into()) {
                            let listen_fn = js_sys::Function::from(listen);

                            let drop_handler = Closure::wrap(Box::new(move |event: JsValue| {
                                set_is_drag_over.set(false);

                                if is_running.get_untracked() {
                                    return;
                                }

                                if let Ok(payload) = js_sys::Reflect::get(&event, &"payload".into())
                                {
                                    if let Some(first_path) =
                                        extract_dropped_paths(&payload).into_iter().next()
                                    {
                                        set_config.update(|c| c.data_path = first_path);
                                        set_error_message.set(None);
                                    }
                                }
                            })
                                as Box<dyn FnMut(JsValue)>);

                            let hover_handler = Closure::wrap(Box::new(move |_: JsValue| {
                                if !is_running.get_untracked() {
                                    set_is_drag_over.set(true);
                                }
                            })
                                as Box<dyn FnMut(JsValue)>);

                            let cancel_handler = Closure::wrap(Box::new(move |_: JsValue| {
                                set_is_drag_over.set(false);
                            })
                                as Box<dyn FnMut(JsValue)>);

                            for event_name in ["tauri://drag-drop", "tauri://file-drop"] {
                                let _ = listen_fn.call2(
                                    &event,
                                    &event_name.into(),
                                    drop_handler.as_ref().unchecked_ref(),
                                );
                            }
                            for event_name in [
                                "tauri://drag-over",
                                "tauri://drag-enter",
                                "tauri://file-drop-hover",
                            ] {
                                let _ = listen_fn.call2(
                                    &event,
                                    &event_name.into(),
                                    hover_handler.as_ref().unchecked_ref(),
                                );
                            }
                            for event_name in ["tauri://drag-leave", "tauri://file-drop-cancelled"]
                            {
                                let _ = listen_fn.call2(
                                    &event,
                                    &event_name.into(),
                                    cancel_handler.as_ref().unchecked_ref(),
                                );
                            }

                            drop_handler.forget();
                            hover_handler.forget();
                            cancel_handler.forget();
                        }
                    }
                }
            }
        });
    });

    let select_folder = move |_| {
        if is_running.get() {
            return;
        }

        spawn_local(async move {
            let options = serde_wasm_bindgen::to_value(&serde_json::json!({
                "directory": true,
                "title": "Select RustFS Data Directory"
            }))
            .unwrap();

            if let Some(result) = open(options).await.as_string() {
                if !result.is_empty() {
                    set_config.update(|c| c.data_path = result);
                    set_error_message.set(None);
                }
            }
        });
    };

    let handle_submit = move |ev: SubmitEvent| {
        ev.prevent_default();

        if is_running.get() {
            if can_stop.get() {
                on_stop.run(());
            }
            return;
        }

        set_error_message.set(None);

        let current_config = config.get();

        if current_config.data_path.trim().is_empty() {
            set_error_message.set(Some("Data path is required".to_string()));
            return;
        }

        let port = match parse_port_input(&port_input.get(), API_PORT_ERROR_MESSAGE) {
            Ok(port) => port,
            Err(message) => {
                set_error_message.set(Some(message.to_string()));
                return;
            }
        };

        let console_port = if current_config.console_enable {
            match parse_port_input(&console_port_input.get(), CONSOLE_PORT_ERROR_MESSAGE) {
                Ok(console_port) => console_port,
                Err(message) => {
                    set_error_message.set(Some(message.to_string()));
                    return;
                }
            }
        } else {
            current_config.console_port
        };

        if let Err(message) = validate_port_conflict(&current_config, port, console_port) {
            set_error_message.set(Some(message.to_string()));
            return;
        }

        set_config.update(|c| {
            c.port = port;
            c.console_port = console_port;
        });

        on_launch.run(ev);
    };

    view! {
        <form class="config-form" class:locked=move || is_running.get() on:submit=handle_submit>
            <Show when=move || is_running.get()>
                <div class="lock-banner">
                    <span class="lock-banner-label">"Configuration Locked"</span>
                </div>
            </Show>

            <div class="control-surface">
                <section class="form-section">
                    <div class="section-header">
                        <span class="section-tag">"Storage"</span>
                        <h2>"Dataset Mount"</h2>
                    </div>

                    <div class="form-group">
                        <label for="data-path">"Data Path" <span class="required">"*"</span></label>
                        <div
                            class="path-input-group"
                            class:drag-over=move || is_drag_over.get() && !is_running.get()
                        >
                            <input
                                id="data-path"
                                type="text"
                                placeholder="Select or drop data directory..."
                                prop:value=move || config.get().data_path
                                disabled=move || is_running.get()
                                on:input=move |ev| {
                                    let value = event_target_value(&ev);
                                    set_config.update(|c| c.data_path = value.clone());
                                    if !value.is_empty() {
                                        set_error_message.set(None);
                                    }
                                }
                            />
                            <button
                                type="button"
                                class="browse-btn"
                                disabled=move || is_running.get()
                                on:click=select_folder
                            >
                                "Browse"
                            </button>
                        </div>
                    </div>
                </section>

                <section class="form-section">
                    <div class="section-header">
                        <span class="section-tag">"Network"</span>
                        <h2>"Ports & Access"</h2>
                    </div>

                    <div class="form-row">
                        <div class="form-group">
                            <label for="port">"API Port"</label>
                            <input
                                id="port"
                                type="number"
                                placeholder="9000"
                                min="1"
                                max="65535"
                                prop:value=move || port_input.get()
                                disabled=move || is_running.get()
                                on:input=move |ev| validate_and_store_api_port(event_target_value(&ev))
                            />
                        </div>
                        <div class="form-group">
                            <label for="host">"Host"</label>
                            <input
                                id="host"
                                type="text"
                                placeholder="127.0.0.1"
                                prop:value=move || config.get().host.unwrap_or_default()
                                disabled=move || is_running.get()
                                on:input=move |ev| {
                                    let value = event_target_value(&ev);
                                    let host = if value.is_empty() { None } else { Some(value) };
                                    set_config.update(|c| c.host = host);
                                }
                            />
                        </div>
                    </div>

                    <div class="runtime-toggle-row">
                        <label class="toggle-card" class:disabled=move || is_running.get()>
                            <input
                                id="console-enable"
                                type="checkbox"
                                prop:checked=move || config.get().console_enable
                                disabled=move || is_running.get()
                                on:change=move |ev| {
                                    let checked = event_target_checked(&ev);
                                    set_config.update(|c| {
                                        c.console_enable = checked;
                                        if c.console_port.is_none() {
                                            c.console_port = Some(DEFAULT_CONSOLE_PORT);
                                        }
                                    });
                                    if checked {
                                        let current_console_port = config
                                            .get_untracked()
                                            .console_port
                                            .unwrap_or(DEFAULT_CONSOLE_PORT);
                                        set_console_port_input.set(current_console_port.to_string());
                                    }
                                }
                            />
                            <span class="toggle-indicator"></span>
                            <span class="toggle-copy">
                                <strong>"Console Endpoint"</strong>
                            </span>
                        </label>
                    </div>

                    <Show when=move || config.get().console_enable>
                        <div class="console-port-stack">
                            <div class="form-group">
                                <label for="console-port">"Console Port"</label>
                                <input
                                    id="console-port"
                                    type="number"
                                    placeholder="9001"
                                    min="1"
                                    max="65535"
                                    prop:value=move || console_port_input.get()
                                    disabled=move || is_running.get()
                                    on:input=move |ev| {
                                        validate_and_store_console_port(event_target_value(&ev))
                                    }
                                />
                            </div>
                            <div class="console-bind-inline">
                                <span class="console-bind-label">"Console Bind"</span>
                                <strong>
                                    {move || {
                                        let current = config.get();
                                        format!(
                                            "{}:{}",
                                            current.host.unwrap_or_else(|| DEFAULT_HOST.to_string()),
                                            current.console_port.unwrap_or(DEFAULT_CONSOLE_PORT)
                                        )
                                    }}
                                </strong>
                            </div>
                        </div>
                    </Show>
                </section>

                <section class="form-section">
                    <div class="section-header">
                        <span class="section-tag">"Security"</span>
                        <h2>"Credentials"</h2>
                    </div>

                    <div class="form-row">
                        <div class="form-group">
                            <label for="access-key">"Access Key"</label>
                            <input
                                id="access-key"
                                type="text"
                                placeholder="Access key"
                                prop:value=move || config.get().access_key.unwrap_or_default()
                                disabled=move || is_running.get()
                                on:input=move |ev| {
                                    let value = event_target_value(&ev);
                                    let access_key = if value.is_empty() { None } else { Some(value) };
                                    set_config.update(|c| c.access_key = access_key);
                                }
                            />
                        </div>
                        <div class="form-group">
                            <label for="secret-key">"Secret Key"</label>
                            <div class="input-with-toggle">
                                <input
                                    id="secret-key"
                                    type=move || if show_secret.get() { "text" } else { "password" }
                                    placeholder="Secret key"
                                    prop:value=move || config.get().secret_key.unwrap_or_default()
                                    disabled=move || is_running.get()
                                    on:input=move |ev| {
                                        let value = event_target_value(&ev);
                                        let secret_key = if value.is_empty() { None } else { Some(value) };
                                        set_config.update(|c| c.secret_key = secret_key);
                                    }
                                />
                                <button
                                    type="button"
                                    class="toggle-visibility"
                                    disabled=move || is_running.get()
                                    aria-pressed=move || show_secret.get()
                                    aria-label=move || {
                                        if show_secret.get() {
                                            "Hide secret key"
                                        } else {
                                            "Show secret key"
                                        }
                                    }
                                    on:click=move |_| {
                                        if !is_running.get() {
                                            set_show_secret.update(|show| *show = !*show);
                                        }
                                    }
                                    title=move || if show_secret.get() { "Hide Secret" } else { "Show Secret" }
                                >
                                    {move || if show_secret.get() {
                                        view! {
                                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                                <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"></path>
                                                <line x1="1" y1="1" x2="23" y2="23"></line>
                                            </svg>
                                        }
                                            .into_any()
                                    } else {
                                        view! {
                                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                                <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"></path>
                                                <circle cx="12" cy="12" r="3"></circle>
                                            </svg>
                                        }
                                            .into_any()
                                    }}
                                </button>
                            </div>
                        </div>
                    </div>
                </section>
            </div>

            <div class="form-actions">
                <button
                    type="submit"
                    class="launch-btn"
                    class:stop-btn=move || can_stop.get()
                    disabled=move || {
                        config.get().data_path.is_empty() || (is_running.get() && !can_stop.get())
                    }
                >
                    {move || {
                        if can_stop.get() {
                            "Stop RustFS"
                        } else if is_running.get() {
                            "RustFS Running"
                        } else {
                            "Launch RustFS"
                        }
                    }}
                </button>

                <Show when=move || is_running.get() && !can_stop.get()>
                    <div class="field-hint">
                        "The configured RustFS service is online, but this launcher is not managing that process."
                    </div>
                </Show>

                <Show when=move || error_message.get().is_some()>
                    <div class="error-message">{move || error_message.get()}</div>
                </Show>
            </div>
        </form>
    }
}
