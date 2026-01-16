import { invoke } from "@tauri-apps/api/core";

import type { IceServer } from "../types";

// Fallback STUN servers (used if TURN fetch fails)
const FALLBACK_ICE_SERVERS: IceServer[] = [
  { urls: "stun:stun.l.google.com:19302" },
  { urls: "stun:stun1.l.google.com:19302" },
];

// Cached ICE servers
let cachedIceServers: IceServer[] | null = null;

export const turnService = {
  /**
   * Fetch ICE servers (STUN + TURN) from backend.
   * Caches the result for subsequent calls.
   */
  getIceServers: async (): Promise<IceServer[]> => {
    if (cachedIceServers) {
      return cachedIceServers;
    }

    try {
      const servers = await invoke<IceServer[]>("get_turn_credentials");
      cachedIceServers = servers;
      return cachedIceServers;
    } catch (error) {
      console.warn("Failed to fetch TURN credentials, using fallback:", error);
      return FALLBACK_ICE_SERVERS;
    }
  },

  /**
   * Clear the cached ICE servers (useful for refreshing credentials)
   */
  clearCache: (): void => {
    cachedIceServers = null;
  },
};
