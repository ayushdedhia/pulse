import { invoke } from "@tauri-apps/api/core";
import {
  createContext,
  ReactNode,
  useCallback,
  useContext,
  useEffect,
  useRef,
  useState,
} from "react";

import { useChatStore } from "../store/chatStore";
import { useMessageStore } from "../store/messageStore";
import { useUserStore } from "../store/userStore";

// Get store functions without subscribing to state changes
const getMessageActions = () => useMessageStore.getState();
const getChatActions = () => useChatStore.getState();

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
  sendTyping: () => { },
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
              const savedMessage = await invoke<{
                id: string;
                chat_id: string;
                sender_id: string;
                content: string | null;
                message_type: string;
                status: string;
                created_at: number;
                sender?: {
                  id: string;
                  name: string;
                };
              }>("receive_message", {
                id: data.id as string,
                chatId: data.chat_id as string,
                senderId: data.sender_id as string,
                senderName: (data.sender_name as string) || null,
                content: data.content as string,
                timestamp: data.timestamp as number,
              });

              // Add message directly to store instead of reloading all
              getMessageActions().addMessage(savedMessage.chat_id, savedMessage as import("../types").Message);
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
                return {
                  ...prev,
                  [chatId]: chatUsers.filter((id) => id !== userId),
                };
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
          // delivered_to is the recipient's ID, so we update if it's NOT us (meaning we sent it)
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
      }
    },
    [currentUser]
  );

  const connect = useCallback(async () => {
    if (wsRef.current?.readyState === WebSocket.OPEN) return;

    try {
      const port = await invoke<number>("get_ws_port");
      const ws = new WebSocket(`ws://127.0.0.1:${port}`);

      ws.onopen = () => {
        console.log("WebSocket connected to port", port);
        setIsConnected(true);

        // Identify ourselves
        if (currentUser) {
          ws.send(
            JSON.stringify({
              type: "connect",
              user_id: currentUser.id,
            })
          );
        }
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
      console.error("Failed to get WS port:", err);
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
    <WebSocketContext.Provider
      value={{ isConnected, typingUsers, sendTyping, onlineUsers }}
    >
      {children}
    </WebSocketContext.Provider>
  );
}

export function useWebSocketContext() {
  return useContext(WebSocketContext);
}
