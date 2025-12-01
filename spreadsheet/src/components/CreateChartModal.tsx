import { useState } from 'preact/hooks';
import { Modal, Select, TextInput, Button, Group, Stack, Text } from '@mantine/core';
import type { ChartType, ChartDataRange } from '../types/chart';
import { CHART_TYPE_LABELS } from '../types/chart';

interface CreateChartModalProps {
  opened: boolean;
  onClose: () => void;
  onSubmit: (type: ChartType, title: string, dataRange: ChartDataRange) => void;
}

const chartTypeOptions = Object.entries(CHART_TYPE_LABELS).map(([value, label]) => ({
  value,
  label,
}));

export function CreateChartModal({ opened, onClose, onSubmit }: CreateChartModalProps) {
  const [chartType, setChartType] = useState<ChartType>('bar');
  const [title, setTitle] = useState('New Chart');
  const [labelsRange, setLabelsRange] = useState('A1:A10');
  const [dataRange, setDataRange] = useState('B1:B10');
  const [error, setError] = useState('');

  const handleSubmit = () => {
    if (!title.trim()) {
      setError('Please enter a chart title');
      return;
    }
    if (!labelsRange.trim()) {
      setError('Please enter a labels range');
      return;
    }
    if (!dataRange.trim()) {
      setError('Please enter a data range');
      return;
    }

    onSubmit(chartType, title.trim(), {
      labelsRange: labelsRange.trim().toUpperCase(),
      dataRange: dataRange.trim().toUpperCase(),
    });

    // Reset form
    setChartType('bar');
    setTitle('New Chart');
    setLabelsRange('A1:A10');
    setDataRange('B1:B10');
    setError('');
    onClose();
  };

  const handleClose = () => {
    setError('');
    onClose();
  };

  return (
    <Modal
      opened={opened}
      onClose={handleClose}
      title="Create New Chart"
      size="md"
    >
      <Stack gap="md">
        <Select
          label="Chart Type"
          data={chartTypeOptions}
          value={chartType}
          onChange={(value: string | null) => value && setChartType(value as ChartType)}
        />

        <TextInput
          label="Chart Title"
          value={title}
          onChange={(e: Event) => setTitle((e.target as HTMLInputElement).value)}
          placeholder="Enter chart title"
        />

        <TextInput
          label="Labels Range"
          value={labelsRange}
          onChange={(e: Event) => setLabelsRange((e.target as HTMLInputElement).value)}
          placeholder="e.g., A1:A10"
          description="Cell range for category labels (e.g., A1:A10)"
        />

        <TextInput
          label="Data Range"
          value={dataRange}
          onChange={(e: Event) => setDataRange((e.target as HTMLInputElement).value)}
          placeholder="e.g., B1:B10"
          description={
            chartType === 'scatter'
              ? 'For scatter: X values range (Y values will be from labels range)'
              : chartType === 'heatmap'
              ? 'Full data range for heatmap (e.g., A1:E5)'
              : 'Cell range for data values'
          }
        />

        {error && <Text c="red" size="sm">{error}</Text>}

        <Group justify="flex-end" mt="md">
          <Button variant="subtle" onClick={handleClose}>
            Cancel
          </Button>
          <Button onClick={handleSubmit}>
            Create Chart
          </Button>
        </Group>
      </Stack>
    </Modal>
  );
}
