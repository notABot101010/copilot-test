import { useRouter, useRoute } from '@copilot-test/preact-router';
import { signal } from '@preact/signals';
import { useEffect } from 'preact/hooks';
import { Button, Card, Group, Text, Title, Stack, LoadingOverlay, Box, Breadcrumbs, Anchor, Code, Badge } from '@mantine/core';
import { getDocument, deleteDocuments } from '../api';
import type { Document } from '../api';

const document = signal<Document | null>(null);
const loading = signal(true);
const error = signal<string | null>(null);

export function DocumentPage() {
  const router = useRouter();
  const route = useRoute();
  const params = route.value.params;
  const namespaceName = decodeURIComponent((params.namespace as string) || '');
  const docId = decodeURIComponent((params.docId as string) || '');

  const loadDocument = async () => {
    if (!namespaceName || !docId) return;
    
    loading.value = true;
    error.value = null;
    try {
      const data = await getDocument(namespaceName, docId);
      document.value = data;
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load document';
    } finally {
      loading.value = false;
    }
  };

  useEffect(() => {
    loadDocument();
  }, [namespaceName, docId]);

  const handleDelete = async () => {
    if (confirm(`Are you sure you want to delete document "${docId}"?`)) {
      try {
        await deleteDocuments(namespaceName, [docId]);
        router.push(`/namespaces/${encodeURIComponent(namespaceName)}/documents`);
      } catch (e) {
        alert(e instanceof Error ? e.message : 'Failed to delete document');
      }
    }
  };

  const breadcrumbs = [
    { title: 'Namespaces', href: '/' },
    { title: namespaceName, href: `/namespaces/${encodeURIComponent(namespaceName)}` },
    { title: 'Documents', href: `/namespaces/${encodeURIComponent(namespaceName)}/documents` },
    { title: docId, href: '#' },
  ].map((item, index) => (
    <Anchor
      key={index}
      href={item.href}
      onClick={(e: Event) => {
        if (item.href !== '#') {
          e.preventDefault();
          router.push(item.href);
        }
      }}
    >
      {item.title}
    </Anchor>
  ));

  const getDocumentAttributes = (doc: Document): [string, unknown][] => {
    return Object.entries(doc).filter(([k]) => k !== 'id' && k !== 'vector');
  };

  return (
    <div>
      <Breadcrumbs mb="md">{breadcrumbs}</Breadcrumbs>

      <Group justify="space-between" mb="lg">
        <Title order={2}>Document: {docId}</Title>
        <Group>
          <Button variant="light" onClick={loadDocument}>
            Refresh
          </Button>
          <Button variant="light" color="red" onClick={handleDelete}>
            Delete
          </Button>
        </Group>
      </Group>

      {error.value && (
        <Card withBorder mb="md" className="bg-red-50">
          <Text c="red">{error.value}</Text>
        </Card>
      )}

      <Box pos="relative">
        <LoadingOverlay visible={loading.value} />
        
        {document.value && (
          <Stack gap="md">
            <Card withBorder>
              <Title order={4} mb="md">Metadata</Title>
              <Group>
                <Text fw={500}>ID:</Text>
                <Code>{document.value.id}</Code>
              </Group>
              <Group mt="sm">
                <Text fw={500}>Has Vector:</Text>
                {document.value.vector ? (
                  <Badge color="green">{document.value.vector.length} dimensions</Badge>
                ) : (
                  <Badge color="gray">No</Badge>
                )}
              </Group>
            </Card>

            {document.value.vector && (
              <Card withBorder>
                <Title order={4} mb="md">Vector</Title>
                <Code block className="max-h-40 overflow-auto">
                  [{document.value.vector.map(v => v.toFixed(6)).join(', ')}]
                </Code>
              </Card>
            )}

            <Card withBorder>
              <Title order={4} mb="md">Attributes</Title>
              <Stack gap="sm">
                {getDocumentAttributes(document.value).map(([key, value]) => (
                  <Group key={key} align="flex-start">
                    <Text fw={500} w={150}>{key}:</Text>
                    <Code block style={{ flex: 1, maxHeight: 100, overflow: 'auto' }}>
                      {typeof value === 'string' ? value : JSON.stringify(value, null, 2)}
                    </Code>
                  </Group>
                ))}
                {getDocumentAttributes(document.value).length === 0 && (
                  <Text c="dimmed">No attributes</Text>
                )}
              </Stack>
            </Card>

            <Card withBorder>
              <Title order={4} mb="md">Raw JSON</Title>
              <Code block className="max-h-80 overflow-auto">
                {JSON.stringify(document.value, null, 2)}
              </Code>
            </Card>
          </Stack>
        )}
      </Box>
    </div>
  );
}
