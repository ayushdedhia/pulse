import { format, isToday, isYesterday } from "date-fns";
import type { CSSProperties } from "react";

import { useWebSocketContext } from "../../context/WebSocketContext";
import type { Chat } from "../../types";
import { MessageStatus } from "../chat/MessageStatus";
import { Avatar } from "../common/Avatar";

interface ChatListItemProps {
  chat: Chat;
  isActive: boolean;
  onClick: () => void;
  style?: CSSProperties;
}

export function ChatListItem({ chat, isActive, onClick, style }: ChatListItemProps) {
  const { onlineUsers } = useWebSocketContext();

  const displayName = chat.chat_type === "group"
    ? chat.name
    : chat.participant?.name || "Unknown";

  const avatarUrl = chat.chat_type === "group"
    ? chat.avatar_url
    : chat.participant?.avatar_url;

  // Check online status from WebSocket context (real-time)
  const participantId = chat.participant?.id;
  const isOnline = chat.chat_type === "individual" && participantId && onlineUsers.has(participantId);

  const lastMessage = chat.last_message;
  const lastMessageTime = lastMessage ? formatMessageTime(lastMessage.created_at) : "";

  const isOwnMessage = lastMessage?.sender_id === "self";
  const hasUnread = chat.unread_count > 0;

  return (
    <button
      onClick={onClick}
      style={style}
      className={`
        w-full flex items-center gap-3 px-3 py-2 text-left
        transition-all duration-200 group relative
        ${isActive
          ? "bg-[var(--bg-hover)]"
          : "hover:bg-[var(--bg-secondary)]"
        }
      `}
    >
      {/* Active indicator */}
      {isActive && (
        <span className="absolute left-0 top-1/2 -translate-y-1/2 w-[3px] h-12 bg-[var(--accent)] rounded-r-full" />
      )}

      {/* Avatar */}
      <div className="relative flex-shrink-0">
        <Avatar
          src={avatarUrl}
          name={displayName || ""}
          size={50}
          className="transition-transform duration-200 group-hover:scale-105"
        />
        {isOnline && (
          <span className="absolute bottom-0.5 right-0.5 w-3 h-3 bg-[var(--online-indicator)] border-2 border-[var(--bg-primary)] rounded-full online-pulse" />
        )}
      </div>

      {/* Content */}
      <div className="flex-1 min-w-0 py-3">
        <div className="flex items-center justify-between gap-2 mb-0.5">
          <span className={`font-medium truncate ${hasUnread ? "text-[var(--text-primary)]" : "text-[var(--text-primary)]"}`}>
            {displayName}
          </span>
          <span className={`text-[11px] flex-shrink-0 ${hasUnread
              ? "text-[var(--accent)] font-medium"
              : "text-[var(--text-secondary)]"
            }`}>
            {lastMessageTime}
          </span>
        </div>

        <div className="flex items-center gap-1.5">
          {/* Message status for own messages */}
          {isOwnMessage && lastMessage && (
            <MessageStatus status={lastMessage.status} className="flex-shrink-0" size={16} />
          )}

          {/* Last message preview */}
          <p className={`text-[13px] truncate flex-1 ${hasUnread ? "text-[var(--text-primary)]" : "text-[var(--text-secondary)]"}`}>
            {chat.chat_type === "group" && lastMessage && !isOwnMessage && (
              <span className="text-[var(--text-secondary)]">
                {lastMessage.sender?.name?.split(" ")[0]}:{" "}
              </span>
            )}
            {lastMessage?.content || "No messages yet"}
          </p>

          {/* Unread badge */}
          {hasUnread && (
            <span className="
              flex-shrink-0 min-w-[20px] h-[20px] px-[6px]
              flex items-center justify-center
              bg-[var(--unread-badge)] text-white
              text-[11px] font-medium rounded-full
              animate-badge-pulse
            ">
              <span className="font-mono">{chat.unread_count > 99 ? "99+" : chat.unread_count}</span>
            </span>
          )}
        </div>
      </div>
    </button>
  );
}

function formatMessageTime(timestamp: number): string {
  const date = new Date(timestamp);

  if (isToday(date)) {
    return format(date, "HH:mm");
  }

  if (isYesterday(date)) {
    return "Yesterday";
  }

  return format(date, "dd/MM/yyyy");
}
