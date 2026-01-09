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

  uploadAvatar: (sourcePath: string): Promise<string> => {
    return invoke<string>("upload_avatar", { sourcePath });
  },

  savePeerAvatar: (userId: string, avatarData: string): Promise<string> => {
    return invoke<string>("save_peer_avatar", { userId, avatarData });
  },

  /** Save a contact with a custom display name (alias) */
  saveContact: (userId: string, displayName: string): Promise<User> => {
    return invoke<User>("save_contact", { userId, displayName });
  },
};
