import { useRouter, useRoute } from '@copilot-test/preact-router';
import { signal } from '@preact/signals';
import { useEffect } from 'preact/hooks';
import { Button, Card, Group, Text, Title, Badge, Stack, Divider, LoadingOverlay, Box, Breadcrumbs, Anchor } from '@mantine/core';
import { getNamespace, deleteNamespace } from '../api';
import type { Namespace } from '../api';

const namespace = signal<Namespace | null>(null);
const loading = signal(true);
const error = signal<string | null>(null);

export function NamespacePage() {
  const router = useRouter();
  const route = useRoute();
  const params = route.value.params;
  const namespaceName = decodeURIComponent((params.namespace as string) || '');

  const loadNamespace = async () => {
    if (!namespaceName) return;
    
    loading.value = true;
    error.value = null;
    try {
      const data = await getNamespace(namespaceName);
      namespace.value = data;
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load namespace';
    } finally {
      loading.value = false;
    }
  };

  useEffect(() => {
    loadNamespace();
  }, [namespaceName]);

  const handleDelete = async () => {
    if (confirm(`Are you sure you want to delete namespace "${namespaceName}"?`)) {
      try {
        await deleteNamespace(namespaceName);
        router.push('/');
      } catch (e) {
        alert(e instanceof Error ? e.message : 'Failed to delete namespace');
      }
    }
  };

  const breadcrumbs = [
    { title: 'Namespaces', href: '/' },
    { title: namespaceName, href: '#' },
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

  return (
    <div>
      <Breadcrumbs mb="md">{breadcrumbs}</Breadcrumbs>

      <Group justify="space-between" mb="lg">
        <Title order={2}>{namespaceName}</Title>
        <Group>
          <Button variant="light" onClick={loadNamespace}>
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
        
        {namespace.value && (
          <Card withBorder>
            <Stack gap="md">
              <Group>
                <Text fw={500}>Document Count:</Text>
                <Badge size="lg">{namespace.value.document_count}</Badge>
              </Group>
              
              <Divider />
              
              <Group>
                <Text fw={500}>Distance Metric:</Text>
                <Badge variant="outline" color="gray">{namespace.value.distance_metric}</Badge>
              </Group>
              
              <Group>
                <Text fw={500}>Vector Dimensions:</Text>
                <Text>{namespace.value.vector_dimensions ?? 'Not set'}</Text>
              </Group>
              
              <Divider />
              
              <Group>
                <Button
                  onClick={() => router.push(`/namespaces/${encodeURIComponent(namespaceName)}/documents`)}
                >
                  View Documents
                </Button>
                <Button
                  color="blue"
                  onClick={() => router.push(`/namespaces/${encodeURIComponent(namespaceName)}/query`)}
                >
                  Query Namespace
                </Button>
              </Group>
            </Stack>
          </Card>
        )}
      </Box>
    </div>
  );
}
