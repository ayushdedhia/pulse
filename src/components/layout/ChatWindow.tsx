import { ChatHeader } from "../chat/ChatHeader";
import { MessageInput } from "../chat/MessageInput";
import { MessageList } from "../chat/MessageList";
import { TypingIndicator } from "../chat/TypingIndicator";

export function ChatWindow() {
  return (
    <div className="flex flex-col h-full">
      <ChatHeader />
      <MessageList />
      <TypingIndicator />
      <MessageInput />
    </div>
  );
}
