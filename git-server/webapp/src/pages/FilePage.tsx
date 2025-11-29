import { useSignal, useSignalEffect } from '@preact/signals';
import { Card, Text, Loader, Alert, Breadcrumbs, Anchor, Badge, Button, Group } from '@mantine/core';
import { useRoute, useRouter } from '@copilot-test/preact-router';
import { getProjectBlob, deleteProjectFile } from '../api';

export function FilePage() {
  const route = useRoute();
  const router = useRouter();
  const content = useSignal<string>('');
  const loading = useSignal(true);
  const deleting = useSignal(false);
  const error = useSignal<string | null>(null);

  const params = route.value.params;
  const query = route.value.query as { ref?: string };
  const orgName = params.org as string;
  const projectName = params.project as string;
  const filePath = params.path as string;
  const gitRef = (query.ref as string) || 'HEAD';

  useSignalEffect(() => {
    loadContent();
  });

  async function loadContent() {
    if (!filePath) {
      error.value = 'No file path provided';
      loading.value = false;
      return;
    }
    try {
      loading.value = true;
      error.value = null;
      content.value = await getProjectBlob(orgName, projectName, filePath, gitRef);
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load file';
    } finally {
      loading.value = false;
    }
  }

  async function handleDelete() {
    if (!confirm(`Are you sure you want to delete ${filePath}?`)) {
      return;
    }

    try {
      deleting.value = true;
      error.value = null;
      await deleteProjectFile(orgName, projectName, filePath, `Delete ${filePath}`);
      router.push(`/${orgName}/${projectName}`);
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to delete file';
      deleting.value = false;
    }
  }

  if (loading.value) {
    return (
      <div class="flex justify-center py-12">
        <Loader size="lg" />
      </div>
    );
  }

  if (error.value) {
    return (
      <Alert color="red" title="Error">
        {error.value}
      </Alert>
    );
  }

  // Build breadcrumb items
  const parts = filePath ? filePath.split('/') : [];
  const breadcrumbItems = [
    <Anchor
      key="root"
      href={`/${orgName}/${projectName}?ref=${encodeURIComponent(gitRef)}`}
      onClick={(e: Event) => {
        e.preventDefault();
        router.push(`/${orgName}/${projectName}?ref=${encodeURIComponent(gitRef)}`);
      }}
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
          onClick={(e: Event) => {
            e.preventDefault();
            router.push(`/${orgName}/${projectName}?ref=${encodeURIComponent(gitRef)}&path=${encodeURIComponent(partPath)}`);
          }}
        >
          {part}
        </Anchor>
      );
    }),
  ];

  return (
    <Card shadow="sm" padding="lg" radius="md" withBorder>
      <div class="border-b border-gray-200 pb-4 mb-4">
        <Group justify="space-between">
          <Group gap="xs">
            <Breadcrumbs>{breadcrumbItems}</Breadcrumbs>
            {gitRef !== 'HEAD' && (
              <Badge color="blue" variant="light">
                {gitRef.substring(0, 7)}
              </Badge>
            )}
          </Group>
          <Group gap="xs">
            <Button
              variant="filled"
              size="sm"
              onClick={() => router.push(`/${orgName}/${projectName}/edit/${filePath}?ref=${encodeURIComponent(gitRef)}`)}
            >
              ✏️ Edit
            </Button>
            <Button
              variant="outline"
              size="sm"
              color="red"
              onClick={handleDelete}
              loading={deleting.value}
            >
              🗑️ Delete
            </Button>
          </Group>
        </Group>
      </div>

      <pre class="bg-gray-100 p-4 overflow-x-auto font-mono text-sm leading-relaxed whitespace-pre-wrap rounded">
        {content.value}
      </pre>
    </Card>
  );
}
