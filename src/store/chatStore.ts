import { invoke } from "@tauri-apps/api/core";
import { create } from "zustand";

import type { Chat, Message, User } from "../types";

interface ChatStore {
  chats: Chat[];
  activeChat: Chat | null;
  messages: Record<string, Message[]>;
  currentUser: User | null;
  isLoading: boolean;
  error: string | null;

  loadChats: () => Promise<void>;
  loadCurrentUser: () => Promise<void>;
  setActiveChat: (chat: Chat | null) => void;
  loadMessages: (chatId: string) => Promise<void>;
  sendMessage: (content: string) => Promise<void>;
  markAsRead: (chatId: string) => Promise<void>;
  createChat: (userId: string) => Promise<Chat>;
  searchMessages: (query: string) => Promise<Message[]>;
}

export const useChatStore = create<ChatStore>((set, get) => ({
  chats: [],
  activeChat: null,
  messages: {},
  currentUser: null,
  isLoading: false,
  error: null,

  loadChats: async () => {
    set({ isLoading: true, error: null });
    try {
      const chats = await invoke<Chat[]>("get_chats");
      set({ chats, isLoading: false });

      // Also load current user
      const currentUser = await invoke<User>("get_current_user");
      set({ currentUser });
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  loadCurrentUser: async () => {
    try {
      const currentUser = await invoke<User>("get_current_user");
      set({ currentUser });
    } catch (error) {
      console.error("Failed to load current user:", error);
    }
  },

  setActiveChat: (chat) => {
    set({ activeChat: chat });
    if (chat) {
      get().loadMessages(chat.id);
      get().markAsRead(chat.id);
    }
  },

  loadMessages: async (chatId: string) => {
    try {
      const messages = await invoke<Message[]>("get_messages", {
        chatId,
        limit: 100,
        offset: 0,
      });
      set((state) => ({
        messages: { ...state.messages, [chatId]: messages },
      }));
    } catch (error) {
      console.error("Failed to load messages:", error);
    }
  },

  sendMessage: async (content: string) => {
    const { activeChat, currentUser } = get();
    if (!activeChat || !content.trim()) return;

    try {
      const message = await invoke<Message>("send_message", {
        chatId: activeChat.id,
        content: content.trim(),
        messageType: "text",
      });

      // Add message to local state
      set((state) => ({
        messages: {
          ...state.messages,
          [activeChat.id]: [...(state.messages[activeChat.id] || []), message],
        },
      }));

      // Update chat's last message and move to top
      set((state) => ({
        chats: state.chats
          .map((chat) =>
            chat.id === activeChat.id
              ? { ...chat, last_message: message, updated_at: message.created_at }
              : chat
          )
          .sort((a, b) => b.updated_at - a.updated_at),
      }));

      // Broadcast via WebSocket for real-time sync
      try {
        await invoke("broadcast_message", {
          chatId: activeChat.id,
          content: content.trim(),
          senderId: currentUser?.id || "self",
        });
      } catch (wsError) {
        // WebSocket broadcast is non-critical
        console.debug("WebSocket broadcast failed:", wsError);
      }
    } catch (error) {
      console.error("Failed to send message:", error);
    }
  },

  markAsRead: async (chatId: string) => {
    try {
      await invoke("mark_as_read", { chatId });
      set((state) => ({
        chats: state.chats.map((chat) =>
          chat.id === chatId ? { ...chat, unread_count: 0 } : chat
        ),
      }));
    } catch (error) {
      console.error("Failed to mark as read:", error);
    }
  },

  createChat: async (userId: string) => {
    const chat = await invoke<Chat>("create_chat", { userId });

    // Add to chats if not already present
    set((state) => {
      const exists = state.chats.some((c) => c.id === chat.id);
      if (exists) return state;
      return { chats: [chat, ...state.chats] };
    });

    return chat;
  },

  searchMessages: async (query: string) => {
    try {
      return await invoke<Message[]>("search_messages", { query });
    } catch (error) {
      console.error("Failed to search messages:", error);
      return [];
    }
  },
}));
