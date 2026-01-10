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
  updateMessageStatus: (messageId: string, status: Message["status"]) => Promise<void>;
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
          message.id,
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
    const existingMessages = useMessageStore.getState().messages[chatId] || [];
    // Check if message already exists (avoid duplicates)
    if (existingMessages.some((m) => m.id === message.id)) {
      return;
    }
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

  updateMessageStatus: async (messageId: string, status: Message["status"]) => {
    try {
      // Update in database first
      await messageService.updateMessageStatus(messageId, status);

      // Update in local state (if message is loaded)
      // Use retries to handle race condition where delivery receipt
      // arrives before the message is added to the store
      const tryUpdate = () => {
        const state = useMessageStore.getState();
        const chatIds = Object.keys(state.messages);

        for (const chatId of chatIds) {
          const messages = state.messages[chatId];
          const msgIndex = messages.findIndex((msg) => msg.id === messageId);

          if (msgIndex !== -1) {
            set((s) => ({
              messages: {
                ...s.messages,
                [chatId]: s.messages[chatId].map((msg) =>
                  msg.id === messageId ? { ...msg, status } : msg
                ),
              },
            }));

            // Also update chat's last_message if it matches
            const chatState = useChatStore.getState();
            const chat = chatState.chats.find((c) => c.id === chatId);
            if (chat?.last_message?.id === messageId) {
              useChatStore.setState((cs) => ({
                chats: cs.chats.map((c) =>
                  c.id === chatId && c.last_message
                    ? { ...c, last_message: { ...c.last_message, status } }
                    : c
                ),
              }));
            }

            return true;
          }
        }
        return false;
      };

      // Try immediately, then retry after delays if not found
      if (!tryUpdate()) {
        setTimeout(tryUpdate, 100);
        setTimeout(tryUpdate, 300);
        setTimeout(tryUpdate, 600);
      }
    } catch (error) {
      console.error("Failed to update message status:", error);
    }
  },
}));
