import { ExternalLink, Globe } from "lucide-react";
import { open } from "@tauri-apps/plugin-shell";

import { cn } from "../../utils/cn";
import type { UrlPreview } from "../../types";

interface UrlPreviewCardProps {
  preview: UrlPreview;
  isOwn: boolean;
}

export function UrlPreviewCard({ preview, isOwn }: UrlPreviewCardProps) {
  const handleClick = async () => {
    try {
      await open(preview.url);
    } catch (error) {
      console.error("Failed to open URL:", error);
    }
  };

  // Extract domain from URL
  const domain = (() => {
    try {
      return new URL(preview.url).hostname.replace("www.", "");
    } catch {
      return preview.site_name || "Link";
    }
  })();

  const hasImage = !!preview.image_url;
  const hasContent = preview.title || preview.description;

  if (!hasContent && !hasImage) {
    // Minimal preview - just show link
    return (
      <button
        onClick={handleClick}
        className={cn(
          "flex items-center gap-2 mx-[6px] mt-[4px] mb-[2px] px-2 py-1.5 rounded-[5px] text-left transition-colors",
          isOwn
            ? "bg-[rgba(0,0,0,0.1)] hover:bg-[rgba(0,0,0,0.15)]"
            : "bg-[rgba(0,0,0,0.05)] hover:bg-[rgba(0,0,0,0.08)]"
        )}
      >
        <Globe size={14} className={isOwn ? "text-[rgba(255,255,255,0.6)]" : "text-[var(--text-secondary)]"} />
        <span
          className={cn(
            "text-[12px] truncate",
            isOwn ? "text-[rgba(255,255,255,0.8)]" : "text-[var(--accent-primary)]"
          )}
        >
          {domain}
        </span>
        <ExternalLink size={12} className={isOwn ? "text-[rgba(255,255,255,0.5)]" : "text-[var(--text-secondary)]"} />
      </button>
    );
  }

  return (
    <button
      onClick={handleClick}
      className={cn(
        "block mx-[6px] mt-[4px] mb-[2px] rounded-[5px] overflow-hidden text-left transition-colors w-[calc(100%-12px)] max-w-[280px]",
        isOwn
          ? "bg-[rgba(0,0,0,0.1)] hover:bg-[rgba(0,0,0,0.15)]"
          : "bg-[rgba(0,0,0,0.05)] hover:bg-[rgba(0,0,0,0.08)]"
      )}
    >
      {/* Image */}
      {hasImage && (
        <div className="w-full h-[140px] relative bg-[rgba(0,0,0,0.1)]">
          <img
            src={preview.image_url}
            alt=""
            className="w-full h-full object-cover"
            loading="lazy"
            onError={(e) => {
              // Hide image on error
              (e.target as HTMLImageElement).style.display = "none";
            }}
          />
        </div>
      )}

      {/* Content */}
      <div className="p-2">
        {/* Site name */}
        <div className="flex items-center gap-1 mb-1">
          <Globe size={10} className={isOwn ? "text-[rgba(255,255,255,0.5)]" : "text-[var(--text-secondary)]"} />
          <span
            className={cn(
              "text-[10px] uppercase tracking-wide",
              isOwn ? "text-[rgba(255,255,255,0.5)]" : "text-[var(--text-secondary)]"
            )}
          >
            {preview.site_name || domain}
          </span>
        </div>

        {/* Title */}
        {preview.title && (
          <p
            className={cn(
              "text-[13px] font-medium line-clamp-2 leading-tight",
              isOwn ? "text-[rgba(255,255,255,0.95)]" : "text-[var(--text-primary)]"
            )}
          >
            {preview.title}
          </p>
        )}

        {/* Description */}
        {preview.description && (
          <p
            className={cn(
              "text-[11px] line-clamp-2 leading-tight mt-0.5",
              isOwn ? "text-[rgba(255,255,255,0.6)]" : "text-[var(--text-secondary)]"
            )}
          >
            {preview.description}
          </p>
        )}
      </div>
    </button>
  );
}
