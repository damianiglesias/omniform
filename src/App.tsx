import { useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { useDownloadQueue } from "./hooks/useDownloadQueue";
import { UrlForm } from "./components/UrlForm";
import { QueueItem } from "./components/QueueItem";
import { DependencyBanner } from "./components/DependencyBanner";
import type { DiagnosticInfo, DownloadOptions, HistoryEntry, OutputFormat, Quality } from "./types";
import "./App.css";

type Tab = "queue" | "history";

export default function App() {
  const { items, deps, ensureDependencies, addDownload, cancelDownload, removeItem, moveItem, clearFinished } =
    useDownloadQueue();
  const [outputDir, setOutputDir] = useState<string | null>(null);
  const [tab, setTab] = useState<Tab>("queue");
  const [history, setHistory] = useState<HistoryEntry[]>([]);
  const [ytdlpUpdate, setYtdlpUpdate] = useState(false);
  const [updating, setUpdating] = useState(false);
  const [diagnostics, setDiagnostics] = useState<DiagnosticInfo | null>(null);
  const [showDiagnostics, setShowDiagnostics] = useState(false);
  const [historyQuery, setHistoryQuery] = useState("");

  useEffect(() => {
    invoke<string>("get_default_output_dir").then(setOutputDir).catch(() => {});
  }, []);

  useEffect(() => {
    if (!deps.ytDlpReady || !deps.ffmpegReady) {
      ensureDependencies();
    }
  }, []);

  useEffect(() => {
    if (deps.ytDlpReady) {
      invoke<boolean>("check_ytdlp_update").then(setYtdlpUpdate).catch(() => {});
    }
  }, [deps.ytDlpReady]);

  function loadHistory() {
    invoke<HistoryEntry[]>("get_history").then(setHistory).catch(() => {});
  }

  useEffect(() => {
    if (tab === "history") loadHistory();
  }, [tab]);

  async function pickOutputDir() {
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected === "string") setOutputDir(selected);
  }

  function handleAdd(urls: string[], format: OutputFormat, quality: Quality, options: DownloadOptions) {
    if (!outputDir) return;
    for (const url of urls) addDownload(url, format, quality, outputDir, options);
  }

  async function handleUpdate() {
    setUpdating(true);
    await invoke("update_ytdlp").catch(() => {});
    setYtdlpUpdate(false);
    setUpdating(false);
  }

  async function handleClearHistory() {
    await invoke("clear_history").catch(() => {});
    setHistory([]);
  }

  function openFolder(path: string) {
    invoke("open_folder", { path }).catch(() => {});
  }

  function redownload(entry: HistoryEntry) {
    if (!outputDir) return;
    const format = entry.format.toLowerCase() as OutputFormat;
    addDownload(entry.url, format, format === "mp3" || format === "wav" || format === "flac" || format === "m4a" || format === "ogg" ? "high" : "best", outputDir, {
      playlist: false, playlistLimit: null, subtitles: false, subtitleLanguage: "en",
    });
    setTab("queue");
  }

  async function toggleDiagnostics() {
    if (!showDiagnostics) {
      const info = await invoke<DiagnosticInfo>("get_diagnostics").catch(() => null);
      setDiagnostics(info);
    }
    setShowDiagnostics((visible) => !visible);
  }

  const depsReady = deps.ytDlpReady && deps.ffmpegReady;
  const hasFinished = items.some((it) => it.status === "done" || it.status === "cancelled");

  return (
    <div className="app">
      <header className="app-header">
        <div>
          <div className="brand"><span className="brand-mark">O</span><h1 className="app-title">Omniform</h1></div>
          <p className="app-subtitle">Downloads, simplified.</p>
        </div>
      </header>

      <DependencyBanner status={deps} onRetry={ensureDependencies} />

      {ytdlpUpdate && (
        <div className="update-banner">
          <span>A new version of yt-dlp is available.</span>
          <button className="update-btn" onClick={handleUpdate} disabled={updating}>
            {updating ? "Updating..." : "Update"}
          </button>
        </div>
      )}

      <section className="output-row">
        <span className="output-label">Save to</span>
        <button className="output-path" onClick={pickOutputDir}>
          <span>{outputDir ?? "Choose a download folder"}</span><span className="path-change">Change</span>
        </button>
      </section>

      <div className="tabs" role="tablist" aria-label="Downloads">
        <button className={`tab-btn${tab === "queue" ? " active" : ""}`} onClick={() => setTab("queue")}>
          Downloads <span>{items.length || ""}</span>
        </button>
        <button className={`tab-btn${tab === "history" ? " active" : ""}`} onClick={() => setTab("history")}>
          History
        </button>
      </div>

      {tab === "queue" && (
        <>
          <UrlForm disabled={!depsReady || !outputDir} onSubmit={handleAdd} />
          <section className="queue-section">
            <div className="queue-header">
              <h2>Current downloads</h2>
              {hasFinished && (
                <button className="clear-btn" onClick={clearFinished}>Clear finished</button>
              )}
            </div>
            {items.length === 0 ? (
              <div className="queue-empty"><span className="empty-icon">↓</span><strong>Your queue is empty</strong><p>Paste a video link above to get started.</p></div>
            ) : (
              <div className="queue-list">
                {items.map((item, index) => (
                  <QueueItem key={item.id} item={item} onCancel={cancelDownload} onRemove={removeItem} onMove={moveItem} canMoveUp={index > 0 && item.status === "queued"} canMoveDown={index < items.length - 1 && item.status === "queued"} />
                ))}
              </div>
            )}
          </section>
        </>
      )}

      {tab === "history" && (
        <section className="queue-section">
          <div className="queue-header">
            <h2>History</h2>
            {history.length > 0 && (
              <button className="clear-btn" onClick={handleClearHistory}>Clear history</button>
            )}
          </div>
          {history.length > 0 && <input className="history-search" placeholder="Search downloads…" value={historyQuery} onChange={(event) => setHistoryQuery(event.target.value)} />}
          {history.filter((entry) => entry.title.toLowerCase().includes(historyQuery.toLowerCase())).length === 0 ? (
            <div className="queue-empty"><span className="empty-icon">◷</span><strong>No downloads in history</strong><p>Completed downloads will appear here.</p></div>
          ) : (
            <div className="queue-list">
              {history.filter((entry) => entry.title.toLowerCase().includes(historyQuery.toLowerCase())).map((entry) => (
                <div key={entry.id} className="queue-item status-done">
                  <div className="queue-thumb"><div className="queue-thumb-placeholder" /></div>
                  <div className="queue-main">
                    <div className="queue-title">{entry.title}</div>
                    <div className="queue-meta">
                      <span className="queue-format">{entry.format.toUpperCase()}</span>
                      <span className="queue-status">
                        {new Date(entry.timestamp * 1000).toLocaleDateString()}
                      </span>
                    </div>
                  </div>
                  <div className="queue-actions">
                    <button className="queue-action-btn" onClick={() => openFolder(entry.output_path)}>
                      Open folder
                    </button>
                    <button className="queue-action-btn" onClick={() => redownload(entry)}>Download again</button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </section>
      )}

      <section className="diagnostics">
        <button className="diagnostics-toggle" onClick={toggleDiagnostics}>{showDiagnostics ? "Hide system diagnostics" : "System diagnostics"}</button>
        {showDiagnostics && <div className="diagnostics-panel">
          <div><span>yt-dlp</span><strong>{diagnostics?.ytDlpVersion ?? "Not installed"}</strong></div>
          <div><span>FFmpeg</span><strong>{diagnostics?.ffmpegVersion ?? "Not installed"}</strong></div>
          <div className="diagnostics-path"><span>Managed binaries</span><strong title={diagnostics?.binPath}>{diagnostics?.binPath ?? "Unavailable"}</strong></div>
          <button className="repair-btn" onClick={ensureDependencies} disabled={deps.downloading}>{deps.downloading ? "Repairing…" : "Repair dependencies"}</button>
        </div>}
      </section>
    </div>
  );
}
