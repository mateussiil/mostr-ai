mod claude;
mod codex;
mod config;
mod cursor;
mod fullscreen;
mod input;
mod menu;
mod usage;

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tauri::{Emitter, Manager, WindowEvent};
use tauri_plugin_autostart::MacosLauncher;
use usage::UsagePayload;

const MARGIN: i32 = 24;
const REFRESH_INTERVAL: Duration = Duration::from_secs(5 * 60);

/// Set right before a programmatic set_size/set_position (from resize_panel
/// or the initial placement) so the Moved handler below doesn't mistake it
/// for a user drag and persist it as the saved position.
static PROGRAMMATIC_MOVE: AtomicBool = AtomicBool::new(false);

/// Resizes the overlay to fit its current content, keeping the right edge
/// anchored in place (shrinking/growing "into" the left side) so hiding a
/// ring doesn't visually detach the panel from wherever it was positioned.
#[tauri::command]
fn resize_panel(window: tauri::WebviewWindow, width: f64) -> Result<(), String> {
    let current_pos = window.outer_position().map_err(|e| e.to_string())?;
    let current_size = window.outer_size().map_err(|e| e.to_string())?;
    let scale = window.scale_factor().map_err(|e| e.to_string())?;
    let new_width = (width * scale).round() as u32;

    if new_width == current_size.width {
        return Ok(());
    }

    let new_x = current_pos.x + current_size.width as i32 - new_width as i32;

    PROGRAMMATIC_MOVE.store(true, Ordering::SeqCst);
    window
        .set_size(tauri::PhysicalSize {
            width: new_width,
            height: current_size.height,
        })
        .map_err(|e| e.to_string())?;
    window
        .set_position(tauri::PhysicalPosition {
            x: new_x,
            y: current_pos.y,
        })
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
fn start_drag(window: tauri::WebviewWindow) -> Result<(), String> {
    window.start_dragging().map_err(|e| e.to_string())
}

#[tauri::command]
fn pin_interactive(pinned: bool) {
    input::pin_interactive(pinned);
}

#[tauri::command]
fn get_autostart_enabled(app: tauri::AppHandle) -> bool {
    menu::is_autostart_enabled(&app)
}

#[tauri::command]
fn toggle_autostart(app: tauri::AppHandle) -> Result<bool, String> {
    menu::toggle_autostart(&app)
}

#[tauri::command]
fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}

async fn refresh_once(app: &tauri::AppHandle) {
    let client = reqwest::Client::new();

    let claude_result = claude::fetch(&client).await;
    let cursor_result = cursor::fetch(&client).await;
    let codex_result = codex::fetch(&client).await;

    let state = app.state::<UsageCache>();
    let mut cache = state.0.lock().unwrap();

    let claude = match claude_result {
        Ok(snapshot) => {
            cache.claude = Some(snapshot.clone());
            Some(snapshot)
        }
        Err(err) => {
            eprintln!("[claude] refresh failed: {err}");
            let fallback = cache.claude.as_ref().map(|s| s.as_stale(err));
            cache.claude = fallback.clone();
            fallback
        }
    };

    let cursor = match cursor_result {
        Ok(snapshot) => {
            cache.cursor = Some(snapshot.clone());
            Some(snapshot)
        }
        Err(err) => {
            eprintln!("[cursor] refresh failed: {err}");
            let fallback = cache.cursor.as_ref().map(|s| s.as_stale(err));
            cache.cursor = fallback.clone();
            fallback
        }
    };

    let codex = match codex_result {
        Ok(snapshot) => {
            cache.codex = Some(snapshot.clone());
            Some(snapshot)
        }
        Err(err) => {
            eprintln!("[codex] refresh failed: {err}");
            let fallback = cache.codex.as_ref().map(|s| s.as_stale(err));
            cache.codex = fallback.clone();
            fallback
        }
    };

    let _ = app.emit("usage-updated", UsagePayload { claude, cursor, codex });
}

struct UsageCache(std::sync::Mutex<UsagePayload>);

fn place_default(window: &tauri::WebviewWindow) {
    let Ok(Some(monitor)) = window.primary_monitor() else {
        return;
    };
    let size = monitor.size();
    let scale = monitor.scale_factor();
    let win_size = window.outer_size().unwrap_or(tauri::PhysicalSize {
        width: 334,
        height: 72,
    });
    let x = size.width as i32 - win_size.width as i32 - (MARGIN as f64 * scale) as i32;
    let y = (MARGIN as f64 * scale) as i32;
    let _ = window.set_position(tauri::PhysicalPosition { x, y });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .manage(UsageCache(std::sync::Mutex::new(UsagePayload::default())))
        .invoke_handler(tauri::generate_handler![
            start_drag,
            pin_interactive,
            get_autostart_enabled,
            toggle_autostart,
            quit_app,
            resize_panel
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            let window = app
                .get_webview_window("overlay")
                .expect("overlay window must exist");

            let saved = config::load(&handle);
            if let (Some(x), Some(y)) = (saved.x, saved.y) {
                let _ = window.set_position(tauri::PhysicalPosition { x, y });
            } else {
                place_default(&window);
            }
            let _ = window.set_ignore_cursor_events(true);
            let _ = window.show();

            if saved.autostart {
                use tauri_plugin_autostart::ManagerExt;
                if !handle.autolaunch().is_enabled().unwrap_or(false) {
                    let _ = handle.autolaunch().enable();
                }
            }

            let move_handle = handle.clone();
            window.on_window_event(move |event| {
                if let WindowEvent::Moved(position) = event {
                    if PROGRAMMATIC_MOVE.swap(false, Ordering::SeqCst) {
                        return;
                    }
                    let mut cfg = config::load(&move_handle);
                    cfg.x = Some(position.x);
                    cfg.y = Some(position.y);
                    let _ = config::save(&move_handle, &cfg);
                }
            });

            input::spawn_unlock_watcher(handle.clone());
            fullscreen::spawn_fullscreen_watcher(handle.clone());

            let refresh_handle = handle.clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    refresh_once(&refresh_handle).await;
                    tokio::time::sleep(REFRESH_INTERVAL).await;
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
