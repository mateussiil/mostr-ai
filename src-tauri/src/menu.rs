use tauri::{AppHandle, Runtime};
use tauri_plugin_autostart::ManagerExt;

pub fn is_autostart_enabled<R: Runtime>(app: &AppHandle<R>) -> bool {
    app.autolaunch().is_enabled().unwrap_or(false)
}

pub fn toggle_autostart<R: Runtime>(app: &AppHandle<R>) -> Result<bool, String> {
    let manager = app.autolaunch();
    let enabled = manager.is_enabled().unwrap_or(false);
    let result = if enabled { manager.disable() } else { manager.enable() };
    result.map_err(|e| e.to_string())?;

    let mut config = crate::config::load(app);
    config.autostart = !enabled;
    crate::config::save(app, &config)?;

    Ok(!enabled)
}
