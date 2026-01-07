import { useEffect } from "react";

import { AppLayout } from "./components/layout/AppLayout";
import { WebSocketProvider } from "./context/WebSocketContext";
import { useChatStore } from "./store/chatStore";
import { useUIStore } from "./store/uiStore";

function App() {
  const theme = useUIStore((state) => state.theme);

  useEffect(() => {
    useChatStore.getState().loadChats();
  }, []);

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
