use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub title: String,
    pub url: String,
    pub format: String,
    pub output_path: String,
    pub timestamp: u64,
}

fn history_path(app: &AppHandle) -> PathBuf {
    app.path()
        .app_data_dir()
        .unwrap_or_else(|_| std::env::temp_dir())
        .join("history.json")
}

pub fn load(app: &AppHandle) -> Vec<HistoryEntry> {
    let path = history_path(app);
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn add(app: &AppHandle, entry: HistoryEntry) -> Result<(), String> {
    let mut entries = load(app);
    entries.insert(0, entry);
    let json = serde_json::to_string(&entries).map_err(|e| e.to_string())?;
    std::fs::write(history_path(app), json).map_err(|e| e.to_string())
}

pub fn clear(app: &AppHandle) -> Result<(), String> {
    std::fs::write(history_path(app), "[]").map_err(|e| e.to_string())
}
