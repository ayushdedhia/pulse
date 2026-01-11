import { createContext, ReactNode, useCallback, useContext, useEffect, useRef, useState } from "react";

import { messageService, userService, websocketService } from "../services";
import { useChatStore } from "../store/chatStore";
import { useMessageStore } from "../store/messageStore";
import { useUserStore } from "../store/userStore";
import type { Message } from "../types";

// Get store functions without subscribing to state changes
const getMessageActions = () => useMessageStore.getState();
const getChatActions = () => useChatStore.getState();

const SERVER_URL = import.meta.env.VITE_SERVER_URL || "ws://localhost:9001";

interface WsMessage {
  type: string;
  [key: string]: unknown;
}

interface WebSocketContextValue {
  isConnected: boolean;
  typingUsers: Record<string, string[]>;
  sendTyping: (chatId: string, isTyping: boolean) => void;
  onlineUsers: Set<string>;
}

const WebSocketContext = createContext<WebSocketContextValue>({
  isConnected: false,
  typingUsers: {},
  sendTyping: () => {},
  onlineUsers: new Set(),
});

export function WebSocketProvider({ children }: { children: ReactNode }) {
  const wsRef = useRef<WebSocket | null>(null);
  const [isConnected, setIsConnected] = useState(false);
  const [typingUsers, setTypingUsers] = useState<Record<string, string[]>>({});
  const [onlineUsers, setOnlineUsers] = useState<Set<string>>(new Set());
  const reconnectTimeoutRef = useRef<number>();

  const currentUser = useUserStore((state) => state.currentUser);

  const handleMessage = useCallback(
    async (data: WsMessage) => {
      switch (data.type) {
        case "message":
          // Save incoming message to local database, then add to store
          if (data.chat_id && data.id && data.sender_id && data.content !== undefined) {
            try {
              // receive_message returns the saved message with the correct deterministic chat_id
              const savedMessage = await messageService.receiveMessage(
                data.id as string,
                data.chat_id as string,
                data.sender_id as string,
                (data.sender_name as string) || null,
                data.content as string,
                data.timestamp as number,
                (data.reply_to_id as string) || undefined
              );

              // Add message directly to store instead of reloading all
              getMessageActions().addMessage(savedMessage.chat_id, savedMessage as Message);
              getChatActions().loadChats();
            } catch (e) {
              // Message might be from ourselves or already exists, that's ok
              console.debug("receive_message:", e);
            }
          }
          break;

        case "typing":
          if (data.chat_id && data.user_id) {
            const chatId = data.chat_id as string;
            const userId = data.user_id as string;
            const isTyping = data.is_typing as boolean;

            setTypingUsers((prev) => {
              const chatUsers = prev[chatId] || [];
              if (isTyping && !chatUsers.includes(userId)) {
                return { ...prev, [chatId]: [...chatUsers, userId] };
              } else if (!isTyping) {
                return { ...prev, [chatId]: chatUsers.filter((id) => id !== userId) };
              }
              return prev;
            });

            // Clear typing indicator after 5 seconds
            setTimeout(() => {
              setTypingUsers((prev) => ({
                ...prev,
                [chatId]: (prev[chatId] || []).filter((id) => id !== userId),
              }));
            }, 5000);
          }
          break;

        case "presence":
          if (data.user_id) {
            const userId = data.user_id as string;
            const isOnline = data.is_online as boolean;

            setOnlineUsers((prev) => {
              const next = new Set(prev);
              if (isOnline) {
                next.add(userId);
              } else {
                next.delete(userId);
              }
              return next;
            });
          }
          break;

        case "delivery_receipt":
          // Update message status to 'delivered' when recipient receives it
          if (data.message_id && data.delivered_to !== currentUser?.id) {
            getMessageActions().updateMessageStatus(data.message_id as string, "delivered");
          }
          break;

        case "read_receipt":
          // Update message status to 'read' when recipient reads it
          if (data.message_ids && data.user_id !== currentUser?.id) {
            const messageIds = data.message_ids as string[];
            for (const messageId of messageIds) {
              getMessageActions().updateMessageStatus(messageId, "read");
            }
          }
          break;

        case "auth_response":
          // Handle authentication response from server
          if (data.success) {
            console.log("Connected to Pulse server");
            setIsConnected(true);
            // Connect the Tauri backend client and broadcast presence
            if (currentUser) {
              websocketService.connect(currentUser.id)
                .then(() => websocketService.broadcastPresence(currentUser.id))
                .catch((err: Error) => console.error("Failed to initialize backend WebSocket:", err));
            }
          } else {
            console.warn("Server authentication failed:", data.message);
            setIsConnected(false);
            wsRef.current?.close();
          }
          break;

        case "error":
          console.error("WebSocket error from server:", data.message);
          break;

        case "profile_update":
          // Handle profile updates from peers
          if (data.user_id && data.user_id !== currentUser?.id) {
            (async () => {
              try {
                let avatarUrl = data.avatar_url as string | undefined;

                // Save avatar locally if bytes are provided
                if (data.avatar_data) {
                  const localPath = await userService.savePeerAvatar(
                    data.user_id as string,
                    data.avatar_data as string
                  );
                  avatarUrl = localPath;
                }

                // Update contact in database
                await userService.updateUser({
                  id: data.user_id as string,
                  name: data.name as string,
                  phone: data.phone as string | undefined,
                  avatar_url: avatarUrl,
                  about: data.about as string | undefined,
                  is_online: true,
                });

                // Refresh chat list to show updated names/avatars
                getChatActions().loadChats();
              } catch (err) {
                console.error("Failed to process profile update:", err);
              }
            })();
          }
          break;
      }
    },
    [currentUser]
  );

  const connect = useCallback(async () => {
    if (wsRef.current?.readyState === WebSocket.OPEN) return;
    if (!currentUser) return;

    try {
      console.log("Connecting to Pulse server at", SERVER_URL);
      const ws = new WebSocket(SERVER_URL);

      ws.onopen = () => {
        console.log("WebSocket connected, authenticating...");
        // Send connect message with user ID
        ws.send(JSON.stringify({ type: "connect", user_id: currentUser.id }));
      };

      ws.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          handleMessage(data);
        } catch (e) {
          console.error("Failed to parse WS message:", e);
        }
      };

      ws.onclose = () => {
        console.log("WebSocket disconnected");
        setIsConnected(false);

        // Reconnect after delay
        reconnectTimeoutRef.current = window.setTimeout(() => {
          connect();
        }, 3000);
      };

      ws.onerror = (err) => {
        console.error("WebSocket error:", err);
      };

      wsRef.current = ws;
    } catch (err) {
      console.error("Failed to connect to server:", err);
      // Retry after delay
      reconnectTimeoutRef.current = window.setTimeout(() => {
        connect();
      }, 3000);
    }
  }, [currentUser, handleMessage]);

  const sendTyping = useCallback(
    (chatId: string, isTyping: boolean) => {
      if (wsRef.current?.readyState === WebSocket.OPEN && currentUser) {
        wsRef.current.send(
          JSON.stringify({
            type: "typing",
            chat_id: chatId,
            user_id: currentUser.id,
            is_typing: isTyping,
          })
        );
      }
    },
    [currentUser]
  );

  useEffect(() => {
    if (currentUser) {
      connect();
    }

    return () => {
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current);
      }
      wsRef.current?.close();
    };
  }, [currentUser, connect]);

  return (
    <WebSocketContext.Provider value={{ isConnected, typingUsers, sendTyping, onlineUsers }}>
      {children}
    </WebSocketContext.Provider>
  );
}

export function useWebSocketContext() {
  return useContext(WebSocketContext);
}
