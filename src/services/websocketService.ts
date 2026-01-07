import { invoke } from "@tauri-apps/api/core";

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
};
