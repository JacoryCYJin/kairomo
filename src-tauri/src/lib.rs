mod commands;
mod platform;
mod runtime;
use runtime::{AppState, Runtime};
use std::{sync::Mutex, time::Duration};
use tauri::{Emitter, Manager};

pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            platform::show(app)
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
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
                if let tauri::WindowEvent::Moved(position) = event {
                    platform::windows::trace(window.app_handle(), &format!("moved {position:?}"));
                }
                if let tauri::WindowEvent::Focused(focused) = event {
                    platform::windows::trace(window.app_handle(), &format!("focus {focused}"));
                }
            }
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::details_view,
            commands::open_details,
            commands::hide_popup,
            commands::snapshot,
            commands::timer_action,
            commands::sessions,
            commands::save_settings,
            commands::delete_session,
            commands::correct_session,
            commands::export_csv
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
