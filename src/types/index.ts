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

export interface Message {
  id: string;
  chat_id: string;
  sender_id: string;
  sender?: User;
  content?: string;
  message_type: "text" | "image" | "video" | "audio" | "document";
  media_url?: string;
  reply_to_id?: string;
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
