import { Camera, Contact, FileText, Image, MapPin, Mic, Plus, Send, Smile, X } from "lucide-react";
import { useEffect, useRef, useState } from "react";

import { useWebSocketContext } from "../../context/WebSocketContext";
import { useChatStore } from "../../store/chatStore";
import { useMessageStore } from "../../store/messageStore";
import { EmojiPicker } from "./EmojiPicker";

export function MessageInput() {
  const [message, setMessage] = useState("");
  const [showEmojiPicker, setShowEmojiPicker] = useState(false);
  const [showAttachMenu, setShowAttachMenu] = useState(false);
  const [isAttachMenuClosing, setIsAttachMenuClosing] = useState(false);
  const [isSending, setIsSending] = useState(false);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const typingTimeoutRef = useRef<number>();

  // Handle attachment menu close with animation
  const closeAttachMenu = () => {
    setIsAttachMenuClosing(true);
    setTimeout(() => {
      setShowAttachMenu(false);
      setIsAttachMenuClosing(false);
    }, 150);
  };

  const activeChat = useChatStore((state) => state.activeChat);
  const sendMessage = useMessageStore((state) => state.sendMessage);
  const { sendTyping } = useWebSocketContext();

  // Auto-resize textarea
  useEffect(() => {
    if (textareaRef.current) {
      textareaRef.current.style.height = "auto";
      textareaRef.current.style.height = `${Math.min(textareaRef.current.scrollHeight, 120)}px`;
    }
  }, [message]);

  // Focus input when chat changes
  useEffect(() => {
    textareaRef.current?.focus();
  }, [activeChat?.id]);

  // Send typing indicator
  const handleTyping = () => {
    if (!activeChat) return;

    sendTyping(activeChat.id, true);

    // Clear previous timeout
    if (typingTimeoutRef.current) {
      clearTimeout(typingTimeoutRef.current);
    }

    // Stop typing after 2 seconds of inactivity
    typingTimeoutRef.current = window.setTimeout(() => {
      sendTyping(activeChat.id, false);
    }, 2000);
  };

  const handleSend = async () => {
    if (!message.trim() || !activeChat || isSending) return;

    setIsSending(true);
    try {
      await sendMessage(activeChat.id, message);
      setMessage("");
      if (textareaRef.current) {
        textareaRef.current.style.height = "auto";
      }
    } finally {
      setIsSending(false);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const handleEmojiSelect = (emoji: string) => {
    setMessage((prev) => prev + emoji);
    textareaRef.current?.focus();
  };

  const hasMessage = message.trim().length > 0;

  return (
    <div className="relative bg-[var(--bg-secondary)] px-4 py-[10px] transition-theme">
      {/* Emoji Picker */}
      {showEmojiPicker && (
        <div className="absolute z-50 mb-2 bottom-full left-0 animate-scale-in">
          <EmojiPicker
            onSelect={handleEmojiSelect}
            onClose={() => setShowEmojiPicker(false)}
          />
        </div>
      )}

      {/* Attachment Menu */}
      {showAttachMenu && (
        <>
          {/* Invisible overlay to close on outside click */}
          <div
            className="fixed inset-0 z-40"
            onClick={closeAttachMenu}
          />
          <div className={`absolute z-50 mb-2 bottom-full left-16 transition-all duration-150 ${
            isAttachMenuClosing
              ? "opacity-0 scale-95 translate-y-2"
              : "animate-scale-in"
          }`}>
            <AttachmentMenu onClose={closeAttachMenu} />
          </div>
        </>
      )}

      <div className="flex items-center">
        {/* Emoji button */}
        <button
          onClick={() => {
            setShowEmojiPicker(!showEmojiPicker);
            if (showAttachMenu) closeAttachMenu();
          }}
          className={`
            w-[42px] h-[42px] flex items-center justify-center rounded-full
            transition-all duration-200 flex-shrink-0 active-press
            ${showEmojiPicker
              ? "text-[var(--accent)] bg-[var(--accent)]/10"
              : "text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)]"
            }
          `}
        >
          <Smile size={26} strokeWidth={1.5} />
        </button>

        {/* Attachment button */}
        <button
          onClick={() => {
            setShowAttachMenu(!showAttachMenu);
            setShowEmojiPicker(false);
          }}
          className={`
            flex items-center justify-center rounded-full p-2
            transition-all duration-200 flex-shrink-0 active-press
            ${showAttachMenu
              ? "text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)]"
              : "text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)]"
            }
          `}
        >
          {showAttachMenu ? <X size={26} strokeWidth={1.5} /> : <Plus size={26} strokeWidth={1.5} className="-translate-x-[0.6px]" />}
        </button>

        {/* Input field */}
        <div className="flex-1 flex items-center ml-2 mr-3 bg-[var(--bg-primary)] rounded-[8px] px-4 min-h-[42px] transition-all duration-200 input-focus-ring">
          <textarea
            ref={textareaRef}
            value={message}
            onChange={(e) => {
              setMessage(e.target.value);
              handleTyping();
            }}
            onKeyDown={handleKeyDown}
            placeholder="Type a message"
            rows={1}
            data-testid="message-input"
            className="w-full bg-transparent text-[var(--text-primary)] placeholder-[var(--text-secondary)] outline-none text-[15px] leading-[21px] resize-none max-h-[120px] no-scrollbar py-[10px]"
          />
        </div>

        {/* Send/Mic button */}
        {hasMessage ? (
          <button
            onClick={handleSend}
            disabled={isSending}
            className="w-[42px] h-[42px] flex items-center justify-center rounded-full bg-[var(--accent)] text-white transition-all duration-200 active-press hover:bg-[var(--accent-hover)] disabled:opacity-50 disabled:cursor-not-allowed flex-shrink-0"
          >
            <Send size={20} strokeWidth={2} className="-translate-x-[1.5px]" />
          </button>
        ) : (
          <button
            className="w-[42px] h-[42px] flex items-center justify-center rounded-full text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)] transition-all duration-200 flex-shrink-0"
          >
            <Mic size={26} strokeWidth={1.5} />
          </button>
        )}
      </div>
    </div>
  );
}

interface AttachmentMenuProps {
  onClose: () => void;
}

function AttachmentMenu({ onClose }: AttachmentMenuProps) {
  const items = [
    { icon: <FileText size={24} />, label: "Document", color: "#7C3AED" },
    { icon: <Camera size={24} />, label: "Camera", color: "#EC4899" },
    { icon: <Image size={24} />, label: "Gallery", color: "#C026D3" },
    { icon: <Contact size={24} />, label: "Contact", color: "#0EA5E9" },
    { icon: <MapPin size={24} />, label: "Location", color: "#22C55E" },
  ];

  return (
    <div className="bg-[var(--bg-tertiary)] rounded-2xl shadow-2xl border border-[var(--border-light)] p-4">
      <div className="grid grid-cols-3 gap-4">
        {items.map((item, index) => (
          <button
            key={index}
            onClick={onClose}
            className="flex flex-col items-center gap-2 p-2 rounded-xl hover:bg-[var(--bg-hover)] transition-all duration-200 group"
            style={{ animationDelay: `${index * 50}ms` }}
          >
            <span
              className="flex items-center justify-center w-[52px] h-[52px] text-white rounded-full transition-all duration-200 group-hover:scale-110 group-hover:shadow-lg"
              style={{
                backgroundColor: item.color,
                boxShadow: `0 4px 12px ${item.color}40`
              }}
            >
              {item.icon}
            </span>
            <span className="text-[var(--text-secondary)] text-[11px] font-medium group-hover:text-[var(--text-primary)] transition-colors">
              {item.label}
            </span>
          </button>
        ))}
      </div>
    </div>
  );
}
