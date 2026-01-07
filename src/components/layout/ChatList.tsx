import { Search, SlidersHorizontal, X } from "lucide-react";
import { useMemo, useState } from "react";

import { useChatStore } from "../../store/chatStore";
import { useUIStore } from "../../store/uiStore";
import { ChatListItem } from "../chat-list/ChatListItem";

export function ChatList() {
  const { chats, activeChat, setActiveChat } = useChatStore();
  const { searchQuery, setSearchQuery } = useUIStore();
  const [isSearchFocused, setIsSearchFocused] = useState(false);

  const filteredChats = useMemo(() => {
    if (!searchQuery.trim()) return chats;

    const query = searchQuery.toLowerCase();
    return chats.filter((chat) => {
      const name = chat.chat_type === "group"
        ? chat.name
        : chat.participant?.name;
      return name?.toLowerCase().includes(query);
    });
  }, [chats, searchQuery]);

  return (
    <div className="flex flex-col h-full bg-[var(--bg-primary)] transition-theme">
      {/* Header */}
      <div className="px-4 pt-4 pb-1">
        <h1 className="text-[22px] font-bold text-[var(--text-primary)] tracking-tight">
          Chats
        </h1>
      </div>

      {/* Search Bar */}
      <div className="px-3 py-2">
        <div
          className={`
            flex items-center gap-3 rounded-lg px-4 py-[9px]
            transition-all duration-200
            ${isSearchFocused
              ? "bg-[var(--bg-primary)] ring-1 ring-[var(--accent)]"
              : "bg-[var(--bg-secondary)]"
            }
          `}
        >
          <Search
            size={18}
            className={`flex-shrink-0 transition-colors duration-200 ${isSearchFocused ? "text-[var(--accent)]" : "text-[var(--text-secondary)]"
              }`}
          />
          <input
            type="text"
            placeholder="Search or start new chat"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            onFocus={() => setIsSearchFocused(true)}
            onBlur={() => setIsSearchFocused(false)}
            className="flex-1 bg-transparent text-[14px] text-[var(--text-primary)] placeholder-[var(--text-secondary)] outline-none"
          />
          {searchQuery ? (
            <button
              onClick={() => setSearchQuery("")}
              className="flex-shrink-0 text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors p-0.5 hover:bg-[var(--bg-hover)] rounded-full"
            >
              <X size={16} />
            </button>
          ) : (
            <button className="flex-shrink-0 text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors p-0.5 hover:bg-[var(--bg-hover)] rounded">
              <SlidersHorizontal size={16} />
            </button>
          )}
        </div>
      </div>

      {/* Filter Chips */}
      <div className="flex gap-2 px-3 pb-2">
        <FilterChip label="All" active />
        <FilterChip label="Unread" />
        <FilterChip label="Groups" />
      </div>

      {/* Chat List */}
      <div className="flex-1 overflow-y-auto scrollbar-hidden">
        {filteredChats.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-[var(--text-secondary)] text-sm px-8 text-center">
            <div className="w-16 h-16 rounded-full bg-[var(--bg-secondary)] flex items-center justify-center mb-4">
              <Search size={24} className="text-[var(--text-secondary)]" />
            </div>
            <p className="font-medium text-[var(--text-primary)] mb-1">No chats found</p>
            <p className="text-xs">Try searching with a different term</p>
          </div>
        ) : (
          <div className="animate-fade-in">
            {filteredChats.map((chat, index) => (
              <ChatListItem
                key={chat.id}
                chat={chat}
                isActive={activeChat?.id === chat.id}
                onClick={() => setActiveChat(chat)}
                style={{ animationDelay: `${index * 30}ms` }}
              />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

interface FilterChipProps {
  label: string;
  active?: boolean;
  onClick?: () => void;
}

function FilterChip({ label, active, onClick }: FilterChipProps) {
  return (
    <button
      onClick={onClick}
      className={`
        px-3 py-1 rounded-full text-xs font-medium
        transition-all duration-200 active-press
        ${active
          ? "bg-[var(--accent)]/15 text-[var(--accent)]"
          : "bg-[var(--bg-secondary)] text-[var(--text-secondary)] hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)]"
        }
      `}
    >
      {label}
    </button>
  );
}
