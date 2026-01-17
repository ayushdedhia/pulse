import { create } from "zustand";
import { chatService } from "../services";
import type { Chat, Message } from "../types";
import { useMessageStore } from "./messageStore";
import { useUserStore } from "./userStore";

interface ChatStore {
  chats: Chat[];
  activeChat: Chat | null;
  isLoading: boolean;
  error: string | null;

  loadChats: () => Promise<void>;
  setActiveChat: (chat: Chat | null) => void;
  createChat: (userId: string) => Promise<Chat>;
  updateChatLastMessage: (chatId: string, message: Message) => void;
  clearUnreadCount: (chatId: string) => void;
  addChat: (chat: Chat) => void;
  updateUserStatus: (userId: string, isOnline: boolean, lastSeen?: number) => void;
}

export const useChatStore = create<ChatStore>((set) => ({
  chats: [],
  activeChat: null,
  isLoading: false,
  error: null,

  loadChats: async () => {
    set({ isLoading: true, error: null });
    try {
      const chats = await chatService.getChats();

      // Update activeChat if it exists (to refresh participant data)
      set((state) => {
        let updatedActiveChat = state.activeChat;
        if (state.activeChat) {
          const freshChat = chats.find((c) => c.id === state.activeChat!.id);
          if (freshChat) {
            updatedActiveChat = freshChat;
          }
        }
        return { chats, activeChat: updatedActiveChat, isLoading: false };
      });

      // Also load current user
      await useUserStore.getState().loadCurrentUser();
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  setActiveChat: (chat) => {
    set({ activeChat: chat });
    if (chat) {
      useMessageStore.getState().loadMessages(chat.id);
      useMessageStore.getState().markAsRead(chat.id);
    }
  },

  createChat: async (userId: string) => {
    const chat = await chatService.createChat(userId);

    // Add to chats if not already present
    set((state) => {
      const exists = state.chats.some((c) => c.id === chat.id);
      if (exists) return state;
      return { chats: [chat, ...state.chats] };
    });

    return chat;
  },

  updateChatLastMessage: (chatId: string, message: Message) => {
    set((state) => ({
      chats: state.chats
        .map((chat) =>
          chat.id === chatId
            ? { ...chat, last_message: message, updated_at: message.created_at }
            : chat
        )
        .sort((a, b) => b.updated_at - a.updated_at),
    }));
  },

  clearUnreadCount: (chatId: string) => {
    set((state) => ({
      chats: state.chats.map((chat) =>
        chat.id === chatId ? { ...chat, unread_count: 0 } : chat
      ),
    }));
  },

  addChat: (chat: Chat) => {
    set((state) => {
      const exists = state.chats.some((c) => c.id === chat.id);
      if (exists) return state;
      return { chats: [chat, ...state.chats] };
    });
  },

  updateUserStatus: (userId: string, isOnline: boolean, lastSeen?: number) => {
    set((state) => ({
      chats: state.chats.map((chat) =>
        chat.participant?.id === userId
          ? {
            ...chat,
            participant: {
              ...chat.participant,
              is_online: isOnline,
              ...(lastSeen ? { last_seen: lastSeen } : {})
            },
          }
          : chat
      ),
      activeChat:
        state.activeChat?.participant?.id === userId
          ? {
            ...state.activeChat,
            participant: {
              ...state.activeChat.participant,
              is_online: isOnline,
              ...(lastSeen ? { last_seen: lastSeen } : {})
            },
          }
          : state.activeChat,
    }));
  },
}));
