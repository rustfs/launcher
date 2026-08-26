mod commands;
mod config;
mod error;
mod process;
mod state;

use state::{add_app_log, set_app_handle, terminate_rustfs_process};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    Manager, WindowEvent,
};

fn focus_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();
    log::info!("Starting RustFS Launcher");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_single_instance::init(
            |app, _arguments, _cwd| {
                focus_main_window(app);
            },
        ))
        .setup(|app| {
            set_app_handle(app.handle().clone());
            add_app_log("RustFS Launcher started".to_string());

            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

            let _tray = TrayIconBuilder::with_id("rustfs-tray")
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        add_app_log("Quit requested from tray, terminating...".to_string());
                        terminate_rustfs_process();
                        app.exit(0);
                    }
                    "show" => focus_main_window(app),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                focus_main_window(app);
                            }
                        }
                    }
                })
                .build(app)?;

            if app.get_webview_window("main").is_some() {
                focus_main_window(app.handle());
                log::info!("Main window shown and focused");
            } else {
                log::warn!("Main window not found");
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::launch_rustfs,
            commands::stop_rustfs,
            commands::validate_config,
            commands::get_app_logs,
            commands::get_rustfs_logs,
            commands::diagnose_rustfs_binary,
            commands::check_tcp_connection,
            commands::is_rustfs_process_running,
            commands::get_runtime_status,
            commands::get_app_version_info,
            commands::open_service_url,
            commands::check_for_update,
            commands::install_update
        ])
        .build(tauri::generate_context!())
        .expect("error building tauri application")
        .run(|app_handle, event| match event {
            tauri::RunEvent::ExitRequested { .. } | tauri::RunEvent::Exit => {
                state::terminate_rustfs_process();
            }
            #[cfg(target_os = "macos")]
            tauri::RunEvent::Reopen { .. } => {
                focus_main_window(app_handle);
            }
            _ => {
                let _ = app_handle;
            }
        });
}
