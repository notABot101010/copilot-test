import type { ChangeEvent, MouseEvent as ReactMouseEvent } from 'react';
import { useState } from 'react';
import { ActionIcon, ScrollArea, Text, TextInput } from '@mantine/core';
import { 
  pages, 
  currentPageId, 
  isSidebarOpen, 
  createPage, 
  setCurrentPage, 
  deletePage,
  toggleSidebar 
} from '../store';

export function Sidebar() {
  const [searchQuery, setSearchQuery] = useState('');

  const filteredPages = pages.value.filter((page) =>
    page.title.toLowerCase().includes(searchQuery.toLowerCase())
  );

  const handleCreatePage = () => {
    const newPage = createPage();
    setCurrentPage(newPage.id);
  };

  const handleDeletePage = (e: ReactMouseEvent<HTMLButtonElement>, pageId: string) => {
    e.stopPropagation();
    deletePage(pageId);
  };

  const handlePageClick = (pageId: string) => {
    setCurrentPage(pageId);
    // Close sidebar on mobile after selecting a page
    if (window.innerWidth < 768) {
      toggleSidebar();
    }
  };

  if (!isSidebarOpen.value) {
    return null;
  }

  return (
    <>
      {/* Mobile overlay */}
      <div 
        className="fixed inset-0 z-30 bg-black/50 md:hidden"
        onClick={toggleSidebar}
      />
      {/* Sidebar */}
      <div className="fixed left-0 top-0 z-40 flex h-screen w-64 flex-shrink-0 flex-col border-r border-zinc-700 bg-zinc-900 md:static md:z-0">
      <div className="flex items-center justify-between border-b border-zinc-700 px-3 py-2">
        <Text fw={600} size="sm">Notes</Text>
        <ActionIcon
          variant="subtle"
          color="gray"
          onClick={toggleSidebar}
          aria-label="Close sidebar"
        >
          <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
            <path fillRule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clipRule="evenodd" />
          </svg>
        </ActionIcon>
      </div>

      <div className="p-2">
        <TextInput
          placeholder="Search pages..."
          size="xs"
          value={searchQuery}
          onChange={(e: ChangeEvent<HTMLInputElement>) => setSearchQuery(e.currentTarget.value)}
          leftSection={
            <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4 text-zinc-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
            </svg>
          }
        />
      </div>

      <ScrollArea className="flex-1 px-2">
        <div className="space-y-0.5">
          {filteredPages.map((page) => (
            <div
              key={page.id}
              className={`group flex w-full cursor-pointer items-center justify-between rounded px-2 py-1.5 text-sm transition-colors ${
                currentPageId.value === page.id
                  ? 'bg-zinc-700 text-white'
                  : 'text-zinc-300 hover:bg-zinc-800'
              }`}
              onClick={() => handlePageClick(page.id)}
            >
              <div className="flex items-center gap-2 overflow-hidden">
                <span className="flex-shrink-0">{page.icon || '📄'}</span>
                <span className="truncate">{page.title || 'Untitled'}</span>
              </div>
              <ActionIcon
                variant="subtle"
                color="red"
                size="xs"
                className="opacity-0 group-hover:opacity-100"
                onClick={(e: ReactMouseEvent<HTMLButtonElement>) => handleDeletePage(e, page.id)}
                aria-label="Delete page"
              >
                <svg xmlns="http://www.w3.org/2000/svg" className="h-3 w-3" viewBox="0 0 20 20" fill="currentColor">
                  <path fillRule="evenodd" d="M9 2a1 1 0 00-.894.553L7.382 4H4a1 1 0 000 2v10a2 2 0 002 2h8a2 2 0 002-2V6a1 1 0 100-2h-3.382l-.724-1.447A1 1 0 0011 2H9zM7 8a1 1 0 012 0v6a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v6a1 1 0 102 0V8a1 1 0 00-1-1z" clipRule="evenodd" />
                </svg>
              </ActionIcon>
            </div>
          ))}
        </div>
      </ScrollArea>

      <div className="border-t border-zinc-700 p-2">
        <button
          className="flex w-full items-center gap-2 rounded px-2 py-1.5 text-sm text-zinc-400 transition-colors hover:bg-zinc-800 hover:text-white"
          onClick={handleCreatePage}
        >
          <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
          </svg>
          <span>New Page</span>
        </button>
      </div>
      </div>
    </>
  );
}
