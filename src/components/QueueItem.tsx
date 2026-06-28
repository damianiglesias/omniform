import { invoke } from "@tauri-apps/api/core";
import type { DownloadItem } from "../types";

interface Props {
  item: DownloadItem;
  onCancel: (id: string) => void;
  onRemove: (id: string) => void;
}

const STATUS_LABEL: Record<DownloadItem["status"], string> = {
  queued: "Queued",
  "fetching-info": "Fetching info...",
  downloading: "Downloading",
  converting: "Converting",
  done: "Done",
  error: "Error",
  cancelled: "Cancelled",
};

export function QueueItem({ item, onCancel, onRemove }: Props) {
  const isActive =
    item.status === "downloading" ||
    item.status === "converting" ||
    item.status === "fetching-info" ||
    item.status === "queued";

  function openFolder() {
    if (item.outputPath) invoke("open_folder", { path: item.outputPath }).catch(() => {});
  }

  return (
    <div className={`queue-item status-${item.status}`}>
      <div className="queue-thumb">
        {item.thumbnail ? (
          <img src={item.thumbnail} alt="" />
        ) : (
          <div className="queue-thumb-placeholder" />
        )}
      </div>
      <div className="queue-main">
        <div className="queue-title" title={item.title ?? item.url}>
          {item.title ?? item.url}
        </div>
        <div className="queue-meta">
          <span className="queue-format">{item.format.toUpperCase()}</span>
          <span className="queue-status">{STATUS_LABEL[item.status]}</span>
          {item.speed && item.status === "downloading" && (
            <span className="queue-speed">{item.speed}</span>
          )}
          {item.eta && item.status === "downloading" && (
            <span className="queue-eta">ETA {item.eta}</span>
          )}
        </div>
        {item.status === "error" && item.errorMessage && (
          <div className="queue-error">{item.errorMessage}</div>
        )}
        <div className="progress-track">
          <div
            className="progress-fill"
            style={{ width: `${Math.min(100, Math.max(0, item.progress))}%` }}
          />
        </div>
      </div>
      <div className="queue-actions">
        {item.status === "done" && item.outputPath && (
          <button className="queue-action-btn" onClick={openFolder}>
            Open folder
          </button>
        )}
        {isActive ? (
          <button className="queue-action-btn" onClick={() => onCancel(item.id)}>
            Cancel
          </button>
        ) : (
          <button className="queue-action-btn" onClick={() => onRemove(item.id)}>
            Remove
          </button>
        )}
      </div>
    </div>
  );
}
