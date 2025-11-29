import { useSignal } from '@preact/signals';
import { Card, Text, TextInput, Textarea, Button, Alert, Group } from '@mantine/core';
import { useRoute, useRouter } from '@copilot-test/preact-router';
import { updateProjectFile } from '../api';

export function CreateFilePage() {
  const route = useRoute();
  const router = useRouter();
  const filePath = useSignal('');
  const content = useSignal('');
  const commitMessage = useSignal('');
  const loading = useSignal(false);
  const error = useSignal<string | null>(null);

  const params = route.value.params;
  const orgName = params.org as string;
  const projectName = params.project as string;

  async function handleSubmit(e: Event) {
    e.preventDefault();

    if (!filePath.value.trim()) {
      error.value = 'File path is required';
      return;
    }

    if (!commitMessage.value.trim()) {
      error.value = 'Commit message is required';
      return;
    }

    try {
      loading.value = true;
      error.value = null;
      await updateProjectFile(
        orgName,
        projectName,
        filePath.value.trim(),
        content.value,
        commitMessage.value.trim()
      );
      router.push(`/${orgName}/${projectName}/blob/${filePath.value.trim()}`);
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to create file';
      loading.value = false;
    }
  }

  return (
    <Card shadow="sm" padding="lg" radius="md" withBorder>
      <Text size="xl" fw={600} mb="lg">
        ➕ Create new file in {projectName}
      </Text>

      {error.value && (
        <Alert color="red" title="Error" mb="md">
          {error.value}
        </Alert>
      )}

      <form onSubmit={handleSubmit}>
        <TextInput
          label="File path"
          description="Path to the file (e.g., src/main.js)"
          placeholder="path/to/file.txt"
          value={filePath.value}
          onChange={(e: Event) => (filePath.value = (e.target as HTMLInputElement).value)}
          required
          mb="md"
        />

        <Textarea
          label="File content"
          description="Content of the file"
          placeholder="Enter file content..."
          value={content.value}
          onChange={(e: Event) => (content.value = (e.target as HTMLTextAreaElement).value)}
          minRows={15}
          autosize
          styles={{
            input: {
              fontFamily: 'monospace',
              fontSize: '14px',
            },
          }}
          mb="md"
        />

        <TextInput
          label="Commit message"
          description="Describe the changes you're making"
          placeholder="Add new file"
          value={commitMessage.value}
          onChange={(e: Event) => (commitMessage.value = (e.target as HTMLInputElement).value)}
          required
          mb="lg"
        />

        <Group>
          <Button type="submit" loading={loading.value} color="green">
            Create file
          </Button>
          <Button
            variant="outline"
            onClick={() => router.push(`/${orgName}/${projectName}`)}
            disabled={loading.value}
          >
            Cancel
          </Button>
        </Group>
      </form>
    </Card>
  );
}
