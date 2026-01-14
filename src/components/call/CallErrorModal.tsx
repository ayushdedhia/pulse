import { AlertTriangle, X } from "lucide-react";

import { useCallStore } from "../../store/callStore";

export function CallErrorModal() {
  const callError = useCallStore((state) => state.callError);
  const setCallError = useCallStore((state) => state.setCallError);

  if (!callError) return null;

  const handleClose = () => {
    setCallError(null);
  };

  return (
    <div className="fixed inset-0 z-[60] flex items-center justify-center">
      {/* Backdrop */}
      <div
        className="absolute inset-0 bg-black/70 backdrop-blur-sm"
        onClick={handleClose}
      />

      {/* Modal */}
      <div className="relative w-[360px] bg-[var(--bg-secondary)] rounded-2xl shadow-2xl overflow-hidden animate-scale-in">
        {/* Header */}
        <div className="relative flex items-center justify-between px-4 py-3 bg-red-500/10 border-b border-red-500/20">
          <div className="flex items-center gap-2">
            <AlertTriangle size={20} className="text-red-500" />
            <h3 className="font-semibold text-[var(--text-primary)]">
              Call Failed
            </h3>
          </div>
          <button
            onClick={handleClose}
            className="p-1 rounded-full hover:bg-[var(--bg-hover)] transition-colors"
          >
            <X size={18} className="text-[var(--text-secondary)]" />
          </button>
        </div>

        {/* Content */}
        <div className="px-4 py-4">
          <p className="text-sm text-[var(--text-secondary)] leading-relaxed">
            {callError}
          </p>
        </div>

        {/* Actions */}
        <div className="flex justify-end px-4 pb-4">
          <button
            onClick={handleClose}
            className="px-4 py-2 bg-[var(--accent)] text-white text-sm font-medium rounded-lg hover:opacity-90 transition-opacity"
          >
            OK
          </button>
        </div>
      </div>

      <style>{`
        @keyframes scale-in {
          from {
            transform: scale(0.9);
            opacity: 0;
          }
          to {
            transform: scale(1);
            opacity: 1;
          }
        }
        .animate-scale-in {
          animation: scale-in 0.2s ease-out;
        }
      `}</style>
    </div>
  );
}
