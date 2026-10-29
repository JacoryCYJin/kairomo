mod commands;
mod platform;
mod runtime;
mod updates;
use runtime::{AppState, Runtime};
use std::{sync::Mutex, time::Duration};
use tauri::{Emitter, Manager};

pub fn run() {
    let builder = tauri::Builder::default();
    #[cfg(target_os = "macos")]
    let builder = builder.plugin(tauri_nspanel::init());
    let app = builder
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            platform::show(app)
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_updater::Builder::new()
                .pubkey(include_str!("../updater.pub").trim())
                .build(),
        )
        .plugin(tauri_plugin_notification::init())
        .manage(updates::Updates::default())
        .manage(platform::popup_startup::PopupStartup::default())
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            let path = app.path().app_data_dir()?;
            std::fs::create_dir_all(&path)?;
            let runtime =
                Runtime::open(&path.join("kairomo.sqlite3")).map_err(std::io::Error::other)?;
            app.manage(AppState(Mutex::new(runtime)));
            platform::setup(app.handle())?;
            let handle = app.handle().clone();
            std::thread::spawn(move || loop {
                std::thread::sleep(Duration::from_secs(1));
                let state = handle.state::<AppState>();
                let result = state
                    .0
                    .lock()
                    .map_err(|_| "runtime_unavailable".to_string())
                    .and_then(|mut rt| rt.tick());
                match result {
                    Ok((snapshot, reached)) => {
                        platform::update(&handle, &snapshot, reached);
                        let _ = handle.emit("timer:changed", snapshot);
                    }
                    Err(error) => {
                        eprintln!("Kairomo checkpoint failed: {error}");
                        let _ = handle.emit("runtime:error", error);
                    }
                }
            });
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == "main" {
                if let tauri::WindowEvent::Resized(size) = event {
                    platform::windows::trace(window.app_handle(), &format!("resized {size:?}"));
                }
                if let tauri::WindowEvent::Moved(position) = event {
                    platform::windows::trace(window.app_handle(), &format!("moved {position:?}"));
                }
                if let tauri::WindowEvent::Focused(focused) = event {
                    platform::windows::trace(window.app_handle(), &format!("focus {focused}"));
                }
            }
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                if window.label() == "main" {
                    platform::windows::hide(window.app_handle());
                } else {
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::details_view,
            commands::open_details,
            commands::hide_popup,
            commands::resize_popup,
            commands::popup_prepared,
            commands::snapshot,
            commands::timer_action,
            commands::cancel_session,
            commands::sessions,
            commands::save_settings,
            commands::save_preferences,
            commands::save_phrases,
            commands::save_bean_range,
            commands::save_theme_options,
            commands::sync_window_appearance,
            commands::delete_session,
            commands::correct_session,
            commands::export_csv,
            updates::release_info,
            updates::check_update,
            updates::install_update
        ])
        .build(tauri::generate_context!())
        .expect("Unable to start Kairomo; check app-data access and SQLite integrity");
    app.run(|handle, event| {
        if let tauri::RunEvent::ExitRequested { ref api, .. } = event {
            if let Some(state) = handle.try_state::<AppState>() {
                let result = state
                    .0
                    .lock()
                    .map_err(|_| "runtime_unavailable".to_string())
                    .and_then(|mut rt| rt.shutdown());
                if let Err(e) = result {
                    api.prevent_exit();
                    let _ = handle.emit("runtime:error", e);
                    platform::show(handle);
                }
            }
        }
        #[cfg(target_os = "macos")]
        if let tauri::RunEvent::Reopen { .. } = event {
            platform::show(handle);
        }
    });
}
