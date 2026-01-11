import { useEffect, useState } from "react";

import { AppLayout } from "./components/layout/AppLayout";
import { UpdateModal } from "./components/modals/UpdateModal";
import { WebSocketProvider } from "./context/WebSocketContext";
import { useCrypto } from "./hooks/useCrypto";
import { updaterService, type UpdateInfo } from "./services";
import { useChatStore } from "./store/chatStore";
import { useUIStore } from "./store/uiStore";

const SKIPPED_VERSION_KEY = "pulse_skipped_version";

function App() {
  const theme = useUIStore((state) => state.theme);
  const { isInitialized, isNewIdentity } = useCrypto();
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
  const [showUpdateModal, setShowUpdateModal] = useState(false);

  useEffect(() => {
    useChatStore.getState().loadChats();
  }, []);

  // Check for updates on startup
  useEffect(() => {
    const checkForUpdates = async () => {
      try {
        const update = await updaterService.checkAndCache();
        if (update) {
          const skippedVersion = localStorage.getItem(SKIPPED_VERSION_KEY);
          if (skippedVersion !== update.version) {
            setUpdateInfo(update);
            setShowUpdateModal(true);
          }
        }
      } catch (err) {
        console.error("Failed to check for updates:", err);
      }
    };

    // Delay check to let the app initialize first
    const timer = setTimeout(checkForUpdates, 3000);
    return () => clearTimeout(timer);
  }, []);

  // Crypto initialization - no longer logging sensitive key info
  useEffect(() => {
    if (isInitialized && isNewIdentity) {
      // Only log on first-time key generation, without exposing key material
      console.log("Identity keys initialized");
    }
  }, [isInitialized, isNewIdentity]);

  useEffect(() => {
    document.documentElement.classList.toggle("light", theme === "light");
  }, [theme]);

  const handleSkipVersion = (version: string) => {
    localStorage.setItem(SKIPPED_VERSION_KEY, version);
  };

  return (
    <WebSocketProvider>
      <AppLayout />
      {showUpdateModal && updateInfo && (
        <UpdateModal
          updateInfo={updateInfo}
          onClose={() => setShowUpdateModal(false)}
          onSkipVersion={handleSkipVersion}
        />
      )}
    </WebSocketProvider>
  );
}

export default App;
