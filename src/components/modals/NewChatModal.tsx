import { ArrowLeft, Search, UserPlus } from "lucide-react";
import { useEffect, useState } from "react";

import { userService } from "../../services";
import { useChatStore } from "../../store/chatStore";
import { useUIStore } from "../../store/uiStore";
import type { User } from "../../types";
import { Avatar } from "../common/Avatar";

export function NewChatModal() {
  const [contacts, setContacts] = useState<User[]>([]);
  const [search, setSearch] = useState("");
  const [loading, setLoading] = useState(true);
  const [showAddContact, setShowAddContact] = useState(false);
  const [newContactId, setNewContactId] = useState("");
  const [newContactName, setNewContactName] = useState("");
  const [addingContact, setAddingContact] = useState(false);
  const [error, setError] = useState("");
  const { setShowNewChat } = useUIStore();
  const { createChat, setActiveChat } = useChatStore();

  useEffect(() => {
    loadContacts();
  }, []);

  const loadContacts = async () => {
    try {
      const data = await userService.getContacts();
      setContacts(data);
    } catch (error) {
      console.error("Failed to load contacts:", error);
    } finally {
      setLoading(false);
    }
  };

  const handleAddContact = async () => {
    if (!newContactId.trim() || !newContactName.trim()) {
      setError("Please fill in both ID and name");
      return;
    }

    setAddingContact(true);
    setError("");

    try {
      await userService.addContact(
        newContactId.trim(),
        newContactName.trim()
      );
      await loadContacts();
      setShowAddContact(false);
      setNewContactId("");
      setNewContactName("");
    } catch (err) {
      setError(String(err));
    } finally {
      setAddingContact(false);
    }
  };

  const handleSelectContact = async (contact: User) => {
    try {
      const chat = await createChat(contact.id);
      setActiveChat(chat);
      setShowNewChat(false);
    } catch (error) {
      console.error("Failed to create chat:", error);
    }
  };

  const filteredContacts = search
    ? contacts.filter((c) =>
      c.name.toLowerCase().includes(search.toLowerCase())
    )
    : contacts;

  return (
    <div className="fixed inset-0 z-50 flex">
      {/* Backdrop */}
      <div
        className="absolute inset-0 bg-black/50"
        onClick={() => setShowNewChat(false)}
      />

      {/* Modal Panel */}
      <div className="relative w-[350px] h-full bg-[var(--bg-primary)] flex flex-col shadow-2xl animate-slide-in">
        {/* Header */}
        <header className="flex items-center gap-6 px-4 py-4 bg-[var(--bg-secondary)]">
          <button
            onClick={() => setShowNewChat(false)}
            className="text-[var(--text-primary)] hover:text-[var(--text-secondary)] transition-colors"
          >
            <ArrowLeft size={24} />
          </button>
          <h2 className="text-lg font-medium text-[var(--text-primary)]">
            New chat
          </h2>
        </header>

        {/* Search */}
        <div className="px-3 py-2 bg-[var(--bg-primary)]">
          <div className="flex items-center gap-3 bg-[var(--bg-secondary)] rounded-lg px-3 py-2">
            <Search size={18} className="text-[var(--text-secondary)]" />
            <input
              type="text"
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              placeholder="Search contacts"
              className="flex-1 bg-transparent text-sm text-[var(--text-primary)] placeholder-[var(--text-secondary)] outline-none"
              autoFocus
            />
          </div>
        </div>

        {/* Add Contact Form */}
        {showAddContact && (
          <div className="px-4 py-3 bg-[var(--bg-secondary)] border-b border-[var(--border)]">
            <div className="space-y-3">
              <input
                type="text"
                value={newContactId}
                onChange={(e) => setNewContactId(e.target.value)}
                placeholder="Contact ID (e.g., user123)"
                className="w-full px-3 py-2 bg-[var(--bg-primary)] text-[var(--text-primary)] placeholder-[var(--text-secondary)] rounded-lg outline-none text-sm font-mono"
                autoFocus
              />
              <input
                type="text"
                value={newContactName}
                onChange={(e) => setNewContactName(e.target.value)}
                placeholder="Contact Name"
                className="w-full px-3 py-2 bg-[var(--bg-primary)] text-[var(--text-primary)] placeholder-[var(--text-secondary)] rounded-lg outline-none text-sm"
              />
              {error && (
                <p className="text-xs text-red-500">{error}</p>
              )}
              <div className="flex gap-2">
                <button
                  onClick={() => setShowAddContact(false)}
                  className="flex-1 px-3 py-2 text-sm text-[var(--text-secondary)] hover:bg-[var(--bg-primary)] rounded-lg transition-colors"
                >
                  Cancel
                </button>
                <button
                  onClick={handleAddContact}
                  disabled={addingContact}
                  className="flex-1 px-3 py-2 text-sm bg-[var(--accent)] text-white rounded-lg hover:opacity-90 transition-opacity disabled:opacity-50"
                >
                  {addingContact ? "Adding..." : "Add Contact"}
                </button>
              </div>
            </div>
          </div>
        )}

        {/* Contacts List */}
        <div className="flex-1 overflow-y-auto">
          {/* Add Contact Button */}
          {!showAddContact && (
            <button
              onClick={() => setShowAddContact(true)}
              className="w-full flex items-center gap-3 px-4 py-3 hover:bg-[var(--bg-secondary)] transition-colors border-b border-[var(--border)]"
            >
              <div className="w-[49px] h-[49px] rounded-full bg-[var(--accent)] flex items-center justify-center">
                <UserPlus size={24} className="text-white" />
              </div>
              <div className="flex-1 text-left">
                <p className="font-medium text-[var(--text-primary)]">
                  Add new contact
                </p>
                <p className="text-sm text-[var(--text-secondary)]">
                  Add a contact by their ID
                </p>
              </div>
            </button>
          )}

          {loading ? (
            <div className="flex items-center justify-center h-32 text-[var(--text-secondary)]">
              Loading contacts...
            </div>
          ) : filteredContacts.length === 0 ? (
            <div className="flex flex-col items-center justify-center h-32 text-[var(--text-secondary)]">
              <p>No contacts yet</p>
              <p className="mt-1 text-xs">Add a contact to start chatting</p>
            </div>
          ) : (
            <>
              <div className="px-4 py-3 text-[var(--accent)] text-sm font-medium">
                CONTACTS ON PULSE
              </div>
              {filteredContacts.map((contact) => (
                <button
                  key={contact.id}
                  onClick={() => handleSelectContact(contact)}
                  className="w-full flex items-center gap-3 px-4 py-3 hover:bg-[var(--bg-secondary)] transition-colors"
                >
                  <div className="relative">
                    <Avatar src={contact.avatar_url} name={contact.name} size={49} />
                    {contact.is_online && (
                      <span className="absolute bottom-0 right-0 w-3 h-3 bg-[var(--accent)] border-2 border-[var(--bg-primary)] rounded-full" />
                    )}
                  </div>
                  <div className="flex-1 text-left">
                    <p className="font-medium text-[var(--text-primary)]">
                      {contact.name}
                    </p>
                    <p className="text-sm text-[var(--text-secondary)] truncate">
                      {contact.about}
                    </p>
                  </div>
                </button>
              ))}
            </>
          )}
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
