mod language;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(any(target_os = "macos", test))]
mod theme_palette;
use crate::runtime::AppState;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager,
};
use tauri_plugin_notification::NotificationExt;
use timer_core::{Kind, Phase, Snapshot, TargetReached};
pub mod popup_startup;
pub mod windows;
struct TrayMenu {
    open: MenuItem<tauri::Wry>,
    toggle: MenuItem<tauri::Wry>,
    quit: MenuItem<tauri::Wry>,
    locale: std::sync::Mutex<String>,
}
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
    let name = app.config().product_name.as_deref().unwrap_or("Kairomo");
    let locale = app
        .state::<AppState>()
        .0
        .lock()
        .map_err(|_| std::io::Error::other("runtime_unavailable"))?
        .engine
        .state
        .settings
        .locale
        .clone();
    let copy = language::text(&locale);
    let open = MenuItem::with_id(
        app,
        "open",
        copy.open.replace("{app}", name),
        true,
        None::<&str>,
    )?;
    let pause = MenuItem::with_id(app, "toggle", copy.pause_resume, true, None::<&str>)?;
    let quit = MenuItem::with_id(
        app,
        "quit",
        copy.quit.replace("{app}", name),
        true,
        None::<&str>,
    )?;
    let menu = Menu::with_items(app, &[&open, &pause, &quit])?;
    app.manage(TrayMenu {
        open,
        toggle: pause,
        quit,
        locale: std::sync::Mutex::new(locale),
    });
    let mut tray = TrayIconBuilder::with_id("timer")
        .tooltip(format!("{name} — Time for what matters"))
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
        // Lay out the fixed-width menu-bar item before the first one-second tick.
        let snapshot = app
            .state::<AppState>()
            .0
            .lock()
            .map_err(|_| std::io::Error::other("runtime_unavailable"))?
            .engine
            .state
            .clone();
        update(app, &snapshot, None);
    }
    Ok(())
}
pub fn update(app: &tauri::AppHandle, state: &Snapshot, reached: Option<TargetReached>) {
    if let Some(tray) = app.tray_by_id("timer") {
        let copy = language::text(&state.settings.locale);
        if let Some(menu) = app.try_state::<TrayMenu>() {
            if let Ok(mut locale) = menu.locale.lock() {
                if *locale != state.settings.locale {
                    let name = app.config().product_name.as_deref().unwrap_or("Kairomo");
                    let updated = menu
                        .open
                        .set_text(copy.open.replace("{app}", name))
                        .and_then(|_| menu.toggle.set_text(copy.pause_resume))
                        .and_then(|_| menu.quit.set_text(copy.quit.replace("{app}", name)));
                    if updated.is_ok() {
                        *locale = state.settings.locale.clone();
                    }
                }
            }
        }
        let resting = state
            .current
            .as_ref()
            .is_some_and(|session| session.kind == Kind::Break);
        let overtime = state
            .current
            .as_ref()
            .is_some_and(|session| session.duration_ms >= session.target_ms);
        let label = if let Some(s) = &state.current {
            let seconds = if s.duration_ms >= s.target_ms {
                (s.duration_ms - s.target_ms) / 1000
            } else {
                (s.target_ms - s.duration_ms).div_ceil(1000)
            };
            format!("{}{}", if overtime { "+" } else { "" }, menu_time(seconds))
        } else {
            menu_time(u64::from(state.settings.focus_minutes) * 60)
        };
        #[cfg(target_os = "macos")]
        macos::update_tray(
            &tray,
            label.trim_start_matches('+'),
            if resting {
                1
            } else if overtime {
                2
            } else {
                0
            },
            &state.settings.appearance,
            &state.settings.visual_series,
        );
        let stage = if resting { copy.rest } else { copy.focus };
        let _ = tray.set_tooltip(Some(format!("Kairomo · {stage} · {label}")));
    }
    if let Some(event) = reached.filter(|_| state.settings.notifications) {
        let focus = event.kind == Kind::Focus;
        let completed = event.completed;
        let copy = language::text(&state.settings.locale);
        let body = if !focus {
            copy.break_done
        } else if completed {
            copy.focus_done
        } else {
            copy.focus_target
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

// This cap belongs only to menu-bar presentation, never the engine's duration.
fn menu_time(seconds: u64) -> String {
    let seconds = seconds.min(99 * 3600 + 59 * 60 + 59);
    if seconds < 3600 {
        format!("{:02}:{:02}", seconds / 60, seconds % 60)
    } else {
        format!(
            "{:02}:{:02}:{:02}",
            seconds / 3600,
            seconds / 60 % 60,
            seconds % 60
        )
    }
}

#[cfg(test)]
mod menu_time_tests {
    use super::menu_time;

    #[test]
    fn menu_time_switches_at_one_hour_and_clamps_only_its_output() {
        for (seconds, expected) in [
            (0, "00:00"),
            (599, "09:59"),
            (3599, "59:59"),
            (3600, "01:00:00"),
            (3601, "01:00:01"),
            (359_999, "99:59:59"),
            (360_000, "99:59:59"),
            (u64::MAX, "99:59:59"),
        ] {
            assert_eq!(menu_time(seconds), expected);
        }
    }
}
