import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { Theme } from "../types";

interface UIStore {
  theme: Theme;
  showEmojiPicker: boolean;
  showNewChat: boolean;
  showProfile: boolean;
  showContactInfo: boolean;
  searchQuery: string;

  toggleTheme: () => void;
  setTheme: (theme: Theme) => void;
  setShowEmojiPicker: (show: boolean) => void;
  setShowNewChat: (show: boolean) => void;
  setShowProfile: (show: boolean) => void;
  setShowContactInfo: (show: boolean) => void;
  setSearchQuery: (query: string) => void;
}

export const useUIStore = create<UIStore>()(
  persist(
    (set) => ({
      theme: "dark",
      showEmojiPicker: false,
      showNewChat: false,
      showProfile: false,
      showContactInfo: false,
      searchQuery: "",

      toggleTheme: () =>
        set((state) => ({ theme: state.theme === "dark" ? "light" : "dark" })),

      setTheme: (theme) => set({ theme }),

      setShowEmojiPicker: (show) => set({ showEmojiPicker: show }),

      setShowNewChat: (show) => set({ showNewChat: show }),

      setShowProfile: (show) => set({ showProfile: show }),

      setShowContactInfo: (show) => set({ showContactInfo: show }),

      setSearchQuery: (query) => set({ searchQuery: query }),
    }),
    {
      name: "pulse-ui-storage",
      partialize: (state) => ({ theme: state.theme }),
    },
  ),
);
