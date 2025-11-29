import { useState } from 'preact/hooks';
import { useRouter } from '@copilot-test/preact-router';
import { Button, Card, Text, TextInput, Title, Stack, Group, ActionIcon, Container, Modal, Loader } from '@mantine/core';
import { spreadsheetList, createSpreadsheet, deleteSpreadsheet, loadSpreadsheetList, isLoadingList } from '../store/spreadsheetStore';
import { useEffect } from 'preact/hooks';

export function HomePage() {
  const router = useRouter();
  const [newSheetName, setNewSheetName] = useState('');
  const [isCreateModalOpen, setIsCreateModalOpen] = useState(false);
  const [deleteConfirmId, setDeleteConfirmId] = useState<string | null>(null);
  const [isCreating, setIsCreating] = useState(false);
  const [isDeleting, setIsDeleting] = useState(false);

  useEffect(() => {
    loadSpreadsheetList();
  }, []);

  const handleCreateSpreadsheet = async () => {
    if (!newSheetName.trim() || isCreating) return;
    setIsCreating(true);
    try {
      const id = await createSpreadsheet(newSheetName.trim());
      setNewSheetName('');
      setIsCreateModalOpen(false);
      if (id) {
        router.push(`/spreadsheets/${id}`);
      }
    } finally {
      setIsCreating(false);
    }
  };

  const handleDeleteSpreadsheet = async (id: string) => {
    if (isDeleting) return;
    setIsDeleting(true);
    try {
      await deleteSpreadsheet(id);
      setDeleteConfirmId(null);
    } finally {
      setIsDeleting(false);
    }
  };

  const handleOpenSpreadsheet = (id: string) => {
    router.push(`/spreadsheets/${id}`);
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

  const sheets = spreadsheetList.value;
  const isLoading = isLoadingList.value;

  return (
    <Container size="lg" className="py-8">
      <div className="flex justify-between items-center mb-8">
        <Title order={1} className="text-white">Spreadsheets</Title>
        <Button
          onClick={() => setIsCreateModalOpen(true)}
          className="bg-blue-600 hover:bg-blue-700"
        >
          New Spreadsheet
        </Button>
      </div>

      {isLoading ? (
        <div className="flex justify-center py-12">
          <Loader size="xl" />
        </div>
      ) : sheets.length === 0 ? (
        <Card className="text-center py-12 bg-gray-800">
          <Text size="lg" c="dimmed" className="mb-4">
            No spreadsheets yet
          </Text>
          <Text size="sm" c="dimmed">
            Create your first spreadsheet to get started
          </Text>
        </Card>
      ) : (
        <Stack gap="md">
          {sheets.map((sheet) => (
            <Card
              key={sheet.id}
              className="bg-gray-800 hover:bg-gray-700 transition-colors cursor-pointer"
              onClick={() => handleOpenSpreadsheet(sheet.id)}
            >
              <Group justify="space-between" align="center">
                <div>
                  <Text size="lg" fw={500} className="text-white">
                    {sheet.name}
                  </Text>
                  <Text size="sm" c="dimmed">
                    Created: {formatDate(sheet.createdAt)} | Updated: {formatDate(sheet.updatedAt)}
                  </Text>
                </div>
                <ActionIcon
                  variant="subtle"
                  color="red"
                  onClick={(e: MouseEvent) => {
                    e.stopPropagation();
                    setDeleteConfirmId(sheet.id);
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
        title="Create New Spreadsheet"
        centered
      >
        <Stack gap="md">
          <TextInput
            label="Spreadsheet Name"
            placeholder="Enter a name for your spreadsheet"
            value={newSheetName}
            onChange={(e: Event) => setNewSheetName((e.target as HTMLInputElement).value)}
            onKeyDown={(e: KeyboardEvent) => {
              if (e.key === 'Enter') handleCreateSpreadsheet();
            }}
            disabled={isCreating}
          />
          <Group justify="flex-end">
            <Button variant="subtle" onClick={() => setIsCreateModalOpen(false)} disabled={isCreating}>
              Cancel
            </Button>
            <Button onClick={handleCreateSpreadsheet} disabled={!newSheetName.trim() || isCreating} loading={isCreating}>
              Create
            </Button>
          </Group>
        </Stack>
      </Modal>

      {/* Delete Confirmation Modal */}
      <Modal
        opened={deleteConfirmId !== null}
        onClose={() => setDeleteConfirmId(null)}
        title="Delete Spreadsheet"
        centered
      >
        <Stack gap="md">
          <Text>Are you sure you want to delete this spreadsheet? This action cannot be undone.</Text>
          <Group justify="flex-end">
            <Button variant="subtle" onClick={() => setDeleteConfirmId(null)} disabled={isDeleting}>
              Cancel
            </Button>
            <Button
              color="red"
              onClick={() => deleteConfirmId && handleDeleteSpreadsheet(deleteConfirmId)}
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
