import { useState } from 'preact/hooks';
import { useRouter } from '@copilot-test/preact-router';
import { Button, Card, Text, TextInput, Title, Stack, Group, ActionIcon, Container, Modal } from '@mantine/core';
import { spreadsheetList, createSpreadsheet, deleteSpreadsheet, loadSpreadsheetList } from '../store/spreadsheetStore';
import { useEffect } from 'preact/hooks';

export function HomePage() {
  const router = useRouter();
  const [newSheetName, setNewSheetName] = useState('');
  const [isCreateModalOpen, setIsCreateModalOpen] = useState(false);
  const [deleteConfirmId, setDeleteConfirmId] = useState<string | null>(null);

  useEffect(() => {
    loadSpreadsheetList();
  }, []);

  const handleCreateSpreadsheet = () => {
    if (!newSheetName.trim()) return;
    const id = createSpreadsheet(newSheetName.trim());
    setNewSheetName('');
    setIsCreateModalOpen(false);
    router.push(`/spreadsheets/${id}`);
  };

  const handleDeleteSpreadsheet = (id: string) => {
    deleteSpreadsheet(id);
    setDeleteConfirmId(null);
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

      {sheets.length === 0 ? (
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
          />
          <Group justify="flex-end">
            <Button variant="subtle" onClick={() => setIsCreateModalOpen(false)}>
              Cancel
            </Button>
            <Button onClick={handleCreateSpreadsheet} disabled={!newSheetName.trim()}>
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
            <Button variant="subtle" onClick={() => setDeleteConfirmId(null)}>
              Cancel
            </Button>
            <Button
              color="red"
              onClick={() => deleteConfirmId && handleDeleteSpreadsheet(deleteConfirmId)}
            >
              Delete
            </Button>
          </Group>
        </Stack>
      </Modal>
    </Container>
  );
}
