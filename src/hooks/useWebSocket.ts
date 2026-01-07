import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useRef, useState } from "react";

import { useMessageStore } from "../store/messageStore";
import { useUserStore } from "../store/userStore";

export interface WsMessage {
  type: "message" | "typing" | "presence" | "read_receipt" | "connect" | "error";
  id?: string;
  chat_id?: string;
  sender_id?: string;
  user_id?: string;
  content?: string;
  timestamp?: number;
  is_typing?: boolean;
  is_online?: boolean;
  last_seen?: number;
  message_id?: string;
  message?: string;
}

export function useWebSocket() {
  const wsRef = useRef<WebSocket | null>(null);
  const [isConnected, setIsConnected] = useState(false);
  const [typingUsers, setTypingUsers] = useState<Record<string, string[]>>({});
  const reconnectTimeoutRef = useRef<number>();

  const { currentUser } = useUserStore();
  const { loadMessages } = useMessageStore();

  const connect = useCallback(async () => {
    if (wsRef.current?.readyState === WebSocket.OPEN) return;

    try {
      const port = await invoke<number>("get_ws_port");
      const ws = new WebSocket(`ws://127.0.0.1:${port}`);

      ws.onopen = () => {
        console.log("WebSocket connected");
        setIsConnected(true);

        // Send connect message with user ID
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
          const message: WsMessage = JSON.parse(event.data);
          handleMessage(message);
        } catch (e) {
          console.error("Failed to parse WebSocket message:", e);
        }
      };

      ws.onclose = () => {
        console.log("WebSocket disconnected");
        setIsConnected(false);

        // Attempt to reconnect after 3 seconds
        reconnectTimeoutRef.current = window.setTimeout(() => {
          connect();
        }, 3000);
      };

      ws.onerror = (error) => {
        console.error("WebSocket error:", error);
      };

      wsRef.current = ws;
    } catch (error) {
      console.error("Failed to connect to WebSocket:", error);
    }
  }, [currentUser]);

  const handleMessage = useCallback(
    (message: WsMessage) => {
      switch (message.type) {
        case "message":
          // Reload messages for the chat
          if (message.chat_id) {
            loadMessages(message.chat_id);
          }
          break;

        case "typing":
          if (message.chat_id && message.user_id) {
            setTypingUsers((prev) => {
              const chatUsers = prev[message.chat_id!] || [];
              if (message.is_typing) {
                if (!chatUsers.includes(message.user_id!)) {
                  return {
                    ...prev,
                    [message.chat_id!]: [...chatUsers, message.user_id!],
                  };
                }
              } else {
                return {
                  ...prev,
                  [message.chat_id!]: chatUsers.filter(
                    (id) => id !== message.user_id
                  ),
                };
              }
              return prev;
            });
          }
          break;

        case "presence":
          // Could update user online status in store
          console.log("Presence update:", message);
          break;

        case "read_receipt":
          // Could update message status in store
          console.log("Read receipt:", message);
          break;

        case "error":
          console.error("WebSocket error message:", message.message);
          break;
      }
    },
    [loadMessages]
  );

  const sendMessage = useCallback(
    (type: string, data: Record<string, unknown>) => {
      if (wsRef.current?.readyState === WebSocket.OPEN) {
        wsRef.current.send(JSON.stringify({ type, ...data }));
      }
    },
    []
  );

  const sendTyping = useCallback(
    (chatId: string, isTyping: boolean) => {
      sendMessage("typing", {
        chat_id: chatId,
        user_id: currentUser?.id,
        is_typing: isTyping,
      });
    },
    [currentUser, sendMessage]
  );

  const disconnect = useCallback(() => {
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
    }
    wsRef.current?.close();
    wsRef.current = null;
    setIsConnected(false);
  }, []);

  useEffect(() => {
    connect();

    return () => {
      disconnect();
    };
  }, [connect, disconnect]);

  return {
    isConnected,
    typingUsers,
    sendMessage,
    sendTyping,
    connect,
    disconnect,
  };
}
