import { create } from "zustand";
import { userService } from "../services";
import type { User } from "../types";

interface UserStore {
  currentUser: User | null;
  isLoading: boolean;
  error: string | null;

  loadCurrentUser: () => Promise<void>;
  updateCurrentUser: (user: User) => Promise<void>;
}

export const useUserStore = create<UserStore>((set) => ({
  currentUser: null,
  isLoading: false,
  error: null,

  loadCurrentUser: async () => {
    set({ isLoading: true, error: null });
    try {
      const currentUser = await userService.getCurrentUser();
      set({ currentUser, isLoading: false });
    } catch (error) {
      set({ error: String(error), isLoading: false });
      console.error("Failed to load current user:", error);
    }
  },

  updateCurrentUser: async (user: User) => {
    try {
      await userService.updateUser(user);
      set({ currentUser: user });
    } catch (error) {
      console.error("Failed to update user:", error);
    }
  },
}));
