import { useEffect, useRef } from 'preact/hooks';
import { Editor } from '@tiptap/core';
import StarterKit from '@tiptap/starter-kit';
import type { MarkdownContent } from '../types';
import { AutomergeDocumentManager } from '../automerge-manager';

interface DocumentEditorProps {
  documentId: string;
  title: string;
}

export function DocumentEditor({ documentId, title }: DocumentEditorProps) {
  const editorRef = useRef<HTMLDivElement>(null);
  const editorInstanceRef = useRef<Editor | null>(null);
  const docManagerRef = useRef<AutomergeDocumentManager<MarkdownContent> | null>(null);
  const isUpdatingRef = useRef(false);

  useEffect(() => {
    if (!editorRef.current) return;

    const docManager = new AutomergeDocumentManager<MarkdownContent>(documentId);
    docManagerRef.current = docManager;

    const editor = new Editor({
      element: editorRef.current,
      extensions: [StarterKit],
      content: '',
      editorProps: {
        attributes: {
          class: 'prose prose-sm sm:prose lg:prose-lg xl:prose-xl mx-auto focus:outline-none min-h-screen p-4',
        },
      },
      onUpdate: ({ editor }) => {
        if (isUpdatingRef.current || !docManager) return;

        const text = editor.getHTML();
        docManager.change((doc) => {
          doc.text = text;
        });
      },
    });

    editorInstanceRef.current = editor;

    docManager.loadFromServer().then(() => {
      docManager.connectWebSocket();

      const unsubscribe = docManager.content.subscribe((content) => {
        if (content && content.text && editor && !editor.isDestroyed) {
          isUpdatingRef.current = true;
          const currentPos = editor.state.selection.anchor;
          editor.commands.setContent(content.text);

          try {
            editor.commands.setTextSelection(currentPos);
          } catch {
            // Position might be invalid after content change
          }

          isUpdatingRef.current = false;
        }
      });

      return () => {
        unsubscribe();
      };
    });

    return () => {
      editor.destroy();
      docManager.disconnect();
    };
  }, [documentId]);

  return (
    <div class="w-full h-full bg-white dark:bg-gray-900">
      <div class="max-w-5xl mx-auto p-6">
        <h1 class="text-3xl font-bold mb-6 text-gray-900 dark:text-white">
          {title}
        </h1>
        <div
          ref={editorRef}
          class="border border-gray-300 dark:border-gray-700 rounded-lg min-h-[600px] bg-white dark:bg-gray-800"
        />
      </div>
    </div>
  );
}
