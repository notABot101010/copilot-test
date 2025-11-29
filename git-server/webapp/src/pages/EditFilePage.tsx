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
} from '@mantine/core';
import { useRoute, useRouter } from '@copilot-test/preact-router';
import { getBlob, updateFile } from '../api';

export function EditFilePage() {
  const route = useRoute();
  const router = useRouter();
  const content = useSignal<string>('');
  const commitMessage = useSignal<string>('');
  const loading = useSignal(true);
  const saving = useSignal(false);
  const error = useSignal<string | null>(null);

  const params = route.value.params;
  const query = route.value.query as { ref?: string };
  const repoName = params.name as string;
  const filePath = params.path as string;
  const gitRef = (query.ref as string) || 'HEAD';

  useSignalEffect(() => {
    loadContent();
  });

  async function loadContent() {
    try {
      loading.value = true;
      error.value = null;
      content.value = await getBlob(repoName, filePath, gitRef);
      commitMessage.value = `Update ${filePath}`;
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load file';
    } finally {
      loading.value = false;
    }
  }

  async function handleSave(e: Event) {
    e.preventDefault();

    if (!commitMessage.value.trim()) {
      error.value = 'Commit message is required';
      return;
    }

    try {
      saving.value = true;
      error.value = null;
      await updateFile(repoName, filePath, content.value, commitMessage.value.trim());
      router.push(
        `/repos/${encodeURIComponent(repoName)}/blob/${encodeURIComponent(filePath)}`
      );
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
  const parts = filePath.split('/');
  const breadcrumbItems = [
    <Anchor
      key="root"
      href={`/repos/${encodeURIComponent(repoName)}?ref=${encodeURIComponent(gitRef)}`}
    >
      {repoName}
    </Anchor>,
    ...parts.map((part, i) => {
      const partPath = parts.slice(0, i + 1).join('/');
      if (i === parts.length - 1) {
        return <Text key={partPath}>{part}</Text>;
      }
      return (
        <Anchor
          key={partPath}
          href={`/repos/${encodeURIComponent(repoName)}?ref=${encodeURIComponent(
            gitRef
          )}&path=${encodeURIComponent(partPath)}`}
        >
          {part}
        </Anchor>
      );
    }),
  ];

  return (
    <Card shadow="sm" padding="lg" radius="md" withBorder>
      <div class="border-b border-gray-200 pb-4 mb-4">
        <div class="flex items-center gap-3">
          <Anchor href="/" c="blue">
            Repositories
          </Anchor>
          <span class="text-gray-400">/</span>
          <Breadcrumbs>{breadcrumbItems}</Breadcrumbs>
          {gitRef !== 'HEAD' && (
            <Badge color="blue" variant="light">
              {gitRef.substring(0, 7)}
            </Badge>
          )}
          <Badge color="yellow" variant="light">
            Editing
          </Badge>
        </div>
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

        <div class="flex gap-3">
          <Button type="submit" loading={saving.value} color="green">
            Commit changes
          </Button>
          <Button
            variant="outline"
            onClick={() =>
              router.push(
                `/repos/${encodeURIComponent(repoName)}/blob/${encodeURIComponent(filePath)}?ref=${encodeURIComponent(gitRef)}`
              )
            }
            disabled={saving.value}
          >
            Cancel
          </Button>
        </div>
      </form>
    </Card>
  );
}
