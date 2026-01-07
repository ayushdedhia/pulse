import { create } from "zustand";
import { messageService, websocketService } from "../services";
import type { Message } from "../types";
import { useChatStore } from "./chatStore";
import { useUserStore } from "./userStore";

interface MessageStore {
  messages: Record<string, Message[]>;
  isLoading: boolean;
  error: string | null;

  loadMessages: (chatId: string) => Promise<void>;
  sendMessage: (chatId: string, content: string) => Promise<void>;
  addMessage: (chatId: string, message: Message) => void;
  markAsRead: (chatId: string) => Promise<void>;
  searchMessages: (query: string) => Promise<Message[]>;
}

export const useMessageStore = create<MessageStore>((set) => ({
  messages: {},
  isLoading: false,
  error: null,

  loadMessages: async (chatId: string) => {
    set({ isLoading: true });
    try {
      const messages = await messageService.getMessages(chatId, 100, 0);
      set((state) => ({
        messages: { ...state.messages, [chatId]: messages },
        isLoading: false,
      }));
    } catch (error) {
      set({ error: String(error), isLoading: false });
      console.error("Failed to load messages:", error);
    }
  },

  sendMessage: async (chatId: string, content: string) => {
    if (!content.trim()) return;

    const currentUser = useUserStore.getState().currentUser;

    try {
      const message = await messageService.sendMessage(
        chatId,
        content.trim(),
        "text"
      );

      // Add message to local state
      set((state) => ({
        messages: {
          ...state.messages,
          [chatId]: [...(state.messages[chatId] || []), message],
        },
      }));

      // Update chat's last message in chat store
      useChatStore.getState().updateChatLastMessage(chatId, message);

      // Broadcast via WebSocket for real-time sync
      try {
        await websocketService.broadcastMessage(
          chatId,
          content.trim(),
          currentUser?.id || "self"
        );
      } catch (wsError) {
        console.debug("WebSocket broadcast failed:", wsError);
      }
    } catch (error) {
      console.error("Failed to send message:", error);
    }
  },

  addMessage: (chatId: string, message: Message) => {
    set((state) => ({
      messages: {
        ...state.messages,
        [chatId]: [...(state.messages[chatId] || []), message],
      },
    }));
  },

  markAsRead: async (chatId: string) => {
    try {
      await messageService.markAsRead(chatId);
      useChatStore.getState().clearUnreadCount(chatId);
    } catch (error) {
      console.error("Failed to mark as read:", error);
    }
  },

  searchMessages: async (query: string) => {
    try {
      return await messageService.searchMessages(query);
    } catch (error) {
      console.error("Failed to search messages:", error);
      return [];
    }
  },
}));
