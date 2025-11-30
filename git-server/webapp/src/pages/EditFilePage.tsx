import { useSignal, useSignalEffect } from '@preact/signals';
import {
  Card,
  Text,
  Loader,
  Alert,
  Breadcrumbs,
  Anchor,
  Badge,
  Button,
  Textarea,
  TextInput,
  Group,
} from '@mantine/core';
import { useRoute, useRouter } from '@copilot-test/preact-router';
import { getProjectBlob, updateProjectFile } from '../api';

export function EditFilePage() {
  const route = useRoute();
  const router = useRouter();
  const content = useSignal<string>('');
  const commitMessage = useSignal<string>('');
  const loading = useSignal(true);
  const saving = useSignal(false);
  const error = useSignal<string | null>(null);

  useSignalEffect(() => {
    // Access route.value inside the effect to track signal changes
    const params = route.value.params;
    const query = route.value.query as { ref?: string };
    const orgName = params.org as string;
    const projectName = params.project as string;
    const filePath = params.path as string;
    const gitRef = (query.ref as string) || 'HEAD';

    if (!orgName || !projectName || !filePath) {
      error.value = 'No file path provided';
      loading.value = false;
      return;
    }

    loadContent(orgName, projectName, filePath, gitRef);
  });

  async function loadContent(orgName: string, projectName: string, filePath: string, gitRef: string) {
    try {
      loading.value = true;
      error.value = null;
      content.value = await getProjectBlob(orgName, projectName, filePath, gitRef);
      commitMessage.value = `Update ${filePath}`;
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to load file';
    } finally {
      loading.value = false;
    }
  }

  // Get current route params for rendering
  const params = route.value.params;
  const query = route.value.query as { ref?: string };
  const orgName = params.org as string;
  const projectName = params.project as string;
  const filePath = params.path as string;
  const gitRef = (query.ref as string) || 'HEAD';

  async function handleSave(e: Event) {
    e.preventDefault();

    if (!commitMessage.value.trim()) {
      error.value = 'Commit message is required';
      return;
    }

    try {
      saving.value = true;
      error.value = null;
      await updateProjectFile(orgName, projectName, filePath, content.value, commitMessage.value.trim());
      router.push(`/${orgName}/${projectName}/blob/${filePath}`);
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to save file';
      saving.value = false;
    }
  }

  if (loading.value) {
    return (
      <div class="flex justify-center py-12">
        <Loader size="lg" />
      </div>
    );
  }

  // Build breadcrumb items
  const parts = filePath ? filePath.split('/') : [];
  const breadcrumbItems = [
    <Anchor
      key="root"
      href={`/${orgName}/${projectName}?ref=${encodeURIComponent(gitRef)}`}
    >
      {projectName}
    </Anchor>,
    ...parts.map((part, i) => {
      const partPath = parts.slice(0, i + 1).join('/');
      if (i === parts.length - 1) {
        return <Text key={partPath}>{part}</Text>;
      }
      return (
        <Anchor
          key={partPath}
          href={`/${orgName}/${projectName}?ref=${encodeURIComponent(gitRef)}&path=${encodeURIComponent(partPath)}`}
        >
          {part}
        </Anchor>
      );
    }),
  ];

  return (
    <Card shadow="sm" padding="lg" radius="md" withBorder>
      <div class="border-b border-gray-200 pb-4 mb-4">
        <Group gap="xs">
          <Breadcrumbs>{breadcrumbItems}</Breadcrumbs>
          {gitRef !== 'HEAD' && (
            <Badge color="blue" variant="light">
              {gitRef.substring(0, 7)}
            </Badge>
          )}
          <Badge color="yellow" variant="light">
            Editing
          </Badge>
        </Group>
      </div>

      {error.value && (
        <Alert color="red" title="Error" mb="md">
          {error.value}
        </Alert>
      )}

      <form onSubmit={handleSave}>
        <Textarea
          value={content.value}
          onChange={(e: Event) => (content.value = (e.target as HTMLTextAreaElement).value)}
          placeholder="File content"
          minRows={20}
          autosize
          styles={{
            input: {
              fontFamily: 'monospace',
              fontSize: '14px',
            },
          }}
          mb="lg"
        />

        <TextInput
          label="Commit message"
          placeholder="Update file"
          value={commitMessage.value}
          onChange={(e: Event) => (commitMessage.value = (e.target as HTMLInputElement).value)}
          required
          mb="lg"
        />

        <Group>
          <Button type="submit" loading={saving.value} color="green">
            Commit changes
          </Button>
          <Button
            variant="outline"
            onClick={() => router.push(`/${orgName}/${projectName}/blob/${filePath}?ref=${encodeURIComponent(gitRef)}`)}
            disabled={saving.value}
          >
            Cancel
          </Button>
        </Group>
      </form>
    </Card>
  );
}
