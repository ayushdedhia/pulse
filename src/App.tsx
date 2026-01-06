import { useEffect } from "react";
import { AppLayout } from "./components/layout/AppLayout";
import { useChatStore } from "./store/chatStore";
import { useUIStore } from "./store/uiStore";
import { WebSocketProvider } from "./context/WebSocketContext";

function App() {
  const { loadChats } = useChatStore();
  const { theme } = useUIStore();

  useEffect(() => {
    loadChats();
  }, [loadChats]);

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
