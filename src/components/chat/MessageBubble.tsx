import { format } from "date-fns";

import { getUserDisplayName, type Message } from "../../types";
import { MessageStatus } from "./MessageStatus";

interface MessageBubbleProps {
  message: Message;
  isOwn: boolean;
  showTail: boolean;
  isGroupChat: boolean;
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

export function MessageBubble({ message, isOwn, showTail, isGroupChat }: MessageBubbleProps) {
  const time = format(new Date(message.created_at), "HH:mm");

  return (
    <div
      className={`
        flex ${isOwn ? "justify-end" : "justify-start"}
        ${showTail ? "mt-3" : "mt-[2px]"}
        px-[2%]
        animate-slide-up
      `}
    >
      <div
        className={`
          relative max-w-[65%] min-w-[80px]
          rounded-[7.5px] shadow-sm
          transition-all duration-200
          hover:shadow-md
          ${isOwn
            ? `bg-[var(--bubble-outgoing)] ${showTail ? "rounded-tr-[0px]" : ""}`
            : `bg-[var(--bubble-incoming)] ${showTail ? "rounded-tl-[0px]" : ""}`
          }
          ${showTail ? (isOwn ? "bubble-tail-out" : "bubble-tail-in") : ""}
        `}
      >
        {/* Sender name for group chats */}
        {isGroupChat && !isOwn && showTail && (
          <p
            className="text-[12.5px] font-medium px-[9px] pt-[6px] pb-0"
            style={{ color: getNameColor(getUserDisplayName(message.sender)) }}
          >
            {getUserDisplayName(message.sender)}
          </p>
        )}

        {/* Message content */}
        <div className={`
          px-[9px] pt-[6px] pb-[8px]
          ${isGroupChat && !isOwn && showTail ? "pt-[2px]" : ""}
        `}>
          <div className="flex flex-wrap items-end">
            <p className="text-[14.2px] text-[var(--text-primary)] whitespace-pre-wrap break-words leading-[19px]">
              {message.content}
            </p>

            {/* Time and status - floating style like WhatsApp */}
            <span className={`
              flex items-center gap-[3px] flex-shrink-0 ml-auto pl-2
              text-[11px] leading-none
              ${isOwn ? "text-[rgba(255,255,255,0.6)]" : "text-[var(--text-secondary)]"}
            `}>
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
  );
}
