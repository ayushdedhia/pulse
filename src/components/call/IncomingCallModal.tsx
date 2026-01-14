import { Phone, PhoneOff, Video } from "lucide-react";

import { callService } from "../../services";
import { useCallStore } from "../../store/callStore";
import { Avatar } from "../common/Avatar";

export function IncomingCallModal() {
  const callStatus = useCallStore((state) => state.callStatus);
  const remoteUserName = useCallStore((state) => state.remoteUserName);
  const remoteUserAvatar = useCallStore((state) => state.remoteUserAvatar);
  const showDeviceSelection = useCallStore((state) => state.showDeviceSelection);
  const showDeviceSelectionForIncoming = useCallStore((state) => state.showDeviceSelectionForIncoming);

  // Don't show if not incoming OR if device selection is showing
  if (callStatus !== "incoming" || showDeviceSelection) return null;

  const handleAccept = () => {
    // Show device selection modal instead of directly accepting
    showDeviceSelectionForIncoming();
  };

  const handleReject = () => {
    callService.rejectCall("declined");
  };

  return (
    <div className="fixed inset-0 z-[60] flex items-center justify-center">
      {/* Backdrop */}
      <div className="absolute inset-0 bg-black/70 backdrop-blur-sm" />

      {/* Modal */}
      <div className="relative w-[320px] bg-[var(--bg-secondary)] rounded-2xl shadow-2xl overflow-hidden animate-scale-in">
        {/* Gradient Header */}
        <div className="relative h-32 bg-gradient-to-br from-[var(--accent)] to-[#006d5b] flex items-center justify-center">
          <div className="absolute inset-0 bg-black/10" />
          <div className="relative">
            <Avatar
              src={remoteUserAvatar || undefined}
              name={remoteUserName || "Unknown"}
              size={80}
              className="ring-4 ring-white/20"
            />
            <span className="absolute -bottom-1 -right-1 w-6 h-6 bg-[var(--accent)] rounded-full flex items-center justify-center ring-2 ring-[var(--bg-secondary)]">
              <Video size={14} className="text-white" />
            </span>
          </div>
        </div>

        {/* Content */}
        <div className="px-6 py-4 text-center">
          <h3 className="text-xl font-semibold text-[var(--text-primary)]">
            {remoteUserName || "Unknown"}
          </h3>
          <p className="mt-1 text-sm text-[var(--text-secondary)] animate-pulse">
            Incoming video call...
          </p>
        </div>

        {/* Action Buttons */}
        <div className="flex items-center justify-center gap-8 px-6 pb-6">
          {/* Reject */}
          <button
            onClick={handleReject}
            className="flex flex-col items-center gap-2 group"
          >
            <span className="w-14 h-14 rounded-full bg-red-500 flex items-center justify-center hover:bg-red-600 transition-colors shadow-lg group-active:scale-95">
              <PhoneOff size={24} className="text-white" />
            </span>
            <span className="text-xs text-[var(--text-secondary)]">Decline</span>
          </button>

          {/* Accept */}
          <button
            onClick={handleAccept}
            className="flex flex-col items-center gap-2 group"
          >
            <span className="w-14 h-14 rounded-full bg-[var(--accent)] flex items-center justify-center hover:opacity-90 transition-opacity shadow-lg group-active:scale-95 animate-pulse-ring">
              <Phone size={24} className="text-white" />
            </span>
            <span className="text-xs text-[var(--text-secondary)]">Accept</span>
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
        @keyframes pulse-ring {
          0% {
            box-shadow: 0 0 0 0 rgba(0, 168, 132, 0.5);
          }
          70% {
            box-shadow: 0 0 0 10px rgba(0, 168, 132, 0);
          }
          100% {
            box-shadow: 0 0 0 0 rgba(0, 168, 132, 0);
          }
        }
        .animate-pulse-ring {
          animation: pulse-ring 1.5s infinite;
        }
      `}</style>
    </div>
  );
}
