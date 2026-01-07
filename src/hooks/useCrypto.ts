import { useCallback, useEffect, useState } from "react";
import { cryptoService, IdentityInfo } from "../services";

export interface EncryptedMessage {
  ciphertext: number[];
  nonce: number[];
  sender_public_key: number[];
}

export function useCrypto() {
  const [publicKey, setPublicKey] = useState<string | null>(null);
  const [userId, setUserId] = useState<string | null>(null);
  const [isInitialized, setIsInitialized] = useState(false);
  const [isNewIdentity, setIsNewIdentity] = useState(false);

  // Initialize identity on mount (loads from persistent storage)
  useEffect(() => {
    const initCrypto = async () => {
      try {
        // Use new persistent identity initialization
        const identity: IdentityInfo = await cryptoService.initIdentity();
        setPublicKey(identity.public_key_hex);
        setUserId(identity.user_id);
        setIsNewIdentity(identity.is_new);
        setIsInitialized(true);

        if (identity.is_new) {
          console.log("Generated new identity keys (stored in OS keyring)");
        } else {
          console.log("Loaded existing identity keys from storage");
        }
      } catch (error) {
        console.error("Failed to initialize crypto identity:", error);
        // Fallback to legacy key generation
        try {
          const key = await cryptoService.generateKeys();
          setPublicKey(key);
          setIsInitialized(true);
          console.log("Fallback: Generated in-memory keys");
        } catch (genError) {
          console.error("Failed to generate keys:", genError);
        }
      }
    };

    initCrypto();
  }, []);

  // Store a peer's public key for future sessions
  const storePeerKey = useCallback(
    async (peerUserId: string, publicKeyHex: string): Promise<boolean> => {
      try {
        await cryptoService.storePeerKey(peerUserId, publicKeyHex);
        return true;
      } catch (error) {
        console.error("Failed to store peer key:", error);
        return false;
      }
    },
    []
  );

  // Get a peer's stored public key
  const getPeerKey = useCallback(
    async (peerUserId: string): Promise<string | null> => {
      try {
        return await cryptoService.getPeerKey(peerUserId);
      } catch (error) {
        console.error("Failed to get peer key:", error);
        return null;
      }
    },
    []
  );

  // Ensure session exists for a chat (auto-derives if possible)
  const ensureSession = useCallback(
    async (peerUserId: string, chatId: string): Promise<boolean> => {
      try {
        return await cryptoService.ensureChatSession(peerUserId, chatId);
      } catch (error) {
        console.error("Failed to ensure session:", error);
        return false;
      }
    },
    []
  );

  // Initialize a chat session with another user's public key (legacy)
  const initSession = useCallback(
    async (theirPublicKey: string, chatId: string): Promise<boolean> => {
      try {
        await cryptoService.initChatSession(theirPublicKey, chatId);
        return true;
      } catch (error) {
        console.error("Failed to init session:", error);
        return false;
      }
    },
    []
  );

  // Check if a session exists
  const hasSession = useCallback(async (chatId: string): Promise<boolean> => {
    try {
      return await cryptoService.hasSession(chatId);
    } catch (error) {
      return false;
    }
  }, []);

  // Encrypt a message
  const encrypt = useCallback(
    async (plaintext: string, chatId: string): Promise<string | null> => {
      try {
        return await cryptoService.encryptMessage(plaintext, chatId);
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
        return await cryptoService.decryptMessage(encryptedJson, chatId);
      } catch (error) {
        console.error("Failed to decrypt:", error);
        return null;
      }
    },
    []
  );

  return {
    publicKey,
    userId,
    isInitialized,
    isNewIdentity,
    storePeerKey,
    getPeerKey,
    ensureSession,
    initSession,
    hasSession,
    encrypt,
    decrypt,
  };
}
