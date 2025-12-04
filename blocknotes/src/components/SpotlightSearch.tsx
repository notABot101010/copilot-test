import type { MouseEvent as ReactMouseEvent } from 'react';
import { useEffect, useRef, useState, useMemo } from 'react';
import { Paper, Text } from '@mantine/core';
import { pages, setCurrentPage, isSidebarOpen, toggleSidebar } from '../store';

export function SpotlightSearch() {
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const [isOpen, setIsOpen] = useState(false);

  // Memoize filtered pages
  const filteredPages = useMemo(() => 
    pages.value.filter(
      (page) =>
        page.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
        page.blocks.some((block) =>
          block.content?.toLowerCase().includes(searchQuery.toLowerCase())
        )
    ),
    [searchQuery, pages.value]
  );

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // CMD+K or Ctrl+K to open spotlight
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault();
        setIsOpen(prev => {
          if (!prev) {
            setSearchQuery('');
            setSelectedIndex(0);
          }
          return !prev;
        });
      }

      // Escape to close
      if (e.key === 'Escape' && isOpen) {
        e.preventDefault();
        setIsOpen(false);
        setSearchQuery('');
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isOpen]);

  useEffect(() => {
    if (isOpen && inputRef.current) {
      inputRef.current.focus();
    }
  }, [isOpen]);

  useEffect(() => {
    setSelectedIndex(0);
  }, [searchQuery]);

  const handleSelectPage = (pageId: string) => {
    setCurrentPage(pageId);
    setIsOpen(false);
    setSearchQuery('');
    // On mobile, ensure sidebar is closed when navigating
    if (window.innerWidth < 768 && isSidebarOpen.value) {
      toggleSidebar();
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      setSelectedIndex(prev => Math.min(prev + 1, filteredPages.length - 1));
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      setSelectedIndex(prev => Math.max(prev - 1, 0));
    } else if (e.key === 'Enter' && filteredPages[selectedIndex]) {
      e.preventDefault();
      handleSelectPage(filteredPages[selectedIndex].id);
    }
  };

  if (!isOpen) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-start justify-center bg-black/50 pt-24"
      onClick={() => {
        setIsOpen(false);
        setSearchQuery('');
      }}
    >
      <Paper
        shadow="lg"
        className="w-full max-w-xl overflow-hidden border border-zinc-700 bg-zinc-800"
        onClick={(e: ReactMouseEvent<HTMLDivElement>) => e.stopPropagation()}
      >
        <div className="border-b border-zinc-700 p-3">
          <div className="flex items-center gap-3">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              className="h-5 w-5 text-zinc-400"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
              />
            </svg>
            <input
              ref={inputRef}
              type="text"
              placeholder="Search pages..."
              className="flex-1 bg-transparent text-white outline-none placeholder:text-zinc-500"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              onKeyDown={handleKeyDown}
            />
            <span className="rounded bg-zinc-700 px-1.5 py-0.5 text-xs text-zinc-400">ESC</span>
          </div>
        </div>

        <div className="max-h-80 overflow-y-auto p-2">
          {filteredPages.length > 0 ? (
            filteredPages.map((page, index) => (
              <button
                key={page.id}
                className={`flex w-full items-center gap-3 rounded px-3 py-2 text-left transition-colors ${
                  selectedIndex === index
                    ? 'bg-zinc-700 text-white'
                    : 'text-zinc-300 hover:bg-zinc-700/50'
                }`}
                onClick={() => handleSelectPage(page.id)}
              >
                <span className="text-lg">{page.icon || '📄'}</span>
                <div className="min-w-0 flex-1">
                  <Text size="sm" fw={500} className="truncate">
                    {page.title || 'Untitled'}
                  </Text>
                  {searchQuery && page.blocks.some((b) => 
                    b.content?.toLowerCase().includes(searchQuery.toLowerCase())
                  ) && (
                    <Text size="xs" c="dimmed" className="truncate">
                      Found in content
                    </Text>
                  )}
                </div>
              </button>
            ))
          ) : (
            <div className="py-8 text-center">
              <Text size="sm" c="dimmed">
                {searchQuery ? 'No pages found' : 'Start typing to search...'}
              </Text>
            </div>
          )}
        </div>

        <div className="flex items-center gap-4 border-t border-zinc-700 px-3 py-2 text-xs text-zinc-500">
          <span className="flex items-center gap-1">
            <span className="rounded bg-zinc-700 px-1">↑</span>
            <span className="rounded bg-zinc-700 px-1">↓</span>
            to navigate
          </span>
          <span className="flex items-center gap-1">
            <span className="rounded bg-zinc-700 px-1">↵</span>
            to select
          </span>
        </div>
      </Paper>
    </div>
  );
}
