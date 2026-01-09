import { invoke } from "@tauri-apps/api/core";
import { NetworkStatus } from "../types";

export const websocketService = {
  broadcastMessage: (
    messageId: string,
    chatId: string,
    content: string,
    senderId: string
  ): Promise<boolean> => {
    return invoke<boolean>("broadcast_message", { messageId, chatId, content, senderId });
  },

  getWsPort: (): Promise<number> => {
    return invoke<number>("get_ws_port");
  },

  getLocalIp: (): Promise<string | null> => {
    return invoke<string | null>("get_local_ip");
  },

  getNetworkStatus: (): Promise<NetworkStatus> => {
    return invoke<NetworkStatus>("get_network_status");
  },

  connectToPeer: (ip: string, port?: number): Promise<void> => {
    return invoke<void>("connect_to_peer", { ip, port });
  },

  /**
   * Get the WebSocket authentication token for this session
   */
  getAuthToken: (): Promise<string> => {
    return invoke<string>("get_ws_auth_token");
  },

  /**
   * Broadcast current user's online presence to all connected peers
   */
  broadcastPresence: (userId: string): Promise<void> => {
    return invoke<void>("broadcast_presence", { userId });
  },
};
