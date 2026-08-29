use leptos::prelude::*;

#[derive(Clone, PartialEq, Copy)]
pub enum ToastType {
    Success,
    Error,
    Info,
}

#[derive(Clone)]
pub struct ToastMessage {
    pub message: String,
    pub toast_type: ToastType,
    pub id: u64,
}

#[component]
pub fn Toast(
    #[prop(into)] toasts: Signal<Vec<ToastMessage>>,
    #[prop(into)] remove_toast: Callback<u64>,
) -> impl IntoView {
    view! {
        <div class="toast-container" role="status" aria-live="polite" aria-atomic="true">
            <For
                each=move || toasts.get()
                key=|toast| toast.id
                let:toast
            >
                <div
                    class="toast-item"
                    class:success=move || toast.toast_type == ToastType::Success
                    class:error=move || toast.toast_type == ToastType::Error
                    class:info=move || toast.toast_type == ToastType::Info
                    on:animationend=move |_| {
                        // Optional: Remove after animation if handled by CSS,
                        // but we use JS timeout usually.
                        // Here we just let CSS handle entrance.
                    }
                >
                    <span class="toast-icon">
                        {match toast.toast_type {
                            ToastType::Success => "✓",
                            ToastType::Error => "✕",
                            ToastType::Info => "ℹ",
                        }}
                    </span>
                    <span class="toast-text">{toast.message}</span>
                    <button
                        class="toast-close"
                        aria-label="Dismiss notification"
                        on:click=move |_| remove_toast.run(toast.id)
                    >
                        "×"
                    </button>
                </div>
            </For>
        </div>
    }
}
