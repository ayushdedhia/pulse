import { format, isSameDay } from "date-fns";
import { useEffect, useRef } from "react";

import { useChatStore } from "../../store/chatStore";
import { DateDivider } from "./DateDivider";
import { MessageBubble } from "./MessageBubble";

export function MessageList() {
  const { activeChat, messages, currentUser } = useChatStore();
  const containerRef = useRef<HTMLDivElement>(null);

  const chatMessages = activeChat ? messages[activeChat.id] || [] : [];

  // Auto-scroll to bottom on new messages
  useEffect(() => {
    if (containerRef.current) {
      containerRef.current.scrollTop = containerRef.current.scrollHeight;
    }
  }, [chatMessages]);

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

            return (
              <MessageBubble
                key={message.id}
                message={message}
                isOwn={isOwnMessage}
                showTail={isFirstInGroup}
                isGroupChat={activeChat?.chat_type === "group"}
              />
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
