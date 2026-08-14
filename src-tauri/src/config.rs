use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayConfig {
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub autostart: bool,
}

impl Default for OverlayConfig {
    fn default() -> Self {
        Self {
            x: None,
            y: None,
            autostart: true,
        }
    }
}

fn config_path<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> Result<PathBuf, String> {
    let dir = tauri::Manager::path(app)
        .app_config_dir()
        .map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("overlay-config.json"))
}

pub fn load<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> OverlayConfig {
    let Ok(path) = config_path(app) else {
        return OverlayConfig::default();
    };
    let Ok(raw) = std::fs::read_to_string(&path) else {
        return OverlayConfig::default();
    };
    serde_json::from_str(&raw).unwrap_or_default()
}

pub fn save<R: tauri::Runtime>(app: &tauri::AppHandle<R>, config: &OverlayConfig) -> Result<(), String> {
    let path = config_path(app)?;
    let raw = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    std::fs::write(&path, raw).map_err(|e| e.to_string())
}
