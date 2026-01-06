import { useChatStore } from "../../store/chatStore";
import { useWebSocketContext } from "../../context/WebSocketContext";

interface TypingIndicatorProps {
  name?: string;
}

export function TypingIndicator({ name }: TypingIndicatorProps) {
  const { activeChat, currentUser } = useChatStore();
  const { typingUsers } = useWebSocketContext();

  // Check if anyone is typing in the active chat (exclude self)
  const typingInChat = activeChat
    ? (typingUsers[activeChat.id] || []).filter(userId => userId !== currentUser?.id)
    : [];

  // If name is provided, show the old style; otherwise use WebSocket data
  const showIndicator = name || typingInChat.length > 0;

  if (!showIndicator) return null;

  return (
    <div className="flex justify-start px-2 sm:px-3 md:px-4 lg:px-[3%] xl:px-[6%] 2xl:px-[10%] pb-2 animate-fade-in">
      <div className="bg-[var(--bubble-incoming)] rounded-[7.5px] rounded-tl-[0px] px-4 py-3 shadow-sm bubble-tail-in">
        <div className="flex items-center gap-1">
          {name && (
            <span className="text-[var(--accent)] text-xs font-medium mr-2">
              {name}
            </span>
          )}
          <div className="flex items-center gap-1">
            <span className="w-2 h-2 rounded-full bg-[var(--typing-dot)] typing-dot" />
            <span className="w-2 h-2 rounded-full bg-[var(--typing-dot)] typing-dot" />
            <span className="w-2 h-2 rounded-full bg-[var(--typing-dot)] typing-dot" />
          </div>
        </div>
      </div>
    </div>
  );
}

// Inline typing text for chat header
export function TypingText({ names }: { names: string[] }) {
  if (names.length === 0) return null;

  const text = names.length === 1
    ? `${names[0]} is typing...`
    : names.length === 2
      ? `${names[0]} and ${names[1]} are typing...`
      : `${names[0]} and ${names.length - 1} others are typing...`;

  return (
    <span className="text-[var(--accent)] animate-pulse">
      {text}
    </span>
  );
}
