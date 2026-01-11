import { invoke } from "@tauri-apps/api/core";
import type { Message } from "../types";

export const messageService = {
  getMessages: (
    chatId: string,
    limit: number = 100,
    offset: number = 0
  ): Promise<Message[]> => {
    return invoke<Message[]>("get_messages", {
      input: { chat_id: chatId, limit, offset },
    });
  },

  sendMessage: (
    chatId: string,
    content: string,
    messageType: string = "text",
    replyToId?: string
  ): Promise<Message> => {
    return invoke<Message>("send_message", {
      input: { chat_id: chatId, content, message_type: messageType, reply_to_id: replyToId },
    });
  },

  markAsRead: (chatId: string): Promise<string[]> => {
    return invoke<string[]>("mark_as_read", { input: { chat_id: chatId } });
  },

  updateMessageStatus: (messageId: string, status: string): Promise<boolean> => {
    return invoke<boolean>("update_message_status", {
      input: { message_id: messageId, status },
    });
  },

  searchMessages: (query: string): Promise<Message[]> => {
    return invoke<Message[]>("search_messages", { input: { query } });
  },

  receiveMessage: (
    id: string,
    chatId: string,
    senderId: string,
    senderName: string | null,
    content: string,
    timestamp: number,
    replyToId?: string
  ): Promise<Message> => {
    return invoke<Message>("receive_message", {
      id,
      chatId,
      senderId,
      senderName,
      content,
      timestamp,
      replyToId,
    });
  },
};
