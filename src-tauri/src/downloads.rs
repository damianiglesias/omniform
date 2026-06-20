use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Child;
use tokio::sync::Mutex;

use crate::dependencies::{ffmpeg_path, yt_dlp_path};

#[derive(Clone, Serialize)]
pub struct ProgressEvent {
    pub id: String,
    pub progress: f64,
    pub speed: Option<String>,
    pub eta: Option<String>,
    pub status: String,
}

#[derive(Clone, Serialize)]
pub struct InfoEvent {
    pub id: String,
    pub title: String,
    pub thumbnail: Option<String>,
}

#[derive(Clone, Serialize)]
pub struct DoneEvent {
    pub id: String,
    #[serde(rename = "outputPath")]
    pub output_path: String,
}

#[derive(Clone, Serialize)]
pub struct ErrorEvent {
    pub id: String,
    pub message: String,
}

#[derive(Default)]
pub struct DownloadRegistry {
    pub running: Mutex<HashMap<String, Arc<Mutex<Child>>>>,
}

#[derive(Deserialize)]
struct VideoInfo {
    title: Option<String>,
    thumbnail: Option<String>,
}

/// Fixed progress template we fully control, with a unique marker
/// at the start of the line so it cannot be confused with other yt-dlp output.
/// Fields are separated by "|" in a known order.
const PROGRESS_MARKER: &str = "OMNIFORM_PROGRESS";
const PROGRESS_TEMPLATE: &str = "OMNIFORM_PROGRESS|%(progress._percent_str)s|%(progress._speed_str)s|%(progress._eta_str)s";

fn audio_quality_to_bitrate(quality: &str) -> &'static str {
    match quality {
        "low" => "5",
        "medium" => "2",
        "high" => "0",
        _ => "0",
    }
}

fn format_to_args(format: &str, quality: &str) -> Vec<String> {
    let audio_formats = ["mp3", "wav", "flac", "m4a", "ogg"];

    if audio_formats.contains(&format) {
        vec![
            "-x".to_string(),
            "--audio-format".to_string(),
            format.to_string(),
            "--audio-quality".to_string(),
            audio_quality_to_bitrate(quality).to_string(),
        ]
    } else {
        let height = match quality {
            "1080p" => Some("1080"),
            "720p" => Some("720"),
            "480p" => Some("480"),
            _ => None,
        };

        let format_selector = match height {
            Some(h) => format!(
                "bestvideo[height<={h}][ext=mp4]+bestaudio[ext=m4a]/best[height<={h}]"
            ),
            None => "bestvideo[ext=mp4]+bestaudio[ext=m4a]/best".to_string(),
        };

        vec![
            "-f".to_string(),
            format_selector,
            "--merge-output-format".to_string(),
            format.to_string(),
        ]
    }
}

/// Fetches the video title and thumbnail before starting the real download,
/// using --dump-json to get an unambiguous, parseable format.
async fn fetch_info(yt_dlp: &std::path::Path, url: &str) -> Option<VideoInfo> {
    let output = tokio::process::Command::new(yt_dlp)
        .args(["--dump-json", "--no-download", "--no-playlist", url])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .await
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let first_line = text.lines().next()?;
    serde_json::from_str::<VideoInfo>(first_line).ok()
}

fn parse_progress_marker_line(line: &str) -> Option<(f64, Option<String>, Option<String>)> {
    let rest = line.strip_prefix(PROGRESS_MARKER)?;
    let parts: Vec<&str> = rest.trim_start_matches('|').split('|').collect();
    if parts.len() < 3 {
        return None;
    }

    let percent_str = parts[0].trim().trim_end_matches('%');
    let progress = percent_str.parse::<f64>().ok()?;

    let speed = {
        let s = parts[1].trim();
        if s.is_empty() || s == "N/A" {
            None
        } else {
            Some(s.to_string())
        }
    };

    let eta = {
        let e = parts[2].trim();
        if e.is_empty() || e == "N/A" {
            None
        } else {
            Some(e.to_string())
        }
    };

    Some((progress, speed, eta))
}

pub async fn start_download(
    app: AppHandle,
    registry: Arc<DownloadRegistry>,
    id: String,
    url: String,
    format: String,
    quality: String,
    output_dir: String,
) -> Result<(), String> {
    let yt_dlp = yt_dlp_path(&app);
    let ffmpeg = ffmpeg_path(&app);

    if !yt_dlp.exists() {
        return Err("yt-dlp is not installed yet".to_string());
    }

    let _ = app.emit(
        "download://progress",
        ProgressEvent {
            id: id.clone(),
            progress: 0.0,
            speed: None,
            eta: None,
            status: "fetching-info".to_string(),
        },
    );

    // Step 1: video info (title/thumbnail), fetched separately
    // and unambiguously, before touching the real download.
    if let Some(info) = fetch_info(&yt_dlp, &url).await {
        let _ = app.emit(
            "download://info",
            InfoEvent {
                id: id.clone(),
                title: info.title.unwrap_or_else(|| url.clone()),
                thumbnail: info.thumbnail,
            },
        );
    }

    let mut args: Vec<String> = vec![
        "--newline".to_string(),
        "--no-playlist".to_string(),
        "--ffmpeg-location".to_string(),
        ffmpeg.to_string_lossy().to_string(),
        "-o".to_string(),
        format!("{output_dir}/%(title)s.%(ext)s"),
        "--progress-template".to_string(),
        PROGRESS_TEMPLATE.to_string(),
    ];

    args.extend(format_to_args(&format, &quality));
    args.push(url.clone());

    let mut cmd = tokio::process::Command::new(&yt_dlp);
    cmd.args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("could not start yt-dlp: {e}"))?;

    let stdout = child.stdout.take().ok_or("could not read stdout")?;
    let stderr = child.stderr.take().ok_or("could not read stderr")?;

    {
        let mut running = registry.running.lock().await;
        running.insert(id.clone(), Arc::new(Mutex::new(child)));
    }

    let app_clone = app.clone();
    let id_clone = id.clone();
    let registry_clone = registry.clone();
    let output_dir_clone = output_dir.clone();

    tokio::spawn(async move {
        let mut reader = BufReader::new(stdout).lines();
        let mut final_status = "done".to_string();

        while let Ok(Some(line)) = reader.next_line().await {
            if let Some((progress, speed, eta)) = parse_progress_marker_line(&line) {
                let status = if progress >= 100.0 {
                    "converting"
                } else {
                    "downloading"
                };
                let _ = app_clone.emit(
                    "download://progress",
                    ProgressEvent {
                        id: id_clone.clone(),
                        progress,
                        speed,
                        eta,
                        status: status.to_string(),
                    },
                );
            } else if line.contains("[Merger]") || line.contains("[ExtractAudio]") {
                let _ = app_clone.emit(
                    "download://progress",
                    ProgressEvent {
                        id: id_clone.clone(),
                        progress: 99.0,
                        speed: None,
                        eta: None,
                        status: "converting".to_string(),
                    },
                );
            }
        }

        // Drain stderr to detect real errors after the process finishes.
        let mut err_reader = BufReader::new(stderr).lines();
        let mut err_text = String::new();
        while let Ok(Some(line)) = err_reader.next_line().await {
            err_text.push_str(&line);
            err_text.push('\n');
        }

        let exit_status = {
            let mut running = registry_clone.running.lock().await;
            if let Some(child_arc) = running.remove(&id_clone) {
                let mut child = child_arc.lock().await;
                child.wait().await.ok()
            } else {
                None
            }
        };

        let succeeded = exit_status.map(|s| s.success()).unwrap_or(false);

        if !succeeded {
            if err_text.trim().is_empty() {
                final_status = "cancelled".to_string();
            } else {
                let _ = app_clone.emit(
                    "download://error",
                    ErrorEvent {
                        id: id_clone.clone(),
                        message: err_text.trim().to_string(),
                    },
                );
                return;
            }
        }

        if final_status == "done" {
            let _ = app_clone.emit(
                "download://progress",
                ProgressEvent {
                    id: id_clone.clone(),
                    progress: 100.0,
                    speed: None,
                    eta: None,
                    status: "done".to_string(),
                },
            );
            let _ = app_clone.emit(
                "download://done",
                DoneEvent {
                    id: id_clone.clone(),
                    output_path: output_dir_clone.clone(),
                },
            );
        }
    });

    Ok(())
}

pub async fn cancel_download(registry: Arc<DownloadRegistry>, id: String) -> Result<(), String> {
    let mut running = registry.running.lock().await;
    if let Some(child_arc) = running.remove(&id) {
        let mut child = child_arc.lock().await;
        let _ = child.kill().await;
    }
    Ok(())
}
