import { Check, Pencil, X } from "lucide-react";
import { useState } from "react";

import { userService } from "../../services";
import { useChatStore } from "../../store/chatStore";
import { useUIStore } from "../../store/uiStore";
import { getUserDisplayName } from "../../types";
import { Avatar } from "../common/Avatar";

export function ContactInfoPanel() {
  const activeChat = useChatStore((state) => state.activeChat);
  const loadChats = useChatStore((state) => state.loadChats);
  const setShowContactInfo = useUIStore((state) => state.setShowContactInfo);

  const participant = activeChat?.participant;

  const [editingName, setEditingName] = useState(false);
  const [customName, setCustomName] = useState(
    participant?.display_name || ""
  );
  const [saving, setSaving] = useState(false);

  if (!activeChat || activeChat.chat_type !== "individual" || !participant) {
    return null;
  }

  const displayName = getUserDisplayName(participant);
  const hasCustomName = !!participant.display_name;

  const handleSaveName = async () => {
    if (!customName.trim()) return;

    setSaving(true);
    try {
      await userService.saveContact(participant.id, customName.trim());
      await loadChats();
      setEditingName(false);
    } catch (e) {
      console.error("Failed to save contact:", e);
    } finally {
      setSaving(false);
    }
  };

  const handleStartEdit = () => {
    setCustomName(participant.display_name || participant.name);
    setEditingName(true);
  };

  return (
    <div className="w-[340px] h-full bg-[var(--bg-primary)] border-l border-[var(--border-light)] flex flex-col animate-slide-in-right">
      {/* Header */}
      <header className="flex items-center gap-4 px-4 py-4 bg-[var(--bg-secondary)]">
        <button
          onClick={() => setShowContactInfo(false)}
          className="text-[var(--text-primary)] hover:text-[var(--text-secondary)] transition-colors"
        >
          <X size={24} />
        </button>
        <h2 className="text-lg font-medium text-[var(--text-primary)]">
          Contact info
        </h2>
      </header>

      {/* Content */}
      <div className="flex-1 overflow-y-auto">
        {/* Avatar Section */}
        <div className="flex flex-col items-center py-8 bg-[var(--bg-secondary)]">
          <Avatar
            src={participant.avatar_url}
            name={displayName}
            size={200}
          />
          <h3 className="mt-4 text-xl font-medium text-[var(--text-primary)]">
            {displayName}
          </h3>
          <p className="text-sm text-[var(--text-secondary)]">
            {participant.phone || participant.id}
          </p>
        </div>

        <div className="h-2 bg-[var(--bg-secondary)]" />

        {/* Custom Name Section */}
        <div className="p-4 bg-[var(--bg-primary)]">
          <label className="text-sm text-[var(--accent)] mb-2 block">
            Contact name
          </label>
          {editingName ? (
            <div className="flex items-center gap-2">
              <input
                type="text"
                value={customName}
                onChange={(e) => setCustomName(e.target.value)}
                placeholder="Enter a name for this contact"
                className="flex-1 bg-transparent text-[var(--text-primary)] border-b-2 border-[var(--accent)] outline-none py-1"
                autoFocus
                disabled={saving}
              />
              <button
                onClick={handleSaveName}
                disabled={saving || !customName.trim()}
                className="text-[var(--accent)] hover:text-[var(--accent-dark)] disabled:opacity-50"
              >
                <Check size={20} />
              </button>
            </div>
          ) : (
            <div className="flex items-center justify-between">
              <div>
                <span className="text-[var(--text-primary)]">
                  {displayName}
                </span>
                {hasCustomName && (
                  <p className="text-xs text-[var(--text-secondary)] mt-1">
                    Original name: {participant.name}
                  </p>
                )}
              </div>
              <button
                onClick={handleStartEdit}
                className="text-[var(--text-secondary)] hover:text-[var(--accent)]"
              >
                <Pencil size={18} />
              </button>
            </div>
          )}
          <p className="text-xs text-[var(--text-secondary)] mt-4">
            This name is only visible to you
          </p>
        </div>

        <div className="h-2 bg-[var(--bg-secondary)]" />

        {/* About Section */}
        {participant.about && (
          <>
            <div className="p-4 bg-[var(--bg-primary)]">
              <label className="text-sm text-[var(--accent)] mb-2 block">
                About
              </label>
              <span className="text-[var(--text-primary)]">
                {participant.about}
              </span>
            </div>
            <div className="h-2 bg-[var(--bg-secondary)]" />
          </>
        )}

        {/* User ID Section */}
        <div className="p-4 bg-[var(--bg-primary)]">
          <label className="text-sm text-[var(--accent)] mb-2 block">
            Pulse ID
          </label>
          <span className="text-[var(--text-primary)] text-sm font-mono break-all">
            {participant.id}
          </span>
        </div>
      </div>

      <style>{`
        @keyframes slide-in-right {
          from {
            transform: translateX(100%);
          }
          to {
            transform: translateX(0);
          }
        }
        .animate-slide-in-right {
          animation: slide-in-right 0.2s ease-out;
        }
      `}</style>
    </div>
  );
}
