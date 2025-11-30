import { useEffect, useState } from 'preact/hooks';
import { Button, TextInput, Modal, Select } from '@mantine/core';
import type { Document } from '../types';
import { listDocuments, createDocument, deleteDocument } from '../api';

interface DocumentListProps {
  path?: string;
}

export function DocumentList(_props: DocumentListProps) {
  const [documents, setDocuments] = useState<Document[]>([]);
  const [loading, setLoading] = useState(true);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [newDocTitle, setNewDocTitle] = useState('');
  const [newDocType, setNewDocType] = useState<'document' | 'presentation'>('document');

  const loadDocuments = async () => {
    try {
      setLoading(true);
      const docs = await listDocuments();
      setDocuments(docs);
    } catch (err) {
      console.error('Failed to load documents:', err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadDocuments();
  }, []);

  const handleCreateDocument = async () => {
    if (!newDocTitle.trim()) return;

    try {
      await createDocument(newDocTitle, newDocType);
      setNewDocTitle('');
      setShowCreateModal(false);
      loadDocuments();
    } catch (err) {
      console.error('Failed to create document:', err);
    }
  };

  const handleDeleteDocument = async (id: string) => {
    if (!confirm('Are you sure you want to delete this document?')) return;

    try {
      await deleteDocument(id);
      loadDocuments();
    } catch (err) {
      console.error('Failed to delete document:', err);
    }
  };

  if (loading) {
    return (
      <div class="flex items-center justify-center h-screen">
        <p class="text-gray-600 dark:text-gray-400">Loading...</p>
      </div>
    );
  }

  return (
    <div class="min-h-screen bg-gray-50 dark:bg-gray-900 p-8">
      <div class="max-w-6xl mx-auto">
        <div class="flex justify-between items-center mb-8">
          <h1 class="text-4xl font-bold text-gray-900 dark:text-white">
            Offline Office Suite
          </h1>
          <Button onClick={() => setShowCreateModal(true)} size="lg">
            New Document
          </Button>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {documents.map((doc) => (
            <a
              key={doc.id}
              href={`/${doc.doc_type}/${doc.id}`}
              class="block bg-white dark:bg-gray-800 rounded-lg shadow-md hover:shadow-lg transition-shadow p-6 border border-gray-200 dark:border-gray-700"
            >
              <div class="flex justify-between items-start mb-3">
                <div class="flex-1">
                  <h3 class="text-xl font-semibold text-gray-900 dark:text-white mb-2">
                    {doc.title}
                  </h3>
                  <span class="inline-block px-3 py-1 text-xs font-medium rounded-full bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200">
                    {doc.doc_type === 'document' ? 'Document' : 'Presentation'}
                  </span>
                </div>
                <button
                  onClick={(e) => {
                    e.preventDefault();
                    handleDeleteDocument(doc.id);
                  }}
                  class="text-red-500 hover:text-red-700 text-sm ml-2"
                >
                  Delete
                </button>
              </div>
              <p class="text-sm text-gray-600 dark:text-gray-400">
                Updated {new Date(doc.updated_at * 1000).toLocaleDateString()}
              </p>
            </a>
          ))}
        </div>

        {documents.length === 0 && (
          <div class="text-center py-12">
            <p class="text-gray-600 dark:text-gray-400 mb-4">
              No documents yet. Create your first document!
            </p>
          </div>
        )}
      </div>

      <Modal
        opened={showCreateModal}
        onClose={() => setShowCreateModal(false)}
        title="Create New Document"
      >
        <div class="space-y-4">
          <TextInput
            label="Title"
            placeholder="Document title"
            value={newDocTitle}
            onChange={(e) => setNewDocTitle((e.target as HTMLInputElement).value)}
          />
          <Select
            label="Type"
            value={newDocType}
            onChange={(value) => setNewDocType(value as 'document' | 'presentation')}
            data={[
              { value: 'document', label: 'Document' },
              { value: 'presentation', label: 'Presentation' },
            ]}
          />
          <div class="flex justify-end gap-2 mt-4">
            <Button variant="outline" onClick={() => setShowCreateModal(false)}>
              Cancel
            </Button>
            <Button onClick={handleCreateDocument}>Create</Button>
          </div>
        </div>
      </Modal>
    </div>
  );
}
