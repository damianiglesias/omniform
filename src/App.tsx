import { useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { useDownloadQueue } from "./hooks/useDownloadQueue";
import { UrlForm } from "./components/UrlForm";
import { QueueItem } from "./components/QueueItem";
import { DependencyBanner } from "./components/DependencyBanner";
import type { OutputFormat, Quality } from "./types";
import "./App.css";

export default function App() {
  const {
    items,
    deps,
    ensureDependencies,
    addDownload,
    cancelDownload,
    removeItem,
    clearFinished,
  } = useDownloadQueue();

  const [outputDir, setOutputDir] = useState<string | null>(null);

  useEffect(() => {
    invoke<string>("get_default_output_dir")
      .then(setOutputDir)
      .catch(() => {});
  }, []);

  useEffect(() => {
    if (!deps.ytDlpReady || !deps.ffmpegReady) {
      ensureDependencies();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  async function pickOutputDir() {
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected === "string") {
      setOutputDir(selected);
    }
  }

  function handleAdd(urls: string[], format: OutputFormat, quality: Quality) {
    if (!outputDir) return;
    for (const url of urls) {
      addDownload(url, format, quality, outputDir);
    }
  }

  const depsReady = deps.ytDlpReady && deps.ffmpegReady;
  const hasFinished = items.some(
    (it) => it.status === "done" || it.status === "cancelled"
  );

  return (
    <div className="app">
      <header className="app-header">
        <h1 className="app-title">
          Omni<span>form</span>
        </h1>
        <p className="app-subtitle">
          Convert videos from any platform to any format
        </p>
      </header>

      <DependencyBanner status={deps} onRetry={ensureDependencies} />

      <section className="output-row">
        <span className="output-label">Output folder</span>
        <button className="output-path" onClick={pickOutputDir}>
          {outputDir ?? "Select a folder..."}
        </button>
      </section>

      <UrlForm disabled={!depsReady || !outputDir} onSubmit={handleAdd} />

      <section className="queue-section">
        <div className="queue-header">
          <h2>Download queue</h2>
          {hasFinished && (
            <button className="clear-btn" onClick={clearFinished}>
              Clear finished
            </button>
          )}
        </div>

        {items.length === 0 ? (
          <div className="queue-empty">No downloads yet.</div>
        ) : (
          <div className="queue-list">
            {items.map((item) => (
              <QueueItem
                key={item.id}
                item={item}
                onCancel={cancelDownload}
                onRemove={removeItem}
              />
            ))}
          </div>
        )}
      </section>
    </div>
  );
}
