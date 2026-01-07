import { Check, CheckCheck } from "lucide-react";

import type { MessageStatus as Status } from "../../types";

interface MessageStatusProps {
  status: Status;
  className?: string;
  size?: number;
}

export function MessageStatus({ status, className = "", size = 16 }: MessageStatusProps) {
  const baseClass = "transition-colors duration-200";

  if (status === "sent") {
    return (
      <Check
        size={size}
        strokeWidth={2.5}
        className={`text-[var(--tick-delivered)] ${baseClass} ${className}`}
      />
    );
  }

  if (status === "delivered") {
    return (
      <CheckCheck
        size={size}
        strokeWidth={2.5}
        className={`text-[var(--tick-delivered)] ${baseClass} ${className}`}
      />
    );
  }

  // Read status - blue ticks
  return (
    <CheckCheck
      size={size}
      strokeWidth={2.5}
      className={`text-[var(--tick-read)] ${baseClass} ${className}`}
    />
  );
}
