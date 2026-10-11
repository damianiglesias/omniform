import React, { FormEvent, useEffect, useRef, useState } from "react";
import { readText } from "@tauri-apps/plugin-clipboard-manager";
import { invoke } from "@tauri-apps/api/core";
import type { DownloadOptions, MediaProbe, OutputFormat, Quality } from "../types";

interface Props {
  disabled: boolean;
  onSubmit: (urls: string[], format: OutputFormat, quality: Quality, options: DownloadOptions) => void;
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
  const [probe, setProbe] = useState<MediaProbe | null>(null);
  const [inspecting, setInspecting] = useState(false);
  const [inspectError, setInspectError] = useState<string | null>(null);
  const [playlist, setPlaylist] = useState(false);
  const [playlistLimit, setPlaylistLimit] = useState(0);
  const [subtitles, setSubtitles] = useState(false);
  const [subtitleLanguage, setSubtitleLanguage] = useState("en");
  const [isDragging, setIsDragging] = useState(false);
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

  useEffect(() => {
    const onDragOver = (event: DragEvent) => { event.preventDefault(); setIsDragging(true); };
    const onDragLeave = (event: DragEvent) => { if (event.relatedTarget === null) setIsDragging(false); };
    const onDrop = (event: DragEvent) => {
      event.preventDefault();
      setIsDragging(false);
      const text = event.dataTransfer?.getData("text/uri-list") || event.dataTransfer?.getData("text/plain");
      if (text) handleUrlChange(text.trim());
    };
    window.addEventListener("dragover", onDragOver);
    window.addEventListener("dragleave", onDragLeave);
    window.addEventListener("drop", onDrop);
    return () => { window.removeEventListener("dragover", onDragOver); window.removeEventListener("dragleave", onDragLeave); window.removeEventListener("drop", onDrop); };
  }, []);

  function handleSubmit(e: FormEvent) {
    e.preventDefault();
    const urls = url
  .split("\n")
  .map((u) => u.trim())
  .filter((u) => URL_PATTERN.test(u));
    if (urls.length === 0) return;
    const quality = selectedKind === "audio" ? audioQuality : videoQuality;
    onSubmit(urls, format, quality, { playlist, playlistLimit: playlistLimit || null, subtitles, subtitleLanguage });
    setUrl("");
    setAutoPasted(false);
  }

  function handleUrlChange(value: string) {
    setUrl(value);
    setAutoPasted(false);
    setProbe(null);
    setInspectError(null);
  }

  async function inspectLink() {
    const candidate = url.trim();
    if (!URL_PATTERN.test(candidate) || candidate.includes("\n")) return;
    setInspecting(true);
    setInspectError(null);
    try {
      const result = await invoke<MediaProbe>("inspect_url", { url: candidate });
      setProbe(result);
      if (result.subtitleLanguages.length && !result.subtitleLanguages.includes(subtitleLanguage)) {
        setSubtitleLanguage(result.subtitleLanguages[0]);
      }
    } catch (error) {
      setInspectError(String(error));
    } finally {
      setInspecting(false);
    }
  }

  function handleKeyDown(e: React.KeyboardEvent<HTMLTextAreaElement>) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSubmit(e as unknown as FormEvent);
    }
  }

  return (
    <form className={`url-form${isDragging ? " drag-active" : ""}`} onSubmit={handleSubmit}>
      <div className="form-heading">
        <div><h2>New download</h2><p>Paste one or more video links.</p></div>
        <kbd>↵ to add</kbd>
      </div>
      <div className="url-input-wrap">
        <textarea
          className="url-input"
          placeholder="Paste a YouTube link…"
          value={url}
          onChange={(e) => handleUrlChange(e.target.value)}
          onKeyDown={handleKeyDown}
          disabled={disabled}
          spellCheck={false}
          rows={url.includes("\n") ? Math.min(6, url.split("\n").length) : 1}
        />
        {autoPasted && <span className="auto-paste-badge">pasted from clipboard</span>}
      </div>
      {isDragging && <div className="drop-hint">Drop the link to add it</div>}

      <div className="inspect-row">
        <button type="button" className="inspect-btn" onClick={inspectLink} disabled={disabled || inspecting || !URL_PATTERN.test(url.trim()) || url.includes("\n")}>
          {inspecting ? "Checking link…" : "Check available options"}
        </button>
        {inspectError && <span className="inspect-error">{inspectError}</span>}
      </div>

      {probe && (
        <div className="media-probe">
          <div className="probe-title">{probe.title}</div>
          <div className="probe-meta">
            {probe.duration !== null && <span>{Math.floor(probe.duration / 60)} min</span>}
            {probe.availableHeights.length > 0 && <span>Up to {probe.availableHeights[0]}</span>}
            {probe.playlistCount !== null && <span>{probe.playlistCount} videos in playlist</span>}
          </div>
        </div>
      )}

      <div className="form-row">
        <div className="field-group">
          <label className="field-label">Save as</label>
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

      <div className="advanced-options">
        <label className="option-toggle"><input type="checkbox" checked={playlist} onChange={(e) => setPlaylist(e.target.checked)} disabled={disabled} /><span>Download playlist</span></label>
        {playlist && <label className="option-field">Limit <input type="number" min="1" placeholder="All" value={playlistLimit || ""} onChange={(e) => setPlaylistLimit(Math.max(0, Number(e.target.value)))} disabled={disabled} /></label>}
        <label className="option-toggle"><input type="checkbox" checked={subtitles} onChange={(e) => setSubtitles(e.target.checked)} disabled={disabled} /><span>Include subtitles</span></label>
        {subtitles && <select className="subtitle-select" value={subtitleLanguage} onChange={(e) => setSubtitleLanguage(e.target.value)} disabled={disabled}>
          {(probe?.subtitleLanguages.length ? probe.subtitleLanguages : ["en", "es", "fr", "de", "pt"]).map((language) => <option key={language} value={language}>{language}</option>)}
        </select>}
      </div>

      <button type="submit" className="submit-btn" disabled={disabled || !url.trim()}>
        {(() => {
          const count = url.split("\n").map((u) => u.trim()).filter(Boolean).length;
          return count > 1 ? `Add ${count} downloads` : "Add download";
        })()}
      </button>
    </form>
  );
}
