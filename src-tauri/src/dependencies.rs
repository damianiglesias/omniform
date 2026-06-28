use serde::Serialize;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter, Manager};

#[derive(Clone, Serialize)]
pub struct DependencyStatus {
    #[serde(rename = "ytDlpReady")]
    pub yt_dlp_ready: bool,
    #[serde(rename = "ffmpegReady")]
    pub ffmpeg_ready: bool,
    pub downloading: bool,
    pub message: Option<String>,
}

/// Folder where the binaries managed by the app are stored.
/// Kept separate from any system installation to avoid version conflicts.
pub fn bin_dir(app: &AppHandle) -> PathBuf {
    let base = app
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| std::env::temp_dir());
    base.join("bin")
}

pub fn yt_dlp_path(app: &AppHandle) -> PathBuf {
    let name = if cfg!(target_os = "windows") {
        "yt-dlp.exe"
    } else {
        "yt-dlp"
    };
    bin_dir(app).join(name)
}

pub fn ffmpeg_path(app: &AppHandle) -> PathBuf {
    let name = if cfg!(target_os = "windows") {
        "ffmpeg.exe"
    } else {
        "ffmpeg"
    };
    bin_dir(app).join(name)
}

pub fn current_status(app: &AppHandle) -> DependencyStatus {
    DependencyStatus {
        yt_dlp_ready: yt_dlp_path(app).exists(),
        ffmpeg_ready: ffmpeg_path(app).exists(),
        downloading: false,
        message: None,
    }
}

fn emit_status(app: &AppHandle, status: &DependencyStatus) {
    let _ = app.emit("deps://status", status.clone());
}

fn yt_dlp_download_url() -> &'static str {
    if cfg!(target_os = "windows") {
        "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe"
    } else if cfg!(target_os = "macos") {
        "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_macos"
    } else {
        "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp"
    }
}

/// ffmpeg does not publish a single official standalone binary per platform
/// the way yt-dlp does, so we rely on widely trusted third-party static builds
/// maintained specifically for self-contained binary distribution.
fn ffmpeg_download_url() -> &'static str {
    if cfg!(target_os = "windows") {
        "https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip"
    } else if cfg!(target_os = "macos") {
        "https://evermeet.cx/ffmpeg/getrelease/zip"
    } else {
        "https://johnvansickle.com/ffmpeg/releases/ffmpeg-release-amd64-static.tar.xz"
    }
}

async fn download_file(url: &str) -> Result<Vec<u8>, String> {
    let resp = reqwest::get(url)
        .await
        .map_err(|e| format!("could not connect: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("download failed with status {}", resp.status()));
    }
    let bytes = resp
        .bytes()
        .await
        .map_err(|e| format!("error reading response: {e}"))?;
    Ok(bytes.to_vec())
}

#[cfg(unix)]
fn make_executable(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(path)
        .map_err(|e| e.to_string())?
        .permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms).map_err(|e| e.to_string())
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> Result<(), String> {
    Ok(())
}

async fn install_yt_dlp(app: &AppHandle) -> Result<(), String> {
    let target = yt_dlp_path(app);
    if target.exists() {
        return Ok(());
    }
    std::fs::create_dir_all(bin_dir(app)).map_err(|e| e.to_string())?;

    let bytes = download_file(yt_dlp_download_url()).await?;
    std::fs::write(&target, bytes).map_err(|e| e.to_string())?;
    make_executable(&target)?;
    Ok(())
}

async fn install_ffmpeg(app: &AppHandle) -> Result<(), String> {
    let target = ffmpeg_path(app);
    if target.exists() {
        return Ok(());
    }
    let dir = bin_dir(app);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let bytes = download_file(ffmpeg_download_url()).await?;

    if cfg!(target_os = "linux") {
        extract_ffmpeg_tar_xz(&bytes, &dir, &target)?;
    } else {
        extract_ffmpeg_zip(&bytes, &dir, &target)?;
    }

    make_executable(&target)?;
    Ok(())
}

fn extract_ffmpeg_tar_xz(bytes: &[u8], dir: &Path, target: &Path) -> Result<(), String> {
    let decompressed = xz_decode(bytes)?;
    let mut archive = tar::Archive::new(decompressed.as_slice());

    for entry in archive.entries().map_err(|e| e.to_string())? {
        let mut entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path().map_err(|e| e.to_string())?.into_owned();
        if let Some(name) = path.file_name() {
            if name == "ffmpeg" {
                entry.unpack(target).map_err(|e| e.to_string())?;
                break;
            }
        }
    }

    let _ = dir;
    Ok(())
}

fn xz_decode(compressed: &[u8]) -> Result<Vec<u8>, String> {
    use std::io::Read;
    let mut decoder = xz2::read::XzDecoder::new(compressed);
    let mut out = Vec::new();
    decoder
        .read_to_end(&mut out)
        .map_err(|e| format!("failed to decompress the ffmpeg archive: {e}"))?;
    Ok(out)
}

fn extract_ffmpeg_zip(bytes: &[u8], dir: &Path, target: &Path) -> Result<(), String> {
    let cursor = std::io::Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor).map_err(|e| e.to_string())?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = file.name().to_string();
        let is_ffmpeg_bin = name.ends_with("ffmpeg.exe") || name.ends_with("/ffmpeg");
        if is_ffmpeg_bin {
            let mut out = std::fs::File::create(target).map_err(|e| e.to_string())?;
            std::io::copy(&mut file, &mut out).map_err(|e| e.to_string())?;
            break;
        }
    }

    let _ = dir;
    Ok(())
}

pub async fn ensure_dependencies(app: AppHandle) -> Result<(), String> {
    let mut status = current_status(&app);

    if status.yt_dlp_ready && status.ffmpeg_ready {
        emit_status(&app, &status);
        return Ok(());
    }

    status.downloading = true;

    if !status.yt_dlp_ready {
        status.message = Some("Downloading yt-dlp...".to_string());
        emit_status(&app, &status);
        if let Err(e) = install_yt_dlp(&app).await {
            status.downloading = false;
            status.message = Some(format!("Error downloading yt-dlp: {e}"));
            emit_status(&app, &status);
            return Err(e);
        }
        status.yt_dlp_ready = true;
    }

    if !status.ffmpeg_ready {
        status.message = Some("Downloading ffmpeg...".to_string());
        emit_status(&app, &status);
        if let Err(e) = install_ffmpeg(&app).await {
            status.downloading = false;
            status.message = Some(format!("Error downloading ffmpeg: {e}"));
            emit_status(&app, &status);
            return Err(e);
        }
        status.ffmpeg_ready = true;
    }

    status.downloading = false;
    status.message = Some("Ready.".to_string());
    emit_status(&app, &status);
    Ok(())
}
pub async fn check_for_update(app: &AppHandle) -> Result<bool, String> {
    let ytdlp = yt_dlp_path(app);
    if !ytdlp.exists() {
        return Ok(false);
    }

    let local = std::process::Command::new(&ytdlp)
        .arg("--version")
        .output()
        .map_err(|e| e.to_string())?;
    let local_ver = String::from_utf8_lossy(&local.stdout).trim().to_string();

    let client = reqwest::Client::builder()
        .user_agent("omniform")
        .build()
        .map_err(|e| e.to_string())?;

    let resp: serde_json::Value = client
        .get("https://api.github.com/repos/yt-dlp/yt-dlp/releases/latest")
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    let latest = resp["tag_name"]
        .as_str()
        .unwrap_or("")
        .to_string();

    Ok(!latest.is_empty() && latest != local_ver)
}

pub async fn update_ytdlp(app: AppHandle) -> Result<(), String> {
    let mut status = current_status(&app);
    status.downloading = true;
    status.message = Some("Updating yt-dlp...".to_string());
    emit_status(&app, &status);

    let target = yt_dlp_path(&app);
    let bytes = download_file(yt_dlp_download_url()).await?;
    std::fs::write(&target, bytes).map_err(|e| e.to_string())?;
    make_executable(&target)?;

    status.downloading = false;
    status.message = Some("yt-dlp updated.".to_string());
    emit_status(&app, &status);
    Ok(())
}
