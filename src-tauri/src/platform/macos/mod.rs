use crate::runtime::AppState;
use std::sync::OnceLock;
use tauri::{Emitter, Manager};
static HANDLE: OnceLock<tauri::AppHandle> = OnceLock::new();
extern "C" {
    fn kairomo_observe_sleep(callback: extern "C" fn());
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
