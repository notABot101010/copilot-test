import { signal, computed } from '@preact/signals';
import { useEffect } from 'preact/hooks';
import { useRoute } from '@copilot-test/preact-router';
import { Button, Table, Modal, Text, Paper, Title, Alert, Breadcrumbs, Anchor, Group, FileInput, Progress } from '@mantine/core';
import { listObjects, uploadObject, deleteObject, type S3Object, formatBytes, formatDate } from '../api';

interface ObjectBrowserState {
  bucket: string;
  prefix: string;
  objects: S3Object[];
  commonPrefixes: string[];
  loading: boolean;
  error: string | null;
  uploadModalOpen: boolean;
  uploadFile: File | null;
  uploading: boolean;
  uploadProgress: number;
}

const state = signal<ObjectBrowserState>({
  bucket: '',
  prefix: '',
  objects: [],
  commonPrefixes: [],
  loading: true,
  error: null,
  uploadModalOpen: false,
  uploadFile: null,
  uploading: false,
  uploadProgress: 0,
});

const breadcrumbItems = computed(() => {
  const items: { title: string; href: string }[] = [];
  items.push({ title: 'Buckets', href: '/' });
  items.push({ title: state.value.bucket, href: `/bucket/${state.value.bucket}` });

  if (state.value.prefix) {
    const parts = state.value.prefix.split('/').filter(Boolean);
    let path = '';
    for (const part of parts) {
      path += part + '/';
      items.push({
        title: part,
        href: `/bucket/${state.value.bucket}?prefix=${encodeURIComponent(path)}`,
      });
    }
  }

  return items;
});

async function loadObjects(bucket: string, prefix: string) {
  state.value = { ...state.value, loading: true, error: null, bucket, prefix };
  try {
    const result = await listObjects(bucket, prefix, '/');
    state.value = {
      ...state.value,
      objects: result.contents,
      commonPrefixes: result.commonPrefixes,
      loading: false,
    };
  } catch (err) {
    state.value = {
      ...state.value,
      error: err instanceof Error ? err.message : 'Failed to load objects',
      loading: false,
    };
  }
}

async function handleUpload() {
  const file = state.value.uploadFile;
  if (!file) return;

  state.value = { ...state.value, uploading: true, uploadProgress: 0, error: null };

  try {
    const key = state.value.prefix + file.name;
    await uploadObject(state.value.bucket, key, file, (progress) => {
      state.value = { ...state.value, uploadProgress: progress };
    });
    state.value = { ...state.value, uploadModalOpen: false, uploadFile: null, uploading: false };
    await loadObjects(state.value.bucket, state.value.prefix);
  } catch (err) {
    state.value = {
      ...state.value,
      error: err instanceof Error ? err.message : 'Failed to upload file',
      uploading: false,
    };
  }
}

async function handleDelete(key: string) {
  if (!confirm(`Are you sure you want to delete "${key}"?`)) return;

  state.value = { ...state.value, error: null };
  try {
    await deleteObject(state.value.bucket, key);
    await loadObjects(state.value.bucket, state.value.prefix);
  } catch (err) {
    state.value = {
      ...state.value,
      error: err instanceof Error ? err.message : 'Failed to delete object',
    };
  }
}

function getDisplayName(key: string, prefix: string): string {
  return key.slice(prefix.length);
}

export function ObjectBrowserPage() {
  const route = useRoute();
  const bucket = route.value.params.bucket as string;

  useEffect(() => {
    const urlParams = new URLSearchParams(window.location.search);
    const prefix = urlParams.get('prefix') || '';
    loadObjects(bucket, prefix);
  }, [bucket]);

  // Listen for URL changes
  useEffect(() => {
    const handlePopState = () => {
      const urlParams = new URLSearchParams(window.location.search);
      const prefix = urlParams.get('prefix') || '';
      loadObjects(bucket, prefix);
    };
    window.addEventListener('popstate', handlePopState);
    return () => window.removeEventListener('popstate', handlePopState);
  }, [bucket]);

  const { objects, commonPrefixes, loading, error, uploadModalOpen, uploadFile, uploading, uploadProgress, prefix } = state.value;

  return (
    <div>
      <Breadcrumbs className="mb-4">
        {breadcrumbItems.value.map((item, index) => (
          <Anchor href={item.href} key={index}>
            {item.title}
          </Anchor>
        ))}
      </Breadcrumbs>

      <div className="flex items-center justify-between mb-6">
        <Title order={2}>
          {prefix ? `${bucket}/${prefix}` : bucket}
        </Title>
        <Button onClick={() => (state.value = { ...state.value, uploadModalOpen: true })}>
          Upload File
        </Button>
      </div>

      {error && (
        <Alert color="red" className="mb-4" onClose={() => (state.value = { ...state.value, error: null })} withCloseButton>
          {error}
        </Alert>
      )}

      <Paper shadow="xs" p="md">
        {loading ? (
          <Text c="dimmed">Loading...</Text>
        ) : objects.length === 0 && commonPrefixes.length === 0 ? (
          <Text c="dimmed">No objects found. Upload a file to get started.</Text>
        ) : (
          <Table striped highlightOnHover>
            <Table.Thead>
              <Table.Tr>
                <Table.Th>Name</Table.Th>
                <Table.Th>Size</Table.Th>
                <Table.Th>Last Modified</Table.Th>
                <Table.Th style={{ width: 150 }}>Actions</Table.Th>
              </Table.Tr>
            </Table.Thead>
            <Table.Tbody>
              {/* Common prefixes (folders) */}
              {commonPrefixes.map((prefixPath) => (
                <Table.Tr key={prefixPath}>
                  <Table.Td>
                    <a
                      href={`/bucket/${bucket}?prefix=${encodeURIComponent(prefixPath)}`}
                      className="text-blue-600 hover:underline no-underline flex items-center gap-2"
                    >
                      <span className="text-gray-400">📁</span>
                      {getDisplayName(prefixPath, prefix)}
                    </a>
                  </Table.Td>
                  <Table.Td>-</Table.Td>
                  <Table.Td>-</Table.Td>
                  <Table.Td>-</Table.Td>
                </Table.Tr>
              ))}
              {/* Objects (files) */}
              {objects.map((obj) => (
                <Table.Tr key={obj.key}>
                  <Table.Td>
                    <a
                      href={`/api/${bucket}/${obj.key}`}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="text-blue-600 hover:underline no-underline flex items-center gap-2"
                    >
                      <span className="text-gray-400">📄</span>
                      {getDisplayName(obj.key, prefix)}
                    </a>
                  </Table.Td>
                  <Table.Td>{formatBytes(obj.size)}</Table.Td>
                  <Table.Td>{formatDate(obj.lastModified)}</Table.Td>
                  <Table.Td>
                    <Group gap="xs">
                      <a
                        href={`/api/${bucket}/${obj.key}`}
                        download
                        className="text-blue-600 hover:underline text-sm no-underline"
                      >
                        Download
                      </a>
                      <button
                        onClick={() => handleDelete(obj.key)}
                        className="text-red-600 hover:underline text-sm bg-transparent border-0 cursor-pointer p-0"
                      >
                        Delete
                      </button>
                    </Group>
                  </Table.Td>
                </Table.Tr>
              ))}
            </Table.Tbody>
          </Table>
        )}
      </Paper>

      <Modal
        opened={uploadModalOpen}
        onClose={() => (state.value = { ...state.value, uploadModalOpen: false })}
        title="Upload File"
      >
        <FileInput
          label="Select file"
          placeholder="Choose a file"
          value={uploadFile}
          onChange={(file) => (state.value = { ...state.value, uploadFile: file })}
          className="mb-4"
        />
        {uploading && (
          <Progress value={uploadProgress} className="mb-4" />
        )}
        <Group justify="flex-end">
          <Button variant="subtle" onClick={() => (state.value = { ...state.value, uploadModalOpen: false })}>
            Cancel
          </Button>
          <Button onClick={handleUpload} loading={uploading} disabled={!uploadFile}>
            Upload
          </Button>
        </Group>
      </Modal>
    </div>
  );
}
