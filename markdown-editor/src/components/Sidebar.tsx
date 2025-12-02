import { useSignal } from '@preact/signals';
import { Button, TextInput, ActionIcon, Modal } from '@mantine/core';
import {
  sortedDocuments,
  activeDocumentId,
  createDocument,
  setActiveDocument,
  deleteDocument,
  updateDocument,
  type Document,
} from '../store/documents';

interface SidebarProps {
  isOpen: boolean;
  onClose: () => void;
}

export function Sidebar({ isOpen, onClose }: SidebarProps) {
  const editingDocId = useSignal<string | null>(null);
  const editTitle = useSignal('');
  const deleteModalOpen = useSignal(false);
  const docToDelete = useSignal<Document | null>(null);

  const handleNewDocument = () => {
    createDocument('Untitled');
    onClose();
  };

  const handleSelectDocument = (id: string) => {
    setActiveDocument(id);
    onClose();
  };

  const handleStartEdit = (doc: Document) => {
    editingDocId.value = doc.id;
    editTitle.value = doc.title;
  };

  const handleSaveEdit = () => {
    if (editingDocId.value && editTitle.value.trim()) {
      updateDocument(editingDocId.value, { title: editTitle.value.trim() });
    }
    editingDocId.value = null;
  };

  const handleCancelEdit = () => {
    editingDocId.value = null;
  };

  const handleDeleteClick = (doc: Document) => {
    docToDelete.value = doc;
    deleteModalOpen.value = true;
  };

  const handleConfirmDelete = () => {
    if (docToDelete.value) {
      deleteDocument(docToDelete.value.id);
    }
    deleteModalOpen.value = false;
    docToDelete.value = null;
  };

  const handleCancelDelete = () => {
    deleteModalOpen.value = false;
    docToDelete.value = null;
  };

  const formatDate = (timestamp: number) => {
    return new Date(timestamp).toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
    });
  };

  return (
    <>
      <aside
        data-testid="sidebar"
        class={`
          fixed lg:static inset-y-0 left-0 z-40
          w-72 bg-zinc-900 border-r border-zinc-800
          flex flex-col h-full
          transform transition-transform duration-300 ease-in-out
          ${isOpen ? 'translate-x-0' : '-translate-x-full lg:translate-x-0'}
        `}
      >
        <div class="p-4 border-b border-zinc-800">
          <Button
            fullWidth
            onClick={handleNewDocument}
            data-testid="new-document-button"
            className="bg-blue-600 hover:bg-blue-700"
          >
            New Document
          </Button>
        </div>

        <div class="flex-1 overflow-y-auto">
          {sortedDocuments.value.length === 0 ? (
            <div class="p-4 text-center text-zinc-500">
              <p>No documents yet</p>
              <p class="text-sm mt-1">Create your first document</p>
            </div>
          ) : (
            <ul class="divide-y divide-zinc-800">
              {sortedDocuments.value.map((doc) => (
                <li
                  key={doc.id}
                  data-testid={`document-item-${doc.id}`}
                  class={`
                    group relative cursor-pointer
                    ${activeDocumentId.value === doc.id ? 'bg-zinc-800' : 'hover:bg-zinc-800/50'}
                  `}
                >
                  {editingDocId.value === doc.id ? (
                    <div class="p-3">
                      <TextInput
                        value={editTitle.value}
                        onChange={(event: Event) => (editTitle.value = (event.currentTarget as HTMLInputElement).value)}
                        onKeyDown={(event: KeyboardEvent) => {
                          if (event.key === 'Enter') handleSaveEdit();
                          if (event.key === 'Escape') handleCancelEdit();
                        }}
                        autoFocus
                        size="sm"
                        data-testid="edit-title-input"
                      />
                      <div class="flex gap-2 mt-2">
                        <Button size="xs" onClick={handleSaveEdit} data-testid="save-title-button">
                          Save
                        </Button>
                        <Button size="xs" variant="subtle" onClick={handleCancelEdit}>
                          Cancel
                        </Button>
                      </div>
                    </div>
                  ) : (
                    <button
                      onClick={() => handleSelectDocument(doc.id)}
                      class="w-full text-left p-3 pr-16 cursor-pointer"
                      data-testid={`select-document-${doc.id}`}
                    >
                      <p class="font-medium text-zinc-100 truncate">{doc.title}</p>
                      <p class="text-xs text-zinc-500 mt-1">{formatDate(doc.updatedAt)}</p>
                    </button>
                  )}

                  {editingDocId.value !== doc.id && (
                    <div class="absolute right-2 top-1/2 -translate-y-1/2 flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                      <ActionIcon
                        size="sm"
                        variant="subtle"
                        onClick={(event: MouseEvent) => {
                          event.stopPropagation();
                          handleStartEdit(doc);
                        }}
                        aria-label="Rename document"
                        data-testid={`rename-document-${doc.id}`}
                      >
                        <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                          <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" />
                          <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" />
                        </svg>
                      </ActionIcon>
                      <ActionIcon
                        size="sm"
                        variant="subtle"
                        color="red"
                        onClick={(event: MouseEvent) => {
                          event.stopPropagation();
                          handleDeleteClick(doc);
                        }}
                        aria-label="Delete document"
                        data-testid={`delete-document-${doc.id}`}
                      >
                        <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                          <polyline points="3 6 5 6 21 6" />
                          <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
                        </svg>
                      </ActionIcon>
                    </div>
                  )}
                </li>
              ))}
            </ul>
          )}
        </div>
      </aside>

      {/* Mobile overlay */}
      {isOpen && (
        <div
          class="fixed inset-0 bg-black/50 z-30 lg:hidden"
          onClick={onClose}
          data-testid="sidebar-overlay"
        />
      )}

      {/* Delete confirmation modal */}
      <Modal
        opened={deleteModalOpen.value}
        onClose={handleCancelDelete}
        title="Delete Document"
        centered
      >
        <p class="text-zinc-300 mb-4">
          Are you sure you want to delete "{docToDelete.value?.title}"? This action cannot be undone.
        </p>
        <div class="flex gap-2 justify-end">
          <Button variant="subtle" onClick={handleCancelDelete}>
            Cancel
          </Button>
          <Button color="red" onClick={handleConfirmDelete} data-testid="confirm-delete-button">
            Delete
          </Button>
        </div>
      </Modal>
    </>
  );
}
