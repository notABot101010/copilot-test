import { useRouter, useRoute } from '@copilot-test/preact-router';
import { signal } from '@preact/signals';
import { useEffect } from 'preact/hooks';
import { Button, Card, Group, Text, Title, Table, Badge, LoadingOverlay, Box, Breadcrumbs, Anchor, Checkbox } from '@mantine/core';
import { getDocuments, deleteDocuments } from '../api';
import type { Document } from '../api';

const documents = signal<Document[]>([]);
const loading = signal(true);
const error = signal<string | null>(null);
const selectedIds = signal<Set<string>>(new Set());

export function DocumentsPage() {
  const router = useRouter();
  const route = useRoute();
  const params = route.value.params;
  const namespaceName = decodeURIComponent((params.namespace as string) || '');

  const loadDocuments = async () => {
    if (!namespaceName) return;
    
    loading.value = true;
    error.value = null;
    selectedIds.value = new Set();
    try {
      const data = await getDocuments(namespaceName);
      documents.value = data;
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load documents';
    } finally {
      loading.value = false;
    }
  };

  useEffect(() => {
    loadDocuments();
  }, [namespaceName]);

  const toggleSelection = (id: string) => {
    const newSelected = new Set(selectedIds.value);
    if (newSelected.has(id)) {
      newSelected.delete(id);
    } else {
      newSelected.add(id);
    }
    selectedIds.value = newSelected;
  };

  const selectAll = () => {
    if (selectedIds.value.size === documents.value.length) {
      selectedIds.value = new Set();
    } else {
      selectedIds.value = new Set(documents.value.map(d => d.id));
    }
  };

  const handleDeleteSelected = async () => {
    const ids = Array.from(selectedIds.value);
    if (ids.length === 0) return;
    
    if (confirm(`Are you sure you want to delete ${ids.length} document(s)?`)) {
      try {
        await deleteDocuments(namespaceName, ids);
        await loadDocuments();
      } catch (e) {
        alert(e instanceof Error ? e.message : 'Failed to delete documents');
      }
    }
  };

  const getDocumentAttributes = (doc: Document): string[] => {
    return Object.keys(doc).filter(k => k !== 'id' && k !== 'vector');
  };

  const breadcrumbs = [
    { title: 'Namespaces', href: '/' },
    { title: namespaceName, href: `/namespaces/${encodeURIComponent(namespaceName)}` },
    { title: 'Documents', href: '#' },
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
        <Title order={2}>Documents</Title>
        <Group>
          {selectedIds.value.size > 0 && (
            <Button color="red" onClick={handleDeleteSelected}>
              Delete Selected ({selectedIds.value.size})
            </Button>
          )}
          <Button variant="light" onClick={loadDocuments}>
            Refresh
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
        
        {documents.value.length === 0 && !loading.value ? (
          <Card withBorder>
            <Text c="dimmed" ta="center">No documents found in this namespace.</Text>
          </Card>
        ) : (
          <Table striped highlightOnHover>
            <Table.Thead>
              <Table.Tr>
                <Table.Th w={40}>
                  <Checkbox
                    checked={selectedIds.value.size === documents.value.length && documents.value.length > 0}
                    indeterminate={selectedIds.value.size > 0 && selectedIds.value.size < documents.value.length}
                    onChange={selectAll}
                  />
                </Table.Th>
                <Table.Th>ID</Table.Th>
                <Table.Th>Has Vector</Table.Th>
                <Table.Th>Attributes</Table.Th>
                <Table.Th>Actions</Table.Th>
              </Table.Tr>
            </Table.Thead>
            <Table.Tbody>
              {documents.value.map((doc) => (
                <Table.Tr key={doc.id}>
                  <Table.Td>
                    <Checkbox
                      checked={selectedIds.value.has(doc.id)}
                      onChange={() => toggleSelection(doc.id)}
                    />
                  </Table.Td>
                  <Table.Td>
                    <Text
                      component="a"
                      href={`/namespaces/${encodeURIComponent(namespaceName)}/documents/${encodeURIComponent(doc.id)}`}
                      onClick={(e: Event) => {
                        e.preventDefault();
                        router.push(`/namespaces/${encodeURIComponent(namespaceName)}/documents/${encodeURIComponent(doc.id)}`);
                      }}
                      className="text-blue-600 hover:underline cursor-pointer font-mono"
                    >
                      {doc.id}
                    </Text>
                  </Table.Td>
                  <Table.Td>
                    {doc.vector ? (
                      <Badge color="green">Yes ({doc.vector.length}D)</Badge>
                    ) : (
                      <Badge color="gray">No</Badge>
                    )}
                  </Table.Td>
                  <Table.Td>
                    <Group gap="xs">
                      {getDocumentAttributes(doc).slice(0, 3).map((attr) => (
                        <Badge key={attr} variant="outline" size="sm">
                          {attr}
                        </Badge>
                      ))}
                      {getDocumentAttributes(doc).length > 3 && (
                        <Text size="xs" c="dimmed">+{getDocumentAttributes(doc).length - 3} more</Text>
                      )}
                    </Group>
                  </Table.Td>
                  <Table.Td>
                    <Button
                      size="xs"
                      variant="light"
                      onClick={() => router.push(`/namespaces/${encodeURIComponent(namespaceName)}/documents/${encodeURIComponent(doc.id)}`)}
                    >
                      View
                    </Button>
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
