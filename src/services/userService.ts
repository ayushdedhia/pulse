import { invoke } from "@tauri-apps/api/core";
import type { User } from "../types";

export const userService = {
  getCurrentUser: (): Promise<User> => {
    return invoke<User>("get_current_user");
  },

  getUser: (userId: string): Promise<User> => {
    return invoke<User>("get_user", { userId });
  },

  updateUser: (user: User): Promise<boolean> => {
    return invoke<boolean>("update_user", { user });
  },

  getContacts: (): Promise<User[]> => {
    return invoke<User[]>("get_contacts");
  },

  addContact: (
    id: string,
    name: string,
    phone?: string
  ): Promise<User> => {
    return invoke<User>("add_contact", { id, name, phone });
  },
};
