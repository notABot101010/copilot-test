import { signal } from '@preact/signals';
import { useEffect } from 'preact/hooks';
import { Button, Card, Group, Text, Title, Table, TextInput, Stack, LoadingOverlay, Box, Code, CopyButton, Alert } from '@mantine/core';
import { listApiKeys, createApiKey, deleteApiKey } from '../api';
import type { ApiKey, CreateApiKeyResponse } from '../api';
import type { JSX } from 'preact';

const apiKeys = signal<ApiKey[]>([]);
const loading = signal(true);
const error = signal<string | null>(null);
const backendUnavailable = signal(false);
const newKeyName = signal('');
const createdKey = signal<CreateApiKeyResponse | null>(null);

export function ApiKeysPage() {
  const loadApiKeys = async () => {
    loading.value = true;
    error.value = null;
    backendUnavailable.value = false;
    try {
      const data = await listApiKeys();
      apiKeys.value = data;
    } catch (e) {
      const errMsg = e instanceof Error ? e.message : 'Failed to load API keys';
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
    loadApiKeys();
  }, []);

  const handleCreate = async () => {
    if (!newKeyName.value.trim()) {
      alert('Please enter a name for the API key');
      return;
    }

    try {
      const result = await createApiKey(newKeyName.value.trim());
      createdKey.value = result;
      newKeyName.value = '';
      await loadApiKeys();
    } catch (e) {
      alert(e instanceof Error ? e.message : 'Failed to create API key');
    }
  };

  const handleDelete = async (id: number) => {
    if (confirm('Are you sure you want to delete this API key?')) {
      try {
        await deleteApiKey(id);
        await loadApiKeys();
      } catch (e) {
        alert(e instanceof Error ? e.message : 'Failed to delete API key');
      }
    }
  };

  return (
    <div>
      <Title order={2} mb="lg">API Keys</Title>

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

      {createdKey.value && (
        <Card withBorder mb="md" className="bg-green-50">
          <Stack gap="sm">
            <Text fw={500} c="green">API Key Created Successfully!</Text>
            <Text size="sm" c="dimmed">
              Copy this key now. You won't be able to see it again.
            </Text>
            <Group>
              <Code className="flex-1">{createdKey.value.key}</Code>
              <CopyButton value={createdKey.value.key}>
                {({ copied, copy }) => (
                  <Button size="xs" color={copied ? 'teal' : 'blue'} onClick={copy}>
                    {copied ? 'Copied!' : 'Copy'}
                  </Button>
                )}
              </CopyButton>
            </Group>
            <Button size="xs" variant="subtle" onClick={() => createdKey.value = null}>
              Dismiss
            </Button>
          </Stack>
        </Card>
      )}

      <Card withBorder mb="md">
        <Title order={4} mb="md">Create New API Key</Title>
        <Group>
          <TextInput
            placeholder="Key name (e.g., 'Production')"
            value={newKeyName.value}
            onChange={(e: JSX.TargetedEvent<HTMLInputElement>) => newKeyName.value = e.currentTarget.value}
            style={{ flex: 1 }}
          />
          <Button onClick={handleCreate}>Create Key</Button>
        </Group>
      </Card>

      <Box pos="relative">
        <LoadingOverlay visible={loading.value} />
        
        {apiKeys.value.length === 0 && !loading.value ? (
          <Card withBorder>
            <Text c="dimmed" ta="center">
              No API keys found. Create one above to secure your API.
            </Text>
            <Text c="dimmed" ta="center" size="sm" mt="xs">
              Note: If no API keys exist, the API is open to all requests.
            </Text>
          </Card>
        ) : (
          <Table striped highlightOnHover>
            <Table.Thead>
              <Table.Tr>
                <Table.Th>ID</Table.Th>
                <Table.Th>Name</Table.Th>
                <Table.Th>Created At</Table.Th>
                <Table.Th>Last Used</Table.Th>
                <Table.Th>Actions</Table.Th>
              </Table.Tr>
            </Table.Thead>
            <Table.Tbody>
              {apiKeys.value.map((key) => (
                <Table.Tr key={key.id}>
                  <Table.Td>{key.id}</Table.Td>
                  <Table.Td>{key.name}</Table.Td>
                  <Table.Td>{new Date(parseInt(key.created_at) * 1000).toLocaleString()}</Table.Td>
                  <Table.Td>
                    {key.last_used_at 
                      ? new Date(parseInt(key.last_used_at) * 1000).toLocaleString()
                      : 'Never'}
                  </Table.Td>
                  <Table.Td>
                    <Button
                      size="xs"
                      variant="light"
                      color="red"
                      onClick={() => handleDelete(key.id)}
                    >
                      Delete
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
