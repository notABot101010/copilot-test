import { useEffect, useRef } from 'preact/hooks';
import { useEditor, EditorContent } from '@tiptap/react';
import StarterKit from '@tiptap/starter-kit';
import Placeholder from '@tiptap/extension-placeholder';
import Link from '@tiptap/extension-link';
import Image from '@tiptap/extension-image';
import TaskList from '@tiptap/extension-task-list';
import TaskItem from '@tiptap/extension-task-item';
import Table from '@tiptap/extension-table';
import TableRow from '@tiptap/extension-table-row';
import TableCell from '@tiptap/extension-table-cell';
import TableHeader from '@tiptap/extension-table-header';
import { 
  updateBlock, 
  openCommandPalette, 
  currentPage, 
  createBlock, 
  deleteBlock, 
  setFocusedBlock, 
  getPreviousBlock 
} from '../store';
import type { Block } from '../types';

interface RichTextEditorProps {
  block: Block;
  pageId: string;
  onFocus?: () => void;
  autoFocus?: boolean;
}

export function RichTextEditor({ block, pageId, onFocus, autoFocus }: RichTextEditorProps) {
  const editorRef = useRef<HTMLDivElement>(null);

  const editor = useEditor({
    extensions: [
      StarterKit.configure({
        heading: {
          levels: [1, 2, 3],
        },
      }),
      Placeholder.configure({
        placeholder: getPlaceholder(block.type),
      }),
      Link.configure({
        openOnClick: true,
        HTMLAttributes: {
          class: 'text-blue-400 underline cursor-pointer',
        },
      }),
      Image.configure({
        HTMLAttributes: {
          class: 'max-w-full rounded-lg',
        },
      }),
      TaskList,
      TaskItem.configure({
        nested: true,
      }),
      Table.configure({
        resizable: true,
      }),
      TableRow,
      TableHeader,
      TableCell,
    ],
    content: block.content,
    editorProps: {
      attributes: {
        class: getEditorClass(block.type),
      },
      handleKeyDown: (view, event) => {
        // Shift+Enter: insert a line break (new line within block)
        if (event.key === 'Enter' && event.shiftKey) {
          // Insert a hard break
          const { state, dispatch } = view;
          const { schema, tr } = state;
          const hardBreak = schema.nodes.hardBreak;
          if (hardBreak) {
            dispatch(tr.replaceSelectionWith(hardBreak.create()).scrollIntoView());
            return true;
          }
          return false;
        }

        // Enter without Shift: create a new block
        if (event.key === 'Enter' && !event.shiftKey) {
          event.preventDefault();
          const page = currentPage.value;
          if (!page) return true;
          
          const blockIndex = page.blocks.findIndex((b) => b.id === block.id);
          // If block not found, append to end (blockIndex will be -1)
          const newBlock = createBlock(pageId, 'text', blockIndex >= 0 ? blockIndex : undefined);
          setFocusedBlock(newBlock.id);
          return true;
        }

        // Backspace on empty block: delete block and focus previous
        if (event.key === 'Backspace') {
          const { state } = view;
          if (state.doc.textContent === '' || state.doc.content.size <= 2) {
            const page = currentPage.value;
            if (!page || page.blocks.length <= 1) return false;
            
            const prevBlock = getPreviousBlock(pageId, block.id);
            if (prevBlock) {
              event.preventDefault();
              deleteBlock(pageId, block.id);
              setFocusedBlock(prevBlock.id);
              return true;
            }
          }
        }

        return false;
      },
    },
    onUpdate: ({ editor }) => {
      updateBlock(pageId, block.id, { content: editor.getHTML() });
    },
    onFocus: () => {
      onFocus?.();
    },
  });

  useEffect(() => {
    if (editor && autoFocus) {
      editor.commands.focus('end');
    }
  }, [editor, autoFocus]);

  const handleKeyDown = (e: KeyboardEvent) => {
    // Show command palette when "/" is pressed at the start of an empty line or empty block
    if (e.key === '/' && editor) {
      const { state } = editor;
      const { from } = state.selection;
      const textBefore = state.doc.textBetween(Math.max(0, from - 1), from, '\n');
      const isAtLineStart = from === 1 || textBefore === '\n' || textBefore === '';
      
      if (editor.isEmpty || isAtLineStart) {
        e.preventDefault();
        const rect = editorRef.current?.getBoundingClientRect();
        if (rect) {
          const page = currentPage.value;
          const blockIndex = page?.blocks.findIndex((b) => b.id === block.id) ?? -1;
          openCommandPalette(rect.left, rect.bottom + 4, blockIndex);
        }
      }
    }
  };

  useEffect(() => {
    const element = editorRef.current;
    if (element) {
      element.addEventListener('keydown', handleKeyDown);
      return () => element.removeEventListener('keydown', handleKeyDown);
    }
  }, [editor]);

  return (
    <div ref={editorRef} className="rich-text-editor">
      <EditorContent editor={editor} />
    </div>
  );
}

function getPlaceholder(type: string): string {
  switch (type) {
    case 'heading1':
      return 'Heading 1';
    case 'heading2':
      return 'Heading 2';
    case 'heading3':
      return 'Heading 3';
    case 'quote':
      return 'Quote...';
    default:
      return "Type '/' for commands...";
  }
}

function getEditorClass(type: string): string {
  const baseClass = 'focus:outline-none w-full';
  switch (type) {
    case 'heading1':
      return `${baseClass} text-3xl font-bold`;
    case 'heading2':
      return `${baseClass} text-2xl font-semibold`;
    case 'heading3':
      return `${baseClass} text-xl font-medium`;
    case 'quote':
      return `${baseClass} border-l-4 border-zinc-600 pl-4 italic text-zinc-400`;
    default:
      return baseClass;
  }
}
