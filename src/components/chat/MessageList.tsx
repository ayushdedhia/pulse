import { format, isSameDay } from "date-fns";
import { useCallback, useEffect, useMemo, useRef } from "react";

import { useChatStore } from "../../store/chatStore";
import { useMessageStore } from "../../store/messageStore";
import { useUserStore } from "../../store/userStore";
import type { Message } from "../../types";
import { DateDivider } from "./DateDivider";
import { MessageBubble } from "./MessageBubble";

export function MessageList() {
  const activeChat = useChatStore((state) => state.activeChat);
  const messages = useMessageStore((state) => state.messages);
  const currentUser = useUserStore((state) => state.currentUser);
  const markAsRead = useMessageStore((state) => state.markAsRead);
  const setReplyingTo = useMessageStore((state) => state.setReplyingTo);
  const containerRef = useRef<HTMLDivElement>(null);
  const messageRefs = useRef<Map<string, HTMLDivElement>>(new Map());

  const chatMessages = activeChat ? messages[activeChat.id] || [] : [];

  // Build a map of message ID to message for quick lookup
  const messageMap = useMemo(() => {
    const map = new Map<string, Message>();
    chatMessages.forEach((msg) => map.set(msg.id, msg));
    return map;
  }, [chatMessages]);

  // Auto-scroll to bottom on new messages
  useEffect(() => {
    if (containerRef.current) {
      containerRef.current.scrollTop = containerRef.current.scrollHeight;
    }
  }, [chatMessages]);

  // Send read receipts and reload messages when window becomes visible/focused
  const loadMessages = useMessageStore((state) => state.loadMessages);

  useEffect(() => {
    if (!activeChat) return;

    const handleVisibilityChange = () => {
      if (document.visibilityState === "visible") {
        // Reload messages in case new ones arrived while unfocused
        loadMessages(activeChat.id);
        markAsRead(activeChat.id);
      }
    };

    const handleFocus = () => {
      // Reload messages in case new ones arrived while unfocused
      loadMessages(activeChat.id);
      markAsRead(activeChat.id);
    };

    // Mark as read immediately if already visible
    if (document.visibilityState === "visible") {
      markAsRead(activeChat.id);
    }

    document.addEventListener("visibilitychange", handleVisibilityChange);
    window.addEventListener("focus", handleFocus);

    return () => {
      document.removeEventListener("visibilitychange", handleVisibilityChange);
      window.removeEventListener("focus", handleFocus);
    };
  }, [activeChat, markAsRead, loadMessages]);

  // Handle reply action
  const handleReply = useCallback((message: Message) => {
    setReplyingTo(message);
  }, [setReplyingTo]);

  // Handle scroll to message
  const handleScrollToMessage = useCallback((messageId: string) => {
    const element = messageRefs.current.get(messageId);
    if (element) {
      element.scrollIntoView({ behavior: "smooth", block: "center" });
      // Add highlight effect
      element.classList.add("message-highlight");
      setTimeout(() => {
        element.classList.remove("message-highlight");
      }, 1500);
    }
  }, []);

  // Group messages by date
  const groupedMessages = chatMessages.reduce<{
    date: Date;
    messages: typeof chatMessages;
  }[]>((groups, message) => {
    const messageDate = new Date(message.created_at);

    const lastGroup = groups[groups.length - 1];
    if (lastGroup && isSameDay(lastGroup.date, messageDate)) {
      lastGroup.messages.push(message);
    } else {
      groups.push({
        date: messageDate,
        messages: [message],
      });
    }

    return groups;
  }, []);

  return (
    <div
      ref={containerRef}
      className="flex-1 overflow-y-auto chat-bg-pattern px-2 sm:px-3 md:px-4 lg:px-[3%] xl:px-[6%] 2xl:px-[10%] py-4"
    >
      {groupedMessages.map((group, groupIndex) => (
        <div key={format(group.date, "yyyy-MM-dd")}>
          <DateDivider date={group.date} />
          {group.messages.map((message, messageIndex) => {
            const prevMessage = messageIndex > 0
              ? group.messages[messageIndex - 1]
              : groupIndex > 0
                ? groupedMessages[groupIndex - 1].messages.slice(-1)[0]
                : null;

            const isFirstInGroup = !prevMessage || prevMessage.sender_id !== message.sender_id;
            const isOwnMessage = currentUser ? message.sender_id === currentUser.id : false;

            // Get the replied message if this message is a reply
            const repliedMessage = message.reply_to_id
              ? messageMap.get(message.reply_to_id)
              : undefined;

            return (
              <div
                key={message.id}
                ref={(el) => {
                  if (el) {
                    messageRefs.current.set(message.id, el);
                  } else {
                    messageRefs.current.delete(message.id);
                  }
                }}
              >
                <MessageBubble
                  message={message}
                  isOwn={isOwnMessage}
                  showTail={isFirstInGroup}
                  isGroupChat={activeChat?.chat_type === "group"}
                  repliedMessage={repliedMessage}
                  currentUserId={currentUser?.id}
                  onReply={handleReply}
                  onScrollToMessage={handleScrollToMessage}
                />
              </div>
            );
          })}
        </div>
      ))}

      {chatMessages.length === 0 && (
        <div className="flex items-center justify-center h-full">
          <div className="bg-[var(--bubble-incoming)] text-[var(--text-secondary)] text-sm px-4 py-2 rounded-lg shadow-sm">
            No messages yet. Say hello!
          </div>
        </div>
      )}
    </div>
  );
}
