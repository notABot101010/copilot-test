import { useEffect } from 'preact/hooks';
import { Button, TextInput } from '@mantine/core';
import {
  documentListItems,
  isLoading,
  errorMessage,
  loadDocuments,
  createNewDocument,
  selectDocument,
} from '../store';
import { signal } from '@preact/signals';

const newDocTitle = signal('');

export function DocumentList() {
  useEffect(() => {
    loadDocuments();
  }, []);

  const handleCreateDocument = async () => {
    try {
      const id = await createNewDocument(newDocTitle.value || 'Untitled');
      newDocTitle.value = '';
      selectDocument(id);
    } catch (err) {
      console.error('Failed to create document:', err);
    }
  };

  const handleSelectDocument = async (id: string) => {
    const { loadDocument } = await import('../store');
    await loadDocument(id);
  };

  return (
    <div class="min-h-screen bg-gray-50 p-6">
      <div class="max-w-4xl mx-auto">
        <h1 class="text-3xl font-bold mb-8 text-gray-900">E2EE CRDT Documents</h1>

        <div class="bg-white rounded-lg shadow p-6 mb-6">
          <h2 class="text-xl font-semibold mb-4 text-gray-800">Create New Document</h2>
          <div class="flex gap-3">
            <TextInput
              placeholder="Document title"
              value={newDocTitle.value}
              onChange={(e) => (newDocTitle.value = e.currentTarget.value)}
              onKeyPress={(e) => {
                if (e.key === 'Enter') {
                  handleCreateDocument();
                }
              }}
              class="flex-1"
            />
            <Button onClick={handleCreateDocument} disabled={isLoading.value}>
              Create
            </Button>
          </div>
        </div>

        {errorMessage.value && (
          <div class="bg-red-50 border border-red-200 rounded-lg p-4 mb-6">
            <p class="text-red-800">{errorMessage.value}</p>
          </div>
        )}

        <div class="bg-white rounded-lg shadow">
          <div class="p-6 border-b border-gray-200">
            <h2 class="text-xl font-semibold text-gray-800">Documents</h2>
          </div>

          {isLoading.value ? (
            <div class="p-6 text-center text-gray-500">Loading...</div>
          ) : documentListItems.value.length === 0 ? (
            <div class="p-6 text-center text-gray-500">
              No documents yet. Create one to get started!
            </div>
          ) : (
            <div class="divide-y divide-gray-200">
              {documentListItems.value.map((doc) => (
                <div
                  key={doc.id}
                  class="p-4 hover:bg-gray-50 cursor-pointer transition-colors"
                  onClick={() => handleSelectDocument(doc.id)}
                >
                  <h3 class="font-medium text-gray-900">{doc.title}</h3>
                  <p class="text-sm text-gray-500 mt-1">
                    Created {new Date(doc.created_at * 1000).toLocaleDateString()}
                  </p>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
