import { useEffect } from "react";

import { AppLayout } from "./components/layout/AppLayout";
import { WebSocketProvider } from "./context/WebSocketContext";
import { useCrypto } from "./hooks/useCrypto";
import { useChatStore } from "./store/chatStore";
import { useUIStore } from "./store/uiStore";

function App() {
  const theme = useUIStore((state) => state.theme);
  const { isInitialized, isNewIdentity, publicKey } = useCrypto();

  useEffect(() => {
    useChatStore.getState().loadChats();
  }, []);

  // Log crypto initialization status
  useEffect(() => {
    if (isInitialized) {
      console.log(
        isNewIdentity
          ? "🔐 Generated new identity keys (stored in OS keyring)"
          : "🔐 Loaded existing identity keys from storage"
      );
      console.log("📋 Public key:", publicKey?.substring(0, 16) + "...");
    }
  }, [isInitialized, isNewIdentity, publicKey]);

  useEffect(() => {
    document.documentElement.classList.toggle("light", theme === "light");
  }, [theme]);

  return (
    <WebSocketProvider>
      <AppLayout />
    </WebSocketProvider>
  );
}

export default App;
