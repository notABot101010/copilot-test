import { useEditor, EditorContent } from '@tiptap/react';
import StarterKit from '@tiptap/starter-kit';
import Placeholder from '@tiptap/extension-placeholder';
import { useEffect, useRef } from 'preact/hooks';
import { Button, ActionIcon, Tooltip } from '@mantine/core';
import {
  activeDocument,
  updateDocument,
  convertToMarkdown,
} from '../store/documents';

interface EditorProps {
  onToggleSidebar: () => void;
}

export function Editor({ onToggleSidebar }: EditorProps) {
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const isLoadingRef = useRef(false);

  const editor = useEditor({
    extensions: [
      StarterKit.configure({
        heading: {
          levels: [1, 2, 3],
        },
      }),
      Placeholder.configure({
        placeholder: 'Start writing...',
      }),
    ],
    content: '',
    onUpdate: ({ editor: ed }) => {
      if (isLoadingRef.current) return;
      
      const doc = activeDocument.value;
      if (!doc) return;

      if (debounceRef.current) {
        clearTimeout(debounceRef.current);
      }

      debounceRef.current = setTimeout(() => {
        updateDocument(doc.id, { content: ed.getHTML() });
      }, 300);
    },
  });

  useEffect(() => {
    if (!editor) return;

    const doc = activeDocument.value;
    if (doc && editor.getHTML() !== doc.content) {
      isLoadingRef.current = true;
      editor.commands.setContent(doc.content || '');
      isLoadingRef.current = false;
    } else if (!doc) {
      isLoadingRef.current = true;
      editor.commands.setContent('');
      isLoadingRef.current = false;
    }
  }, [activeDocument.value?.id, editor]);

  const handleExport = () => {
    const doc = activeDocument.value;
    if (!doc || !editor) return;

    const html = editor.getHTML();
    const markdown = convertToMarkdown(html);
    
    const blob = new Blob([markdown], { type: 'text/markdown' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = `${doc.title}.md`;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    URL.revokeObjectURL(url);
  };

  if (!activeDocument.value) {
    return (
      <div class="flex-1 flex flex-col items-center justify-center bg-zinc-950 text-zinc-500">
        <button
          onClick={onToggleSidebar}
          class="lg:hidden mb-4 p-2 rounded-lg bg-zinc-800 hover:bg-zinc-700 transition-colors"
          data-testid="mobile-menu-button-empty"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="3" y1="12" x2="21" y2="12" />
            <line x1="3" y1="6" x2="21" y2="6" />
            <line x1="3" y1="18" x2="21" y2="18" />
          </svg>
        </button>
        <svg xmlns="http://www.w3.org/2000/svg" width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round" class="mb-4">
          <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
          <polyline points="14 2 14 8 20 8" />
          <line x1="16" y1="13" x2="8" y2="13" />
          <line x1="16" y1="17" x2="8" y2="17" />
          <polyline points="10 9 9 9 8 9" />
        </svg>
        <p class="text-lg">Select or create a document</p>
        <p class="text-sm mt-2">to start writing</p>
      </div>
    );
  }

  return (
    <div class="flex-1 flex flex-col bg-zinc-950 h-full overflow-hidden">
      {/* Toolbar */}
      <div class="flex items-center gap-2 p-2 md:p-3 border-b border-zinc-800 flex-wrap">
        <button
          onClick={onToggleSidebar}
          class="lg:hidden p-2 rounded-lg bg-zinc-800 hover:bg-zinc-700 transition-colors"
          data-testid="mobile-menu-button"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="3" y1="12" x2="21" y2="12" />
            <line x1="3" y1="6" x2="21" y2="6" />
            <line x1="3" y1="18" x2="21" y2="18" />
          </svg>
        </button>

        <div class="hidden md:flex items-center gap-1 border-r border-zinc-700 pr-2">
          <Tooltip label="Heading 1">
            <ActionIcon
              variant={editor?.isActive('heading', { level: 1 }) ? 'filled' : 'subtle'}
              onClick={() => editor?.chain().focus().toggleHeading({ level: 1 }).run()}
              data-testid="heading1-button"
            >
              <span class="text-xs font-bold">H1</span>
            </ActionIcon>
          </Tooltip>
          <Tooltip label="Heading 2">
            <ActionIcon
              variant={editor?.isActive('heading', { level: 2 }) ? 'filled' : 'subtle'}
              onClick={() => editor?.chain().focus().toggleHeading({ level: 2 }).run()}
              data-testid="heading2-button"
            >
              <span class="text-xs font-bold">H2</span>
            </ActionIcon>
          </Tooltip>
          <Tooltip label="Heading 3">
            <ActionIcon
              variant={editor?.isActive('heading', { level: 3 }) ? 'filled' : 'subtle'}
              onClick={() => editor?.chain().focus().toggleHeading({ level: 3 }).run()}
              data-testid="heading3-button"
            >
              <span class="text-xs font-bold">H3</span>
            </ActionIcon>
          </Tooltip>
        </div>

        <div class="flex items-center gap-1 border-r border-zinc-700 pr-2">
          <Tooltip label="Bold">
            <ActionIcon
              variant={editor?.isActive('bold') ? 'filled' : 'subtle'}
              onClick={() => editor?.chain().focus().toggleBold().run()}
              data-testid="bold-button"
            >
              <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M6 4h8a4 4 0 0 1 4 4 4 4 0 0 1-4 4H6z" />
                <path d="M6 12h9a4 4 0 0 1 4 4 4 4 0 0 1-4 4H6z" />
              </svg>
            </ActionIcon>
          </Tooltip>
          <Tooltip label="Italic">
            <ActionIcon
              variant={editor?.isActive('italic') ? 'filled' : 'subtle'}
              onClick={() => editor?.chain().focus().toggleItalic().run()}
              data-testid="italic-button"
            >
              <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <line x1="19" y1="4" x2="10" y2="4" />
                <line x1="14" y1="20" x2="5" y2="20" />
                <line x1="15" y1="4" x2="9" y2="20" />
              </svg>
            </ActionIcon>
          </Tooltip>
          <Tooltip label="Strikethrough">
            <ActionIcon
              variant={editor?.isActive('strike') ? 'filled' : 'subtle'}
              onClick={() => editor?.chain().focus().toggleStrike().run()}
              data-testid="strike-button"
            >
              <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <line x1="5" y1="12" x2="19" y2="12" />
                <path d="M16 6C16 6 14.5 4 12 4C9.5 4 7 6 7 8.5C7 11 9 12 12 12" />
                <path d="M8 18C8 18 9.5 20 12 20C14.5 20 17 18 17 15.5C17 13 15 12 12 12" />
              </svg>
            </ActionIcon>
          </Tooltip>
        </div>

        <div class="hidden md:flex items-center gap-1 border-r border-zinc-700 pr-2">
          <Tooltip label="Bullet List">
            <ActionIcon
              variant={editor?.isActive('bulletList') ? 'filled' : 'subtle'}
              onClick={() => editor?.chain().focus().toggleBulletList().run()}
              data-testid="bullet-list-button"
            >
              <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <line x1="8" y1="6" x2="21" y2="6" />
                <line x1="8" y1="12" x2="21" y2="12" />
                <line x1="8" y1="18" x2="21" y2="18" />
                <line x1="3" y1="6" x2="3.01" y2="6" />
                <line x1="3" y1="12" x2="3.01" y2="12" />
                <line x1="3" y1="18" x2="3.01" y2="18" />
              </svg>
            </ActionIcon>
          </Tooltip>
          <Tooltip label="Numbered List">
            <ActionIcon
              variant={editor?.isActive('orderedList') ? 'filled' : 'subtle'}
              onClick={() => editor?.chain().focus().toggleOrderedList().run()}
              data-testid="ordered-list-button"
            >
              <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <line x1="10" y1="6" x2="21" y2="6" />
                <line x1="10" y1="12" x2="21" y2="12" />
                <line x1="10" y1="18" x2="21" y2="18" />
                <path d="M4 6h1v4" />
                <path d="M4 10h2" />
                <path d="M6 18H4c0-1 2-2 2-3s-1-1.5-2-1" />
              </svg>
            </ActionIcon>
          </Tooltip>
          <Tooltip label="Blockquote">
            <ActionIcon
              variant={editor?.isActive('blockquote') ? 'filled' : 'subtle'}
              onClick={() => editor?.chain().focus().toggleBlockquote().run()}
              data-testid="blockquote-button"
            >
              <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M3 21c3 0 7-1 7-8V5c0-1.25-.756-2.017-2-2H4c-1.25 0-2 .75-2 1.972V11c0 1.25.75 2 2 2 1 0 1 0 1 1v1c0 1-1 2-2 2s-1 .008-1 1.031V21z" />
                <path d="M15 21c3 0 7-1 7-8V5c0-1.25-.757-2.017-2-2h-4c-1.25 0-2 .75-2 1.972V11c0 1.25.75 2 2 2h.75c0 2.25.25 4-2.75 4v3z" />
              </svg>
            </ActionIcon>
          </Tooltip>
        </div>

        <div class="hidden md:flex items-center gap-1 border-r border-zinc-700 pr-2">
          <Tooltip label="Code">
            <ActionIcon
              variant={editor?.isActive('code') ? 'filled' : 'subtle'}
              onClick={() => editor?.chain().focus().toggleCode().run()}
              data-testid="code-button"
            >
              <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <polyline points="16 18 22 12 16 6" />
                <polyline points="8 6 2 12 8 18" />
              </svg>
            </ActionIcon>
          </Tooltip>
          <Tooltip label="Code Block">
            <ActionIcon
              variant={editor?.isActive('codeBlock') ? 'filled' : 'subtle'}
              onClick={() => editor?.chain().focus().toggleCodeBlock().run()}
              data-testid="code-block-button"
            >
              <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
                <polyline points="8 8 4 12 8 16" />
                <polyline points="16 8 20 12 16 16" />
              </svg>
            </ActionIcon>
          </Tooltip>
        </div>

        <div class="flex-1" />

        <h2 class="hidden md:block text-zinc-300 font-medium truncate max-w-xs" data-testid="document-title">
          {activeDocument.value.title}
        </h2>

        <div class="flex-1" />

        <Button
          variant="light"
          onClick={handleExport}
          data-testid="export-button"
          size="sm"
        >
          Export
        </Button>
      </div>

      {/* Mobile document title */}
      <div class="md:hidden px-3 py-2 border-b border-zinc-800">
        <h2 class="text-zinc-300 font-medium truncate" data-testid="document-title-mobile">
          {activeDocument.value.title}
        </h2>
      </div>

      {/* Editor area */}
      <div class="flex-1 overflow-y-auto p-4 md:p-8">
        <div class="max-w-3xl mx-auto">
          <EditorContent
            editor={editor}
            class="prose prose-invert prose-zinc max-w-none min-h-[calc(100vh-200px)]"
            data-testid="editor-content"
          />
        </div>
      </div>
    </div>
  );
}
