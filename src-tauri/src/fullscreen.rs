use std::time::Duration;
use tauri::Manager;

#[cfg(target_os = "windows")]
fn foreground_window_is_fullscreen(overlay_hwnd: windows::Win32::Foundation::HWND) -> bool {
    use windows::Win32::Foundation::RECT;
    use windows::Win32::Graphics::Gdi::{
        GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        GetClassNameW, GetForegroundWindow, GetWindowRect,
    };

    unsafe {
        let fg = GetForegroundWindow();
        if fg.0.is_null() || fg == overlay_hwnd {
            return false;
        }

        let mut class_buf = [0u16; 256];
        let len = GetClassNameW(fg, &mut class_buf);
        let class_name = String::from_utf16_lossy(&class_buf[..len as usize]);
        // Desktop/shell windows report as covering the whole monitor too; ignore them.
        if class_name == "Progman" || class_name == "WorkerW" || class_name == "Shell_TrayWnd" {
            return false;
        }

        let mut win_rect = RECT::default();
        if GetWindowRect(fg, &mut win_rect).is_err() {
            return false;
        }

        let monitor = MonitorFromWindow(fg, MONITOR_DEFAULTTONEAREST);
        let mut mi = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if GetMonitorInfoW(monitor, &mut mi).as_bool() == false {
            return false;
        }

        win_rect == mi.rcMonitor
    }
}

#[cfg(not(target_os = "windows"))]
fn foreground_window_is_fullscreen(_overlay_hwnd: ()) -> bool {
    false
}

pub fn spawn_fullscreen_watcher(app: tauri::AppHandle) {
    std::thread::spawn(move || {
        let mut hidden_for_fullscreen = false;
        loop {
            std::thread::sleep(Duration::from_secs(2));
            let Some(window) = app.get_webview_window("overlay") else {
                continue;
            };

            #[cfg(target_os = "windows")]
            let is_fullscreen = {
                let Ok(hwnd) = window.hwnd() else { continue };
                foreground_window_is_fullscreen(windows::Win32::Foundation::HWND(
                    hwnd.0 as *mut _,
                ))
            };
            #[cfg(not(target_os = "windows"))]
            let is_fullscreen = false;

            if is_fullscreen && !hidden_for_fullscreen {
                hidden_for_fullscreen = true;
                let _ = window.hide();
            } else if !is_fullscreen && hidden_for_fullscreen {
                hidden_for_fullscreen = false;
                let _ = window.show();
            }
        }
    });
}
