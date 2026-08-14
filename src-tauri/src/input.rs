use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tauri::{Emitter, Manager};

/// While an in-webview affordance that outlives a single click (the
/// settings menu) is open, the window must stay interactive even if Alt
/// gets released mid-interaction — Windows synthesizes an Alt-up the moment
/// certain UI interactions start, well before the user's finger actually
/// leaves the key. Without this, re-enabling click-through mid-interaction
/// would yank mouse input away from whatever is still open in the webview.
static INTERACTION_PINNED: AtomicBool = AtomicBool::new(false);

pub fn pin_interactive(pinned: bool) {
    INTERACTION_PINNED.store(pinned, Ordering::SeqCst);
}

#[cfg(target_os = "windows")]
fn alt_is_down() -> bool {
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_MENU};
    // High-order bit set means the key is currently down.
    unsafe { (GetAsyncKeyState(VK_MENU.0 as i32) as u16) & 0x8000 != 0 }
}

#[cfg(not(target_os = "windows"))]
fn alt_is_down() -> bool {
    false
}

/// Polls the Alt key and toggles the overlay window between click-through
/// (default) and interactive (while Alt is held) so the HUD never steals
/// clicks from whatever is underneath it.
pub fn spawn_unlock_watcher(app: tauri::AppHandle) {
    std::thread::spawn(move || {
        let mut unlocked = false;
        loop {
            std::thread::sleep(Duration::from_millis(60));
            let down = alt_is_down();
            if down == unlocked {
                continue;
            }
            // Alt was released while a pinned affordance (the settings
            // menu) is open: keep the window interactive until it closes.
            if !down && INTERACTION_PINNED.load(Ordering::SeqCst) {
                continue;
            }
            unlocked = down;
            let Some(window) = app.get_webview_window("overlay") else {
                continue;
            };
            let _ = window.set_ignore_cursor_events(!unlocked);
            let _ = app.emit("unlock-changed", unlocked);
        }
    });
}
