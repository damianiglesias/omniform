import type { DependencyStatus } from "../types";

interface Props {
  status: DependencyStatus;
  onRetry: () => void;
}

export function DependencyBanner({ status, onRetry }: Props) {
  if (status.ytDlpReady && status.ffmpegReady) return null;

  return (
    <div className="dep-banner">
      {status.downloading ? (
        <>
          <div className="dep-spinner" />
          <span>{status.message ?? "Setting up required components..."}</span>
        </>
      ) : (
        <>
          <span>
            {status.message ?? "Some required components are missing."}
          </span>
          <button className="dep-retry-btn" onClick={onRetry}>
            Retry
          </button>
        </>
      )}
    </div>
  );
}
