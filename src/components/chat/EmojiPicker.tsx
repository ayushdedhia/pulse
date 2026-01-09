import data from "@emoji-mart/data";
import Picker from "@emoji-mart/react";
import { useEffect, useRef } from "react";

import { useUIStore } from "../../store/uiStore";

interface EmojiPickerProps {
  onSelect: (emoji: string) => void;
  onClose: () => void;
}

interface EmojiData {
  native: string;
}

export function EmojiPicker({ onSelect, onClose }: EmojiPickerProps) {
  const pickerRef = useRef<HTMLDivElement>(null);
  const theme = useUIStore((state) => state.theme);

  // Close on click outside
  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (pickerRef.current && !pickerRef.current.contains(e.target as Node)) {
        onClose();
      }
    };

    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [onClose]);

  const handleEmojiSelect = (emoji: EmojiData) => {
    onSelect(emoji.native);
  };

  return (
    <div ref={pickerRef} className="emoji-picker-container">
      <Picker
        data={data}
        onEmojiSelect={handleEmojiSelect}
        theme={theme}
        set="native"
        previewPosition="none"
        skinTonePosition="search"
        navPosition="top"
        perLine={11}
        emojiSize={32}
        emojiButtonSize={44}
        maxFrequentRows={2}
        icons="outline"
        noCountryFlags={true}
      />
    </div>
  );
}
