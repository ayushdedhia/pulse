import { useCallStore } from "../store/callStore";
import { useUserStore } from "../store/userStore";
import type { CallMessage } from "../types";

// STUN server for NAT traversal (free Google STUN)
const ICE_SERVERS: RTCIceServer[] = [
  { urls: "stun:stun.l.google.com:19302" },
  { urls: "stun:stun1.l.google.com:19302" },
];

// Call timeout in milliseconds (30 seconds)
const CALL_TIMEOUT_MS = 30000;

// LocalStorage keys for device preferences
const PREFERRED_VIDEO_DEVICE_KEY = "pulse_preferred_video_device";
const PREFERRED_AUDIO_DEVICE_KEY = "pulse_preferred_audio_device";

// Audio elements for call sounds
let ringtoneAudio: HTMLAudioElement | null = null;
let dialtoneAudio: HTMLAudioElement | null = null;
let hangupAudio: HTMLAudioElement | null = null;

export interface MediaDeviceInfo {
  deviceId: string;
  label: string;
  kind: "videoinput" | "audioinput";
}

class CallService {
  private pc: RTCPeerConnection | null = null;
  private sendMessage: ((msg: CallMessage) => void) | null = null;
  private callTimeoutId: number | null = null;
  private reconnectTimeoutId: number | null = null;
  private pendingIceCandidates: RTCIceCandidateInit[] = [];

  /**
   * Initialize the call service with a WebSocket send function
   */
  setSendMessage(sendFn: (msg: CallMessage) => void) {
    this.sendMessage = sendFn;
  }

  /**
   * Get available media devices (cameras and microphones)
   * Note: This only enumerates devices without requesting media access.
   * The preview stream request will trigger the permission prompt if needed.
   */
  async getAvailableDevices(): Promise<{ videoDevices: MediaDeviceInfo[]; audioDevices: MediaDeviceInfo[] }> {
    try {
      const devices = await navigator.mediaDevices.enumerateDevices();

      const videoDevices = devices
        .filter(d => d.kind === "videoinput")
        .map(d => ({
          deviceId: d.deviceId,
          label: d.label || `Camera ${d.deviceId.slice(0, 8)}`,
          kind: "videoinput" as const,
        }));

      const audioDevices = devices
        .filter(d => d.kind === "audioinput")
        .map(d => ({
          deviceId: d.deviceId,
          label: d.label || `Microphone ${d.deviceId.slice(0, 8)}`,
          kind: "audioinput" as const,
        }));

      return { videoDevices, audioDevices };
    } catch (error) {
      console.error("Failed to enumerate devices:", error);
      return { videoDevices: [], audioDevices: [] };
    }
  }

  /**
   * Get preferred device IDs from localStorage
   */
  getPreferredDevices(): { videoDeviceId: string | null; audioDeviceId: string | null } {
    return {
      videoDeviceId: localStorage.getItem(PREFERRED_VIDEO_DEVICE_KEY),
      audioDeviceId: localStorage.getItem(PREFERRED_AUDIO_DEVICE_KEY),
    };
  }

  /**
   * Save preferred device IDs to localStorage
   */
  setPreferredDevices(videoDeviceId: string | null, audioDeviceId: string | null): void {
    if (videoDeviceId) {
      localStorage.setItem(PREFERRED_VIDEO_DEVICE_KEY, videoDeviceId);
    }
    if (audioDeviceId) {
      localStorage.setItem(PREFERRED_AUDIO_DEVICE_KEY, audioDeviceId);
    }
  }

  /**
   * Get a preview stream for device selection (without starting a call)
   */
  async getPreviewStream(videoDeviceId?: string, audioDeviceId?: string): Promise<MediaStream> {
    // If specific devices requested, try exact first to ensure the right device is used
    if (videoDeviceId || audioDeviceId) {
      try {
        const exactConstraints: MediaStreamConstraints = {
          video: videoDeviceId ? { deviceId: { exact: videoDeviceId } } : true,
          audio: audioDeviceId ? { deviceId: { exact: audioDeviceId } } : true,
        };
        return await navigator.mediaDevices.getUserMedia(exactConstraints);
      } catch (error) {
        console.warn("Failed with exact constraints, trying ideal:", error);
        // Fall back to ideal (for virtual cameras that don't support exact)
        try {
          const idealConstraints: MediaStreamConstraints = {
            video: videoDeviceId ? { deviceId: { ideal: videoDeviceId } } : true,
            audio: audioDeviceId ? { deviceId: { ideal: audioDeviceId } } : true,
          };
          return await navigator.mediaDevices.getUserMedia(idealConstraints);
        } catch (error2) {
          console.warn("Failed with ideal constraints, trying defaults:", error2);
        }
      }
    }

    // Fall back to default devices
    return navigator.mediaDevices.getUserMedia({ video: true, audio: true });
  }

  /**
   * Get local media stream (camera + microphone) with optional device selection
   */
  async getLocalMedia(videoDeviceId?: string, audioDeviceId?: string): Promise<MediaStream> {
    // Use provided device IDs, or fall back to stored preferences, or use defaults
    const preferredDevices = this.getPreferredDevices();
    const videoId = videoDeviceId || preferredDevices.videoDeviceId;
    const audioId = audioDeviceId || preferredDevices.audioDeviceId;

    // If specific devices, try exact first to ensure the right device is used
    if (videoId || audioId) {
      try {
        const exactConstraints: MediaStreamConstraints = {
          video: videoId ? { deviceId: { exact: videoId } } : true,
          audio: audioId ? { deviceId: { exact: audioId } } : true,
        };
        return await navigator.mediaDevices.getUserMedia(exactConstraints);
      } catch (error) {
        console.warn("Failed with exact constraints, trying ideal:", error);
        // Fall back to ideal (for virtual cameras that don't support exact)
        try {
          const idealConstraints: MediaStreamConstraints = {
            video: videoId ? { deviceId: { ideal: videoId } } : true,
            audio: audioId ? { deviceId: { ideal: audioId } } : true,
          };
          return await navigator.mediaDevices.getUserMedia(idealConstraints);
        } catch (error2) {
          console.warn("Failed with ideal constraints, trying defaults:", error2);
        }
      }
    }

    // Fall back to default devices
    try {
      return await navigator.mediaDevices.getUserMedia({ video: true, audio: true });
    } catch (fallbackError) {
      console.error("Failed to get local media:", fallbackError);
      throw fallbackError;
    }
  }

  /**
   * Initialize RTCPeerConnection with event handlers
   */
  private initPeerConnection(): RTCPeerConnection {
    if (this.pc) {
      this.pc.close();
    }

    this.pc = new RTCPeerConnection({ iceServers: ICE_SERVERS });

    // Handle ICE candidates
    this.pc.onicecandidate = (event) => {
      if (event.candidate && this.sendMessage) {
        const { callId, remoteUserId } = useCallStore.getState();
        const currentUserId = this.getCurrentUserId();
        if (callId && remoteUserId && currentUserId) {
          this.sendMessage({
            type: "rtc_ice_candidate",
            call_id: callId,
            from_user_id: currentUserId,
            to_user_id: remoteUserId,
            candidate: JSON.stringify(event.candidate),
          });
        }
      }
    };

    // Handle connection state changes
    this.pc.onconnectionstatechange = () => {
      console.log("Connection state:", this.pc?.connectionState);
      if (this.pc?.connectionState === "connected") {
        useCallStore.getState().setCallStatus("connected");
        useCallStore.getState().setCallStartTime(Date.now());
        this.stopDialtone();
        this.stopRingtone();
        // Clear any reconnection timeout
        if (this.reconnectTimeoutId) {
          clearTimeout(this.reconnectTimeoutId);
          this.reconnectTimeoutId = null;
        }
      } else if (this.pc?.connectionState === "disconnected") {
        // "disconnected" is often temporary - wait before hanging up
        console.log("Connection disconnected, waiting for reconnection...");
        if (!this.reconnectTimeoutId) {
          this.reconnectTimeoutId = window.setTimeout(() => {
            if (this.pc?.connectionState === "disconnected" || this.pc?.connectionState === "failed") {
              console.log("Reconnection timeout - hanging up");
              this.hangup("connection_failed");
            }
          }, 10000); // 10 seconds to reconnect
        }
      } else if (this.pc?.connectionState === "failed") {
        this.hangup("connection_failed");
      }
    };

    // Handle remote tracks
    this.pc.ontrack = (event) => {
      console.log("Received remote track:", event.track.kind);
      const [remoteStream] = event.streams;
      if (remoteStream) {
        useCallStore.getState().setRemoteStream(remoteStream);
      }
    };

    return this.pc;
  }

  /**
   * Start an outgoing call
   */
  async startCall(remoteUserId: string, remoteUserName: string, avatarUrl?: string): Promise<void> {
    console.log("startCall called:", { remoteUserId, remoteUserName, avatarUrl });

    if (!this.sendMessage) {
      console.error("Call service not initialized - sendMessage is null");
      return;
    }

    const currentUserId = this.getCurrentUserId();
    console.log("currentUserId:", currentUserId);

    if (!currentUserId) {
      console.error("No current user");
      return;
    }

    // Generate call ID and update store
    const callId = useCallStore.getState().startOutgoingCall(remoteUserId, remoteUserName, avatarUrl);

    // Get local media first
    try {
      const localStream = await this.getLocalMedia();
      useCallStore.getState().setLocalStream(localStream);
    } catch (err) {
      console.error("Failed to get local media, aborting call:", err);
      useCallStore.getState().reset();
      useCallStore.getState().setCallError(this.getMediaErrorMessage(err));
      return;
    }

    // Get caller's name and avatar for display on recipient's side
    const currentUserName = useUserStore.getState().currentUser?.name || "Unknown";
    const currentUserAvatar = useUserStore.getState().currentUser?.avatar_url;

    // Send call invite
    this.sendMessage({
      type: "call_invite",
      call_id: callId,
      from_user_id: currentUserId,
      to_user_id: remoteUserId,
      kind: "video",
      from_user_name: currentUserName,
      from_user_avatar: currentUserAvatar,
    });

    // Play dialtone
    this.playDialtone();

    // Set timeout for no answer
    this.callTimeoutId = window.setTimeout(() => {
      const { callStatus } = useCallStore.getState();
      if (callStatus === "outgoing") {
        console.log("Call timeout - no answer");
        this.hangup("no_answer");
      }
    }, CALL_TIMEOUT_MS);
  }

  /**
   * Accept an incoming call
   */
  async acceptCall(): Promise<void> {
    console.log("[acceptCall] Starting...");
    const { callId, remoteUserId, callStatus } = useCallStore.getState();
    const currentUserId = this.getCurrentUserId();

    console.log("[acceptCall] State:", { callId, remoteUserId, callStatus, currentUserId, hasSendMessage: !!this.sendMessage });

    if (callStatus !== "incoming" || !callId || !remoteUserId || !currentUserId || !this.sendMessage) {
      console.error("[acceptCall] Cannot accept call - invalid state");
      return;
    }

    // Stop ringtone
    this.stopRingtone();

    // Get local media
    try {
      console.log("[acceptCall] Getting local media...");
      const localStream = await this.getLocalMedia();
      console.log("[acceptCall] Got local stream with tracks:", localStream.getTracks().map(t => t.kind));
      useCallStore.getState().setLocalStream(localStream);

      // Initialize peer connection and add tracks
      console.log("[acceptCall] Initializing peer connection...");
      this.initPeerConnection();
      localStream.getTracks().forEach((track) => {
        this.pc?.addTrack(track, localStream);
      });
      console.log("[acceptCall] Peer connection initialized, tracks added");
    } catch (err) {
      console.error("[acceptCall] Failed to get local media:", err);
      this.rejectCall("media_error");
      useCallStore.getState().setCallError(this.getMediaErrorMessage(err));
      return;
    }

    // Update status
    console.log("[acceptCall] Setting status to connecting...");
    useCallStore.getState().setCallStatus("connecting");

    // Send accept message
    console.log("[acceptCall] Sending call_accept message...");
    this.sendMessage({
      type: "call_accept",
      call_id: callId,
      from_user_id: currentUserId,
      to_user_id: remoteUserId,
    });
    console.log("[acceptCall] Done!");
  }

  /**
   * Start an outgoing call with specific devices (called from DeviceSelectionModal)
   */
  async startCallWithDevices(
    remoteUserId: string,
    remoteUserName: string,
    avatarUrl?: string,
    videoDeviceId?: string,
    audioDeviceId?: string
  ): Promise<void> {
    console.log("startCallWithDevices called:", { remoteUserId, remoteUserName, videoDeviceId, audioDeviceId });

    if (!this.sendMessage) {
      console.error("Call service not initialized - sendMessage is null");
      return;
    }

    const currentUserId = this.getCurrentUserId();
    if (!currentUserId) {
      console.error("No current user");
      return;
    }

    // Generate call ID and update store
    const callId = useCallStore.getState().startOutgoingCall(remoteUserId, remoteUserName, avatarUrl);

    // Get local media with specified devices
    try {
      const localStream = await this.getLocalMedia(videoDeviceId, audioDeviceId);
      useCallStore.getState().setLocalStream(localStream);
    } catch (err) {
      console.error("Failed to get local media, aborting call:", err);
      useCallStore.getState().reset();
      useCallStore.getState().setCallError(this.getMediaErrorMessage(err));
      return;
    }

    // Get caller's name and avatar for display on recipient's side
    const currentUserName = useUserStore.getState().currentUser?.name || "Unknown";
    const currentUserAvatar = useUserStore.getState().currentUser?.avatar_url;

    // Send call invite
    this.sendMessage({
      type: "call_invite",
      call_id: callId,
      from_user_id: currentUserId,
      to_user_id: remoteUserId,
      kind: "video",
      from_user_name: currentUserName,
      from_user_avatar: currentUserAvatar,
    });

    // Play dialtone
    this.playDialtone();

    // Set timeout for no answer
    this.callTimeoutId = window.setTimeout(() => {
      const { callStatus } = useCallStore.getState();
      if (callStatus === "outgoing") {
        console.log("Call timeout - no answer");
        this.hangup("no_answer");
      }
    }, CALL_TIMEOUT_MS);
  }

  /**
   * Accept an incoming call with specific devices (called from DeviceSelectionModal)
   */
  async acceptCallWithDevices(videoDeviceId?: string, audioDeviceId?: string): Promise<void> {
    console.log("[acceptCallWithDevices] Starting with devices:", { videoDeviceId, audioDeviceId });
    const { callId, remoteUserId, callStatus } = useCallStore.getState();
    const currentUserId = this.getCurrentUserId();

    if (callStatus !== "incoming" || !callId || !remoteUserId || !currentUserId || !this.sendMessage) {
      console.error("[acceptCallWithDevices] Cannot accept call - invalid state");
      return;
    }

    // Stop ringtone
    this.stopRingtone();

    // Get local media with specified devices
    try {
      const localStream = await this.getLocalMedia(videoDeviceId, audioDeviceId);
      useCallStore.getState().setLocalStream(localStream);

      // Initialize peer connection and add tracks
      this.initPeerConnection();
      localStream.getTracks().forEach((track) => {
        this.pc?.addTrack(track, localStream);
      });
    } catch (err) {
      console.error("[acceptCallWithDevices] Failed to get local media:", err);
      this.rejectCall("media_error");
      useCallStore.getState().setCallError(this.getMediaErrorMessage(err));
      return;
    }

    // Update status
    useCallStore.getState().setCallStatus("connecting");

    // Send accept message
    this.sendMessage({
      type: "call_accept",
      call_id: callId,
      from_user_id: currentUserId,
      to_user_id: remoteUserId,
    });
  }

  /**
   * Reject an incoming call
   */
  rejectCall(reason: string = "rejected"): void {
    const { callId, remoteUserId } = useCallStore.getState();
    const currentUserId = this.getCurrentUserId();

    if (callId && remoteUserId && currentUserId && this.sendMessage) {
      this.sendMessage({
        type: "call_reject",
        call_id: callId,
        from_user_id: currentUserId,
        to_user_id: remoteUserId,
        reason,
      });
    }

    this.stopRingtone();
    this.playHangup();
    this.cleanup();
  }

  /**
   * Hang up the current call
   */
  hangup(_reason: string = "user_hangup"): void {
    const { callId, remoteUserId, callStatus } = useCallStore.getState();
    const currentUserId = this.getCurrentUserId();

    // Send hangup message if in an active call
    if (callId && remoteUserId && currentUserId && this.sendMessage && callStatus !== "idle") {
      this.sendMessage({
        type: "call_hangup",
        call_id: callId,
        from_user_id: currentUserId,
        to_user_id: remoteUserId,
      });
    }

    // Stop any playing sounds
    this.stopDialtone();
    this.stopRingtone();
    this.playHangup();
    this.cleanup();
  }

  /**
   * Handle incoming call invite
   */
  handleCallInvite(callId: string, fromUserId: string, fromUserName: string, avatarUrl?: string): void {
    useCallStore.getState().receiveIncomingCall(callId, fromUserId, fromUserName, avatarUrl);
    this.playRingtone();

    // Auto-reject after timeout
    this.callTimeoutId = window.setTimeout(() => {
      const { callStatus } = useCallStore.getState();
      if (callStatus === "incoming") {
        this.rejectCall("timeout");
      }
    }, CALL_TIMEOUT_MS);
  }

  /**
   * Handle call accepted by remote peer (caller side)
   */
  async handleCallAccepted(): Promise<void> {
    const { callStatus, localStream, callId, remoteUserId } = useCallStore.getState();
    const currentUserId = this.getCurrentUserId();

    if (callStatus !== "outgoing" || !localStream || !callId || !remoteUserId || !currentUserId) {
      console.error("Invalid state for call accept");
      return;
    }

    // Clear timeout
    if (this.callTimeoutId) {
      clearTimeout(this.callTimeoutId);
      this.callTimeoutId = null;
    }

    // Stop dialtone
    this.stopDialtone();

    // Initialize peer connection and add tracks
    this.initPeerConnection();
    localStream.getTracks().forEach((track) => {
      this.pc?.addTrack(track, localStream);
    });

    // Update status
    useCallStore.getState().setCallStatus("connecting");

    // Create and send offer
    try {
      const offer = await this.pc!.createOffer();
      await this.pc!.setLocalDescription(offer);

      if (this.sendMessage) {
        this.sendMessage({
          type: "rtc_offer",
          call_id: callId,
          from_user_id: currentUserId,
          to_user_id: remoteUserId,
          sdp: JSON.stringify(offer),
        });
      }
    } catch (error) {
      console.error("Failed to create offer:", error);
      this.hangup("offer_error");
    }
  }

  /**
   * Handle call rejected by remote peer
   */
  handleCallRejected(reason: string): void {
    console.log("Call rejected:", reason);
    if (this.callTimeoutId) {
      clearTimeout(this.callTimeoutId);
      this.callTimeoutId = null;
    }
    this.stopDialtone();
    this.playHangup();
    this.cleanup();
  }

  /**
   * Handle remote hangup
   */
  handleRemoteHangup(): void {
    this.stopRingtone();
    this.stopDialtone();
    this.playHangup();
    this.cleanup();
  }

  /**
   * Handle RTC offer (callee side)
   */
  async handleRtcOffer(sdp: string): Promise<void> {
    console.log("[handleRtcOffer] Received RTC offer");
    const { callStatus, callId, remoteUserId } = useCallStore.getState();
    const currentUserId = this.getCurrentUserId();

    console.log("[handleRtcOffer] State:", { callStatus, callId, remoteUserId, currentUserId, hasPc: !!this.pc });

    if (callStatus !== "connecting" || !this.pc || !callId || !remoteUserId || !currentUserId) {
      console.error("[handleRtcOffer] Invalid state for RTC offer");
      return;
    }

    try {
      const offer = JSON.parse(sdp) as RTCSessionDescriptionInit;
      await this.pc.setRemoteDescription(offer);

      // Process any pending ICE candidates
      for (const candidate of this.pendingIceCandidates) {
        await this.pc.addIceCandidate(candidate);
      }
      this.pendingIceCandidates = [];

      // Create and send answer
      const answer = await this.pc.createAnswer();
      await this.pc.setLocalDescription(answer);

      if (this.sendMessage) {
        this.sendMessage({
          type: "rtc_answer",
          call_id: callId,
          from_user_id: currentUserId,
          to_user_id: remoteUserId,
          sdp: JSON.stringify(answer),
        });
      }
    } catch (error) {
      console.error("Failed to handle RTC offer:", error);
      this.hangup("rtc_error");
    }
  }

  /**
   * Handle RTC answer (caller side)
   */
  async handleRtcAnswer(sdp: string): Promise<void> {
    if (!this.pc) {
      console.error("No peer connection for RTC answer");
      return;
    }

    try {
      const answer = JSON.parse(sdp) as RTCSessionDescriptionInit;
      await this.pc.setRemoteDescription(answer);

      // Process any pending ICE candidates
      for (const candidate of this.pendingIceCandidates) {
        await this.pc.addIceCandidate(candidate);
      }
      this.pendingIceCandidates = [];
    } catch (error) {
      console.error("Failed to handle RTC answer:", error);
      this.hangup("rtc_error");
    }
  }

  /**
   * Handle ICE candidate from remote peer
   */
  async handleIceCandidate(candidateJson: string): Promise<void> {
    try {
      const candidate = JSON.parse(candidateJson) as RTCIceCandidateInit;

      if (this.pc?.remoteDescription) {
        await this.pc.addIceCandidate(candidate);
      } else {
        // Queue candidate until remote description is set
        this.pendingIceCandidates.push(candidate);
      }
    } catch (error) {
      console.error("Failed to add ICE candidate:", error);
    }
  }

  /**
   * Clean up call resources
   */
  private cleanup(): void {
    // Clear timeouts
    if (this.callTimeoutId) {
      clearTimeout(this.callTimeoutId);
      this.callTimeoutId = null;
    }
    if (this.reconnectTimeoutId) {
      clearTimeout(this.reconnectTimeoutId);
      this.reconnectTimeoutId = null;
    }

    // Close peer connection
    if (this.pc) {
      this.pc.close();
      this.pc = null;
    }

    // Clear pending candidates
    this.pendingIceCandidates = [];

    // Reset store (this also stops local tracks)
    useCallStore.getState().reset();
  }

  /**
   * Get current user ID from the user store
   */
  private getCurrentUserId(): string | null {
    return useUserStore.getState().currentUser?.id || null;
  }

  /**
   * Convert media error to user-friendly message
   */
  private getMediaErrorMessage(error: unknown): string {
    if (error instanceof DOMException) {
      switch (error.name) {
        case "NotReadableError":
          return "Camera or microphone is being used by another application. Please close other apps using your camera and try again.";
        case "NotAllowedError":
          return "Camera and microphone access was denied. Please allow access in your browser settings and try again.";
        case "NotFoundError":
          return "No camera or microphone found. Please connect a camera and microphone to make video calls.";
        case "OverconstrainedError":
          return "Your camera doesn't support the required settings. Please try a different camera.";
        default:
          return `Could not access camera or microphone: ${error.message}`;
      }
    }
    return "Could not access camera or microphone. Please check your device settings and try again.";
  }

  // === Sound Methods ===

  playRingtone(): void {
    if (!ringtoneAudio) {
      ringtoneAudio = new Audio("/sounds/ringtone.mp3");
      ringtoneAudio.loop = true;
    }
    ringtoneAudio.currentTime = 0;
    ringtoneAudio.play().catch((e) => console.warn("Could not play ringtone:", e));
  }

  stopRingtone(): void {
    if (ringtoneAudio) {
      ringtoneAudio.pause();
      ringtoneAudio.currentTime = 0;
    }
  }

  playDialtone(): void {
    if (!dialtoneAudio) {
      dialtoneAudio = new Audio("/sounds/dialtone.mp3");
      dialtoneAudio.loop = true;
    }
    dialtoneAudio.currentTime = 0;
    dialtoneAudio.play().catch((e) => console.warn("Could not play dialtone:", e));
  }

  stopDialtone(): void {
    if (dialtoneAudio) {
      dialtoneAudio.pause();
      dialtoneAudio.currentTime = 0;
    }
  }

  playHangup(): void {
    if (!hangupAudio) {
      hangupAudio = new Audio("/sounds/hangup.mp3");
    }
    hangupAudio.currentTime = 0;
    hangupAudio.play().catch((e) => console.warn("Could not play hangup:", e));
  }
}

// Export singleton instance
export const callService = new CallService();
