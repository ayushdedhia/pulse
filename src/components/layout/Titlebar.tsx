import { getCurrentWindow } from "@tauri-apps/api/window";
import { Minus, Square, X } from "lucide-react";
import { useEffect, useRef } from "react";

export function Titlebar() {
  const titlebarRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const titlebar = titlebarRef.current;
    if (!titlebar) return;

    const handleMouseDown = async (e: MouseEvent) => {
      // Only handle left mouse button
      if (e.buttons !== 1) return;

      // Don't drag if clicking on buttons or their children
      const target = e.target as HTMLElement;
      if (target.closest("button")) return;

      // Double click to maximize, single click to drag
      if (e.detail === 2) {
        await getCurrentWindow().toggleMaximize();
      } else {
        await getCurrentWindow().startDragging();
      }
    };

    titlebar.addEventListener("mousedown", handleMouseDown);
    return () => titlebar.removeEventListener("mousedown", handleMouseDown);
  }, []);

  const handleMinimize = () => {
    getCurrentWindow().minimize();
  };

  const handleMaximize = () => {
    getCurrentWindow().toggleMaximize();
  };

  const handleClose = () => {
    getCurrentWindow().close();
  };

  return (
    <div
      ref={titlebarRef}
      className="h-10 flex items-center justify-between bg-[var(--bg-titlebar)] select-none"
    >
      {/* App Logo & Title */}
      <div className="flex items-center flex-1 h-full px-4">
        <img src="/logo-with-name.png" alt="Pulse" className="object-contain w-auto h-6" />
      </div>

      {/* Window Controls */}
      <div className="flex h-full">
        <button
          type="button"
          onClick={handleMinimize}
          className="w-12 h-full flex items-center justify-center text-[var(--text-secondary)] hover:bg-[var(--bg-hover)] transition-colors"
        >
          <Minus size={16} />
        </button>
        <button
          type="button"
          onClick={handleMaximize}
          className="w-12 h-full flex items-center justify-center text-[var(--text-secondary)] hover:bg-[var(--bg-hover)] transition-colors"
        >
          <Square size={14} />
        </button>
        <button
          type="button"
          onClick={handleClose}
          className="w-12 h-full flex items-center justify-center text-[var(--text-secondary)] hover:bg-red-500 hover:text-white transition-colors"
        >
          <X size={16} />
        </button>
      </div>
    </div>
  );
}
