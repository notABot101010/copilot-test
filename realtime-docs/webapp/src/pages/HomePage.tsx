import { useState } from 'preact/hooks';
import { useSignal } from '@preact/signals';
import { useRouter } from '@copilot-test/preact-router';
import { Button, Card, TextInput, Stack, Group, ActionIcon, Container, Modal, Loader } from '@mantine/core';
import { fetchDocumentList, createDocument, deleteDocument } from '../store/documentStore';
import type { DocumentInfo } from '../store/documentStore';
import { useEffect } from 'preact/hooks';

export function HomePage() {
  const router = useRouter();
  const [newDocTitle, setNewDocTitle] = useState('');
  const [isCreateModalOpen, setIsCreateModalOpen] = useState(false);
  const [deleteConfirmId, setDeleteConfirmId] = useState<string | null>(null);
  const [isCreating, setIsCreating] = useState(false);
  const [isDeleting, setIsDeleting] = useState(false);

  // Use local signals for document list and loading state
  const documentList = useSignal<DocumentInfo[]>([]);
  const isLoadingList = useSignal(false);

  useEffect(() => {
    const loadDocuments = async () => {
      isLoadingList.value = true;
      const docs = await fetchDocumentList();
      documentList.value = docs;
      isLoadingList.value = false;
    };
    loadDocuments();
  }, [documentList, isLoadingList]);

  const handleCreateDocument = async () => {
    if (!newDocTitle.trim() || isCreating) return;
    setIsCreating(true);
    try {
      const id = await createDocument(newDocTitle.trim());
      setNewDocTitle('');
      setIsCreateModalOpen(false);
      if (id) {
        router.push(`/documents/${id}`);
      }
    } finally {
      setIsCreating(false);
    }
  };

  const handleDeleteDocument = async (id: string) => {
    if (isDeleting) return;
    setIsDeleting(true);
    try {
      const success = await deleteDocument(id);
      if (success) {
        // Update local list
        documentList.value = documentList.value.filter(doc => doc.id !== id);
      }
      setDeleteConfirmId(null);
    } finally {
      setIsDeleting(false);
    }
  };

  const handleOpenDocument = (id: string) => {
    router.push(`/documents/${id}`);
  };

  const formatDate = (timestamp: number) => {
    return new Date(timestamp).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  const docs = documentList.value;
  const isLoading = isLoadingList.value;

  return (
    <Container size="lg" className="py-8">
      <div className="flex justify-between items-center mb-8">
        <h1 className="text-3xl font-bold text-white">Realtime Docs</h1>
        <Button
          onClick={() => setIsCreateModalOpen(true)}
          className="bg-blue-600 hover:bg-blue-700"
        >
          New Document
        </Button>
      </div>

      {isLoading ? (
        <div className="flex justify-center py-12">
          <Loader size="xl" />
        </div>
      ) : docs.length === 0 ? (
        <Card className="text-center py-12 bg-gray-800">
          <p className="text-lg text-gray-400 mb-4">
            No documents yet
          </p>
          <p className="text-sm text-gray-500">
            Create your first document to get started
          </p>
        </Card>
      ) : (
        <Stack gap="md">
          {docs.map((doc) => (
            <Card
              key={doc.id}
              className="bg-gray-800 hover:bg-gray-700 transition-colors cursor-pointer"
              onClick={() => handleOpenDocument(doc.id)}
            >
              <Group justify="space-between" align="center">
                <div>
                  <p className="text-lg font-medium text-white">
                    {doc.title}
                  </p>
                  <p className="text-sm text-gray-400">
                    Created: {formatDate(doc.createdAt)} | Updated: {formatDate(doc.updatedAt)}
                  </p>
                </div>
                <ActionIcon
                  variant="subtle"
                  color="red"
                  onClick={(event: MouseEvent) => {
                    event.stopPropagation();
                    setDeleteConfirmId(doc.id);
                  }}
                  className="hover:bg-red-900"
                >
                  <span className="text-lg">🗑️</span>
                </ActionIcon>
              </Group>
            </Card>
          ))}
        </Stack>
      )}

      {/* Create Modal */}
      <Modal
        opened={isCreateModalOpen}
        onClose={() => setIsCreateModalOpen(false)}
        title="Create New Document"
        centered
      >
        <Stack gap="md">
          <TextInput
            label="Document Title"
            placeholder="Enter a title for your document"
            value={newDocTitle}
            onChange={(event: Event) => setNewDocTitle((event.target as HTMLInputElement).value)}
            onKeyDown={(event: KeyboardEvent) => {
              if (event.key === 'Enter') handleCreateDocument();
            }}
            disabled={isCreating}
          />
          <Group justify="flex-end">
            <Button variant="subtle" onClick={() => setIsCreateModalOpen(false)} disabled={isCreating}>
              Cancel
            </Button>
            <Button onClick={handleCreateDocument} disabled={!newDocTitle.trim() || isCreating} loading={isCreating}>
              Create
            </Button>
          </Group>
        </Stack>
      </Modal>

      {/* Delete Confirmation Modal */}
      <Modal
        opened={deleteConfirmId !== null}
        onClose={() => setDeleteConfirmId(null)}
        title="Delete Document"
        centered
      >
        <Stack gap="md">
          <p>Are you sure you want to delete this document? This action cannot be undone.</p>
          <Group justify="flex-end">
            <Button variant="subtle" onClick={() => setDeleteConfirmId(null)} disabled={isDeleting}>
              Cancel
            </Button>
            <Button
              color="red"
              onClick={() => deleteConfirmId && handleDeleteDocument(deleteConfirmId)}
              loading={isDeleting}
            >
              Delete
            </Button>
          </Group>
        </Stack>
      </Modal>
    </Container>
  );
}
