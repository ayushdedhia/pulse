import { Lock } from "lucide-react";
import { useCallback, useState } from "react";

import { useChatStore } from "../../store/chatStore";
import { useUIStore } from "../../store/uiStore";
import { NewChatModal } from "../modals/NewChatModal";
import { ProfileModal } from "../modals/ProfileModal";
import { ContactInfoPanel } from "../panels/ContactInfoPanel";
import { ChatList } from "./ChatList";
import { ChatWindow } from "./ChatWindow";
import { Sidebar } from "./Sidebar";
import { Titlebar } from "./Titlebar";

const MIN_CHAT_LIST_WIDTH = 280;
const MAX_CHAT_LIST_WIDTH = 500;
const DEFAULT_CHAT_LIST_WIDTH = 380;

export function AppLayout() {
  const activeChat = useChatStore((state) => state.activeChat);
  const showNewChat = useUIStore((state) => state.showNewChat);
  const showProfile = useUIStore((state) => state.showProfile);
  const showContactInfo = useUIStore((state) => state.showContactInfo);
  const [chatListWidth, setChatListWidth] = useState(DEFAULT_CHAT_LIST_WIDTH);
  const [isResizing, setIsResizing] = useState(false);

  const handleMouseDown = useCallback(
    (e: React.MouseEvent) => {
      e.preventDefault();
      setIsResizing(true);

      const startX = e.clientX;
      const startWidth = chatListWidth;

      const handleMouseMove = (e: MouseEvent) => {
        const delta = e.clientX - startX;
        const newWidth = Math.min(
          MAX_CHAT_LIST_WIDTH,
          Math.max(MIN_CHAT_LIST_WIDTH, startWidth + delta),
        );
        setChatListWidth(newWidth);
      };

      const handleMouseUp = () => {
        setIsResizing(false);
        document.removeEventListener("mousemove", handleMouseMove);
        document.removeEventListener("mouseup", handleMouseUp);
      };

      document.addEventListener("mousemove", handleMouseMove);
      document.addEventListener("mouseup", handleMouseUp);
    },
    [chatListWidth],
  );

  return (
    <div className="flex flex-col h-screen w-screen overflow-hidden bg-[var(--bg-primary)] transition-theme">
      {/* Custom Titlebar */}
      <Titlebar />

      {/* Main Content */}
      <div className="flex flex-1 overflow-hidden">
        {/* Left Sidebar with icons */}
        <Sidebar />

        {/* Chat List Panel with Resizable Handle */}
        <div
          className="flex flex-col border-r border-[var(--border-light)] bg-[var(--bg-primary)] relative"
          style={{ width: chatListWidth, minWidth: MIN_CHAT_LIST_WIDTH }}
        >
          <ChatList />

          {/* Resize Handle */}
          <div
            className={`
              absolute right-0 top-0 bottom-0 w-1 cursor-col-resize
              hover:bg-[var(--accent)]/50 transition-colors
              ${isResizing ? "bg-[var(--accent)]" : ""}
            `}
            onMouseDown={handleMouseDown}
          />
        </div>

        {/* Main Chat Area */}
        <div
          className={`flex-1 flex flex-col ${isResizing ? "select-none" : ""}`}
        >
          {activeChat ? <ChatWindow /> : <EmptyState />}
        </div>

        {/* Contact Info Panel (slides in from right) */}
        {showContactInfo && activeChat && <ContactInfoPanel />}
      </div>

      {/* Modals */}
      {showNewChat && <NewChatModal />}
      {showProfile && <ProfileModal />}
    </div>
  );
}

function EmptyState() {
  return (
    <div className="flex-1 flex flex-col items-center justify-center bg-[var(--bg-secondary)] text-center p-8 relative transition-theme">
      <div className="max-w-[420px] animate-fade-in">
        {/* App Logo */}
        <div className="relative -mb-4">
          <img
            src="/logo.png"
            alt="Pulse"
            className="w-[140px] h-[140px] mx-auto object-contain"
          />
        </div>

        <h1 className="text-[28px] font-semibold text-[var(--text-primary)] mb-3 tracking-tight">
          Pulse for Desktop
        </h1>
        <p className="text-[15px] text-[var(--text-secondary)] leading-relaxed mb-6">
          Send and receive messages without keeping your phone online.
          <br />
          Use Pulse on up to 4 linked devices and 1 phone at the same time.
        </p>

        {/* Feature highlights */}
        <div className="flex justify-center gap-6 text-[var(--text-secondary)] text-xs">
          <div className="flex items-center gap-1.5">
            <div className="w-2 h-2 rounded-full bg-[var(--accent)]" />
            <span>Fast</span>
          </div>
          <div className="flex items-center gap-1.5">
            <div className="w-2 h-2 rounded-full bg-[var(--accent)]" />
            <span>Secure</span>
          </div>
          <div className="flex items-center gap-1.5">
            <div className="w-2 h-2 rounded-full bg-[var(--accent)]" />
            <span>Private</span>
          </div>
        </div>
      </div>

      {/* Bottom encryption notice */}
      <div className="absolute bottom-6 flex items-center gap-2 text-[var(--text-muted)] text-[13px]">
        <Lock size={14} />
        <span>End-to-end encrypted</span>
      </div>
    </div>
  );
}
