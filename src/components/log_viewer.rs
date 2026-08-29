use crate::types::{LogEntry, LogType};
use leptos::prelude::*;
use std::collections::VecDeque;

#[component]
pub fn LogViewer(
    #[prop(into)] app_logs: Signal<VecDeque<LogEntry>>,
    #[prop(into)] set_app_logs: WriteSignal<VecDeque<LogEntry>>,
    #[prop(into)] rustfs_logs: Signal<VecDeque<LogEntry>>,
    #[prop(into)] set_rustfs_logs: WriteSignal<VecDeque<LogEntry>>,
    #[prop(into)] current_log_type: Signal<LogType>,
    #[prop(into)] set_current_log_type: WriteSignal<LogType>,
) -> impl IntoView {
    let (auto_scroll, set_auto_scroll) = signal(true);
    let logs_ref = NodeRef::<leptos::html::Div>::new();

    // Auto-scroll effect
    Effect::new(move |_| {
        // Track log changes
        let _ = app_logs.get();
        let _ = rustfs_logs.get();

        if auto_scroll.get() {
            if let Some(element) = logs_ref.get() {
                request_animation_frame(move || {
                    element.set_scroll_top(element.scroll_height());
                });
            }
        }
    });

    let clear_logs = move |_| {
        set_app_logs.set(VecDeque::new());
        set_rustfs_logs.set(VecDeque::new());
    };

    view! {
        <div class="log-panel">
            <div class="log-header">
                <div class="log-tabs" role="tablist" aria-label="Log sources">
                    <button
                        class="log-tab"
                        class:active=move || current_log_type.get() == LogType::App
                        role="tab"
                        aria-selected=move || current_log_type.get() == LogType::App
                        on:click=move |_| set_current_log_type.set(LogType::App)
                    >
                        "App Logs"
                    </button>
                    <button
                        class="log-tab"
                        class:active=move || current_log_type.get() == LogType::RustFS
                        role="tab"
                        aria-selected=move || current_log_type.get() == LogType::RustFS
                        on:click=move |_| set_current_log_type.set(LogType::RustFS)
                    >
                        "RustFS Output"
                    </button>
                </div>
                <div class="log-actions">
                    <label class="auto-scroll-toggle">
                        <input
                            type="checkbox"
                            prop:checked=move || auto_scroll.get()
                            on:change=move |ev| set_auto_scroll.set(event_target_checked(&ev))
                        />
                        "Auto-scroll"
                    </label>
                    <button class="clear-btn" on:click=clear_logs title="Clear Logs">
                        "Clear"
                    </button>
                </div>
            </div>
            <div class="log-output" node_ref=logs_ref>
                <For
                    each=move || {
                        match current_log_type.get() {
                            LogType::App => app_logs.get(),
                            LogType::RustFS => rustfs_logs.get(),
                        }
                        .into_iter()
                        .collect::<Vec<_>>()
                    }
                    key=|log| log.id
                    let:log
                >
                    <div class="log-line">{log.message}</div>
                </For>
                <Show when=move || {
                    match current_log_type.get() {
                        LogType::App => app_logs.get().is_empty(),
                        LogType::RustFS => rustfs_logs.get().is_empty(),
                    }
                }>
                    <div class="empty-logs">"No logs available"</div>
                </Show>
            </div>
        </div>
    }
}
