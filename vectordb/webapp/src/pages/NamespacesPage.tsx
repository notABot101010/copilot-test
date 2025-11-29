import { useRouter } from '@copilot-test/preact-router';
import { signal } from '@preact/signals';
import { useEffect } from 'preact/hooks';
import { Button, Card, Group, Table, Text, Title, Badge, LoadingOverlay, Box, Alert } from '@mantine/core';
import { listNamespaces, deleteNamespace } from '../api';
import type { Namespace } from '../api';

const namespaces = signal<Namespace[]>([]);
const loading = signal(true);
const error = signal<string | null>(null);
const backendUnavailable = signal(false);

export function NamespacesPage() {
  const router = useRouter();

  const loadNamespaces = async () => {
    loading.value = true;
    error.value = null;
    backendUnavailable.value = false;
    try {
      const data = await listNamespaces();
      namespaces.value = data;
    } catch (e) {
      const errMsg = e instanceof Error ? e.message : 'Failed to load namespaces';
      // Check if it's a connection error (backend not running)
      if (errMsg.includes('Internal Server Error') || errMsg.includes('Failed to fetch') || errMsg.includes('NetworkError')) {
        backendUnavailable.value = true;
      } else {
        error.value = errMsg;
      }
    } finally {
      loading.value = false;
    }
  };

  useEffect(() => {
    loadNamespaces();
  }, []);

  const handleDelete = async (name: string) => {
    if (confirm(`Are you sure you want to delete namespace "${name}"?`)) {
      try {
        await deleteNamespace(name);
        await loadNamespaces();
      } catch (e) {
        alert(e instanceof Error ? e.message : 'Failed to delete namespace');
      }
    }
  };

  return (
    <div>
      <Group justify="space-between" mb="lg">
        <Title order={2}>Namespaces</Title>
        <Button onClick={loadNamespaces} variant="light">
          Refresh
        </Button>
      </Group>

      {error.value && (
        <Card withBorder mb="md" className="bg-red-50">
          <Text c="red">{error.value}</Text>
        </Card>
      )}

      {backendUnavailable.value && (
        <Alert color="yellow" title="Backend Server Not Running" mb="md">
          <Text size="sm">
            The VectorDB backend server is not running. Start the server with:
          </Text>
          <Text size="sm" mt="xs" ff="monospace" className="bg-gray-100 p-2 rounded">
            S3_BUCKET=your-bucket cargo run -p vectordb
          </Text>
          <Text size="sm" mt="xs" c="dimmed">
            Make sure to set the required environment variables (S3_BUCKET, AWS credentials, etc.)
          </Text>
        </Alert>
      )}

      <Box pos="relative">
        <LoadingOverlay visible={loading.value} />
        
        {namespaces.value.length === 0 && !loading.value ? (
          <Card withBorder>
            <Text c="dimmed" ta="center">No namespaces found. Create one by upserting documents via the API.</Text>
          </Card>
        ) : (
          <Table striped highlightOnHover>
            <Table.Thead>
              <Table.Tr>
                <Table.Th>Name</Table.Th>
                <Table.Th>Documents</Table.Th>
                <Table.Th>Distance Metric</Table.Th>
                <Table.Th>Dimensions</Table.Th>
                <Table.Th>Actions</Table.Th>
              </Table.Tr>
            </Table.Thead>
            <Table.Tbody>
              {namespaces.value.map((ns) => (
                <Table.Tr key={ns.name}>
                  <Table.Td>
                    <Text
                      component="a"
                      href={`/namespaces/${encodeURIComponent(ns.name)}`}
                      onClick={(e: Event) => {
                        e.preventDefault();
                        router.push(`/namespaces/${encodeURIComponent(ns.name)}`);
                      }}
                      className="text-blue-600 hover:underline cursor-pointer"
                    >
                      {ns.name}
                    </Text>
                  </Table.Td>
                  <Table.Td>
                    <Badge variant="light">{ns.document_count}</Badge>
                  </Table.Td>
                  <Table.Td>
                    <Badge color="gray" variant="outline">{ns.distance_metric}</Badge>
                  </Table.Td>
                  <Table.Td>{ns.vector_dimensions ?? '-'}</Table.Td>
                  <Table.Td>
                    <Group gap="xs">
                      <Button
                        size="xs"
                        variant="light"
                        onClick={() => router.push(`/namespaces/${encodeURIComponent(ns.name)}/documents`)}
                      >
                        Documents
                      </Button>
                      <Button
                        size="xs"
                        variant="light"
                        color="blue"
                        onClick={() => router.push(`/namespaces/${encodeURIComponent(ns.name)}/query`)}
                      >
                        Query
                      </Button>
                      <Button
                        size="xs"
                        variant="light"
                        color="red"
                        onClick={() => handleDelete(ns.name)}
                      >
                        Delete
                      </Button>
                    </Group>
                  </Table.Td>
                </Table.Tr>
              ))}
            </Table.Tbody>
          </Table>
        )}
      </Box>
    </div>
  );
}
