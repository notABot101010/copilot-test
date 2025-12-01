import { useEffect, useMemo } from 'preact/hooks';
import { useSignal, useSignalEffect } from '@preact/signals';
import { useRouter, useRoute } from '@copilot-test/preact-router';
import { Container, Loader, ActionIcon, TextInput, Group } from '@mantine/core';
import { 
  createDocumentState,
  loadDocument, 
  closeDocument, 
  updateDocumentContent, 
  updateDocumentTitle,

} from '../store/documentStore';

export function DocumentPage() {
  const router = useRouter();
  const route = useRoute();
  const documentId = route.value.params?.id as string;
  
  // Create local document state - this is scoped to this component instance
  const documentState = useMemo(() => createDocumentState(), []);  

  const isEditingTitle = useSignal(false);
  const titleInput = useSignal('');
  const lastSyncedContent = useSignal('');

  useEffect(() => {
    if (documentId) {
      loadDocument(documentId, documentState);
    }

    return () => {
      closeDocument(documentState);
    };
  }, [documentId, documentState]);

  // Update title input when document changes - using useSignalEffect for signal reactivity
  useSignalEffect(() => {
    if (documentState.document.value) {
      titleInput.value = String(documentState.document.value.title);
    }
  });

  const handleGoBack = () => {
    router.push('/');
  };

  const handleTitleSubmit = () => {
    if (titleInput.value.trim()) {
      updateDocumentTitle(titleInput.value.trim(), documentState);
    }
    isEditingTitle.value = false;
  };

  const handleContentChange = (event: Event) => {
    const target = event.target as HTMLTextAreaElement;
    const newContent = target.value;

    // Only update if content actually changed
    if (newContent !== lastSyncedContent.value) {
      lastSyncedContent.value = newContent;
      updateDocumentContent(newContent, documentState);
    }
  };

  if (documentState.isLoading.value) {
    return (
      <Container size="lg" className="py-8">
        <div className="flex justify-center items-center h-64">
          <Loader size="xl" />
        </div>
      </Container>
    );
  }

  if (!documentState.document.value) {
    return (
      <Container size="lg" className="py-8">
        <div className="flex flex-col items-center justify-center h-64">
          <p className="text-lg text-gray-400 mb-4">Document not found</p>
          <button
            onClick={handleGoBack}
            className="text-blue-400 hover:text-blue-300 underline"
          >
            Go back to documents
          </button>
        </div>
      </Container>
    );
  }

  const content = String(documentState.document.value.content || '');

  return (
    <div className="min-h-screen flex flex-col bg-gray-900">
      {/* Header */}
      <div className="border-b border-gray-700 bg-gray-800">
        <Container size="lg" className="py-4">
          <Group justify="space-between" align="center">
            <Group align="center" gap="md">
              <a href="/">
                <ActionIcon
                  variant="subtle"
                  onClick={handleGoBack}
                  className="text-gray-400 hover:text-white"
                >
                  <span className="text-xl">←</span>
                </ActionIcon>
              </a>

              {isEditingTitle.value ? (
                <TextInput
                  value={titleInput.value}
                  onChange={(event: Event) => {
                    titleInput.value = (event.target as HTMLInputElement).value;
                  }}
                  onBlur={handleTitleSubmit}
                  onKeyDown={(event: KeyboardEvent) => {
                    if (event.key === 'Enter') {
                      handleTitleSubmit();
                    } else if (event.key === 'Escape') {
                      titleInput.value = String(documentState.document.value?.title || '');
                      isEditingTitle.value = false;
                    }
                  }}
                  autoFocus
                  className="w-64"
                />
              ) : (
                <h1
                  className="text-xl font-semibold text-white cursor-pointer hover:text-blue-400"
                  onClick={() => isEditingTitle.value = true}
                  title="Click to edit title"
                >
                  {String(documentState.document.value.title)}
                </h1>
              )}
            </Group>

            <div className="flex items-center gap-2">
              <span className="text-sm text-gray-500">
                ID: {documentId.substring(0, 8)}...
              </span>
              <div className="w-2 h-2 rounded-full bg-green-500" title="Connected" />
            </div>
          </Group>
        </Container>
      </div>

      {/* Editor */}
      <Container size="lg" className="flex-1 py-6">
        <div className="bg-gray-800 rounded-lg shadow-lg h-full min-h-[600px] flex flex-col">
          {/* Toolbar */}
          <div className="border-b border-gray-700 px-4 py-2">
            <div className="flex items-center gap-2 text-sm text-gray-400">
              <span className="px-2 py-1 bg-gray-700 rounded">Markdown</span>
              <span className="text-gray-600">|</span>
              <span>Use **bold**, *italic*, # headers, - lists, etc.</span>
            </div>
          </div>

          {/* Content Area */}
          <div className="flex-1 flex">
            {/* Editor */}
            <div className="flex-1 flex flex-col">
              <textarea
                value={content}
                onInput={handleContentChange}
                placeholder="Start writing your document in Markdown..."
                className="flex-1 w-full p-4 bg-transparent text-white font-mono text-sm resize-none focus:outline-none"
                spellcheck={false}
              />
            </div>

            {/* Preview */}
            <div className="flex-1 border-l border-gray-700 p-4 overflow-auto">
              <div className="prose prose-invert max-w-none">
                <MarkdownPreview content={content} />
              </div>
            </div>
          </div>
        </div>
      </Container>
    </div>
  );
}

// Simple Markdown preview component
function MarkdownPreview({ content }: { content: string }) {
  if (!content) {
    return (
      <p className="text-gray-500 italic">Preview will appear here...</p>
    );
  }

  // Simple markdown rendering (basic support)
  const renderMarkdown = (text: string): string => {
    let html = text
      // Escape HTML
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      // Headers
      .replace(/^### (.+)$/gm, '<h3 class="text-lg font-bold mt-4 mb-2">$1</h3>')
      .replace(/^## (.+)$/gm, '<h2 class="text-xl font-bold mt-6 mb-3">$1</h2>')
      .replace(/^# (.+)$/gm, '<h1 class="text-2xl font-bold mt-8 mb-4">$1</h1>')
      // Bold and Italic
      .replace(/\*\*\*(.+?)\*\*\*/g, '<strong><em>$1</em></strong>')
      .replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
      .replace(/\*(.+?)\*/g, '<em>$1</em>')
      .replace(/___(.+?)___/g, '<strong><em>$1</em></strong>')
      .replace(/__(.+?)__/g, '<strong>$1</strong>')
      .replace(/_(.+?)_/g, '<em>$1</em>')
      // Code
      .replace(/`(.+?)`/g, '<code class="bg-gray-700 px-1 rounded">$1</code>')
      // Links - only allow http/https URLs to prevent javascript: XSS
      .replace(/\[(.+?)\]\((https?:\/\/[^\)]+)\)/g, '<a href="$2" class="text-blue-400 hover:underline" target="_blank" rel="noopener noreferrer">$1</a>')
      // Unordered lists
      .replace(/^- (.+)$/gm, '<li class="ml-4">$1</li>')
      .replace(/^\* (.+)$/gm, '<li class="ml-4">$1</li>')
      // Blockquotes
      .replace(/^> (.+)$/gm, '<blockquote class="border-l-4 border-gray-600 pl-4 italic text-gray-400">$1</blockquote>')
      // Horizontal rules
      .replace(/^---$/gm, '<hr class="my-4 border-gray-700" />')
      .replace(/^\*\*\*$/gm, '<hr class="my-4 border-gray-700" />')
      // Line breaks (double newline = paragraph)
      .replace(/\n\n/g, '</p><p class="mb-2">')
      // Single line breaks
      .replace(/\n/g, '<br />');

    // Wrap consecutive list items
    html = html.replace(/(<li[^>]*>.*?<\/li>)+/g, '<ul class="list-disc mb-2">$&</ul>');

    return `<p class="mb-2">${html}</p>`;
  };

  return (
    <div
      className="text-gray-200 leading-relaxed"
      dangerouslySetInnerHTML={{ __html: renderMarkdown(content) }}
    />
  );
}
