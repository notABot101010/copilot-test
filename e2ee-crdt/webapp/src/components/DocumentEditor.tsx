import { Button, TextInput, Textarea } from '@mantine/core';
import { currentDocument, selectDocument, updateDocumentTitle, updateDocumentContent } from '../store';
import { signal } from '@preact/signals';

const hasUnsavedChanges = signal(false);

export function DocumentEditor() {
  const doc = currentDocument.value;

  if (!doc) {
    return (
      <div class="min-h-screen bg-gray-50 flex items-center justify-center">
        <p class="text-gray-500">No document selected</p>
      </div>
    );
  }

  const handleTitleChange = async (newTitle: string) => {
    hasUnsavedChanges.value = true;
    await updateDocumentTitle(doc.id, newTitle);
    hasUnsavedChanges.value = false;
  };

  const handleContentChange = async (newContent: string) => {
    hasUnsavedChanges.value = true;
    await updateDocumentContent(doc.id, newContent);
    hasUnsavedChanges.value = false;
  };

  const handleBack = () => {
    selectDocument(null);
  };

  return (
    <div class="min-h-screen bg-gray-50">
      <div class="bg-white shadow-sm border-b border-gray-200">
        <div class="max-w-4xl mx-auto px-6 py-4">
          <div class="flex items-center justify-between mb-4">
            <Button variant="subtle" onClick={handleBack}>
              ← Back to Documents
            </Button>
            {hasUnsavedChanges.value && (
              <span class="text-sm text-gray-500">Saving...</span>
            )}
          </div>
          <TextInput
            value={doc.doc.title}
            onChange={(e) => handleTitleChange(e.currentTarget.value)}
            placeholder="Document title"
            class="text-2xl font-bold"
            styles={{
              input: {
                fontSize: '1.5rem',
                fontWeight: 'bold',
                border: 'none',
                padding: '0.5rem 0',
              },
            }}
          />
        </div>
      </div>

      <div class="max-w-4xl mx-auto px-6 py-8">
        <Textarea
          value={doc.doc.content}
          onChange={(e) => handleContentChange(e.currentTarget.value)}
          placeholder="Start writing..."
          minRows={20}
          autosize
          styles={{
            input: {
              fontSize: '1rem',
              lineHeight: '1.75',
              border: 'none',
              padding: '1rem',
              backgroundColor: 'white',
              borderRadius: '0.5rem',
              boxShadow: '0 1px 3px 0 rgba(0, 0, 0, 0.1)',
            },
          }}
        />

        <div class="mt-6 p-4 bg-blue-50 rounded-lg border border-blue-200">
          <p class="text-sm text-blue-800">
            <strong>Real-time collaboration enabled:</strong> Changes are encrypted end-to-end
            and synced across all connected clients using CRDT technology.
          </p>
        </div>
      </div>
    </div>
  );
}
