mod dependencies;
mod downloads;
mod history;

use serde::Serialize;
use std::sync::Arc;
use tauri::{AppHandle, State};

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
    options: downloads::DownloadOptions,
) -> Result<(), String> {
    let registry = registry.inner().clone();
    downloads::start_download(app, registry, id, url, format, quality, output_dir, options).await
}

#[tauri::command]
async fn inspect_url(app: AppHandle, url: String) -> Result<downloads::MediaProbe, String> {
    downloads::inspect_url(app, url).await
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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DiagnosticInfo {
    bin_path: String,
    yt_dlp_path: String,
    ffmpeg_path: String,
    yt_dlp_version: Option<String>,
    ffmpeg_version: Option<String>,
}

async fn binary_version(path: &std::path::Path, arg: &str) -> Option<String> {
    let output = tokio::process::Command::new(path)
        .arg(arg)
        .output()
        .await
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .map(str::to_string)
}

#[tauri::command]
async fn get_diagnostics(app: AppHandle) -> DiagnosticInfo {
    let bin = dependencies::bin_dir(&app);
    let yt_dlp = dependencies::yt_dlp_path(&app);
    let ffmpeg = dependencies::ffmpeg_path(&app);
    DiagnosticInfo {
        bin_path: bin.to_string_lossy().to_string(),
        yt_dlp_version: binary_version(&yt_dlp, "--version").await,
        ffmpeg_version: binary_version(&ffmpeg, "-version").await,
        yt_dlp_path: yt_dlp.to_string_lossy().to_string(),
        ffmpeg_path: ffmpeg.to_string_lossy().to_string(),
    }
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
            inspect_url,
            get_default_output_dir,
            start_download,
            cancel_download,
            open_folder,
            get_history,
            add_history_entry,
            clear_history,
            check_ytdlp_update,
            update_ytdlp,
            get_diagnostics,
        ])
        .run(tauri::generate_context!())
        .expect("error while running the Tauri application");
}
