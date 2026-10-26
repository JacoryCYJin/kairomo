#[cfg(target_os = "macos")]
mod macos;
use crate::runtime::AppState;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager,
};
use tauri_plugin_notification::NotificationExt;
use timer_core::{Kind, Phase, Snapshot};
pub mod windows;
pub async fn sync_window_appearance(
    window: tauri::WebviewWindow,
    appearance: timer_core::Appearance,
) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    return macos::sync_window_appearance(window, appearance).await;
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (window, appearance);
        Ok(())
    }
}
pub fn show(app: &tauri::AppHandle) {
    windows::show(app);
}
pub fn setup(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let open = MenuItem::with_id(app, "open", "打开 Kairomo / Open", true, None::<&str>)?;
    let pause = MenuItem::with_id(
        app,
        "toggle",
        "暂停 / 继续 · Pause / Resume",
        true,
        None::<&str>,
    )?;
    let quit = MenuItem::with_id(app, "quit", "退出 Kairomo / Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &pause, &quit])?;
    let mut tray = TrayIconBuilder::with_id("timer")
        .tooltip("Kairomo — Time for what matters")
        .menu(&menu)
        .show_menu_on_left_click(false);
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    #[cfg(target_os = "macos")]
    {
        tray = tray
            .icon(tauri::include_image!("icons/tray.png"))
            .icon_as_template(true);
    }
    tray.on_menu_event(|app, event| match event.id.as_ref() {
        "open" => show(app),
        "toggle" => {
            let state = app.state::<AppState>();
            let result = state
                .0
                .lock()
                .map_err(|_| "runtime_unavailable".to_string())
                .and_then(|mut rt| {
                    let action = match rt.engine.state.phase {
                        Phase::Running => "pause",
                        Phase::Paused => "resume",
                        Phase::Idle => return Ok(rt.engine.state.clone()),
                    };
                    rt.action(action, "")
                });
            match result {
                Ok(s) => {
                    let _ = app.emit("timer:changed", s);
                }
                Err(e) => {
                    let _ = app.emit("runtime:error", e);
                    show(app);
                }
            }
        }
        "quit" => app.exit(0),
        _ => (),
    })
    .on_tray_icon_event(|tray, event| {
        if let TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        } = event
        {
            windows::trace(tray.app_handle(), "tray left release");
            windows::toggle(tray.app_handle());
        }
    })
    .build(app)?;
    #[cfg(target_os = "macos")]
    {
        macos::observe(app);
        macos::configure_popup(app).map_err(std::io::Error::other)?;
    }
    Ok(())
}
pub fn update(app: &tauri::AppHandle, state: &Snapshot, reached: bool) {
    if let Some(tray) = app.tray_by_id("timer") {
        let resting = state
            .current
            .as_ref()
            .is_some_and(|session| session.kind == Kind::Break);
        #[cfg(target_os = "macos")]
        update_tray_icon(&tray, resting);
        let label = if let Some(s) = &state.current {
            let seconds = if s.duration_ms >= s.target_ms {
                (s.duration_ms - s.target_ms) / 1000
            } else {
                (s.target_ms - s.duration_ms).div_ceil(1000)
            };
            format!(
                "{}{:02}:{:02}",
                if s.duration_ms >= s.target_ms {
                    "+"
                } else {
                    ""
                },
                seconds / 60,
                seconds % 60
            )
        } else {
            format!("{:02}:00", state.settings.focus_minutes)
        };
        #[cfg(target_os = "macos")]
        let _ = tray.set_title(Some(&label));
        let stage = match (resting, state.settings.locale.as_str()) {
            (true, "zh-CN") => "休息",
            (false, "zh-CN") => "专注",
            (true, _) => "Rest",
            (false, _) => "Focus",
        };
        let _ = tray.set_tooltip(Some(format!("Kairomo · {stage} · {label}")));
    }
    if reached && state.settings.notifications {
        let focus = state
            .current
            .as_ref()
            .is_some_and(|s| s.kind == Kind::Focus);
        let zh = state.settings.locale == "zh-CN";
        let body = match (focus, zh) {
            (true, true) => "已达到专注目标。继续计时，准备好后再休息。",
            (false, true) => "休息目标已到。准备好后再开始专注。",
            (true, false) => "Focus target reached. Still tracking — take a break when ready.",
            (false, false) => "Break target reached. Resume focus when you are ready.",
        };
        if let Err(error) = app
            .notification()
            .builder()
            .title("Kairomo")
            .body(body)
            .show()
        {
            let _ = app.emit("notification:error", error.to_string());
        }
    }
}

#[cfg(target_os = "macos")]
fn update_tray_icon(tray: &tauri::tray::TrayIcon, resting: bool) {
    static LAST_STYLE: std::sync::Mutex<Option<bool>> = std::sync::Mutex::new(None);
    let Ok(mut previous) = LAST_STYLE.lock() else {
        return;
    };
    if *previous == Some(resting) {
        return;
    }
    let icon = if resting {
        tauri::include_image!("icons/tray-rest.png")
    } else {
        tauri::include_image!("icons/tray.png")
    };
    // Templates follow macOS menu-bar contrast; the rest icon retains its amber color.
    if tray.set_icon_with_as_template(Some(icon), !resting).is_ok() {
        *previous = Some(resting);
    }
}
