import { useCallback } from 'preact/hooks';
import type { JSX } from 'preact';
import { TextInput, ScrollArea, ActionIcon } from '@mantine/core';
import { useCreateBlockNote } from '@blocknote/react';
import { BlockNoteView } from '@blocknote/mantine';
import type { Block, PartialBlock } from '@blocknote/core';
import '@blocknote/mantine/style.css';
import {
  currentPage,
  updatePage,
  isSidebarOpen,
  toggleSidebar,
} from '../store';

// Custom dark theme matching the zinc color scheme
const darkTheme = {
  colors: {
    editor: {
      text: '#ffffff',
      background: 'transparent',
    },
    menu: {
      text: '#ffffff',
      background: '#27272a',
    },
    tooltip: {
      text: '#ffffff',
      background: '#3f3f46',
    },
    hovered: {
      text: '#ffffff',
      background: '#3f3f46',
    },
    selected: {
      text: '#ffffff',
      background: '#52525b',
    },
    disabled: {
      text: '#71717a',
      background: '#27272a',
    },
    shadow: '#00000066',
    border: '#3f3f46',
    sideMenu: '#a1a1aa',
    highlights: {
      gray: { text: '#d4d4d8', background: '#3f3f46' },
      brown: { text: '#fcd34d', background: '#78350f' },
      red: { text: '#fca5a5', background: '#7f1d1d' },
      orange: { text: '#fdba74', background: '#7c2d12' },
      yellow: { text: '#fde047', background: '#713f12' },
      green: { text: '#86efac', background: '#14532d' },
      blue: { text: '#93c5fd', background: '#1e3a8a' },
      purple: { text: '#c4b5fd', background: '#4c1d95' },
      pink: { text: '#f9a8d4', background: '#831843' },
    },
  },
  borderRadius: 6,
  fontFamily: 'system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
};

// Convert old block format to BlockNote format
function convertLegacyBlocks(legacyBlocks: Array<{ type: string; content: string; id?: string }>): PartialBlock[] | undefined {
  if (!legacyBlocks || legacyBlocks.length === 0) return undefined;
  
  return legacyBlocks.map((block) => {
    // Map old block types to BlockNote types
    switch (block.type) {
      case 'heading1':
        return {
          type: 'heading' as const,
          props: { level: 1 },
          content: block.content ? [{ type: 'text' as const, text: block.content.replace(/<[^>]*>/g, ''), styles: {} }] : [],
        };
      case 'heading2':
        return {
          type: 'heading' as const,
          props: { level: 2 },
          content: block.content ? [{ type: 'text' as const, text: block.content.replace(/<[^>]*>/g, ''), styles: {} }] : [],
        };
      case 'heading3':
        return {
          type: 'heading' as const,
          props: { level: 3 },
          content: block.content ? [{ type: 'text' as const, text: block.content.replace(/<[^>]*>/g, ''), styles: {} }] : [],
        };
      case 'bulletList':
        return {
          type: 'bulletListItem' as const,
          content: block.content ? [{ type: 'text' as const, text: block.content.replace(/<[^>]*>/g, ''), styles: {} }] : [],
        };
      case 'numberedList':
        return {
          type: 'numberedListItem' as const,
          content: block.content ? [{ type: 'text' as const, text: block.content.replace(/<[^>]*>/g, ''), styles: {} }] : [],
        };
      case 'todoList':
        return {
          type: 'checkListItem' as const,
          props: { checked: false },
          content: block.content ? [{ type: 'text' as const, text: block.content.replace(/<[^>]*>/g, ''), styles: {} }] : [],
        };
      default:
        return {
          type: 'paragraph' as const,
          content: block.content ? [{ type: 'text' as const, text: block.content.replace(/<[^>]*>/g, ''), styles: {} }] : [],
        };
    }
  });
}

interface BlockNoteEditorWrapperProps {
  pageId: string;
  initialContent?: PartialBlock[];
  onChange: (blocks: Block[]) => void;
}

function BlockNoteEditorWrapper({ pageId, initialContent, onChange }: BlockNoteEditorWrapperProps) {
  const editor = useCreateBlockNote({
    initialContent: initialContent && initialContent.length > 0 ? initialContent : undefined,
  }, [pageId]); // Recreate editor when page changes

  return (
    <BlockNoteView
      editor={editor}
      theme={darkTheme}
      onChange={() => {
        onChange(editor.document);
      }}
    />
  );
}

export function PageEditor() {
  // Get initial content for BlockNote
  const getInitialContent = useCallback(() => {
    const page = currentPage.value;
    if (!page) return undefined;
    
    // Check if we have blocknote content stored
    if (page.blocknoteContent && Array.isArray(page.blocknoteContent) && page.blocknoteContent.length > 0) {
      return page.blocknoteContent as PartialBlock[];
    }
    
    // Convert legacy blocks
    if (page.blocks && page.blocks.length > 0) {
      return convertLegacyBlocks(page.blocks);
    }
    
    return undefined;
  }, [currentPage.value?.id, currentPage.value?.blocks, currentPage.value?.blocknoteContent]);

  const handleEditorChange = useCallback((blocks: Block[]) => {
    if (!currentPage.value) return;
    updatePage(currentPage.value.id, { blocknoteContent: blocks });
  }, []);

  const handleTitleKeyDown = (e: KeyboardEvent) => {
    if (e.key === 'Enter') {
      e.preventDefault();
      // Focus the editor
      const editorElement = document.querySelector('.bn-editor');
      if (editorElement instanceof HTMLElement) {
        editorElement.focus();
      }
    }
  };

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
  const initialContent = getInitialContent();

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
        <div className="page-content mx-auto max-w-3xl px-4 py-8 sm:px-12">
          {/* Icon and Title */}
          <div className="mb-8">
            <button className="mb-4 text-5xl transition-transform hover:scale-110">
              {page.icon || '📄'}
            </button>
            <TextInput
              variant="unstyled"
              placeholder="Untitled"
              value={page.title}
              onChange={(e: JSX.TargetedEvent<HTMLInputElement>) => updatePage(page.id, { title: e.currentTarget.value })}
              onKeyDown={handleTitleKeyDown}
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
          <div className="min-h-[300px]">
            <BlockNoteEditorWrapper
              key={page.id}
              pageId={page.id}
              initialContent={initialContent}
              onChange={handleEditorChange}
            />
          </div>
        </div>
      </ScrollArea>
    </div>
  );
}
