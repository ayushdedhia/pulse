import { create } from "zustand";

import type { CallStatus } from "../types";

interface PendingCallInfo {
  type: "outgoing" | "incoming";
  // For outgoing calls
  remoteUserId?: string;
  remoteUserName?: string;
  remoteUserAvatar?: string;
  // For incoming calls (already have callId from invite)
  callId?: string;
}

interface CallState {
  // Call metadata
  callId: string | null;
  remoteUserId: string | null;
  remoteUserName: string | null;
  remoteUserAvatar: string | null;
  callStatus: CallStatus;

  // Media state
  isMuted: boolean;
  isCameraOff: boolean;
  localStream: MediaStream | null;
  remoteStream: MediaStream | null;

  // Timing
  callStartTime: number | null;

  // Error state
  callError: string | null;

  // Device selection state
  showDeviceSelection: boolean;
  pendingCallInfo: PendingCallInfo | null;

  // Actions
  startOutgoingCall: (userId: string, userName: string, avatarUrl?: string) => string;
  receiveIncomingCall: (callId: string, fromUserId: string, fromUserName: string, avatarUrl?: string) => void;
  setCallStatus: (status: CallStatus) => void;
  setLocalStream: (stream: MediaStream | null) => void;
  setRemoteStream: (stream: MediaStream | null) => void;
  toggleMute: () => void;
  toggleCamera: () => void;
  setCallStartTime: (time: number) => void;
  setCallError: (error: string | null) => void;
  // Device selection actions
  showDeviceSelectionForOutgoing: (userId: string, userName: string, avatarUrl?: string) => void;
  showDeviceSelectionForIncoming: () => void;
  hideDeviceSelection: () => void;
  reset: () => void;
}

const initialState = {
  callId: null,
  remoteUserId: null,
  remoteUserName: null,
  remoteUserAvatar: null,
  callStatus: "idle" as CallStatus,
  isMuted: false,
  isCameraOff: false,
  localStream: null,
  remoteStream: null,
  callStartTime: null,
  callError: null,
  showDeviceSelection: false,
  pendingCallInfo: null,
};

export const useCallStore = create<CallState>((set, get) => ({
  ...initialState,

  startOutgoingCall: (userId: string, userName: string, avatarUrl?: string) => {
    const callId = crypto.randomUUID();
    set({
      callId,
      remoteUserId: userId,
      remoteUserName: userName,
      remoteUserAvatar: avatarUrl || null,
      callStatus: "outgoing",
      isMuted: false,
      isCameraOff: false,
    });
    return callId;
  },

  receiveIncomingCall: (callId: string, fromUserId: string, fromUserName: string, avatarUrl?: string) => {
    // Ignore if already in a call
    if (get().callStatus !== "idle") {
      console.log("Already in a call, ignoring incoming call");
      return;
    }
    set({
      callId,
      remoteUserId: fromUserId,
      remoteUserName: fromUserName,
      remoteUserAvatar: avatarUrl || null,
      callStatus: "incoming",
    });
  },

  setCallStatus: (status: CallStatus) => {
    set({ callStatus: status });
  },

  setLocalStream: (stream: MediaStream | null) => {
    set({ localStream: stream });
  },

  setRemoteStream: (stream: MediaStream | null) => {
    set({ remoteStream: stream });
  },

  toggleMute: () => {
    const { localStream, isMuted } = get();
    if (localStream) {
      localStream.getAudioTracks().forEach((track) => {
        track.enabled = isMuted; // Toggle: if muted, enable; if not muted, disable
      });
    }
    set({ isMuted: !isMuted });
  },

  toggleCamera: () => {
    const { localStream, isCameraOff } = get();
    if (localStream) {
      localStream.getVideoTracks().forEach((track) => {
        track.enabled = isCameraOff; // Toggle: if off, enable; if on, disable
      });
    }
    set({ isCameraOff: !isCameraOff });
  },

  setCallStartTime: (time: number) => {
    set({ callStartTime: time });
  },

  setCallError: (error: string | null) => {
    set({ callError: error });
  },

  showDeviceSelectionForOutgoing: (userId: string, userName: string, avatarUrl?: string) => {
    set({
      showDeviceSelection: true,
      pendingCallInfo: {
        type: "outgoing",
        remoteUserId: userId,
        remoteUserName: userName,
        remoteUserAvatar: avatarUrl,
      },
    });
  },

  showDeviceSelectionForIncoming: () => {
    const { callId, remoteUserId, remoteUserName, remoteUserAvatar } = get();
    set({
      showDeviceSelection: true,
      pendingCallInfo: {
        type: "incoming",
        callId: callId || undefined,
        remoteUserId: remoteUserId || undefined,
        remoteUserName: remoteUserName || undefined,
        remoteUserAvatar: remoteUserAvatar || undefined,
      },
    });
  },

  hideDeviceSelection: () => {
    set({
      showDeviceSelection: false,
      pendingCallInfo: null,
    });
  },

  reset: () => {
    const { localStream } = get();
    // Stop all local tracks
    if (localStream) {
      localStream.getTracks().forEach((track) => track.stop());
    }
    set(initialState);
  },
}));
