import { ArrowLeft, Camera, Check, Copy, Pencil } from "lucide-react";
import { useState } from "react";

import { useUIStore } from "../../store/uiStore";
import { useUserStore } from "../../store/userStore";
import { Avatar } from "../common/Avatar";

export function ProfileModal() {
  const setShowProfile = useUIStore((state) => state.setShowProfile);
  const currentUser = useUserStore((state) => state.currentUser);
  const [editingName, setEditingName] = useState(false);
  const [editingAbout, setEditingAbout] = useState(false);
  const [name, setName] = useState(currentUser?.name || "");
  const [about, setAbout] = useState(currentUser?.about || "");
  const [copied, setCopied] = useState(false);

  const handleCopyId = async () => {
    if (currentUser?.id) {
      await navigator.clipboard.writeText(currentUser.id);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  const handleSaveName = () => {
    // TODO: Save to backend
    setEditingName(false);
  };

  const handleSaveAbout = () => {
    // TODO: Save to backend
    setEditingAbout(false);
  };

  return (
    <div className="fixed inset-0 z-50 flex">
      {/* Backdrop */}
      <div
        className="absolute inset-0 bg-black/50"
        onClick={() => setShowProfile(false)}
      />

      {/* Modal Panel */}
      <div className="relative w-[350px] h-full bg-[var(--bg-primary)] flex flex-col shadow-2xl animate-slide-in">
        {/* Header */}
        <header className="flex items-center gap-6 px-4 py-4 bg-[var(--bg-secondary)]">
          <button
            onClick={() => setShowProfile(false)}
            className="text-[var(--text-primary)] hover:text-[var(--text-secondary)] transition-colors"
          >
            <ArrowLeft size={24} />
          </button>
          <h2 className="text-lg font-medium text-[var(--text-primary)]">
            Profile
          </h2>
        </header>

        {/* Profile Content */}
        <div className="flex-1 overflow-y-auto">
          {/* Avatar Section */}
          <div className="flex flex-col items-center py-8 bg-[var(--bg-secondary)]">
            <div className="relative group">
              <Avatar
                src={currentUser?.avatar_url}
                name={currentUser?.name || "You"}
                size={200}
              />
              <button className="absolute inset-0 flex flex-col items-center justify-center transition-opacity rounded-full opacity-0 bg-black/50 group-hover:opacity-100">
                <Camera size={24} className="mb-1 text-white" />
                <span className="text-xs text-white">CHANGE PROFILE PHOTO</span>
              </button>
            </div>
          </div>

          {/* Name Section */}
          <div className="p-4 bg-[var(--bg-primary)]">
            <label className="text-sm text-[var(--accent)] mb-2 block">
              Your name
            </label>
            {editingName ? (
              <div className="flex items-center gap-2">
                <input
                  type="text"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  className="flex-1 bg-transparent text-[var(--text-primary)] border-b-2 border-[var(--accent)] outline-none py-1"
                  autoFocus
                />
                <button
                  onClick={handleSaveName}
                  className="text-[var(--accent)] hover:text-[var(--accent-dark)]"
                >
                  <Check size={20} />
                </button>
              </div>
            ) : (
              <div className="flex items-center justify-between">
                <span className="text-[var(--text-primary)]">
                  {currentUser?.name}
                </span>
                <button
                  onClick={() => setEditingName(true)}
                  className="text-[var(--text-secondary)] hover:text-[var(--accent)]"
                >
                  <Pencil size={18} />
                </button>
              </div>
            )}
            <p className="text-xs text-[var(--text-secondary)] mt-4">
              This is not your username or pin. This name will be visible to your Pulse contacts.
            </p>
          </div>

          <div className="h-2 bg-[var(--bg-secondary)]" />

          {/* About Section */}
          <div className="p-4 bg-[var(--bg-primary)]">
            <label className="text-sm text-[var(--accent)] mb-2 block">
              About
            </label>
            {editingAbout ? (
              <div className="flex items-center gap-2">
                <input
                  type="text"
                  value={about}
                  onChange={(e) => setAbout(e.target.value)}
                  className="flex-1 bg-transparent text-[var(--text-primary)] border-b-2 border-[var(--accent)] outline-none py-1"
                  autoFocus
                />
                <button
                  onClick={handleSaveAbout}
                  className="text-[var(--accent)] hover:text-[var(--accent-dark)]"
                >
                  <Check size={20} />
                </button>
              </div>
            ) : (
              <div className="flex items-center justify-between">
                <span className="text-[var(--text-primary)]">
                  {currentUser?.about}
                </span>
                <button
                  onClick={() => setEditingAbout(true)}
                  className="text-[var(--text-secondary)] hover:text-[var(--accent)]"
                >
                  <Pencil size={18} />
                </button>
              </div>
            )}
          </div>

          <div className="h-2 bg-[var(--bg-secondary)]" />

          {/* Phone Section */}
          <div className="p-4 bg-[var(--bg-primary)]">
            <label className="text-sm text-[var(--accent)] mb-2 block">
              Phone
            </label>
            <span className="text-[var(--text-primary)] font-mono">
              {currentUser?.phone || "Not set"}
            </span>
          </div>

          <div className="h-2 bg-[var(--bg-secondary)]" />

          {/* User ID Section */}
          <div className="p-4 bg-[var(--bg-primary)]">
            <label className="text-sm text-[var(--accent)] mb-2 block">
              Your Pulse ID
            </label>
            <div className="flex items-center justify-between gap-2">
              <span className="text-[var(--text-primary)] text-sm font-mono truncate">
                {currentUser?.id}
              </span>
              <button
                onClick={handleCopyId}
                className="flex items-center gap-1 px-2 py-1 text-xs text-[var(--accent)] hover:bg-[var(--bg-secondary)] rounded transition-colors"
              >
                <Copy size={14} />
                {copied ? "Copied!" : "Copy"}
              </button>
            </div>
            <p className="text-xs text-[var(--text-secondary)] mt-2">
              Share this ID with others so they can add you as a contact.
            </p>
          </div>
        </div>
      </div>

      <style>{`
        @keyframes slide-in {
          from {
            transform: translateX(-100%);
          }
          to {
            transform: translateX(0);
          }
        }
        .animate-slide-in {
          animation: slide-in 0.2s ease-out;
        }
      `}</style>
    </div>
  );
}
