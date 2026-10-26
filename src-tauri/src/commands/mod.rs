use crate::runtime::AppState;
use tauri::{Emitter, State};
use timer_core::{Repository, Session, Settings, Snapshot};
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
) -> Result<Snapshot, String> {
    let snapshot = state
        .0
        .lock()
        .map_err(|_| "runtime_unavailable")?
        .action(&action, title.as_deref().unwrap_or(""))?;
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
