import { invoke } from "@tauri-apps/api/core";
import { NetworkStatus } from "../types";

export const websocketService = {
  broadcastMessage: (
    chatId: string,
    content: string,
    senderId: string
  ): Promise<boolean> => {
    return invoke<boolean>("broadcast_message", { chatId, content, senderId });
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
};
