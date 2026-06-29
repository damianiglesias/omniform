import React, { FormEvent, useEffect, useRef, useState } from "react";
import { readText } from "@tauri-apps/plugin-clipboard-manager";
import type { OutputFormat, Quality } from "../types";

interface Props {
  disabled: boolean;
  onSubmit: (urls: string[], format: OutputFormat, quality: Quality) => void;
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

const URL_PATTERN = /^https?:\/\/\S+$/i;

export function UrlForm({ disabled, onSubmit }: Props) {
  const [url, setUrl] = useState("");
  const [format, setFormat] = useState<OutputFormat>("mp4");
  const [videoQuality, setVideoQuality] = useState<Quality>("best");
  const [audioQuality, setAudioQuality] = useState<Quality>("high");
  const [autoPasted, setAutoPasted] = useState(false);
  const lastClipboardRef = useRef<string | null>(null);

  const selectedKind = FORMATS.find((f) => f.value === format)?.kind ?? "video";

  useEffect(() => {
    async function checkClipboard() {
      try {
        const text = await readText();
        if (!text) return;
        const trimmed = text.trim();
        if (!URL_PATTERN.test(trimmed)) return;
        if (trimmed === lastClipboardRef.current) return;

        lastClipboardRef.current = trimmed;

        setUrl((current) => {
          if (current.trim().length === 0) {
            setAutoPasted(true);
            return trimmed;
          }
          return current;
        });
      } catch {
        // Clipboard may be unavailable or hold non-text content; safe to ignore.
      }
    }

    checkClipboard();
    window.addEventListener("focus", checkClipboard);
    return () => window.removeEventListener("focus", checkClipboard);
  }, []);

  function handleSubmit(e: FormEvent) {
    e.preventDefault();
    const urls = url
  .split("\n")
  .map((u) => u.trim())
  .filter((u) => URL_PATTERN.test(u));
    if (urls.length === 0) return;
    const quality = selectedKind === "audio" ? audioQuality : videoQuality;
    onSubmit(urls, format, quality);
    setUrl("");
    setAutoPasted(false);
  }

  function handleUrlChange(value: string) {
    setUrl(value);
    setAutoPasted(false);
  }

  function handleKeyDown(e: React.KeyboardEvent<HTMLTextAreaElement>) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSubmit(e as unknown as FormEvent);
    }
  }

  return (
    <form className="url-form" onSubmit={handleSubmit}>
      <div className="url-input-wrap">
        <textarea
          className="url-input"
          placeholder="Paste one or more links, one per line..."
          value={url}
          onChange={(e) => handleUrlChange(e.target.value)}
          onKeyDown={handleKeyDown}
          disabled={disabled}
          spellCheck={false}
          rows={url.includes("\n") ? Math.min(6, url.split("\n").length) : 1}
        />
        {autoPasted && <span className="auto-paste-badge">pasted from clipboard</span>}
      </div>

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
        {(() => {
          const count = url.split("\n").map((u) => u.trim()).filter(Boolean).length;
          return count > 1 ? `Add ${count} to queue` : "Add to queue";
        })()}
      </button>
    </form>
  );
}
