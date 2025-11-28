import { useState, useRef, useCallback } from 'preact/hooks';
import type { ComponentChildren } from 'preact';

interface ResizableSidebarProps {
  children: ComponentChildren;
  minWidth?: number;
  maxWidth?: number;
  defaultWidth?: number;
}

export function ResizableSidebar({
  children,
  minWidth = 200,
  maxWidth = 400,
  defaultWidth = 240,
}: ResizableSidebarProps) {
  const [width, setWidth] = useState(defaultWidth);
  const [isResizing, setIsResizing] = useState(false);
  const sidebarRef = useRef<HTMLDivElement>(null);

  const startResizing = useCallback((e: MouseEvent) => {
    e.preventDefault();
    setIsResizing(true);
  }, []);

  const stopResizing = useCallback(() => {
    setIsResizing(false);
  }, []);

  const resize = useCallback(
    (e: MouseEvent) => {
      if (!isResizing || !sidebarRef.current) return;
      
      const sidebarRect = sidebarRef.current.getBoundingClientRect();
      const newWidth = e.clientX - sidebarRect.left;
      
      if (newWidth >= minWidth && newWidth <= maxWidth) {
        setWidth(newWidth);
      }
    },
    [isResizing, minWidth, maxWidth]
  );

  // Add event listeners for mouse move and mouse up
  if (typeof window !== 'undefined') {
    if (isResizing) {
      window.addEventListener('mousemove', resize);
      window.addEventListener('mouseup', stopResizing);
    } else {
      window.removeEventListener('mousemove', resize);
      window.removeEventListener('mouseup', stopResizing);
    }
  }

  return (
    <div
      ref={sidebarRef}
      className="relative flex shrink-0"
      style={{ width: `${width}px` }}
    >
      <div className="flex-1 overflow-hidden">
        {children}
      </div>
      {/* Resize handle */}
      <div
        onMouseDown={startResizing}
        className={`absolute right-0 top-0 bottom-0 w-1 cursor-ew-resize hover:bg-[#5865f2] transition-colors ${
          isResizing ? 'bg-[#5865f2]' : 'bg-transparent'
        }`}
      />
    </div>
  );
}
