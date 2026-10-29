use crate::runtime::AppState;
use std::sync::OnceLock;
use tauri::{Emitter, Manager};
use tauri_nspanel::{tauri_panel, ManagerExt, StyleMask, WebviewWindowExt};

tauri_panel! {
    panel!(TimerPanel {
        config: {
            can_become_key_window: true,
            can_become_main_window: false,
            is_floating_panel: true
        }
    })
}
mod theme_icons {
    include!(concat!(env!("OUT_DIR"), "/theme_icons.rs"));
}
static HANDLE: OnceLock<tauri::AppHandle> = OnceLock::new();
extern "C" {
    fn kairomo_update_tray(
        item: *mut std::ffi::c_void,
        label: *const std::ffi::c_char,
        style: i32,
        rest_appearance: i32,
        focus: *const u8,
        focus_len: usize,
        rest: *const u8,
        rest_len: usize,
        palette: *const u8,
    );
}

pub fn update_tray(
    tray: &tauri::tray::TrayIcon,
    label: &str,
    style: i32,
    appearance: &timer_core::Appearance,
    theme_id: &str,
) {
    let rest_appearance = match appearance {
        timer_core::Appearance::System => 0,
        timer_core::Appearance::Light => 1,
        timer_core::Appearance::Dark => 2,
    };
    let theme = timer_core::themes::theme(theme_id);
    let rest_icon = theme_icons::rest_icon(theme_id);
    let palette = crate::platform::theme_palette::menu_palette(theme);
    let Ok(label) = std::ffi::CString::new(label) else {
        return;
    };
    let _ = tray.with_inner_tray_icon(move |tray| {
        if let Some(item) = tray.ns_status_item() {
            let focus = include_bytes!("../../../icons/tray.png");
            let rest = rest_icon;
            // SAFETY: with_inner_tray_icon runs on the main thread. The retained
            // status item and CString outlive this synchronous AppKit call.
            unsafe {
                kairomo_update_tray(
                    &*item as *const _ as *mut std::ffi::c_void,
                    label.as_ptr(),
                    style,
                    rest_appearance,
                    focus.as_ptr(),
                    focus.len(),
                    rest.as_ptr(),
                    rest.len(),
                    palette.as_ptr(),
                );
            }
        }
    });
}
extern "C" {
    fn kairomo_set_window_appearance(window: *mut std::ffi::c_void, mode: i32);
}
pub async fn sync_window_appearance(
    window: tauri::WebviewWindow,
    appearance: timer_core::Appearance,
) -> Result<(), String> {
    let mode = match appearance {
        timer_core::Appearance::System => 0,
        timer_core::Appearance::Light => 1,
        timer_core::Appearance::Dark => 2,
    };
    let (sender, receiver) = std::sync::mpsc::channel();
    let target = window.clone();
    window
        .run_on_main_thread(move || {
            let result = target
                .ns_window()
                .map_err(|error| error.to_string())
                .map(|pointer| {
                    // SAFETY: AppKit runs on the main thread. The captured WebviewWindow
                    // keeps its NSWindow alive for this synchronous call; no pointer is retained.
                    unsafe {
                        kairomo_set_window_appearance(pointer, mode);
                    }
                });
            let _ = sender.send(result);
        })
        .map_err(|error| error.to_string())?;
    tauri::async_runtime::spawn_blocking(move || receiver.recv().map_err(|error| error.to_string()))
        .await
        .map_err(|error| error.to_string())??
}
extern "C" {
    fn kairomo_observe_sleep(callback: extern "C" fn());
    fn kairomo_align_details_titlebar(window: *mut std::ffi::c_void);
}
extern "C" fn suspend() {
    let Some(app) = HANDLE.get() else {
        return;
    };
    let state = app.state::<AppState>();
    match state.0.lock() {
        Ok(mut rt) => match rt.sleep() {
            Ok(()) => {
                let _ = app.emit("timer:changed", &rt.engine.state);
            }
            Err(error) => {
                let _ = app.emit("runtime:error", error);
            }
        },
        Err(_) => {
            let _ = app.emit("runtime:error", "runtime_unavailable");
        }
    };
}
pub fn observe(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("details") {
        if let Ok(pointer) = window.ns_window() {
            // SAFETY: setup runs on AppKit's main thread. The native observer is
            // owned by this NSWindow and holds only a weak reference back to it.
            unsafe { kairomo_align_details_titlebar(pointer) };
        }
    }
    if HANDLE.set(app.clone()).is_ok() {
        // SAFETY: registered once on main thread; static callback and AppHandle
        // outlive the NSWorkspace observer. The C shim does not retain pointers.
        unsafe {
            kairomo_observe_sleep(suspend);
        }
    }
}

#[repr(C)]
#[derive(Default, Debug)]
struct Placement {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    display_id: u32,
}
extern "C" {
    fn kairomo_configure_popup(window: *mut std::ffi::c_void);
    fn kairomo_ensure_popup_visible(
        window: *mut std::ffi::c_void,
        placement: *mut Placement,
    ) -> bool;
}
// These functions are only called on Tauri's main thread; the NSWindow remains
// owned by its WebviewWindow throughout each synchronous AppKit call.
pub fn configure_popup(app: &tauri::AppHandle) -> Result<(), String> {
    let window = app.get_webview_window("main").ok_or("window_unavailable")?;
    let pointer = window.ns_window().map_err(|e| e.to_string())?;
    unsafe {
        kairomo_configure_popup(pointer);
    }
    let panel = window.to_panel::<TimerPanel>().map_err(|e| e.to_string())?;
    panel
        .add_style_mask(StyleMask::empty().nonactivating_panel().into())
        .map_err(|e| e.to_string())?;
    panel.set_hides_on_deactivate(false);
    panel.set_released_when_closed(false);
    super::windows::trace(app, "timer converted to nonactivating floating NSPanel");
    Ok(())
}

pub fn popup_visible(app: &tauri::AppHandle) -> Result<bool, String> {
    app.get_webview_panel("main")
        .map(|panel| panel.is_visible())
        .map_err(|_| "panel_unavailable".into())
}

pub fn show_popup(app: &tauri::AppHandle) -> Result<(), String> {
    let panel = app
        .get_webview_panel("main")
        .map_err(|_| "panel_unavailable")?;
    panel.show_and_make_key();
    Ok(())
}

pub fn hide_popup(app: &tauri::AppHandle) -> Result<(), String> {
    let panel = app
        .get_webview_panel("main")
        .map_err(|_| "panel_unavailable")?;
    panel.hide();
    Ok(())
}
pub fn ensure_popup_visible(
    app: &tauri::AppHandle,
    window: &tauri::WebviewWindow,
) -> Result<(), String> {
    let pointer = window.ns_window().map_err(|e| e.to_string())?;
    let mut placement = Placement::default();
    if !unsafe { kairomo_ensure_popup_visible(pointer, &mut placement) } {
        return Err("pointer_display_unavailable".into());
    }
    super::windows::trace(app, &format!("position {placement:?}"));
    Ok(())
}

#[cfg(test)]
mod popup_tests {
    use super::Placement;
    extern "C" {
        fn kairomo_popup_grip_visible(frame: Placement, area: Placement) -> bool;
        fn kairomo_popup_geometry(
            area: Placement,
            mouse_x: f64,
            width: f64,
            height: f64,
        ) -> Placement;
    }
    #[test]
    fn external_display_above_laptop_keeps_its_logical_origin() {
        let area = Placement {
            x: 0.0,
            y: 900.0,
            width: 2560.0,
            height: 1415.0,
            display_id: 2,
        };
        // Pure geometry function; no Cocoa objects or UI are accessed by this test.
        let p = unsafe { kairomo_popup_geometry(area, 2520.0, 360.0, 280.0) };
        assert_eq!((p.x, p.y, p.display_id), (2200.0, 2029.0, 2));
    }
    #[test]
    fn display_left_of_laptop_clamps_inside_negative_coordinates() {
        let area = Placement {
            x: -1920.0,
            y: 0.0,
            width: 1920.0,
            height: 1055.0,
            display_id: 3,
        };
        let p = unsafe { kairomo_popup_geometry(area, -1800.0, 360.0, 280.0) };
        assert_eq!((p.x, p.y, p.display_id), (-1920.0, 769.0, 3));
    }
    #[test]
    fn remembered_position_is_kept_on_an_existing_external_screen() {
        let frame = Placement {
            x: -1700.0,
            y: 400.0,
            width: 360.0,
            height: 280.0,
            display_id: 3,
        };
        let area = Placement {
            x: -1920.0,
            y: 0.0,
            width: 1920.0,
            height: 1055.0,
            display_id: 3,
        };
        assert!(unsafe { kairomo_popup_grip_visible(frame, area) });
    }
    #[test]
    fn disconnected_external_screen_requires_recovery() {
        let frame = Placement {
            x: -1700.0,
            y: 400.0,
            width: 360.0,
            height: 280.0,
            display_id: 3,
        };
        let laptop = Placement {
            x: 0.0,
            y: 0.0,
            width: 1512.0,
            height: 944.0,
            display_id: 1,
        };
        assert!(!unsafe { kairomo_popup_grip_visible(frame, laptop) });
    }
}
