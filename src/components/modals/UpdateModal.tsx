import { Download, RefreshCw, X } from "lucide-react";
import { useState } from "react";

import { updaterService } from "../../services";
import type { UpdateInfo, DownloadProgress } from "../../services";

interface UpdateModalProps {
  updateInfo: UpdateInfo;
  onClose: () => void;
  onSkipVersion: (version: string) => void;
}

export function UpdateModal({ updateInfo, onClose, onSkipVersion }: UpdateModalProps) {
  const [downloading, setDownloading] = useState(false);
  const [progress, setProgress] = useState<DownloadProgress | null>(null);
  const [error, setError] = useState<string | null>(null);

  const handleUpdate = async () => {
    setDownloading(true);
    setError(null);

    try {
      await updaterService.installCached((prog) => {
        setProgress(prog);
      });
    } catch (err) {
      console.error("Update failed:", err);
      setError(err instanceof Error ? err.message : "Update failed");
      setDownloading(false);
    }
  };

  const handleSkip = () => {
    onSkipVersion(updateInfo.version);
    onClose();
  };

  const formatBytes = (bytes: number): string => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      {/* Backdrop */}
      <div
        className="absolute inset-0 bg-black/50"
        onClick={downloading ? undefined : onClose}
      />

      {/* Modal */}
      <div className="relative w-[420px] bg-[var(--bg-primary)] rounded-lg shadow-2xl animate-fade-in">
        {/* Header */}
        <header className="flex items-center justify-between px-5 py-4 border-b border-[var(--border)]">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-full bg-[var(--accent)]/10 flex items-center justify-center">
              <Download size={20} className="text-[var(--accent)]" />
            </div>
            <div>
              <h2 className="text-lg font-medium text-[var(--text-primary)]">
                Update Available
              </h2>
              <p className="text-xs text-[var(--text-secondary)]">
                v{updateInfo.currentVersion} &rarr; v{updateInfo.version}
              </p>
            </div>
          </div>
          {!downloading && (
            <button
              onClick={onClose}
              className="text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors"
            >
              <X size={20} />
            </button>
          )}
        </header>

        {/* Content */}
        <div className="p-5">
          {/* Release Notes */}
          {updateInfo.body && (
            <div className="mb-4">
              <label className="text-xs text-[var(--accent)] mb-2 block font-medium">
                What's New
              </label>
              <div className="max-h-32 overflow-y-auto p-3 rounded-lg bg-[var(--bg-secondary)] text-sm text-[var(--text-secondary)]">
                {updateInfo.body}
              </div>
            </div>
          )}

          {/* Progress Bar */}
          {downloading && (
            <div className="mb-4">
              <div className="flex items-center justify-between mb-2">
                <span className="text-sm text-[var(--text-primary)]">
                  {progress ? "Downloading..." : "Preparing..."}
                </span>
                {progress && (
                  <span className="text-xs text-[var(--text-secondary)]">
                    {formatBytes(progress.downloaded)}
                  </span>
                )}
              </div>
              <div className="h-2 rounded-full bg-[var(--bg-secondary)] overflow-hidden">
                <div
                  className="h-full bg-[var(--accent)] transition-all duration-300"
                  style={{
                    width: progress?.total
                      ? `${(progress.downloaded / progress.total) * 100}%`
                      : "0%",
                  }}
                />
              </div>
              {!progress?.total && (
                <div className="h-2 rounded-full bg-[var(--bg-secondary)] overflow-hidden mt-1">
                  <div className="h-full w-1/3 bg-[var(--accent)] animate-pulse-bar" />
                </div>
              )}
            </div>
          )}

          {/* Error */}
          {error && (
            <div className="mb-4 p-3 rounded-lg bg-red-500/10 text-red-500 text-sm">
              {error}
            </div>
          )}

          {/* Actions */}
          <div className="flex gap-3">
            {!downloading ? (
              <>
                <button
                  onClick={handleSkip}
                  className="flex-1 py-2.5 px-4 rounded-lg bg-[var(--bg-secondary)] text-[var(--text-primary)] font-medium hover:bg-[var(--bg-hover)] transition-colors"
                >
                  Skip Version
                </button>
                <button
                  onClick={handleUpdate}
                  className="flex-1 py-2.5 px-4 rounded-lg bg-[var(--accent)] text-white font-medium hover:bg-[var(--accent-dark)] transition-colors flex items-center justify-center gap-2"
                >
                  <RefreshCw size={16} />
                  Update Now
                </button>
              </>
            ) : (
              <button
                disabled
                className="flex-1 py-2.5 px-4 rounded-lg bg-[var(--accent)] text-white font-medium opacity-50 cursor-not-allowed flex items-center justify-center gap-2"
              >
                <RefreshCw size={16} className="animate-spin" />
                Installing...
              </button>
            )}
          </div>

          {!downloading && (
            <button
              onClick={onClose}
              className="w-full mt-3 py-2 text-sm text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors"
            >
              Remind me later
            </button>
          )}
        </div>
      </div>

      <style>{`
        @keyframes fade-in {
          from {
            opacity: 0;
            transform: scale(0.95);
          }
          to {
            opacity: 1;
            transform: scale(1);
          }
        }
        .animate-fade-in {
          animation: fade-in 0.15s ease-out;
        }
        @keyframes pulse-bar {
          0%, 100% {
            transform: translateX(-100%);
          }
          50% {
            transform: translateX(300%);
          }
        }
        .animate-pulse-bar {
          animation: pulse-bar 1.5s ease-in-out infinite;
        }
      `}</style>
    </div>
  );
}
