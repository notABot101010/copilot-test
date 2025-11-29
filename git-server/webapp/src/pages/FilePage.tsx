import { useSignal, useSignalEffect } from '@preact/signals';
import { Card, Text, Loader, Alert, Breadcrumbs, Anchor, Badge, Button } from '@mantine/core';
import { useRoute } from '@copilot-test/preact-router';
import { getBlob } from '../api';

export function FilePage() {
  const route = useRoute();
  const content = useSignal<string>('');
  const loading = useSignal(true);
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
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load file';
    } finally {
      loading.value = false;
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
        <div class="flex items-center justify-between">
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
          </div>
          <Button
            variant="filled"
            size="sm"
            component="a"
            href={`/repos/${encodeURIComponent(repoName)}/edit/${encodeURIComponent(
              filePath
            )}?ref=${encodeURIComponent(gitRef)}`}
          >
            Edit file
          </Button>
        </div>
      </div>

      <pre class="bg-gray-100 p-4 overflow-x-auto font-mono text-sm leading-relaxed whitespace-pre-wrap rounded">
        {content.value}
      </pre>
    </Card>
  );
}
