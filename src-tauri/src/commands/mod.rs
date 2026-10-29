use crate::runtime::AppState;
use tauri::{Emitter, Manager, State};
use timer_core::{Phase, Repository, Session, Settings, Snapshot};
#[tauri::command]
pub async fn sync_window_appearance(
    window: tauri::WebviewWindow,
    appearance: timer_core::Appearance,
) -> Result<(), String> {
    crate::platform::sync_window_appearance(window, appearance).await
}
#[tauri::command]
pub fn snapshot(state: State<'_, AppState>) -> Result<Snapshot, String> {
    Ok(state
        .0
        .lock()
        .map_err(|_| "runtime_unavailable")?
        .engine
        .state
        .clone())
}
#[tauri::command]
pub fn timer_action(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    action: String,
    title: Option<String>,
    expected_session_id: Option<String>,
    expected_phase: Phase,
) -> Result<Snapshot, String> {
    let snapshot = state
        .0
        .lock()
        .map_err(|_| "runtime_unavailable")?
        .action_for(
            &action,
            title.as_deref().unwrap_or(""),
            expected_session_id.as_deref(),
            expected_phase,
        )?;
    let _ = app.emit("timer:changed", &snapshot);
    Ok(snapshot)
}
#[tauri::command]
pub fn cancel_session(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    expected_id: String,
) -> Result<Snapshot, String> {
    let snapshot = state
        .0
        .lock()
        .map_err(|_| "runtime_unavailable")?
        .cancel_session(&expected_id)?;
    let _ = app.emit("timer:changed", &snapshot);
    Ok(snapshot)
}
#[tauri::command]
pub fn sessions(state: State<'_, AppState>) -> Result<Vec<Session>, String> {
    state
        .0
        .lock()
        .map_err(|_| "runtime_unavailable")?
        .repository
        .sessions()
        .map_err(|e| e.to_string())
}
#[tauri::command]
pub fn save_settings(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    settings: Settings,
) -> Result<Snapshot, String> {
    let snapshot = state
        .0
        .lock()
        .map_err(|_| "runtime_unavailable")?
        .configure(settings)?;
    let _ = app.emit("timer:changed", &snapshot);
    Ok(snapshot)
}
#[tauri::command]
pub fn save_theme_options(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    theme_id: String,
    values: serde_json::Map<String, serde_json::Value>,
) -> Result<Snapshot, String> {
    let snapshot = state
        .0
        .lock()
        .map_err(|_| "runtime_unavailable")?
        .theme_options(&theme_id, values)?;
    let _ = app.emit("timer:changed", &snapshot);
    Ok(snapshot)
}
#[tauri::command]
pub fn save_bean_range(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    range: timer_core::BeanRange,
) -> Result<Snapshot, String> {
    let snapshot = state
        .0
        .lock()
        .map_err(|_| "runtime_unavailable")?
        .bean_range(range)?;
    let _ = app.emit("timer:changed", &snapshot);
    Ok(snapshot)
}
#[tauri::command]
pub fn save_phrases(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    phrases: timer_core::Phrases,
) -> Result<Snapshot, String> {
    let snapshot = state
        .0
        .lock()
        .map_err(|_| "runtime_unavailable")?
        .phrases(phrases)?;
    let _ = app.emit("timer:changed", &snapshot);
    Ok(snapshot)
}
#[tauri::command]
pub fn save_preferences(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    locale: Option<String>,
    appearance: Option<timer_core::Appearance>,
    visual_series: Option<String>,
) -> Result<Snapshot, String> {
    let snapshot = state
        .0
        .lock()
        .map_err(|_| "runtime_unavailable")?
        .preferences(locale, appearance, visual_series)?;
    let _ = app.emit("timer:changed", &snapshot);
    Ok(snapshot)
}
#[tauri::command]
pub fn delete_session(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state
        .0
        .lock()
        .map_err(|_| "runtime_unavailable")?
        .repository
        .delete_session(&id)
        .map_err(|e| e.to_string())
}
#[tauri::command]
pub fn correct_session(
    state: State<'_, AppState>,
    id: String,
    title: String,
    minutes: f64,
) -> Result<(), String> {
    state
        .0
        .lock()
        .map_err(|_| "runtime_unavailable")?
        .correct(&id, &title, minutes)
}
#[tauri::command]
pub fn export_csv(state: State<'_, AppState>, path: String) -> Result<(), String> {
    state
        .0
        .lock()
        .map_err(|_| "runtime_unavailable")?
        .export(&path)
}

#[tauri::command]
pub fn open_details(app: tauri::AppHandle, view: String) -> Result<(), String> {
    crate::platform::windows::open_details(&app, &view)
}
#[tauri::command]
pub fn hide_popup(app: tauri::AppHandle) {
    crate::platform::windows::hide(&app);
}

#[tauri::command]
pub fn details_view() -> String {
    crate::platform::windows::details_view()
}

/// Only the timer can select a known fixed layout; no general window-control capability is exposed.
#[tauri::command]
pub fn resize_popup(window: tauri::WebviewWindow, height: u32) -> Result<(), String> {
    if window.label() != "main" || !matches!(height, 284 | 302 | 320) {
        return Err("invalid_timer_layout".into());
    }
    window
        .set_size(tauri::LogicalSize::new(360.0, f64::from(height)))
        .map_err(|error| error.to_string())
}

/// The details webview cannot release the timer's startup gate.
#[tauri::command]
pub fn popup_prepared(window: tauri::WebviewWindow) -> Result<(), String> {
    if window.label() != "main" {
        return Err("invalid_timer_window".into());
    }
    crate::platform::windows::prepared(window.app_handle())
}
