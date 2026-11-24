use crate::helpers::{
    control_class, first_screen_state, primary_action, ControlWeight, OperationFailure,
};
use crate::i18n::{use_i18n, Strings};
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

/// Kept as a value rather than text so an alert already on screen follows a
/// language switch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FormError {
    ApiPort,
    ConsolePort,
    PortConflict,
}

impl FormError {
    fn failure(self, t: &Strings) -> OperationFailure {
        let (reason, fix) = match self {
            FormError::ApiPort => (t.api_port_invalid, t.fix_port_number),
            FormError::ConsolePort => (t.console_port_invalid, t.fix_port_number),
            FormError::PortConflict => (t.port_conflict, t.fix_different_ports),
        };
        OperationFailure {
            reason: reason.to_string(),
            fix: fix.to_string(),
        }
    }
}

fn extract_dropped_paths(payload: &JsValue) -> Vec<String> {
    if let Ok(paths) = js_sys::Reflect::get(payload, &"paths".into()) {
        if let Ok(parsed) = serde_wasm_bindgen::from_value::<Vec<String>>(paths) {
            return parsed;
        }
    }

    serde_wasm_bindgen::from_value::<Vec<String>>(payload.clone()).unwrap_or_default()
}

static DRAG_LISTENERS_REGISTERED: AtomicBool = AtomicBool::new(false);

fn parse_port_input(value: &str, error: FormError) -> Result<Option<u16>, FormError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    let Ok(port) = trimmed.parse::<u16>() else {
        return Err(error);
    };

    if port == 0 {
        return Err(error);
    }

    Ok(Some(port))
}

fn validate_port_conflict(
    config: &RustFsConfig,
    api_port: Option<u16>,
    console_port: Option<u16>,
) -> Result<(), FormError> {
    if !config.console_enable {
        return Ok(());
    }

    let effective_api_port = api_port.unwrap_or(DEFAULT_API_PORT);
    let effective_console_port = console_port.unwrap_or(DEFAULT_CONSOLE_PORT);

    if effective_api_port == effective_console_port {
        return Err(FormError::PortConflict);
    }

    Ok(())
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
    let i18n = use_i18n();
    let (is_drag_over, set_is_drag_over) = signal(false);
    let (settings_error, set_settings_error) = signal(Option::<FormError>::None);
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

    let clear_related_error = move |related: &[FormError]| {
        set_settings_error.update(|error| {
            if let Some(current) = error.as_ref() {
                if related.contains(current) {
                    *error = None;
                }
            }
        });
    };

    let validate_and_store_api_port = move |value: String| {
        set_port_input.set(value.clone());

        match parse_port_input(&value, FormError::ApiPort) {
            Ok(port) => {
                let current = config.get_untracked();
                set_config.update(|c| c.port = port);

                match validate_port_conflict(&current, port, current.console_port) {
                    Ok(()) => clear_related_error(&[FormError::ApiPort, FormError::PortConflict]),
                    Err(error) => set_settings_error.set(Some(error)),
                }
            }
            Err(error) => {
                set_config.update(|c| c.port = None);
                set_settings_error.set(Some(error));
            }
        }
    };

    let validate_and_store_console_port = move |value: String| {
        set_console_port_input.set(value.clone());

        match parse_port_input(&value, FormError::ConsolePort) {
            Ok(console_port) => {
                let current = config.get_untracked();
                set_config.update(|c| c.console_port = console_port);

                match validate_port_conflict(&current, current.port, console_port) {
                    Ok(()) => {
                        clear_related_error(&[FormError::ConsolePort, FormError::PortConflict])
                    }
                    Err(error) => set_settings_error.set(Some(error)),
                }
            }
            Err(error) => {
                set_config.update(|c| c.console_port = None);
                set_settings_error.set(Some(error));
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

        let title = i18n.t_untracked().select_data_dir;
        spawn_local(async move {
            let options = serde_wasm_bindgen::to_value(&serde_json::json!({
                "directory": true,
                "title": title
            }))
            .unwrap();

            if let Some(result) = open(options).await.as_string() {
                if !result.is_empty() {
                    set_config.update(|c| c.data_path = result);
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

        let port = match parse_port_input(&port_input.get(), FormError::ApiPort) {
            Ok(port) => port,
            Err(error) => {
                set_settings_error.set(Some(error));
                set_settings_open.set(true);
                return;
            }
        };

        let console_port = if current_config.console_enable {
            match parse_port_input(&console_port_input.get(), FormError::ConsolePort) {
                Ok(console_port) => console_port,
                Err(error) => {
                    set_settings_error.set(Some(error));
                    set_settings_open.set(true);
                    return;
                }
            }
        } else {
            current_config.console_port
        };

        if let Err(error) = validate_port_conflict(&current_config, port, console_port) {
            set_settings_error.set(Some(error));
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
                    <label for="data-path">
                        {move || i18n.t().data_path}
                        " "
                        <span class="required">"*"</span>
                    </label>
                    <div
                        class="path-input-group"
                        dir="ltr"
                        class:drag-over=move || is_drag_over.get() && !can_stop.get() && !action_busy.get()
                    >
                        <input
                            id="data-path"
                            type="text"
                            dir="ltr"
                            spellcheck="false"
                            autocomplete="off"
                            placeholder=move || i18n.t().data_path_placeholder
                            prop:value=move || config.get().data_path
                            disabled=move || action_busy.get()
                            on:input=move |ev| {
                                let value = event_target_value(&ev);
                                set_config.update(|c| c.data_path = value);
                            }
                        />
                        <button
                            type="button"
                            class=control_class(ControlWeight::Outline)
                            disabled=move || action_busy.get()
                            on:click=select_folder
                        >
                            {move || i18n.t().browse}
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
                        {move || i18n.t().action_label(screen().launch_label)}
                    </button>
                </div>
            </Show>

            <Show when=move || is_running.get() && !can_stop.get()>
                <p class="field-hint">
                    {move || i18n.t().external_listener}
                </p>
            </Show>
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
                        <h2 id="settings-title">{move || i18n.t().settings}</h2>
                        <button
                            type="button"
                            class="icon-button"
                            aria-label=move || i18n.t().close_settings
                            on:click=move |_| on_dismiss_settings.run(())
                        >
                            "×"
                        </button>
                    </div>
                    <p class="overlay-note">
                        {move || i18n.t().settings_note}
                    </p>
                    <div class="form-row">
                        <div class="form-group">
                            <label for="host">{move || i18n.t().host}</label>
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
                            <label for="port">{move || i18n.t().api_port}</label>
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
                        <span>{move || i18n.t().console}</span>
                    </label>
                    <Show when=move || config.get().console_enable>
                        <div class="form-group">
                            <label for="console-port">{move || i18n.t().console_port}</label>
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
                    {move || {
                        settings_error.get().map(|error| {
                            let failure = error.failure(i18n.t());
                            view! {
                                <div class="alert-error" role="alert">
                                    <p>{failure.reason}</p>
                                    <p>{failure.fix}</p>
                                </div>
                            }
                        })
                    }}
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
                        <h2 id="admin-title">{move || i18n.t().change_admin}</h2>
                        <button
                            type="button"
                            class="icon-button"
                            aria-label=move || i18n.t().close_change_admin
                            on:click=move |_| on_dismiss_admin.run(())
                        >
                            "×"
                        </button>
                    </div>
                    <p class="overlay-note">
                        {move || i18n.t().admin_note}
                    </p>
                    <div class="form-group">
                        <label for="access-key">{move || i18n.t().access_key}</label>
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
                        <label for="secret-key">{move || i18n.t().secret_key}</label>
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
