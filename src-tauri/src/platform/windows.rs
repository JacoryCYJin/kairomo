use std::sync::Mutex;
#[cfg(any(not(target_os = "macos"), test))]
use tauri::PhysicalPosition;
use tauri::{Emitter, Manager};

static DETAILS_VIEW: Mutex<&str> = Mutex::new("settings");
pub fn details_view() -> String {
    DETAILS_VIEW
        .lock()
        .map(|v| v.to_string())
        .unwrap_or_else(|_| "settings".into())
}

pub fn trace(app: &tauri::AppHandle, message: &str) {
    #[cfg(debug_assertions)]
    {
        use std::io::Write;
        static LOG_LOCK: Mutex<()> = Mutex::new(());
        let Ok(_guard) = LOG_LOCK.lock() else {
            return;
        };
        let Ok(directory) = app.path().app_log_dir() else {
            return;
        };
        if std::fs::create_dir_all(&directory).is_err() {
            return;
        }
        let path = directory.join("window.log");
        let append = std::fs::metadata(&path).map_or(true, |m| m.len() < 131072);
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(append)
            .truncate(!append)
            .open(path)
        {
            let stamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.as_millis());
            let _ = writeln!(file, "{stamp} {message}");
        }
    }
    #[cfg(not(debug_assertions))]
    let _ = (app, message);
}
fn report(app: &tauri::AppHandle, result: Result<(), String>) {
    if let Err(error) = result {
        trace(app, &format!("window error: {error}"));
        eprintln!("Kairomo window error: {error}");
        let _ = app.emit("window:error", error);
    }
}
fn dispatch(app: &tauri::AppHandle, operation: fn(&tauri::AppHandle) -> Result<(), String>) {
    let handle = app.clone();
    if let Err(error) = app.run_on_main_thread(move || report(&handle, operation(&handle))) {
        report(app, Err(error.to_string()));
    }
}
pub fn hide(app: &tauri::AppHandle) {
    dispatch(app, hide_on_main);
}
fn hide_on_main(app: &tauri::AppHandle) -> Result<(), String> {
    let window = app.get_webview_window("main").ok_or("window_unavailable")?;
    window.hide().map_err(|e| e.to_string())?;
    trace(app, "hidden explicitly");
    Ok(())
}
pub fn toggle(app: &tauri::AppHandle) {
    dispatch(app, toggle_on_main);
}
fn toggle_on_main(app: &tauri::AppHandle) -> Result<(), String> {
    let window = app.get_webview_window("main").ok_or("window_unavailable")?;
    let visible = window.is_visible().map_err(|e| e.to_string())?;
    trace(app, &format!("toggle visible={visible}"));
    if visible {
        hide_on_main(app)
    } else {
        show_on_main(app)
    }
}
pub fn show(app: &tauri::AppHandle) {
    dispatch(app, show_on_main);
}
fn show_on_main(app: &tauri::AppHandle) -> Result<(), String> {
    let window = app.get_webview_window("main").ok_or("window_unavailable")?;
    #[cfg(target_os = "macos")]
    super::macos::ensure_popup_visible(app, &window)?;
    #[cfg(not(target_os = "macos"))]
    {
        // Preserve the user's position. Recover only if the window has become off-screen.
        let position = window.outer_position().map_err(|e| e.to_string())?;
        let size = window.outer_size().map_err(|e| e.to_string())?;
        let monitors = app.available_monitors().map_err(|e| e.to_string())?;
        let visible = monitors.iter().any(|monitor| {
            let area = monitor.work_area();
            position.x + size.width as i32 > area.position.x
                && position.x < area.position.x + area.size.width as i32
                && position.y + size.height as i32 > area.position.y
                && position.y < area.position.y + area.size.height as i32
        });
        if !visible {
            let pointer = app.cursor_position().map_err(|e| e.to_string())?;
            let monitor = app
                .monitor_from_point(pointer.x, pointer.y)
                .map_err(|e| e.to_string())?
                .or(app.primary_monitor().map_err(|e| e.to_string())?)
                .ok_or("display_unavailable")?;
            let area = monitor.work_area();
            let point = popup_position(
                (pointer.x as i32, pointer.y as i32),
                (0, 0),
                monitor.scale_factor(),
                (
                    area.position.x,
                    area.position.y,
                    area.size.width,
                    area.size.height,
                ),
            );
            window.set_position(point).map_err(|e| e.to_string())?;
        }
    }
    window.show().map_err(|e| e.to_string())?;
    window.set_focus().map_err(|e| e.to_string())?;
    window.emit("timer:opened", ()).map_err(|e| e.to_string())?;
    trace(
        app,
        "shown; neutral focus requested; remains visible when focus changes",
    );
    Ok(())
}
pub fn open_details(app: &tauri::AppHandle, view: &str) -> Result<(), String> {
    if !matches!(view, "history" | "settings") {
        return Err("invalid_view".into());
    }
    *DETAILS_VIEW.lock().map_err(|_| "window_unavailable")? = if view == "history" {
        "history"
    } else {
        "settings"
    };
    let window = app
        .get_webview_window("details")
        .ok_or("window_unavailable")?;
    window
        .emit("details:view", view)
        .map_err(|e| e.to_string())?;
    hide(app);
    window.show().map_err(|e| e.to_string())?;
    window.unminimize().map_err(|e| e.to_string())?;
    window.set_focus().map_err(|e| e.to_string())
}

#[cfg(any(not(target_os = "macos"), test))]
fn popup_position(
    anchor: (i32, i32),
    icon: (u32, u32),
    scale: f64,
    area: (i32, i32, u32, u32),
) -> PhysicalPosition<i32> {
    let width = (360.0 * scale) as i32;
    let height = (280.0 * scale) as i32;
    let gap = (6.0 * scale) as i32;
    let right = area.0 + area.2 as i32;
    let bottom = area.1 + area.3 as i32;
    let x = (anchor.0 + icon.0 as i32 / 2 - width / 2).clamp(area.0, (right - width).max(area.0));
    let below = anchor.1 + icon.1 as i32 + gap;
    let y = if below + height <= bottom {
        below.max(area.1)
    } else {
        (anchor.1 - height - gap).max(area.1)
    };
    PhysicalPosition::new(x, y)
}

#[cfg(test)]
mod tests {
    use super::popup_position;
    #[test]
    fn retina_menu_popup_stays_inside_right_edge() {
        let p = popup_position((2800, 0), (80, 48), 2.0, (0, 48, 2880, 1752));
        assert_eq!((p.x, p.y), (2160, 60));
    }
    #[test]
    fn bottom_tray_opens_above_on_negative_origin_monitor() {
        let p = popup_position((-50, 1050), (40, 30), 1.0, (-1920, 0, 1920, 1050));
        assert_eq!((p.x, p.y), (-360, 764));
    }
}
