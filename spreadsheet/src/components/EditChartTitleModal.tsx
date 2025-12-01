import { useState, useEffect } from 'preact/hooks';
import { Modal, TextInput, Button, Group, Stack, Text } from '@mantine/core';

interface EditChartTitleModalProps {
  opened: boolean;
  onClose: () => void;
  onSubmit: (newTitle: string) => void;
  currentTitle: string;
}

export function EditChartTitleModal({ opened, onClose, onSubmit, currentTitle }: EditChartTitleModalProps) {
  const [title, setTitle] = useState(currentTitle);
  const [error, setError] = useState('');

  // Reset title when modal opens with new currentTitle
  useEffect(() => {
    if (opened) {
      setTitle(currentTitle);
      setError('');
    }
  }, [opened, currentTitle]);

  const handleSubmit = () => {
    if (!title.trim()) {
      setError('Please enter a chart title');
      return;
    }

    onSubmit(title.trim());
    onClose();
  };

  const handleClose = () => {
    setError('');
    onClose();
  };

  const handleKeyDown = (event: KeyboardEvent) => {
    if (event.key === 'Enter') {
      event.preventDefault();
      handleSubmit();
    } else if (event.key === 'Escape') {
      event.preventDefault();
      handleClose();
    }
  };

  return (
    <Modal
      opened={opened}
      onClose={handleClose}
      title="Edit Chart Title"
      size="sm"
    >
      <Stack gap="md">
        <TextInput
          label="Chart Title"
          value={title}
          onChange={(event: Event) => setTitle((event.target as HTMLInputElement).value)}
          onKeyDown={handleKeyDown}
          placeholder="Enter chart title"
          autoFocus
          data-testid="chart-title-input"
        />

        {error && <Text c="red" size="sm">{error}</Text>}

        <Group justify="flex-end" mt="md">
          <Button variant="subtle" onClick={handleClose}>
            Cancel
          </Button>
          <Button onClick={handleSubmit} data-testid="save-chart-title-button">
            Save
          </Button>
        </Group>
      </Stack>
    </Modal>
  );
}
