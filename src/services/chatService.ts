import { invoke } from "@tauri-apps/api/core";
import type { Chat } from "../types";

export const chatService = {
  getChats: (): Promise<Chat[]> => {
    return invoke<Chat[]>("get_chats");
  },

  createChat: (userId: string): Promise<Chat> => {
    return invoke<Chat>("create_chat", { userId });
  },
};
