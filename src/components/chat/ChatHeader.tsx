import { MoreVertical, Phone, Search, Video } from "lucide-react";

import { useWebSocketContext } from "../../context/WebSocketContext";
import { useCallStore } from "../../store/callStore";
import { useChatStore } from "../../store/chatStore";
import { useUIStore } from "../../store/uiStore";
import { getUserDisplayName } from "../../types";
import { formatLastSeen } from "../../utils/formatTime";
import { Avatar } from "../common/Avatar";

export function ChatHeader() {
  const activeChat = useChatStore((state) => state.activeChat);
  const setShowContactInfo = useUIStore((state) => state.setShowContactInfo);
  const showContactInfo = useUIStore((state) => state.showContactInfo);
  const callStatus = useCallStore((state) => state.callStatus);
  const { onlineUsers } = useWebSocketContext();

  if (!activeChat) return null;

  const isInCall = callStatus !== "idle";

  const displayName = activeChat.chat_type === "group"
    ? activeChat.name
    : getUserDisplayName(activeChat.participant);

  const avatarUrl = activeChat.chat_type === "group"
    ? activeChat.avatar_url
    : activeChat.participant?.avatar_url;

  // Check online status from WebSocket context (real-time) rather than stored data
  const participantId = activeChat.participant?.id;
  const isOnline = activeChat.chat_type === "individual" && participantId && onlineUsers.has(participantId);

  const status = activeChat.chat_type === "group"
    ? "Click here for group info"
    : isOnline
      ? "online"
      : formatLastSeen(activeChat.participant?.last_seen);

  const showDeviceSelectionForOutgoing = useCallStore((state) => state.showDeviceSelectionForOutgoing);

  const handleVideoCall = () => {
    // Only allow calls for individual chats when not already in a call
    if (activeChat.chat_type !== "individual" || !activeChat.participant || isInCall) {
      return;
    }
    // Show device selection modal before starting the call
    showDeviceSelectionForOutgoing(
      activeChat.participant.id,
      displayName || "Unknown",
      avatarUrl
    );
  };

  return (
    <header className="flex items-center gap-4 px-4 h-[60px] bg-[var(--bg-secondary)] border-b border-[var(--border-light)] transition-theme">
      {/* Avatar and Info */}
      <button
        onClick={() => setShowContactInfo(!showContactInfo)}
        className="flex items-center flex-1 min-w-0 gap-3 group"
      >
        <div className="relative flex-shrink-0">
          <Avatar
            src={avatarUrl}
            name={displayName || ""}
            size={42}
            className="transition-transform duration-200 group-hover:scale-105"
          />
          {isOnline && (
            <span className="absolute bottom-0 right-0 w-3 h-3 bg-[var(--online-indicator)] border-2 border-[var(--bg-secondary)] rounded-full online-pulse" />
          )}
        </div>
        <div className="flex-1 min-w-0 text-left">
          <h2 className="font-medium text-[var(--text-primary)] text-[16px] truncate group-hover:text-[var(--accent)] transition-colors">
            {displayName}
          </h2>
          <p className={`text-[13px] truncate ${isOnline ? "text-[var(--accent)]" : "text-[var(--text-secondary)]"}`}>
            {status}
          </p>
        </div>
      </button>

      {/* Action Buttons */}
      <div className="flex items-center gap-[2px]">
        <HeaderButton
          icon={<Video size={22} strokeWidth={1.75} />}
          tooltip="Video call"
          onClick={handleVideoCall}
          disabled={isInCall || activeChat.chat_type === "group"}
        />
        <HeaderButton icon={<Phone size={20} strokeWidth={1.75} />} tooltip="Voice call" disabled />
        <HeaderButton icon={<Search size={20} strokeWidth={1.75} />} tooltip="Search" />
        <HeaderButton icon={<MoreVertical size={20} strokeWidth={1.75} />} tooltip="Menu" />
      </div>
    </header>
  );
}

interface HeaderButtonProps {
  icon: React.ReactNode;
  tooltip?: string;
  onClick?: () => void;
  disabled?: boolean;
}

function HeaderButton({ icon, tooltip, onClick, disabled }: HeaderButtonProps) {
  return (
    <button
      onClick={onClick}
      disabled={disabled}
      className={`
        w-10 h-10 flex items-center justify-center rounded-full
        text-[var(--text-secondary)]
        transition-all duration-200 active-press
        group relative
        ${disabled ? "opacity-50 cursor-not-allowed" : "hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)]"}
      `}
    >
      <span className="transition-transform duration-200 group-hover:scale-110">
        {icon}
      </span>

      {/* Tooltip */}
      {tooltip && (
        <span className="
          absolute top-full mt-2 px-2 py-1
          bg-[var(--bg-tertiary)] text-[var(--text-primary)]
          text-xs font-medium rounded-md
          opacity-0 invisible group-hover:opacity-100 group-hover:visible
          -translate-y-1 group-hover:translate-y-0
          transition-all duration-200
          whitespace-nowrap pointer-events-none z-50
          shadow-lg border border-[var(--border)]
        ">
          {tooltip}
        </span>
      )}
    </button>
  );
}
