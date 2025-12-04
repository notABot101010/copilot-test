import type { ChangeEvent } from 'react';
import { useRef } from 'react';
import { TextInput, ScrollArea, ActionIcon } from '@mantine/core';
import { BlockNoteEditor } from './BlockNoteEditor';
import {
  currentPage,
  updatePage,
  isSidebarOpen,
  toggleSidebar,
} from '../store';

export function PageEditor() {
  const titleInputRef = useRef<HTMLInputElement | null>(null);

  if (!currentPage.value) {
    return (
      <div className="flex h-screen flex-1 items-center justify-center">
        <div className="text-center">
          <p className="mb-4 text-xl text-zinc-400">No page selected</p>
          <p className="text-sm text-zinc-500">Select a page from the sidebar or create a new one</p>
        </div>
      </div>
    );
  }

  const page = currentPage.value;

  return (
    <div className="flex h-screen flex-1 flex-col overflow-hidden">
      {/* Header */}
      <div className="flex items-center gap-2 border-b border-zinc-700 px-4 py-2">
        <ActionIcon
          variant="subtle"
          color="gray"
          onClick={toggleSidebar}
          aria-label={isSidebarOpen.value ? "Close sidebar" : "Open sidebar"}
        >
          <svg xmlns="http://www.w3.org/2000/svg" className="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
            <path fillRule="evenodd" d="M3 5a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zM3 10a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zM3 15a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1z" clipRule="evenodd" />
          </svg>
        </ActionIcon>
        <div className="flex items-center gap-2">
          <span className="text-lg">{page.icon || '📄'}</span>
          <span className="text-sm text-zinc-300">{page.title || 'Untitled'}</span>
        </div>
      </div>

      {/* Page content */}
      <ScrollArea className="flex-1">
        <div className="page-content mx-auto max-w-3xl px-4 py-8 md:px-12">
          {/* Icon and Title */}
          <div className="mb-8">
            <button className="mb-4 text-5xl transition-transform hover:scale-110">
              {page.icon || '📄'}
            </button>
            <TextInput
              ref={titleInputRef}
              variant="unstyled"
              placeholder="Untitled"
              value={page.title}
              onChange={(e: ChangeEvent<HTMLInputElement>) => updatePage(page.id, { title: e.currentTarget.value })}
              className="text-4xl font-bold"
              styles={{
                input: {
                  fontSize: '2.25rem',
                  fontWeight: 700,
                  padding: 0,
                  height: 'auto',
                },
              }}
            />
          </div>

          {/* BlockNote Editor */}
          <BlockNoteEditor key={page.id} page={page} />
        </div>
      </ScrollArea>
    </div>
  );
}
