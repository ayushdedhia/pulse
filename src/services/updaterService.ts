import { check, Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

export interface UpdateInfo {
  version: string;
  currentVersion: string;
  body: string | undefined;
  date: string | undefined;
}

export interface DownloadProgress {
  downloaded: number;
  total: number | undefined;
}

export const updaterService = {
  /**
   * Check for available updates
   * Returns update info if available, null otherwise
   */
  checkForUpdate: async (): Promise<UpdateInfo | null> => {
    try {
      const update = await check();
      if (update) {
        return {
          version: update.version,
          currentVersion: update.currentVersion,
          body: update.body,
          date: update.date,
        };
      }
      return null;
    } catch (error) {
      console.error("Failed to check for updates:", error);
      return null;
    }
  },

  /**
   * Download and install the update, then relaunch the app
   */
  downloadAndInstall: async (
    onProgress?: (progress: DownloadProgress) => void
  ): Promise<void> => {
    const update = await check();
    if (!update) {
      throw new Error("No update available");
    }

    let downloaded = 0;

    await update.downloadAndInstall((event) => {
      if (event.event === "Started") {
        const contentLength = event.data.contentLength;
        console.log(`Download started, total size: ${contentLength ?? "unknown"}`);
      } else if (event.event === "Progress") {
        downloaded += event.data.chunkLength;
        onProgress?.({
          downloaded,
          total: undefined,
        });
      } else if (event.event === "Finished") {
        console.log("Download finished");
      }
    });

    // Relaunch the app to apply the update
    await relaunch();
  },

  /**
   * Store a cached reference to the update object for later installation
   */
  _cachedUpdate: null as Update | null,

  /**
   * Check and cache the update for later installation
   */
  checkAndCache: async (): Promise<UpdateInfo | null> => {
    try {
      const update = await check();
      if (update) {
        updaterService._cachedUpdate = update;
        return {
          version: update.version,
          currentVersion: update.currentVersion,
          body: update.body,
          date: update.date,
        };
      }
      return null;
    } catch (error) {
      console.error("Failed to check for updates:", error);
      return null;
    }
  },

  /**
   * Install a previously cached update
   */
  installCached: async (
    onProgress?: (progress: DownloadProgress) => void
  ): Promise<void> => {
    const update = updaterService._cachedUpdate;
    if (!update) {
      throw new Error("No cached update available");
    }

    let downloaded = 0;

    await update.downloadAndInstall((event) => {
      if (event.event === "Progress") {
        downloaded += event.data.chunkLength;
        onProgress?.({
          downloaded,
          total: undefined,
        });
      }
    });

    await relaunch();
  },
};
