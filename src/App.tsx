import { useEffect, useState } from "react";

import { CallErrorModal } from "./components/call/CallErrorModal";
import { CallOverlay } from "./components/call/CallOverlay";
import { DeviceSelectionModal } from "./components/call/DeviceSelectionModal";
import { IncomingCallModal } from "./components/call/IncomingCallModal";
import { AppLayout } from "./components/layout/AppLayout";
import { OnboardingModal } from "./components/modals/OnboardingModal";
import { UpdateModal } from "./components/modals/UpdateModal";
import { WebSocketProvider } from "./context/WebSocketContext";
import { useCrypto } from "./hooks/useCrypto";
import { updaterService, type UpdateInfo } from "./services";
import { useChatStore } from "./store/chatStore";
import { useUserStore } from "./store/userStore";
import { useUIStore } from "./store/uiStore";

const SKIPPED_VERSION_KEY = "pulse_skipped_version";

// Test mode: ?test=onboarding forces the modal to show for e2e testing
const isTestMode =
  new URLSearchParams(window.location.search).get("test") === "onboarding";

function App() {
  const theme = useUIStore((state) => state.theme);
  const { isInitialized, isNewIdentity } = useCrypto();
  const currentUser = useUserStore((state) => state.currentUser);
  const chats = useChatStore((state) => state.chats);
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
  const [showUpdateModal, setShowUpdateModal] = useState(false);
  const [onboardingComplete, setOnboardingComplete] = useState(false);

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

  // Only show onboarding if no chats exist (prevents error when keys regenerated but DB has history)
  // Test mode (?test=onboarding) forces the modal to show for e2e testing
  const showOnboarding =
    !onboardingComplete &&
    (isTestMode ||
      (isInitialized && isNewIdentity && !!currentUser && chats.length === 0));

  return (
    <WebSocketProvider>
      <AppLayout />
      {/* Video Call Components */}
      <IncomingCallModal />
      <DeviceSelectionModal />
      <CallOverlay />
      <CallErrorModal />
      {showOnboarding && (
        <OnboardingModal onComplete={() => setOnboardingComplete(true)} />
      )}
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
