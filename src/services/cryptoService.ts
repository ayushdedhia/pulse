import { invoke } from "@tauri-apps/api/core";

export interface IdentityInfo {
  user_id: string;
  public_key_hex: string;
  is_new: boolean;
}

export const cryptoService = {
  /**
   * Initialize identity from persistent storage or generate new keys
   * Should be called once during app startup
   */
  initIdentity: (): Promise<IdentityInfo> => {
    return invoke<IdentityInfo>("init_identity");
  },

  /**
   * Store a peer's public key for future sessions
   */
  storePeerKey: (peerUserId: string, publicKeyHex: string): Promise<boolean> => {
    return invoke<boolean>("store_peer_key", {
      peerUserId,
      publicKeyHex,
    });
  },

  /**
   * Get a peer's stored public key
   */
  getPeerKey: (peerUserId: string): Promise<string | null> => {
    return invoke<string | null>("get_peer_key", { peerUserId });
  },

  /**
   * Ensure a session exists for a chat (auto-derives if possible)
   */
  ensureChatSession: (peerUserId: string, chatId: string): Promise<boolean> => {
    return invoke<boolean>("ensure_chat_session", { peerUserId, chatId });
  },

  // Legacy methods (for backward compatibility)

  /**
   * Generate new keys (legacy - prefer initIdentity)
   */
  generateKeys: (): Promise<string> => {
    return invoke<string>("generate_keys");
  },

  /**
   * Get current public key
   */
  getPublicKey: (): Promise<string> => {
    return invoke<string>("get_public_key");
  },

  /**
   * Initialize a chat session with peer's public key (legacy)
   */
  initChatSession: (theirPublicKey: string, chatId: string): Promise<boolean> => {
    return invoke<boolean>("init_chat_session", {
      theirPublicKey,
      chatId,
    });
  },

  /**
   * Check if a session exists for a chat
   */
  hasSession: (chatId: string): Promise<boolean> => {
    return invoke<boolean>("has_chat_session", { chatId });
  },

  /**
   * Encrypt a message for a chat
   */
  encryptMessage: (plaintext: string, chatId: string): Promise<string> => {
    return invoke<string>("encrypt_message", { plaintext, chatId });
  },

  /**
   * Decrypt a message from a chat
   */
  decryptMessage: (encryptedJson: string, chatId: string): Promise<string> => {
    return invoke<string>("decrypt_message", { encryptedJson, chatId });
  },
};
