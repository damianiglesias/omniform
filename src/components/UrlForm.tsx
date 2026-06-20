import { FormEvent, useState } from "react";
import type { OutputFormat, Quality } from "../types";

interface Props {
  disabled: boolean;
  onSubmit: (url: string, format: OutputFormat, quality: Quality) => void;
}

const FORMATS: { value: OutputFormat; label: string; kind: "video" | "audio" }[] = [
  { value: "mp4", label: "MP4", kind: "video" },
  { value: "webm", label: "WebM", kind: "video" },
  { value: "mp3", label: "MP3", kind: "audio" },
  { value: "wav", label: "WAV", kind: "audio" },
  { value: "flac", label: "FLAC", kind: "audio" },
  { value: "m4a", label: "M4A", kind: "audio" },
  { value: "ogg", label: "OGG", kind: "audio" },
];

const VIDEO_QUALITIES: { value: Quality; label: string }[] = [
  { value: "best", label: "Best available" },
  { value: "1080p", label: "1080p" },
  { value: "720p", label: "720p" },
  { value: "480p", label: "480p" },
];

const AUDIO_QUALITIES: { value: Quality; label: string }[] = [
  { value: "high", label: "High (320kbps)" },
  { value: "medium", label: "Medium (160kbps)" },
  { value: "low", label: "Low (96kbps)" },
];

export function UrlForm({ disabled, onSubmit }: Props) {
  const [url, setUrl] = useState("");
  const [format, setFormat] = useState<OutputFormat>("mp4");
  const [videoQuality, setVideoQuality] = useState<Quality>("best");
  const [audioQuality, setAudioQuality] = useState<Quality>("high");

  const selectedKind = FORMATS.find((f) => f.value === format)?.kind ?? "video";

  function handleSubmit(e: FormEvent) {
    e.preventDefault();
    const trimmed = url.trim();
    if (!trimmed) return;
    const quality = selectedKind === "audio" ? audioQuality : videoQuality;
    onSubmit(trimmed, format, quality);
    setUrl("");
  }

  return (
    <form className="url-form" onSubmit={handleSubmit}>
      <input
        type="text"
        className="url-input"
        placeholder="Paste a YouTube, TikTok, Instagram link..."
        value={url}
        onChange={(e) => setUrl(e.target.value)}
        disabled={disabled}
        spellCheck={false}
      />

      <div className="form-row">
        <div className="field-group">
          <label className="field-label">Format</label>
          <div className="format-grid">
            {FORMATS.map((f) => (
              <button
                type="button"
                key={f.value}
                className={`format-chip ${format === f.value ? "active" : ""}`}
                onClick={() => setFormat(f.value)}
                disabled={disabled}
              >
                {f.label}
              </button>
            ))}
          </div>
        </div>

        <div className="field-group">
          <label className="field-label">Quality</label>
          {selectedKind === "video" ? (
            <select
              className="quality-select"
              value={videoQuality}
              onChange={(e) => setVideoQuality(e.target.value as Quality)}
              disabled={disabled}
            >
              {VIDEO_QUALITIES.map((q) => (
                <option key={q.value} value={q.value}>
                  {q.label}
                </option>
              ))}
            </select>
          ) : (
            <select
              className="quality-select"
              value={audioQuality}
              onChange={(e) => setAudioQuality(e.target.value as Quality)}
              disabled={disabled}
            >
              {AUDIO_QUALITIES.map((q) => (
                <option key={q.value} value={q.value}>
                  {q.label}
                </option>
              ))}
            </select>
          )}
        </div>
      </div>

      <button type="submit" className="submit-btn" disabled={disabled || !url.trim()}>
        Add to queue
      </button>
    </form>
  );
}
