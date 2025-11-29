import { useState, useRef, useCallback, useEffect } from 'preact/hooks';
import type { ComponentChildren } from 'preact';
import { Drawer } from '@mantine/core';
import { useMediaQuery } from '@mantine/hooks';

/** Breakpoint for mobile responsive behavior */
export const MOBILE_BREAKPOINT = '768px';

interface ResizableSidebarProps {
  children: ComponentChildren;
  minWidth?: number;
  maxWidth?: number;
  defaultWidth?: number;
  mobileOpen?: boolean;
  onMobileClose?: () => void;
}

export function ResizableSidebar({
  children,
  minWidth = 200,
  maxWidth = 400,
  defaultWidth = 240,
  mobileOpen = false,
  onMobileClose,
}: ResizableSidebarProps) {
  const [width, setWidth] = useState(defaultWidth);
  const [isResizing, setIsResizing] = useState(false);
  const sidebarRef = useRef<HTMLDivElement>(null);
  const isMobile = useMediaQuery(`(max-width: ${MOBILE_BREAKPOINT})`);

  const startResizing = useCallback((e: MouseEvent) => {
    e.preventDefault();
    setIsResizing(true);
  }, []);

  const stopResizing = useCallback(() => {
    setIsResizing(false);
  }, []);

  const resize = useCallback(
    (e: MouseEvent) => {
      if (!sidebarRef.current) return;
      
      const sidebarRect = sidebarRef.current.getBoundingClientRect();
      const newWidth = e.clientX - sidebarRect.left;
      
      if (newWidth >= minWidth && newWidth <= maxWidth) {
        setWidth(newWidth);
      }
    },
    [minWidth, maxWidth]
  );

  // Add event listeners for mouse move and mouse up with proper cleanup
  useEffect(() => {
    if (!isResizing) return;

    window.addEventListener('mousemove', resize);
    window.addEventListener('mouseup', stopResizing);

    return () => {
      window.removeEventListener('mousemove', resize);
      window.removeEventListener('mouseup', stopResizing);
    };
  }, [isResizing, resize, stopResizing]);

  // On mobile, render as a drawer
  if (isMobile) {
    return (
      <Drawer
        opened={mobileOpen}
        onClose={onMobileClose || (() => {})}
        position="left"
        size="80%"
        withCloseButton={false}
        styles={{
          body: { padding: 0, height: '100%' },
          content: { backgroundColor: '#2b2d31' },
        }}
      >
        <div className="h-full overflow-hidden">
          {children}
        </div>
      </Drawer>
    );
  }

  // On desktop, render as resizable sidebar
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
