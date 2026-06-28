import { useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { useDownloadQueue } from "./hooks/useDownloadQueue";
import { UrlForm } from "./components/UrlForm";
import { QueueItem } from "./components/QueueItem";
import { DependencyBanner } from "./components/DependencyBanner";
import type { HistoryEntry, OutputFormat, Quality } from "./types";
import "./App.css";

type Tab = "queue" | "history";

export default function App() {
  const { items, deps, ensureDependencies, addDownload, cancelDownload, removeItem, clearFinished } =
    useDownloadQueue();
  const [outputDir, setOutputDir] = useState<string | null>(null);
  const [tab, setTab] = useState<Tab>("queue");
  const [history, setHistory] = useState<HistoryEntry[]>([]);
  const [ytdlpUpdate, setYtdlpUpdate] = useState(false);
  const [updating, setUpdating] = useState(false);

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

  function handleAdd(urls: string[], format: OutputFormat, quality: Quality) {
    if (!outputDir) return;
    for (const url of urls) addDownload(url, format, quality, outputDir);
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

  const depsReady = deps.ytDlpReady && deps.ffmpegReady;
  const hasFinished = items.some((it) => it.status === "done" || it.status === "cancelled");

  return (
    <div className="app">
      <header className="app-header">
        <h1 className="app-title">Omni<span>form</span></h1>
        <p className="app-subtitle">Convert videos from any platform to any format</p>
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
        <span className="output-label">Output folder</span>
        <button className="output-path" onClick={pickOutputDir}>
          {outputDir ?? "Select a folder..."}
        </button>
      </section>

      <div className="tabs">
        <button className={`tab-btn${tab === "queue" ? " active" : ""}`} onClick={() => setTab("queue")}>
          Queue
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
              <h2>Download queue</h2>
              {hasFinished && (
                <button className="clear-btn" onClick={clearFinished}>Clear finished</button>
              )}
            </div>
            {items.length === 0 ? (
              <div className="queue-empty">No downloads yet.</div>
            ) : (
              <div className="queue-list">
                {items.map((item) => (
                  <QueueItem key={item.id} item={item} onCancel={cancelDownload} onRemove={removeItem} />
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
          {history.length === 0 ? (
            <div className="queue-empty">No history yet.</div>
          ) : (
            <div className="queue-list">
              {history.map((entry) => (
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
                  </div>
                </div>
              ))}
            </div>
          )}
        </section>
      )}
    </div>
  );
}
