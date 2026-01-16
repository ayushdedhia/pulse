export interface User {
  id: string;
  /** Original name broadcast by the user */
  name: string;
  /** Local alias set by current user (overrides name in UI if set) */
  display_name?: string;
  phone?: string;
  avatar_url?: string;
  about?: string;
  last_seen?: number;
  is_online: boolean;
  /** Whether to fetch and show link previews (default: true) */
  link_previews_enabled: boolean;
}

/** Get the display name for a user (alias if set, otherwise original name) */
export function getUserDisplayName(user: User | undefined): string {
  if (!user) return "Unknown";
  return user.display_name || user.name;
}

export interface Chat {
  id: string;
  chat_type: "individual" | "group";
  name?: string;
  avatar_url?: string;
  created_at: number;
  updated_at: number;
  last_message?: Message;
  unread_count: number;
  participant?: User;
}

export interface UrlPreview {
  url: string;
  title?: string;
  description?: string;
  image_url?: string;
  site_name?: string;
  fetched_at: number;
}

export interface Message {
  id: string;
  chat_id: string;
  sender_id: string;
  sender?: User;
  content?: string;
  message_type: "text" | "image" | "video" | "audio" | "document";
  media_url?: string;
  reply_to_id?: string;
  url_preview?: UrlPreview;
  status: "sent" | "delivered" | "read";
  created_at: number;
  edited_at?: number;
}

export type MessageStatus = "sent" | "delivered" | "read";

export type Theme = "dark" | "light";

export interface PeerInfo {
  ip: string;
  port: number;
  connected: boolean;
}

export interface NetworkStatus {
  is_server: boolean;
  local_ip: string | null;
  port: number;
  connected_peers: PeerInfo[];
}

export interface IceServer {
  urls: string;
  username?: string;
  credential?: string;
}

export type CallStatus = "idle" | "outgoing" | "incoming" | "connecting" | "connected";

export interface CallInviteMessage {
  type: "call_invite";
  call_id: string;
  from_user_id: string;
  to_user_id: string;
  kind: "video" | "audio";
  from_user_name?: string;
  from_user_avatar?: string;
}

export interface CallRingingMessage {
  type: "call_ringing";
  call_id: string;
  from_user_id: string;
  to_user_id: string;
}

export interface CallAcceptMessage {
  type: "call_accept";
  call_id: string;
  from_user_id: string;
  to_user_id: string;
}

export interface CallRejectMessage {
  type: "call_reject";
  call_id: string;
  from_user_id: string;
  to_user_id: string;
  reason: string;
}

export interface CallHangupMessage {
  type: "call_hangup";
  call_id: string;
  from_user_id: string;
  to_user_id: string;
}

export interface RtcOfferMessage {
  type: "rtc_offer";
  call_id: string;
  from_user_id: string;
  to_user_id: string;
  sdp: string;
}

export interface RtcAnswerMessage {
  type: "rtc_answer";
  call_id: string;
  from_user_id: string;
  to_user_id: string;
  sdp: string;
}

export interface RtcIceCandidateMessage {
  type: "rtc_ice_candidate";
  call_id: string;
  from_user_id: string;
  to_user_id: string;
  candidate: string;
}

export type CallMessage =
  | CallInviteMessage
  | CallRingingMessage
  | CallAcceptMessage
  | CallRejectMessage
  | CallHangupMessage
  | RtcOfferMessage
  | RtcAnswerMessage
  | RtcIceCandidateMessage;
