import { invoke } from "@tauri-apps/api/core";
import type { Message } from "../types";

export const messageService = {
  getMessages: (
    chatId: string,
    limit: number = 100,
    offset: number = 0
  ): Promise<Message[]> => {
    return invoke<Message[]>("get_messages", { chatId, limit, offset });
  },

  sendMessage: (
    chatId: string,
    content: string,
    messageType: string = "text"
  ): Promise<Message> => {
    return invoke<Message>("send_message", { chatId, content, messageType });
  },

  markAsRead: (chatId: string): Promise<string[]> => {
    return invoke<string[]>("mark_as_read", { chatId });
  },

  updateMessageStatus: (messageId: string, status: string): Promise<boolean> => {
    return invoke<boolean>("update_message_status", { messageId, status });
  },

  searchMessages: (query: string): Promise<Message[]> => {
    return invoke<Message[]>("search_messages", { query });
  },

  receiveMessage: (
    id: string,
    chatId: string,
    senderId: string,
    senderName: string | null,
    content: string,
    timestamp: number
  ): Promise<Message> => {
    return invoke<Message>("receive_message", {
      id,
      chatId,
      senderId,
      senderName,
      content,
      timestamp,
    });
  },
};
