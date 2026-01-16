import { invoke } from "@tauri-apps/api/core";

import type { IceServer } from "../types";

// Fallback STUN servers (used if TURN fetch fails)
const FALLBACK_ICE_SERVERS: IceServer[] = [
  { urls: "stun:stun.l.google.com:19302" },
  { urls: "stun:stun1.l.google.com:19302" },
];

// TURN server cache TTL (12 hours)
const TURN_CACHE_TTL_MS = 12 * 60 * 60 * 1000;

// Cached ICE servers with metadata
interface CachedIceServersMetadata {
  servers: IceServer[];
  expiryTime: number;
}

let cachedIceServersWithMeta: CachedIceServersMetadata | null = null;

export const turnService = {
  /**
   * Fetch ICE servers (STUN + TURN) from backend.
   * Caches the result for subsequent calls with TTL-based expiry.
   */
  getIceServers: async (): Promise<IceServer[]> => {
    const now = Date.now();

    // Return cached servers if still valid
    if (cachedIceServersWithMeta && now < cachedIceServersWithMeta.expiryTime) {
      return cachedIceServersWithMeta.servers;
    }

    try {
      const servers = await invoke<IceServer[]>("get_turn_credentials");
      cachedIceServersWithMeta = {
        servers,
        expiryTime: now + TURN_CACHE_TTL_MS,
      };
      return servers;
    } catch (error) {
      console.warn("Failed to fetch TURN credentials, using fallback:", error);
      return FALLBACK_ICE_SERVERS;
    }
  },

  /**
   * Clear the cached ICE servers and expiry metadata (useful for refreshing credentials)
   */
  clearCache: (): void => {
    cachedIceServersWithMeta = null;
  },
};
