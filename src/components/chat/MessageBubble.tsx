import { format } from "date-fns";
import { ChevronDown, Reply } from "lucide-react";
import { useEffect, useRef, useState } from "react";

import { getUserDisplayName, type Message } from "../../types";
import { cn } from "../../utils/cn";
import { MessageStatus } from "./MessageStatus";

interface MessageBubbleProps {
  message: Message;
  isOwn: boolean;
  showTail: boolean;
  isGroupChat: boolean;
  repliedMessage?: Message;
  currentUserId?: string;
  onReply?: (message: Message) => void;
  onScrollToMessage?: (messageId: string) => void;
}

// Generate consistent color for sender name in group chats
const nameColors = [
  "#FF6B6B", "#06CF9C", "#53BDEB", "#FFB347", "#DDA0DD",
  "#87CEEB", "#F0E68C", "#98FB98", "#DEB887", "#87CEFA",
];

function getNameColor(name: string): string {
  const index = name.split("").reduce((acc, char) => acc + char.charCodeAt(0), 0) % nameColors.length;
  return nameColors[index];
}

export function MessageBubble({
  message,
  isOwn,
  showTail,
  isGroupChat,
  repliedMessage,
  currentUserId,
  onReply,
  onScrollToMessage,
}: MessageBubbleProps) {
  const time = format(new Date(message.created_at), "HH:mm");
  const [showDropdown, setShowDropdown] = useState(false);
  const [isHovered, setIsHovered] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);

  // Close dropdown when clicking outside
  useEffect(() => {
    if (!showDropdown) return;

    const handleClickOutside = (e: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(e.target as Node)) {
        setShowDropdown(false);
      }
    };

    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [showDropdown]);

  const handleReplyClick = () => {
    setShowDropdown(false);
    if (onReply) {
      onReply(message);
    }
  };

  // Get reply quote color
  const getReplyColor = () => {
    if (!repliedMessage) return "#53BDEB";
    if (isOwn) {
      return repliedMessage.sender_id === message.sender_id ? "rgba(255,255,255,0.7)" : "#53BDEB";
    }
    return getNameColor(getUserDisplayName(repliedMessage.sender));
  };

  return (
    <>
      <div
        className={cn(
          "flex px-[2%] animate-slide-up group",
          isOwn ? "justify-end" : "justify-start",
          showTail ? "mt-3" : "mt-[2px]"
        )}
        onMouseEnter={() => setIsHovered(true)}
        onMouseLeave={() => setIsHovered(false)}
      >
        <div
          className={cn(
            "relative max-w-[65%] min-w-[80px] rounded-[7.5px] shadow-sm transition-all duration-200 hover:shadow-md",
            isOwn ? "bg-[var(--bubble-outgoing)]" : "bg-[var(--bubble-incoming)]",
            isOwn && showTail && "rounded-tr-[0px]",
            !isOwn && showTail && "rounded-tl-[0px]",
            showTail && (isOwn ? "bubble-tail-out" : "bubble-tail-in")
          )}
        >
          {/* Dropdown arrow - appears on hover */}
          <button
            onClick={() => setShowDropdown(!showDropdown)}
            className={cn(
              "absolute top-0 right-0 h-7 px-1 flex items-center justify-center transition-opacity duration-150 z-10 rounded-tr-[7.5px]",
              isHovered || showDropdown ? "opacity-100" : "opacity-0",
              isOwn && showTail && "rounded-tr-[0px]"
            )}
            style={{
              background: isOwn
                ? "linear-gradient(to left, var(--bubble-outgoing) 60%, transparent)"
                : "linear-gradient(to left, var(--bubble-incoming) 60%, transparent)",
              paddingLeft: "16px",
            }}
          >
            <ChevronDown size={16} className={isOwn ? "text-[rgba(255,255,255,0.9)]" : "text-[var(--text-primary)] opacity-60"} />
          </button>

          {/* Dropdown menu */}
          {showDropdown && (
            <div
              ref={dropdownRef}
              className="absolute bottom-full right-0 z-50 bg-[var(--bg-primary)] rounded-lg shadow-lg py-1 min-w-[100px] animate-scale-in mb-1"
              style={{ boxShadow: "0 2px 12px rgba(0,0,0,0.25)" }}
            >
              <button
                onClick={handleReplyClick}
                className="w-full px-3 py-1.5 text-left text-[13px] text-[var(--text-primary)] hover:bg-[var(--bg-hover)] flex items-center gap-2"
              >
                <Reply size={14} className="text-[var(--text-secondary)]" />
                Reply
              </button>
            </div>
          )}

          {/* Sender name for group chats */}
          {isGroupChat && !isOwn && showTail && (
            <p
              className="text-[12.5px] font-medium px-[9px] pt-[6px] pb-0"
              style={{ color: getNameColor(getUserDisplayName(message.sender)) }}
            >
              {getUserDisplayName(message.sender)}
            </p>
          )}

          {/* Reply quote */}
          {repliedMessage && (
            <div
              className={cn(
                "mx-[6px] mt-[6px] rounded-[5px] overflow-hidden cursor-pointer",
                isOwn ? "bg-[rgba(0,0,0,0.1)]" : "bg-[rgba(0,0,0,0.05)]"
              )}
              onClick={() => onScrollToMessage?.(repliedMessage.id)}
            >
              <div className="flex">
                <div className="w-[3px] flex-shrink-0" style={{ backgroundColor: getReplyColor() }} />
                <div className="px-2 py-[6px] min-w-0">
                  <p className="text-[12px] font-medium truncate" style={{ color: getReplyColor() }}>
                    {currentUserId && repliedMessage.sender_id === currentUserId
                      ? "You"
                      : getUserDisplayName(repliedMessage.sender)}
                  </p>
                  <p className={cn("text-[12px] truncate", isOwn ? "text-[rgba(255,255,255,0.7)]" : "text-[var(--text-secondary)]")}>
                    {repliedMessage.content || "[Media]"}
                  </p>
                </div>
              </div>
            </div>
          )}

          {/* Message content */}
          <div
            className={cn(
              "px-[9px] pt-[6px] pb-[8px]",
              isGroupChat && !isOwn && showTail && !repliedMessage && "pt-[2px]",
              repliedMessage && "pt-[4px]"
            )}
          >
            <div className="flex flex-wrap items-end">
              <p className="text-[14.2px] text-[var(--text-primary)] whitespace-pre-wrap break-words leading-[19px]">
                {message.content}
              </p>

              {/* Time and status - floating style like WhatsApp */}
              <span className={cn("flex items-center gap-[3px] flex-shrink-0 ml-auto pl-2 text-[11px] leading-none", isOwn ? "text-[rgba(255,255,255,0.6)]" : "text-[var(--text-secondary)]")}>
                <span className="translate-y-[1px]">{time}</span>
                {isOwn && (
                  <span className="translate-y-[1px]">
                    <MessageStatus status={message.status} size={16} />
                  </span>
                )}
              </span>
            </div>
          </div>
        </div>
      </div>
    </>
  );
}
