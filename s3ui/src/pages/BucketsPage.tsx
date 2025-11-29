import { signal } from '@preact/signals';
import { useEffect } from 'preact/hooks';
import type { ChangeEvent } from 'preact/compat';
import { Button, TextInput, Modal, Table, ActionIcon, Group, Text, Paper, Title, Alert } from '@mantine/core';
import { listBuckets, createBucket, deleteBucket, type Bucket, formatDate } from '../api';

const buckets = signal<Bucket[]>([]);
const loading = signal(true);
const error = signal<string | null>(null);
const createModalOpen = signal(false);
const newBucketName = signal('');
const creating = signal(false);

async function loadBuckets() {
  loading.value = true;
  error.value = null;
  try {
    buckets.value = await listBuckets();
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Failed to load buckets';
  } finally {
    loading.value = false;
  }
}

async function handleCreateBucket() {
  if (!newBucketName.value.trim()) return;
  creating.value = true;
  error.value = null;
  try {
    await createBucket(newBucketName.value.trim());
    createModalOpen.value = false;
    newBucketName.value = '';
    await loadBuckets();
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Failed to create bucket';
  } finally {
    creating.value = false;
  }
}

async function handleDeleteBucket(name: string) {
  if (!confirm(`Are you sure you want to delete bucket "${name}"?`)) return;
  error.value = null;
  try {
    await deleteBucket(name);
    await loadBuckets();
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Failed to delete bucket';
  }
}

export function BucketsPage() {
  useEffect(() => {
    loadBuckets();
  }, []);

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <Title order={2}>Buckets</Title>
        <Button onClick={() => (createModalOpen.value = true)}>
          Create Bucket
        </Button>
      </div>

      {error.value && (
        <Alert color="red" className="mb-4" onClose={() => (error.value = null)} withCloseButton>
          {error.value}
        </Alert>
      )}

      <Paper shadow="xs" p="md">
        {loading.value ? (
          <Text c="dimmed">Loading...</Text>
        ) : buckets.value.length === 0 ? (
          <Text c="dimmed">No buckets found. Create one to get started.</Text>
        ) : (
          <Table striped highlightOnHover>
            <Table.Thead>
              <Table.Tr>
                <Table.Th>Name</Table.Th>
                <Table.Th>Created</Table.Th>
                <Table.Th style={{ width: 100 }}>Actions</Table.Th>
              </Table.Tr>
            </Table.Thead>
            <Table.Tbody>
              {buckets.value.map((bucket) => (
                <Table.Tr key={bucket.name}>
                  <Table.Td>
                    <a href={`/bucket/${bucket.name}`} className="text-blue-600 hover:underline no-underline">
                      {bucket.name}
                    </a>
                  </Table.Td>
                  <Table.Td>{formatDate(bucket.creationDate)}</Table.Td>
                  <Table.Td>
                    <Group gap="xs">
                      <ActionIcon
                        color="red"
                        variant="subtle"
                        onClick={() => handleDeleteBucket(bucket.name)}
                        title="Delete bucket"
                      >
                        <span className="text-sm">Delete</span>
                      </ActionIcon>
                    </Group>
                  </Table.Td>
                </Table.Tr>
              ))}
            </Table.Tbody>
          </Table>
        )}
      </Paper>

      <Modal
        opened={createModalOpen.value}
        onClose={() => (createModalOpen.value = false)}
        title="Create Bucket"
      >
        <TextInput
          label="Bucket Name"
          placeholder="my-bucket"
          value={newBucketName.value}
          onChange={(event: ChangeEvent<HTMLInputElement>) => (newBucketName.value = event.currentTarget.value)}
          className="mb-4"
        />
        <Group justify="flex-end">
          <Button variant="subtle" onClick={() => (createModalOpen.value = false)}>
            Cancel
          </Button>
          <Button onClick={handleCreateBucket} loading={creating.value}>
            Create
          </Button>
        </Group>
      </Modal>
    </div>
  );
}
