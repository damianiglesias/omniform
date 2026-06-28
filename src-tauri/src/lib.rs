mod dependencies;
mod downloads;
mod history;

use std::sync::Arc;
use tauri::{AppHandle, Manager, State};

use dependencies::DependencyStatus;
use downloads::DownloadRegistry;
use history::HistoryEntry;

#[tauri::command]
fn check_dependencies(app: AppHandle) -> DependencyStatus {
    dependencies::current_status(&app)
}

#[tauri::command]
async fn ensure_dependencies(app: AppHandle) -> Result<(), String> {
    dependencies::ensure_dependencies(app).await
}

#[tauri::command]
fn get_default_output_dir() -> Result<String, String> {
    dirs::download_dir()
        .or_else(dirs::home_dir)
        .map(|p| p.to_string_lossy().to_string())
        .ok_or_else(|| "could not determine a default folder".to_string())
}

#[tauri::command]
async fn start_download(
    app: AppHandle,
    registry: State<'_, Arc<DownloadRegistry>>,
    id: String,
    url: String,
    format: String,
    quality: String,
    output_dir: String,
) -> Result<(), String> {
    let registry = registry.inner().clone();
    downloads::start_download(app, registry, id, url, format, quality, output_dir).await
}

#[tauri::command]
async fn cancel_download(
    registry: State<'_, Arc<DownloadRegistry>>,
    id: String,
) -> Result<(), String> {
    let registry = registry.inner().clone();
    downloads::cancel_download(registry, id).await
}

#[tauri::command]
async fn open_folder(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
{
    std::process::Command::new("explorer")
        .arg(&path)
        .spawn()
        .map_err(|e| e.to_string())?;
}
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .args(["-R", &path])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        let parent = std::path::Path::new(&path)
            .parent()
            .unwrap_or(std::path::Path::new("/"))
            .to_string_lossy()
            .to_string();
        std::process::Command::new("xdg-open")
            .arg(parent)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn get_history(app: AppHandle) -> Vec<HistoryEntry> {
    history::load(&app)
}

#[tauri::command]
fn add_history_entry(app: AppHandle, entry: HistoryEntry) -> Result<(), String> {
    history::add(&app, entry)
}

#[tauri::command]
fn clear_history(app: AppHandle) -> Result<(), String> {
    history::clear(&app)
}

#[tauri::command]
async fn check_ytdlp_update(app: AppHandle) -> Result<bool, String> {
    dependencies::check_for_update(&app).await
}

#[tauri::command]
async fn update_ytdlp(app: AppHandle) -> Result<(), String> {
    dependencies::update_ytdlp(app).await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(Arc::new(DownloadRegistry::default()))
        .setup(|app| {
            let handle = app.handle().clone();
            std::fs::create_dir_all(dependencies::bin_dir(&handle)).ok();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            check_dependencies,
            ensure_dependencies,
            get_default_output_dir,
            start_download,
            cancel_download,
            open_folder,
            get_history,
            add_history_entry,
            clear_history,
            check_ytdlp_update,
            update_ytdlp,
        ])
        .run(tauri::generate_context!())
        .expect("error while running the Tauri application");
}
