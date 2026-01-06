import {
  MessageCircle,
  Users,
  CircleDashed,
  Settings,
  Moon,
  Sun,
} from "lucide-react";
import { useUIStore } from "../../store/uiStore";
import { useChatStore } from "../../store/chatStore";
import { Avatar } from "../common/Avatar";

export function Sidebar() {
  const { theme, toggleTheme, setShowProfile, setShowNewChat } = useUIStore();
  const { currentUser } = useChatStore();

  return (
    <div className="flex flex-col w-[72px] min-w-[72px] bg-[var(--bg-secondary)] border-r border-[var(--border-light)] transition-theme">
      {/* Profile Avatar */}
      <div className="flex justify-center py-4">
        <button
          onClick={() => setShowProfile(true)}
          className="relative group"
        >
          <Avatar
            src={currentUser?.avatar_url}
            name={currentUser?.name || "You"}
            size={44}
            className="ring-2 ring-transparent group-hover:ring-[var(--accent)] transition-all duration-300"
          />
          <div className="absolute inset-0 rounded-full bg-black/0 group-hover:bg-black/10 transition-colors duration-200" />
        </button>
      </div>

      {/* Navigation Icons */}
      <nav className="flex-1 flex flex-col items-center pt-2 gap-1">
        <SidebarButton
          icon={<MessageCircle size={24} strokeWidth={1.75} />}
          active
          tooltip="Chats"
        />
        <SidebarButton
          icon={<CircleDashed size={24} strokeWidth={1.75} />}
          tooltip="Status"
        />
        <SidebarButton
          icon={<Users size={24} strokeWidth={1.75} />}
          tooltip="New chat"
          onClick={() => setShowNewChat(true)}
        />
      </nav>

      {/* Bottom Icons */}
      <div className="flex flex-col items-center pb-4 gap-1">
        <SidebarButton
          icon={theme === "dark" ? <Sun size={24} strokeWidth={1.75} /> : <Moon size={24} strokeWidth={1.75} />}
          tooltip={theme === "dark" ? "Light mode" : "Dark mode"}
          onClick={toggleTheme}
        />
        <SidebarButton
          icon={<Settings size={24} strokeWidth={1.75} />}
          tooltip="Settings"
        />
      </div>
    </div>
  );
}

interface SidebarButtonProps {
  icon: React.ReactNode;
  active?: boolean;
  tooltip?: string;
  onClick?: () => void;
}

function SidebarButton({ icon, active, tooltip, onClick }: SidebarButtonProps) {
  return (
    <button
      onClick={onClick}
      className={`
        relative w-12 h-12 flex items-center justify-center rounded-2xl
        transition-all duration-200 group active-press
        ${active
          ? "bg-[var(--accent)]/15 text-[var(--accent)]"
          : "text-[var(--text-secondary)] hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)]"
        }
      `}
    >
      {/* Active indicator bar */}
      {active && (
        <span className="absolute left-0 top-1/2 -translate-y-1/2 w-[3px] h-7 bg-[var(--accent)] rounded-r-full transition-all duration-300" />
      )}

      <span className={`transition-transform duration-200 ${!active ? "group-hover:scale-110 group-active:scale-95" : ""}`}>
        {icon}
      </span>

      {/* Tooltip */}
      {tooltip && (
        <span className="
          absolute left-full ml-3 px-3 py-1.5
          bg-[var(--bg-tertiary)] text-[var(--text-primary)]
          text-xs font-medium rounded-lg
          opacity-0 invisible group-hover:opacity-100 group-hover:visible
          -translate-x-2 group-hover:translate-x-0
          transition-all duration-200
          whitespace-nowrap pointer-events-none z-50
          shadow-lg border border-[var(--border)]
        ">
          {tooltip}
          {/* Arrow */}
          <span className="absolute left-0 top-1/2 -translate-y-1/2 -translate-x-1 w-2 h-2 bg-[var(--bg-tertiary)] border-l border-b border-[var(--border)] rotate-45" />
        </span>
      )}
    </button>
  );
}
