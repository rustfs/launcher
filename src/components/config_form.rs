use crate::helpers::{
    control_class, first_screen_state, operation_failure, primary_action, ControlWeight,
    OperationKind,
};
use crate::types::{RustFsConfig, DEFAULT_CONSOLE_PORT, DEFAULT_HOST};
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

    let effective_api_port = api_port.unwrap_or(crate::types::DEFAULT_API_PORT);
    let effective_console_port = console_port.unwrap_or(DEFAULT_CONSOLE_PORT);

    if effective_api_port == effective_console_port {
        return Err(PORT_CONFLICT_ERROR_MESSAGE);
    }

    Ok(())
}

fn alert_for(raw: String) -> impl IntoView {
    let failure = operation_failure(OperationKind::Launch, &raw);
    view! {
        <div class="alert-error" role="alert">
            <p>{failure.reason}</p>
            <p>{failure.fix}</p>
        </div>
    }
}

#[component]
pub fn ConfigForm(
    #[prop(into)] config: Signal<RustFsConfig>,
    #[prop(into)] set_config: WriteSignal<RustFsConfig>,
    #[prop(into)] is_running: Signal<bool>,
    #[prop(into)] can_stop: Signal<bool>,
    #[prop(into)] action_busy: Signal<bool>,
    #[prop(into)] on_launch: Callback<SubmitEvent>,
    #[prop(into)] clear_action_failure: Callback<()>,
    #[prop(into)] settings_open: Signal<bool>,
    #[prop(into)] set_settings_open: WriteSignal<bool>,
    #[prop(into)] on_dismiss_settings: Callback<()>,
    #[prop(into)] admin_open: Signal<bool>,
    #[prop(into)] on_dismiss_admin: Callback<()>,
    host_input_ref: NodeRef<leptos::html::Input>,
    access_key_input_ref: NodeRef<leptos::html::Input>,
) -> impl IntoView {
    let (is_drag_over, set_is_drag_over) = signal(false);
    let (error_message, set_error_message) = signal(Option::<String>::None);
    let (settings_error, set_settings_error) = signal(Option::<String>::None);
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
        set_settings_error.update(|error| {
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
                    Err(message) => set_settings_error.set(Some(message.to_string())),
                }
            }
            Err(message) => {
                set_config.update(|c| c.port = None);
                set_settings_error.set(Some(message.to_string()));
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
                    Err(message) => set_settings_error.set(Some(message.to_string())),
                }
            }
            Err(message) => {
                set_config.update(|c| c.console_port = None);
                set_settings_error.set(Some(message.to_string()));
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

                                if primary_action(
                                    can_stop.get_untracked(),
                                    action_busy.get_untracked(),
                                    false,
                                )
                                .lock_form
                                {
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
                                if !primary_action(
                                    can_stop.get_untracked(),
                                    action_busy.get_untracked(),
                                    false,
                                )
                                .lock_form
                                {
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
        if primary_action(can_stop.get(), action_busy.get(), false).lock_form {
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
        clear_action_failure.run(());

        let current_config = config.get();
        let path_empty = current_config.data_path.trim().is_empty();
        let action = primary_action(can_stop.get(), action_busy.get(), path_empty);
        if action_busy.get() || action.stops_service || action.disabled {
            return;
        }

        set_error_message.set(None);

        let port = match parse_port_input(&port_input.get(), API_PORT_ERROR_MESSAGE) {
            Ok(port) => port,
            Err(message) => {
                set_settings_error.set(Some(message.to_string()));
                set_settings_open.set(true);
                return;
            }
        };

        let console_port = if current_config.console_enable {
            match parse_port_input(&console_port_input.get(), CONSOLE_PORT_ERROR_MESSAGE) {
                Ok(console_port) => console_port,
                Err(message) => {
                    set_settings_error.set(Some(message.to_string()));
                    set_settings_open.set(true);
                    return;
                }
            }
        } else {
            current_config.console_port
        };

        if let Err(message) = validate_port_conflict(&current_config, port, console_port) {
            set_settings_error.set(Some(message.to_string()));
            set_settings_open.set(true);
            return;
        }

        set_config.update(|c| {
            c.port = port;
            c.console_port = console_port;
        });

        on_launch.run(ev);
    };

    let screen = move || {
        first_screen_state(
            can_stop.get(),
            action_busy.get(),
            config.get().data_path.trim().is_empty(),
            false,
            false,
        )
    };

    view! {
        <form class="config-form" on:submit=handle_submit>
            <Show when=move || screen().show_path>
                <div class="form-group">
                    <label for="data-path">"Data path" <span class="required">"*"</span></label>
                    <div
                        class="path-input-group"
                        class:drag-over=move || is_drag_over.get() && !can_stop.get() && !action_busy.get()
                    >
                        <input
                            id="data-path"
                            type="text"
                            dir="ltr"
                            spellcheck="false"
                            autocomplete="off"
                            placeholder="Select or drop a data folder"
                            prop:value=move || config.get().data_path
                            disabled=move || action_busy.get()
                            on:input=move |ev| {
                                let value = event_target_value(&ev);
                                set_config.update(|c| c.data_path = value.clone());
                                if !value.trim().is_empty() {
                                    set_error_message.set(None);
                                }
                            }
                        />
                        <button
                            type="button"
                            class=control_class(ControlWeight::Outline)
                            disabled=move || action_busy.get()
                            on:click=select_folder
                        >
                            "Browse"
                        </button>
                    </div>
                </div>
            </Show>

            <Show when=move || screen().launch_visible>
                <div class="form-actions">
                    <button
                        type="submit"
                        class=control_class(ControlWeight::Primary)
                        disabled=move || screen().launch_disabled
                    >
                        {move || screen().launch_label}
                    </button>
                </div>
            </Show>

            <Show when=move || is_running.get() && !can_stop.get()>
                <p class="field-hint">
                    "Something is already listening on this host and port, but this launcher did not start it. Change the port in Settings, or stop that program, then launch."
                </p>
            </Show>

            {move || error_message.get().map(alert_for)}
        </form>

        <Show when=move || settings_open.get()>
            <div class="overlay-layer">
                <div class="overlay-backdrop" on:click=move |_| on_dismiss_settings.run(())></div>
                <div
                    class="overlay-panel"
                    role="dialog"
                    aria-modal="true"
                    aria-labelledby="settings-title"
                >
                    <div class="overlay-heading">
                        <h2 id="settings-title">"Settings"</h2>
                        <button
                            type="button"
                            class="icon-button"
                            aria-label="Close settings"
                            on:click=move |_| on_dismiss_settings.run(())
                        >
                            "×"
                        </button>
                    </div>
                    <p class="overlay-note">
                        "Host, port, and console apply the next time you launch. A new install fills in 0.0.0.0, so other devices on your network can connect. An empty host stays on this computer."
                    </p>
                    <div class="form-row">
                        <div class="form-group">
                            <label for="host">"Host"</label>
                            <input
                                id="host"
                                node_ref=host_input_ref
                                type="text"
                                dir="ltr"
                                spellcheck="false"
                                autocomplete="off"
                                placeholder=DEFAULT_HOST
                                prop:value=move || config.get().host.unwrap_or_default()
                                on:input=move |ev| {
                                    let value = event_target_value(&ev);
                                    let host = if value.trim().is_empty() { None } else { Some(value) };
                                    set_config.update(|c| c.host = host);
                                }
                            />
                        </div>
                        <div class="form-group">
                            <label for="port">"API port"</label>
                            <input
                                id="port"
                                type="number"
                                placeholder="9000"
                                min="1"
                                max="65535"
                                prop:value=move || port_input.get()
                                on:input=move |ev| validate_and_store_api_port(event_target_value(&ev))
                            />
                        </div>
                    </div>
                    <label class="check-row" for="console-enable">
                        <input
                            id="console-enable"
                            type="checkbox"
                            prop:checked=move || config.get().console_enable
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
                                set_settings_error.set(None);
                            }
                        />
                        <span>"Console"</span>
                    </label>
                    <Show when=move || config.get().console_enable>
                        <div class="form-group">
                            <label for="console-port">"Console port"</label>
                            <input
                                id="console-port"
                                type="number"
                                placeholder="9001"
                                min="1"
                                max="65535"
                                prop:value=move || console_port_input.get()
                                on:input=move |ev| validate_and_store_console_port(event_target_value(&ev))
                            />
                        </div>
                    </Show>
                    {move || settings_error.get().map(alert_for)}
                </div>
            </div>
        </Show>

        <Show when=move || admin_open.get()>
            <div class="overlay-layer">
                <div class="overlay-backdrop" on:click=move |_| on_dismiss_admin.run(())></div>
                <div
                    class="overlay-panel"
                    role="dialog"
                    aria-modal="true"
                    aria-labelledby="admin-title"
                >
                    <div class="overlay-heading">
                        <h2 id="admin-title">"Change administrator"</h2>
                        <button
                            type="button"
                            class="icon-button"
                            aria-label="Close change administrator"
                            on:click=move |_| on_dismiss_admin.run(())
                        >
                            "×"
                        </button>
                    </div>
                    <p class="overlay-note">
                        "These keys are used the next time you launch. RustFS keeps the keys shown on the main screen until then. The published default is rustfsadmin."
                    </p>
                    <div class="form-group">
                        <label for="access-key">"Access key"</label>
                        <input
                            id="access-key"
                            node_ref=access_key_input_ref
                            type="text"
                            dir="ltr"
                            spellcheck="false"
                            autocomplete="off"
                            placeholder="rustfsadmin"
                            prop:value=move || config.get().access_key.unwrap_or_default()
                            on:input=move |ev| {
                                let value = event_target_value(&ev);
                                let access_key = if value.is_empty() { None } else { Some(value) };
                                set_config.update(|c| c.access_key = access_key);
                            }
                        />
                    </div>
                    <div class="form-group">
                        <label for="secret-key">"Secret key"</label>
                        <input
                            id="secret-key"
                            type="text"
                            dir="ltr"
                            spellcheck="false"
                            autocomplete="off"
                            placeholder="rustfsadmin"
                            prop:value=move || config.get().secret_key.unwrap_or_default()
                            on:input=move |ev| {
                                let value = event_target_value(&ev);
                                let secret_key = if value.is_empty() { None } else { Some(value) };
                                set_config.update(|c| c.secret_key = secret_key);
                            }
                        />
                    </div>
                </div>
            </div>
        </Show>
    }
}
