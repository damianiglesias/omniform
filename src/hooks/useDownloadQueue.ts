import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  DependencyStatus,
  DownloadDoneEvent,
  DownloadErrorEvent,
  DownloadInfoEvent,
  DownloadItem,
  DownloadProgressEvent,
  OutputFormat,
  Quality,
} from "../types";

function makeId(): string {
  return `${Date.now()}-${Math.random().toString(36).slice(2, 9)}`;
}

export function useDownloadQueue() {
  const [items, setItems] = useState<DownloadItem[]>([]);
  const [deps, setDeps] = useState<DependencyStatus>({
    ytDlpReady: false,
    ffmpegReady: false,
    downloading: false,
    message: null,
  });
  const itemsRef = useRef(items);
  itemsRef.current = items;

  useEffect(() => {
    invoke<DependencyStatus>("check_dependencies").then(setDeps).catch(() => {});

    const unlistenInfo = listen<DownloadInfoEvent>("download://info", (e) => {
      setItems((prev) =>
        prev.map((it) =>
          it.id === e.payload.id
            ? { ...it, title: e.payload.title, thumbnail: e.payload.thumbnail }
            : it
        )
      );
    });

    const unlistenProgress = listen<DownloadProgressEvent>(
      "download://progress",
      (e) => {
        setItems((prev) =>
          prev.map((it) =>
            it.id === e.payload.id
              ? {
                  ...it,
                  progress: e.payload.progress,
                  speed: e.payload.speed,
                  eta: e.payload.eta,
                  status: e.payload.status,
                }
              : it
          )
        );
      }
    );

    const unlistenDone = listen<DownloadDoneEvent>("download://done", (e) => {
      setItems((prev) =>
        prev.map((it) =>
          it.id === e.payload.id
            ? { ...it, status: "done", progress: 100, outputPath: e.payload.outputPath }
            : it
        )
      );
    });

    const unlistenError = listen<DownloadErrorEvent>("download://error", (e) => {
      setItems((prev) =>
        prev.map((it) =>
          it.id === e.payload.id
            ? { ...it, status: "error", errorMessage: e.payload.message }
            : it
        )
      );
    });

    const unlistenDeps = listen<DependencyStatus>("deps://status", (e) => {
      setDeps(e.payload);
    });

    return () => {
      unlistenInfo.then((f) => f());
      unlistenProgress.then((f) => f());
      unlistenDone.then((f) => f());
      unlistenError.then((f) => f());
      unlistenDeps.then((f) => f());
    };
  }, []);

  const ensureDependencies = useCallback(async () => {
    await invoke("ensure_dependencies");
  }, []);

  const addDownload = useCallback(
    async (url: string, format: OutputFormat, quality: Quality, outputDir: string) => {
      const id = makeId();
      const newItem: DownloadItem = {
        id,
        url,
        title: null,
        thumbnail: null,
        format,
        quality,
        status: "queued",
        progress: 0,
        speed: null,
        eta: null,
        errorMessage: null,
        outputPath: null,
      };
      setItems((prev) => [...prev, newItem]);
      try {
        await invoke("start_download", { id, url, format, quality, outputDir });
      } catch (err) {
        setItems((prev) =>
          prev.map((it) =>
            it.id === id
              ? { ...it, status: "error", errorMessage: String(err) }
              : it
          )
        );
      }
      return id;
    },
    []
  );

  const cancelDownload = useCallback(async (id: string) => {
    await invoke("cancel_download", { id });
    setItems((prev) =>
      prev.map((it) => (it.id === id ? { ...it, status: "cancelled" } : it))
    );
  }, []);

  const removeItem = useCallback((id: string) => {
    setItems((prev) => prev.filter((it) => it.id !== id));
  }, []);

  const clearFinished = useCallback(() => {
    setItems((prev) =>
      prev.filter((it) => it.status !== "done" && it.status !== "cancelled")
    );
  }, []);

  return {
    items,
    deps,
    ensureDependencies,
    addDownload,
    cancelDownload,
    removeItem,
    clearFinished,
  };
}
