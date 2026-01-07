import { formatDistanceToNow, format, isToday, isYesterday } from "date-fns";

export function formatLastSeen(timestamp?: number): string {
  if (!timestamp) return "Offline";

  const date = new Date(timestamp);
  const now = new Date();
  const diffMinutes = Math.floor((now.getTime() - date.getTime()) / 60000);

  if (diffMinutes < 1) return "last seen just now";
  if (diffMinutes < 60) return `last seen ${diffMinutes} min ago`;

  if (isToday(date)) {
    return `last seen today at ${format(date, "HH:mm")}`;
  }

  if (isYesterday(date)) {
    return `last seen yesterday at ${format(date, "HH:mm")}`;
  }

  return `last seen ${format(date, "MMM d")} at ${format(date, "HH:mm")}`;
}

export function formatMessageTime(timestamp: number): string {
  return format(new Date(timestamp), "HH:mm");
}

export function formatRelativeTime(timestamp: number): string {
  return formatDistanceToNow(new Date(timestamp), { addSuffix: true });
}
