import { Camera, ChevronDown, Mic, Phone, X } from "lucide-react";
import { useEffect, useRef, useState } from "react";

import { callService, type MediaDeviceInfo } from "../../services/callService";
import { useCallStore } from "../../store/callStore";
import { Avatar } from "../common/Avatar";

export function DeviceSelectionModal() {
  const showDeviceSelection = useCallStore((state) => state.showDeviceSelection);
  const pendingCallInfo = useCallStore((state) => state.pendingCallInfo);
  const hideDeviceSelection = useCallStore((state) => state.hideDeviceSelection);

  const [videoDevices, setVideoDevices] = useState<MediaDeviceInfo[]>([]);
  const [audioDevices, setAudioDevices] = useState<MediaDeviceInfo[]>([]);
  const [selectedVideoId, setSelectedVideoId] = useState<string>("");
  const [selectedAudioId, setSelectedAudioId] = useState<string>("");
  const [previewStream, setPreviewStream] = useState<MediaStream | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [retryCount, setRetryCount] = useState(0);

  const videoRef = useRef<HTMLVideoElement>(null);
  const streamRef = useRef<MediaStream | null>(null);

  // Load devices on mount
  useEffect(() => {
    if (!showDeviceSelection) return;

    const loadDevices = async () => {
      setIsLoading(true);
      setError(null);

      try {
        // First, get the preview stream with NO device constraints (just use defaults)
        // This avoids issues with stale device IDs from localStorage
        const stream = await navigator.mediaDevices.getUserMedia({ video: true, audio: true });
        streamRef.current = stream;
        setPreviewStream(stream);

        // Now enumerate devices - we'll have labels since we have an active stream
        const { videoDevices: vd, audioDevices: ad } = await callService.getAvailableDevices();
        setVideoDevices(vd);
        setAudioDevices(ad);

        // Get the actual device IDs from the active stream tracks
        const videoTrack = stream.getVideoTracks()[0];
        const audioTrack = stream.getAudioTracks()[0];
        const activeVideoId = videoTrack?.getSettings().deviceId || "";
        const activeAudioId = audioTrack?.getSettings().deviceId || "";

        // Use active devices, or fall back to preferred/first available
        const preferred = callService.getPreferredDevices();
        const videoId = activeVideoId ||
          (preferred.videoDeviceId && vd.some(d => d.deviceId === preferred.videoDeviceId) ? preferred.videoDeviceId : vd[0]?.deviceId || "");
        const audioId = activeAudioId ||
          (preferred.audioDeviceId && ad.some(d => d.deviceId === preferred.audioDeviceId) ? preferred.audioDeviceId : ad[0]?.deviceId || "");

        setSelectedVideoId(videoId);
        setSelectedAudioId(audioId);
      } catch (err) {
        console.error("Failed to load devices:", err);
        // Show more specific error message
        if (err instanceof DOMException) {
          if (err.name === "NotReadableError") {
            setError("Camera or microphone is in use by another app. Please close it and try again.");
          } else if (err.name === "NotAllowedError") {
            setError("Permission denied. Please allow camera and microphone access.");
          } else {
            setError(`Could not access camera or microphone: ${err.message}`);
          }
        } else {
          setError("Could not access camera or microphone. Please check permissions.");
        }
      } finally {
        setIsLoading(false);
      }
    };

    loadDevices();

    // Cleanup preview stream on unmount (use ref to avoid stale closure)
    return () => {
      if (streamRef.current) {
        streamRef.current.getTracks().forEach(track => track.stop());
        streamRef.current = null;
      }
    };
  }, [showDeviceSelection, retryCount]);

  const handleRetry = () => {
    // Stop any existing stream before retrying
    if (streamRef.current) {
      streamRef.current.getTracks().forEach(track => track.stop());
      streamRef.current = null;
      setPreviewStream(null);
    }
    setError(null);
    setRetryCount(c => c + 1);
  };

  // Update video preview when stream or loading state changes
  useEffect(() => {
    if (videoRef.current && previewStream) {
      videoRef.current.srcObject = previewStream;
    }
  }, [previewStream, isLoading]);

  // Handle device change
  const handleDeviceChange = async (type: "video" | "audio", deviceId: string) => {
    if (type === "video") {
      setSelectedVideoId(deviceId);
    } else {
      setSelectedAudioId(deviceId);
    }

    // Stop current preview and start new one
    if (streamRef.current) {
      streamRef.current.getTracks().forEach(track => track.stop());
    }

    try {
      const newVideoId = type === "video" ? deviceId : selectedVideoId;
      const newAudioId = type === "audio" ? deviceId : selectedAudioId;
      const stream = await callService.getPreviewStream(newVideoId, newAudioId);
      streamRef.current = stream;
      setPreviewStream(stream);
    } catch (err) {
      console.error("Failed to switch device:", err);
    }
  };

  // Start the call
  const handleStartCall = async () => {
    // Stop preview stream
    if (streamRef.current) {
      streamRef.current.getTracks().forEach(track => track.stop());
      streamRef.current = null;
      setPreviewStream(null);
    }

    // Save preferences
    callService.setPreferredDevices(selectedVideoId, selectedAudioId);

    // Hide modal
    hideDeviceSelection();

    // Start or accept the call based on type
    if (pendingCallInfo?.type === "outgoing" && pendingCallInfo.remoteUserId) {
      await callService.startCallWithDevices(
        pendingCallInfo.remoteUserId,
        pendingCallInfo.remoteUserName || "Unknown",
        pendingCallInfo.remoteUserAvatar,
        selectedVideoId,
        selectedAudioId
      );
    } else if (pendingCallInfo?.type === "incoming") {
      await callService.acceptCallWithDevices(selectedVideoId, selectedAudioId);
    }
  };

  // Cancel
  const handleCancel = () => {
    // Stop preview stream
    if (streamRef.current) {
      streamRef.current.getTracks().forEach(track => track.stop());
      streamRef.current = null;
      setPreviewStream(null);
    }

    // If incoming call, reject it
    if (pendingCallInfo?.type === "incoming") {
      callService.rejectCall("cancelled");
    }

    hideDeviceSelection();
  };

  if (!showDeviceSelection || !pendingCallInfo) return null;

  const isOutgoing = pendingCallInfo.type === "outgoing";
  const remoteName = pendingCallInfo.remoteUserName || "Unknown";
  const remoteAvatar = pendingCallInfo.remoteUserAvatar;

  return (
    <div className="fixed inset-0 z-[60] flex items-center justify-center">
      {/* Backdrop */}
      <div className="absolute inset-0 bg-black/70 backdrop-blur-sm" />

      {/* Modal */}
      <div className="relative w-[480px] max-h-[90vh] bg-[var(--bg-secondary)] rounded-2xl shadow-2xl overflow-hidden animate-scale-in">
        {/* Header */}
        <div className="flex items-center justify-between px-5 py-4 border-b border-[var(--border)]">
          <div className="flex items-center gap-3">
            <Avatar
              src={remoteAvatar}
              name={remoteName}
              size={40}
            />
            <div>
              <h3 className="font-semibold text-[var(--text-primary)]">
                {isOutgoing ? `Call ${remoteName}` : `Call from ${remoteName}`}
              </h3>
              <p className="text-sm text-[var(--text-secondary)]">
                Select your camera and microphone
              </p>
            </div>
          </div>
          <button
            onClick={handleCancel}
            className="p-2 rounded-full hover:bg-[var(--bg-hover)] transition-colors"
          >
            <X size={20} className="text-[var(--text-secondary)]" />
          </button>
        </div>

        {/* Content */}
        <div className="p-5 space-y-5">
          {/* Video Preview */}
          <div className="relative aspect-video bg-black rounded-xl overflow-hidden">
            {isLoading ? (
              <div className="absolute inset-0 flex items-center justify-center">
                <div className="w-8 h-8 border-2 border-[var(--accent)] border-t-transparent rounded-full animate-spin" />
              </div>
            ) : error ? (
              <div className="absolute inset-0 flex flex-col items-center justify-center text-center p-4 gap-3">
                <p className="text-red-400 text-sm">{error}</p>
                <button
                  onClick={handleRetry}
                  className="px-4 py-1.5 text-sm bg-[var(--accent)] text-white rounded-lg hover:opacity-90 transition-opacity"
                >
                  Retry
                </button>
              </div>
            ) : (
              <video
                ref={videoRef}
                autoPlay
                playsInline
                muted
                className="w-full h-full object-cover"
              />
            )}
          </div>

          {/* Device Selectors */}
          <div className="space-y-3">
            {/* Camera Selector */}
            <div className="flex items-center gap-3">
              <div className="w-10 h-10 flex items-center justify-center rounded-full bg-[var(--bg-tertiary)]">
                <Camera size={18} className="text-[var(--text-secondary)]" />
              </div>
              <div className="flex-1 relative">
                <select
                  value={selectedVideoId}
                  onChange={(e) => handleDeviceChange("video", e.target.value)}
                  disabled={isLoading || videoDevices.length === 0}
                  className="w-full appearance-none bg-[var(--bg-tertiary)] text-[var(--text-primary)] px-4 py-2.5 pr-10 rounded-lg border border-[var(--border)] focus:outline-none focus:border-[var(--accent)] disabled:opacity-50 cursor-pointer"
                >
                  {videoDevices.length === 0 ? (
                    <option>No cameras found</option>
                  ) : (
                    videoDevices.map((device) => (
                      <option key={device.deviceId} value={device.deviceId}>
                        {device.label}
                      </option>
                    ))
                  )}
                </select>
                <ChevronDown size={16} className="absolute right-3 top-1/2 -translate-y-1/2 text-[var(--text-secondary)] pointer-events-none" />
              </div>
            </div>

            {/* Microphone Selector */}
            <div className="flex items-center gap-3">
              <div className="w-10 h-10 flex items-center justify-center rounded-full bg-[var(--bg-tertiary)]">
                <Mic size={18} className="text-[var(--text-secondary)]" />
              </div>
              <div className="flex-1 relative">
                <select
                  value={selectedAudioId}
                  onChange={(e) => handleDeviceChange("audio", e.target.value)}
                  disabled={isLoading || audioDevices.length === 0}
                  className="w-full appearance-none bg-[var(--bg-tertiary)] text-[var(--text-primary)] px-4 py-2.5 pr-10 rounded-lg border border-[var(--border)] focus:outline-none focus:border-[var(--accent)] disabled:opacity-50 cursor-pointer"
                >
                  {audioDevices.length === 0 ? (
                    <option>No microphones found</option>
                  ) : (
                    audioDevices.map((device) => (
                      <option key={device.deviceId} value={device.deviceId}>
                        {device.label}
                      </option>
                    ))
                  )}
                </select>
                <ChevronDown size={16} className="absolute right-3 top-1/2 -translate-y-1/2 text-[var(--text-secondary)] pointer-events-none" />
              </div>
            </div>
          </div>
        </div>

        {/* Actions */}
        <div className="flex items-center justify-end gap-3 px-5 py-4 border-t border-[var(--border)]">
          <button
            onClick={handleCancel}
            className="px-5 py-2.5 text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors"
          >
            Cancel
          </button>
          <button
            onClick={handleStartCall}
            disabled={isLoading || !!error}
            className="flex items-center gap-2 px-5 py-2.5 bg-[var(--accent)] text-white font-medium rounded-lg hover:opacity-90 transition-opacity disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <Phone size={18} />
            {isOutgoing ? "Start Call" : "Join Call"}
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
