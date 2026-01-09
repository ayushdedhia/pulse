import { create } from "zustand";
import { userService, websocketService } from "../services";
import type { User } from "../types";

interface UserStore {
  currentUser: User | null;
  isLoading: boolean;
  error: string | null;

  loadCurrentUser: () => Promise<void>;
  updateCurrentUser: (
    user: User,
    broadcast?: boolean,
    avatarData?: string
  ) => Promise<void>;
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

  updateCurrentUser: async (
    user: User,
    broadcast = false,
    avatarData?: string
  ) => {
    try {
      await userService.updateUser(user);
      set({ currentUser: user });

      if (broadcast) {
        await websocketService.broadcastProfile(
          user.id,
          user.name,
          user.phone,
          user.avatar_url,
          user.about,
          avatarData
        );
      }
    } catch (error) {
      console.error("Failed to update user:", error);
    }
  },
}));
