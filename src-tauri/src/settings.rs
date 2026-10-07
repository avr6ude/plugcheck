//! Tiny JSON-file settings store under the app config dir.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::Manager;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub notifications: bool,
    pub poll_secs: u64,
    pub launch_at_login: bool,
    /// Hide the Dock icon and run as a pure menu-bar app.
    pub menu_bar_only: bool,
    /// Show the raw-data "Technical" tab on ports.
    pub show_technical: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            notifications: true,
            poll_secs: 3,
            launch_at_login: false,
            menu_bar_only: false,
            show_technical: false,
        }
    }
}

impl Settings {
    pub fn poll_secs(&self) -> u64 {
        self.poll_secs.clamp(1, 60)
    }
}

fn path(app: &tauri::AppHandle) -> PathBuf {
    let dir = app
        .path()
        .app_config_dir()
        .unwrap_or_else(|_| std::env::temp_dir());
    let _ = std::fs::create_dir_all(&dir);
    dir.join("settings.json")
}

pub fn load(app: &tauri::AppHandle) -> Settings {
    std::fs::read_to_string(path(app))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save(app: &tauri::AppHandle, s: &Settings) -> Result<(), String> {
    let json = serde_json::to_string_pretty(s).map_err(|e| e.to_string())?;
    std::fs::write(path(app), json).map_err(|e| e.to_string())
}
