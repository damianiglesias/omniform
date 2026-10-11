use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Child;
use tokio::sync::Mutex;

use crate::dependencies::{ffmpeg_path, yt_dlp_path};

/// On Windows, child processes spawned from a windowed (GUI) app still pop up
/// their own console window unless explicitly told not to. CREATE_NO_WINDOW
/// (0x08000000) suppresses that. No-op on other platforms.
#[cfg(target_os = "windows")]
fn no_window(cmd: &mut tokio::process::Command) {
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    cmd.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(target_os = "windows"))]
fn no_window(_cmd: &mut tokio::process::Command) {}

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

#[derive(Deserialize)]
pub struct DownloadOptions {
    pub playlist: bool,
    #[serde(rename = "playlistLimit")]
    pub playlist_limit: Option<u32>,
    pub subtitles: bool,
    #[serde(rename = "subtitleLanguage")]
    pub subtitle_language: String,
}

#[derive(Serialize)]
pub struct MediaProbe {
    pub title: String,
    pub thumbnail: Option<String>,
    pub duration: Option<u64>,
    #[serde(rename = "playlistCount")]
    pub playlist_count: Option<u32>,
    #[serde(rename = "availableHeights")]
    pub available_heights: Vec<String>,
    #[serde(rename = "subtitleLanguages")]
    pub subtitle_languages: Vec<String>,
}

#[derive(Deserialize)]
struct ProbeJson {
    title: Option<String>,
    thumbnail: Option<String>,
    duration: Option<u64>,
    playlist_count: Option<u32>,
    formats: Option<Vec<ProbeFormat>>,
    subtitles: Option<serde_json::Map<String, serde_json::Value>>,
    automatic_captions: Option<serde_json::Map<String, serde_json::Value>>,
}

#[derive(Deserialize)]
struct ProbeFormat {
    height: Option<u32>,
}

/// Fixed progress template we fully control, with a unique marker
/// at the start of the line so it cannot be confused with other yt-dlp output.
/// Fields are separated by "|" in a known order.
const PROGRESS_MARKER: &str = "OMNIFORM_PROGRESS";
const PROGRESS_TEMPLATE: &str =
    "OMNIFORM_PROGRESS|%(progress._percent_str)s|%(progress._speed_str)s|%(progress._eta_str)s";

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
            "--embed-metadata".to_string(),
            "--embed-thumbnail".to_string(),
        ]
    } else {
        let height = match quality {
            "1080p" => Some("1080"),
            "720p" => Some("720"),
            "480p" => Some("480"),
            _ => None,
        };

        let format_selector = match height {
            Some(h) => {
                format!("bestvideo[height<={h}][ext=mp4]+bestaudio[ext=m4a]/best[height<={h}]")
            }
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
    let mut cmd = tokio::process::Command::new(yt_dlp);
    cmd.args(["--dump-json", "--no-download", "--no-playlist", url])
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    no_window(&mut cmd);

    let output = cmd.output().await.ok()?;

    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let first_line = text.lines().next()?;
    serde_json::from_str::<VideoInfo>(first_line).ok()
}

pub async fn inspect_url(app: AppHandle, url: String) -> Result<MediaProbe, String> {
    let yt_dlp = yt_dlp_path(&app);
    if !yt_dlp.is_file() {
        return Err("yt-dlp is not installed yet".to_string());
    }
    let mut cmd = tokio::process::Command::new(yt_dlp);
    cmd.args([
        "--dump-single-json",
        "--no-download",
        "--flat-playlist",
        &url,
    ])
    .stdout(Stdio::piped())
    .stderr(Stdio::piped());
    no_window(&mut cmd);
    let output = cmd
        .output()
        .await
        .map_err(|e| format!("could not inspect link: {e}"))?;
    if !output.status.success() {
        return Err(friendly_error(&String::from_utf8_lossy(&output.stderr)));
    }
    let value: ProbeJson = serde_json::from_slice(&output.stdout)
        .map_err(|_| "couldn't read the available download options".to_string())?;
    let mut heights: Vec<u32> = value
        .formats
        .unwrap_or_default()
        .into_iter()
        .filter_map(|format| format.height)
        .collect();
    heights.sort_unstable_by(|a, b| b.cmp(a));
    heights.dedup();
    let mut subtitles: Vec<String> = value
        .subtitles
        .into_iter()
        .flatten()
        .chain(value.automatic_captions.into_iter().flatten())
        .map(|(language, _)| language)
        .collect();
    subtitles.sort();
    subtitles.dedup();
    Ok(MediaProbe {
        title: value.title.unwrap_or_else(|| "Untitled video".to_string()),
        thumbnail: value.thumbnail,
        duration: value.duration,
        playlist_count: value.playlist_count,
        available_heights: heights
            .into_iter()
            .map(|height| format!("{height}p"))
            .collect(),
        subtitle_languages: subtitles,
    })
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

fn friendly_error(raw: &str) -> String {
    let lower = raw.to_lowercase();

    if lower.contains("video unavailable") || lower.contains("this video is not available") {
        return "This video is unavailable.".to_string();
    }
    if lower.contains("private video") {
        return "This video is private.".to_string();
    }
    if lower.contains("sign in to confirm") || lower.contains("age-restricted") {
        return "This video is age-restricted and can't be downloaded.".to_string();
    }
    if lower.contains("unsupported url") || lower.contains("is not a valid url") {
        return "This link isn't supported.".to_string();
    }
    if lower.contains("unable to download webpage")
        || lower.contains("failed to resolve")
        || lower.contains("name or service not known")
        || lower.contains("network is unreachable")
    {
        return "Couldn't connect — check your internet connection.".to_string();
    }
    if lower.contains("404") || lower.contains("not found") {
        return "This content no longer exists.".to_string();
    }
    if lower.contains("ffmpeg") {
        return "Conversion failed while processing the file.".to_string();
    }

    let first_real_line = raw
        .lines()
        .find(|l| l.trim_start().starts_with("ERROR:"))
        .unwrap_or_else(|| raw.lines().next().unwrap_or(raw));

    let cleaned = first_real_line.trim_start_matches("ERROR:").trim();
    if cleaned.is_empty() {
        "Something went wrong during the download.".to_string()
    } else if cleaned.len() > 160 {
        format!("{}...", &cleaned[..160])
    } else {
        cleaned.to_string()
    }
}

pub async fn start_download(
    app: AppHandle,
    registry: Arc<DownloadRegistry>,
    id: String,
    url: String,
    format: String,
    quality: String,
    output_dir: String,
    options: DownloadOptions,
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
        "--ffmpeg-location".to_string(),
        ffmpeg.to_string_lossy().to_string(),
        "-o".to_string(),
        format!("{output_dir}/%(title)s.%(ext)s"),
        "--progress-template".to_string(),
        PROGRESS_TEMPLATE.to_string(),
    ];

    if !options.playlist {
        args.push("--no-playlist".to_string());
    } else if let Some(limit) = options.playlist_limit.filter(|limit| *limit > 0) {
        args.extend(["--playlist-end".to_string(), limit.to_string()]);
    }

    if options.subtitles {
        args.extend([
            "--write-subs".to_string(),
            "--write-auto-subs".to_string(),
            "--sub-langs".to_string(),
            options.subtitle_language,
            "--convert-subs".to_string(),
            "srt".to_string(),
        ]);
        if !["mp3", "wav", "flac", "m4a", "ogg"].contains(&format.as_str()) {
            args.push("--embed-subs".to_string());
        }
    }

    args.extend(format_to_args(&format, &quality));
    args.push(url.clone());

    let mut cmd = tokio::process::Command::new(&yt_dlp);
    cmd.args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    no_window(&mut cmd);

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
                        message: friendly_error(err_text.trim()),
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
