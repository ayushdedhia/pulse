import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

export interface EncryptedMessage {
  ciphertext: number[];
  nonce: number[];
  sender_public_key: number[];
}

export function useCrypto() {
  const [publicKey, setPublicKey] = useState<string | null>(null);
  const [isInitialized, setIsInitialized] = useState(false);

  // Generate or retrieve public key on mount
  useEffect(() => {
    const initCrypto = async () => {
      try {
        const key = await invoke<string>("get_public_key");
        setPublicKey(key);
        setIsInitialized(true);
      } catch (error) {
        console.error("Failed to initialize crypto:", error);
        // Try generating new keys
        try {
          const newKey = await invoke<string>("generate_keys");
          setPublicKey(newKey);
          setIsInitialized(true);
        } catch (genError) {
          console.error("Failed to generate keys:", genError);
        }
      }
    };

    initCrypto();
  }, []);

  // Initialize a chat session with another user's public key
  const initSession = useCallback(
    async (theirPublicKey: string, chatId: string): Promise<boolean> => {
      try {
        await invoke("init_chat_session", {
          theirPublicKey,
          chatId,
        });
        return true;
      } catch (error) {
        console.error("Failed to init session:", error);
        return false;
      }
    },
    []
  );

  // Encrypt a message
  const encrypt = useCallback(
    async (plaintext: string, chatId: string): Promise<string | null> => {
      try {
        const encrypted = await invoke<string>("encrypt_message", {
          plaintext,
          chatId,
        });
        return encrypted;
      } catch (error) {
        console.error("Failed to encrypt:", error);
        return null;
      }
    },
    []
  );

  // Decrypt a message
  const decrypt = useCallback(
    async (encryptedJson: string, chatId: string): Promise<string | null> => {
      try {
        const plaintext = await invoke<string>("decrypt_message", {
          encryptedJson,
          chatId,
        });
        return plaintext;
      } catch (error) {
        console.error("Failed to decrypt:", error);
        return null;
      }
    },
    []
  );

  return {
    publicKey,
    isInitialized,
    initSession,
    encrypt,
    decrypt,
  };
}
