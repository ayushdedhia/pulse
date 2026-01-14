import { Camera, CameraOff, Mic, MicOff, PhoneOff } from "lucide-react";
import { useEffect, useRef, useState } from "react";

import { callService } from "../../services";
import { useCallStore } from "../../store/callStore";
import { Avatar } from "../common/Avatar";

export function CallOverlay() {
  const callStatus = useCallStore((state) => state.callStatus);
  const remoteUserName = useCallStore((state) => state.remoteUserName);
  const remoteUserAvatar = useCallStore((state) => state.remoteUserAvatar);
  const isMuted = useCallStore((state) => state.isMuted);
  const isCameraOff = useCallStore((state) => state.isCameraOff);
  const localStream = useCallStore((state) => state.localStream);
  const remoteStream = useCallStore((state) => state.remoteStream);
  const callStartTime = useCallStore((state) => state.callStartTime);
  const toggleMute = useCallStore((state) => state.toggleMute);
  const toggleCamera = useCallStore((state) => state.toggleCamera);

  const localVideoRef = useRef<HTMLVideoElement>(null);
  const remoteVideoRef = useRef<HTMLVideoElement>(null);
  const [duration, setDuration] = useState("00:00");

  // Connect local stream to video element
  useEffect(() => {
    if (localVideoRef.current && localStream) {
      localVideoRef.current.srcObject = localStream;
    }
  }, [localStream]);

  // Connect remote stream to video element
  useEffect(() => {
    if (remoteVideoRef.current && remoteStream) {
      remoteVideoRef.current.srcObject = remoteStream;
    }
  }, [remoteStream]);

  // Update call duration timer
  useEffect(() => {
    if (callStatus !== "connected" || !callStartTime) return;

    const updateDuration = () => {
      const elapsed = Math.floor((Date.now() - callStartTime) / 1000);
      const minutes = Math.floor(elapsed / 60);
      const seconds = elapsed % 60;
      setDuration(`${minutes.toString().padStart(2, "0")}:${seconds.toString().padStart(2, "0")}`);
    };

    updateDuration();
    const interval = setInterval(updateDuration, 1000);
    return () => clearInterval(interval);
  }, [callStatus, callStartTime]);

  // Only show for outgoing, connecting, and connected states
  if (callStatus === "idle" || callStatus === "incoming") return null;

  const handleHangup = () => {
    callService.hangup();
  };

  const getStatusText = () => {
    switch (callStatus) {
      case "outgoing":
        return "Calling...";
      case "connecting":
        return "Connecting...";
      case "connected":
        return duration;
      default:
        return "";
    }
  };

  return (
    <div className="fixed inset-0 z-[60] bg-[#0a0f14] flex flex-col">
      {/* Remote Video (Background) */}
      {remoteStream && callStatus === "connected" ? (
        <video
          ref={remoteVideoRef}
          autoPlay
          playsInline
          className="absolute inset-0 w-full h-full object-cover"
        />
      ) : (
        // Avatar placeholder when no remote stream
        <div className="absolute inset-0 flex items-center justify-center bg-gradient-to-br from-[#1a2530] to-[#0a0f14]">
          <div className="text-center">
            <Avatar
              src={remoteUserAvatar || undefined}
              name={remoteUserName || "Unknown"}
              size={120}
              className="mx-auto ring-4 ring-white/10"
            />
            <h2 className="mt-4 text-2xl font-semibold text-white">
              {remoteUserName || "Unknown"}
            </h2>
            <p className="mt-2 text-[var(--text-secondary)] animate-pulse">
              {getStatusText()}
            </p>
          </div>
        </div>
      )}

      {/* Top Bar - Status */}
      {callStatus === "connected" && (
        <div className="relative z-10 flex items-center justify-center py-4 bg-gradient-to-b from-black/50 to-transparent">
          <div className="px-4 py-2 bg-black/40 rounded-full backdrop-blur-sm">
            <p className="text-sm text-white font-medium">
              {remoteUserName} • {duration}
            </p>
          </div>
        </div>
      )}

      {/* Local Video (Picture-in-Picture) */}
      {localStream && (
        <div className="absolute top-20 right-4 w-32 h-44 rounded-2xl overflow-hidden shadow-2xl ring-2 ring-white/20 z-20">
          <video
            ref={localVideoRef}
            autoPlay
            playsInline
            muted
            className={`w-full h-full object-cover ${isCameraOff ? "hidden" : ""}`}
          />
          {isCameraOff && (
            <div className="w-full h-full bg-[var(--bg-secondary)] flex items-center justify-center">
              <CameraOff size={32} className="text-[var(--text-secondary)]" />
            </div>
          )}
        </div>
      )}

      {/* Bottom Controls */}
      <div className="absolute bottom-0 left-0 right-0 z-10 pb-10 pt-20 bg-gradient-to-t from-black/70 to-transparent">
        <div className="flex items-center justify-center gap-6">
          {/* Mute */}
          <button
            onClick={toggleMute}
            className={`w-14 h-14 rounded-full flex items-center justify-center transition-all active:scale-95 ${
              isMuted
                ? "bg-white text-black"
                : "bg-white/20 text-white hover:bg-white/30"
            }`}
          >
            {isMuted ? <MicOff size={24} /> : <Mic size={24} />}
          </button>

          {/* Hangup */}
          <button
            onClick={handleHangup}
            className="w-16 h-16 rounded-full bg-red-500 flex items-center justify-center hover:bg-red-600 transition-colors active:scale-95 shadow-lg"
          >
            <PhoneOff size={28} className="text-white" />
          </button>

          {/* Camera Toggle */}
          <button
            onClick={toggleCamera}
            className={`w-14 h-14 rounded-full flex items-center justify-center transition-all active:scale-95 ${
              isCameraOff
                ? "bg-white text-black"
                : "bg-white/20 text-white hover:bg-white/30"
            }`}
          >
            {isCameraOff ? <CameraOff size={24} /> : <Camera size={24} />}
          </button>
        </div>
      </div>
    </div>
  );
}
