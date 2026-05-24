mod commands;
mod config;
mod monitor;
mod tray;

use config::store::ConfigStore;
use tauri::Manager;
use tauri::Emitter;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, ShortcutState};

fn position_window_at_bottom(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let window = app.get_webview_window("main").ok_or("main window not found")?;
    let monitor = window.primary_monitor()?.ok_or("no primary monitor")?;
    let screen_size = monitor.size();
    let bar_height = 100i32;
    let y = (screen_size.height as i32).saturating_sub(bar_height);
    window.set_position(tauri::PhysicalPosition::new(0, y))?;
    window.set_size(tauri::PhysicalSize::new(screen_size.width, bar_height as u32))?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config_store = ConfigStore::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        if shortcut.mods == Modifiers::ALT && shortcut.key == Code::Space {
                            tray::toggle_dock_visibility(app);
                        } else if shortcut.mods == Modifiers::CONTROL && shortcut.key == Code::Space {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.emit("toggle-search", ());
                            }
                        }
                    }
                })
                .build(),
        )
        .manage(config_store)
        .setup(|app| {
            if let Err(e) = position_window_at_bottom(app) {
                eprintln!("failed to position window: {e}");
            }

            if let Err(e) = tray::create_tray(app.handle()) {
                eprintln!("failed to create tray: {e}");
            }

            let _ = crate::monitor::events::APP_HANDLE.set(app.handle().clone());
            crate::monitor::events::start_window_event_listener();

            app.handle().plugin(
                tauri_plugin_global_shortcut::Builder::new()
                    .build(),
            ).ok();
            if let Err(e) = app.global_shortcut().register(
                tauri_plugin_global_shortcut::Shortcut::new(Some(Modifiers::ALT), Code::Space)
            ) {
                eprintln!("failed to register shortcut: {e}");
            }
            if let Err(e) = app.global_shortcut().register(
                tauri_plugin_global_shortcut::Shortcut::new(Some(Modifiers::CONTROL), Code::Space)
            ) {
                eprintln!("failed to register Ctrl+Space shortcut: {e}");
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::dock::get_pinned_items,
            commands::dock::add_pinned_item,
            commands::dock::remove_pinned_item,
            commands::dock::reorder_items,
            commands::dock::launch_app,
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::tray::get_active_windows,
            commands::menubar::get_volume,
            commands::menubar::set_volume,
            commands::menubar::get_battery,
            commands::menubar::get_wifi,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
