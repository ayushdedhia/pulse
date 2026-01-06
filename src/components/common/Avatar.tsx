import { useState } from "react";
import { User } from "lucide-react";

interface AvatarProps {
  src?: string;
  name: string;
  size?: number;
  className?: string;
  showStatus?: boolean;
  isOnline?: boolean;
}

// WhatsApp-style gradient colors for avatars
const gradients = [
  "linear-gradient(135deg, #FF6B6B 0%, #FF8E53 100%)",
  "linear-gradient(135deg, #4ECDC4 0%, #44A08D 100%)",
  "linear-gradient(135deg, #667EEA 0%, #764BA2 100%)",
  "linear-gradient(135deg, #F093FB 0%, #F5576C 100%)",
  "linear-gradient(135deg, #4FACFE 0%, #00F2FE 100%)",
  "linear-gradient(135deg, #43E97B 0%, #38F9D7 100%)",
  "linear-gradient(135deg, #FA709A 0%, #FEE140 100%)",
  "linear-gradient(135deg, #A8EDEA 0%, #FED6E3 100%)",
  "linear-gradient(135deg, #D299C2 0%, #FEF9D7 100%)",
  "linear-gradient(135deg, #89F7FE 0%, #66A6FF 100%)",
];

function getGradient(name: string): string {
  const index = name.split("").reduce((acc, char) => acc + char.charCodeAt(0), 0) % gradients.length;
  return gradients[index];
}

export function Avatar({ src, name, size = 40, className = "" }: AvatarProps) {
  const [imageError, setImageError] = useState(false);

  const initials = name
    .split(" ")
    .map((n) => n[0])
    .filter(Boolean)
    .slice(0, 2)
    .join("")
    .toUpperCase();

  const fontSize = size * 0.38;

  if (src && !imageError) {
    return (
      <img
        src={src}
        alt={name}
        onError={() => setImageError(true)}
        className={`rounded-full object-cover bg-[var(--bg-secondary)] ${className}`}
        style={{ width: size, height: size, minWidth: size, minHeight: size }}
      />
    );
  }

  // Fallback to initials with gradient
  if (initials) {
    return (
      <div
        className={`flex items-center justify-center rounded-full text-white font-semibold select-none ${className}`}
        style={{
          width: size,
          height: size,
          minWidth: size,
          minHeight: size,
          background: getGradient(name),
          fontSize,
          textShadow: "0 1px 2px rgba(0,0,0,0.2)",
        }}
      >
        {initials}
      </div>
    );
  }

  // Default user icon
  return (
    <div
      className={`flex items-center justify-center rounded-full bg-[var(--bg-tertiary)] text-[var(--text-secondary)] ${className}`}
      style={{ width: size, height: size, minWidth: size, minHeight: size }}
    >
      <User size={size * 0.5} />
    </div>
  );
}
