mod dependencies;
mod downloads;

use std::sync::Arc;
#[allow(unused_imports)]
use tauri::{AppHandle, Manager, State};

use dependencies::DependencyStatus;
use downloads::DownloadRegistry;

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running the Tauri application");
}
