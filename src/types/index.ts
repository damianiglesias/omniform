export type OutputFormat =
  | "mp4"
  | "mp3"
  | "wav"
  | "flac"
  | "m4a"
  | "webm"
  | "ogg";
export type Quality =
  | "best"
  | "1080p"
  | "720p"
  | "480p"
  | "audio-only"
  | "high"
  | "medium"
  | "low";
export type DownloadStatus =
  | "queued"
  | "fetching-info"
  | "downloading"
  | "converting"
  | "done"
  | "error"
  | "cancelled";
export interface DownloadItem {
  id: string;
  url: string;
  title: string | null;
  thumbnail: string | null;
  format: OutputFormat;
  quality: Quality;
  status: DownloadStatus;
  progress: number;
  speed: string | null;
  eta: string | null;
  errorMessage: string | null;
  outputPath: string | null;
}
export interface DownloadProgressEvent {
  id: string;
  progress: number;
  speed: string | null;
  eta: string | null;
  status: DownloadStatus;
}
export interface DownloadInfoEvent {
  id: string;
  title: string;
  thumbnail: string | null;
}
export interface DownloadDoneEvent {
  id: string;
  outputPath: string;
}
export interface DownloadErrorEvent {
  id: string;
  message: string;
}
export interface DependencyStatus {
  ytDlpReady: boolean;
  ffmpegReady: boolean;
  downloading: boolean;
  message: string | null;
}
export interface HistoryEntry {
  id: string;
  title: string;
  url: string;
  format: string;
  output_path: string;
  timestamp: number;
}
