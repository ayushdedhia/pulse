import { useEffect } from "react";

import { AppLayout } from "./components/layout/AppLayout";
import { WebSocketProvider } from "./context/WebSocketContext";
import { useCrypto } from "./hooks/useCrypto";
import { useChatStore } from "./store/chatStore";
import { useUIStore } from "./store/uiStore";

function App() {
  const theme = useUIStore((state) => state.theme);
  const { isInitialized, isNewIdentity } = useCrypto();

  useEffect(() => {
    useChatStore.getState().loadChats();
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

  return (
    <WebSocketProvider>
      <AppLayout />
    </WebSocketProvider>
  );
}

export default App;
