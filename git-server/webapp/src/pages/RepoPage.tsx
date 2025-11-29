import { useSignal, useSignalEffect } from '@preact/signals';
import { Card, Text, Loader, Alert, Tabs, Badge, Breadcrumbs, Anchor, Button, Group, CopyButton, ActionIcon, Tooltip, Modal, Code } from '@mantine/core';
import { useDisclosure } from '@mantine/hooks';
import { useRoute, useRouter } from '@copilot-test/preact-router';
import {
  getRepoTree,
  getRepoCommits,
  getRepo,
  type FileEntry,
  type CommitInfo,
  type RepoInfo,
  formatSize,
  formatDate,
} from '../api';

export function RepoPage() {
  const route = useRoute();
  const router = useRouter();
  const repo = useSignal<RepoInfo | null>(null);
  const files = useSignal<FileEntry[]>([]);
  const commits = useSignal<CommitInfo[]>([]);
  const loading = useSignal(true);
  const error = useSignal<string | null>(null);
  const activeTab = useSignal<string>('files');
  const [cloneModalOpened, { open: openCloneModal, close: closeCloneModal }] = useDisclosure(false);

  // Get query params
  const params = route.value.params;
  const query = route.value.query as { ref?: string; path?: string };
  const orgName = params.org as string;
  const repoName = params.name as string;
  const gitRef = (query.ref as string) || 'HEAD';
  const currentPath = (query.path as string) || '';

  // Clone URLs
  const sshCloneUrl = `git@localhost:${orgName}/${repoName}.git`;
  const httpCloneUrl = `http://localhost:3000/${orgName}/${repoName}.git`;

  useSignalEffect(() => {
    loadData();
  });

  async function loadData() {
    try {
      loading.value = true;
      error.value = null;
      const [repoData, filesData, commitsData] = await Promise.all([
        getRepo(orgName, repoName),
        getRepoTree(orgName, repoName, gitRef, currentPath),
        getRepoCommits(orgName, repoName),
      ]);
      repo.value = repoData;
      files.value = filesData;
      commits.value = commitsData;
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load repository';
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
  const breadcrumbItems = [];
  if (currentPath) {
    const parts = currentPath.split('/');
    breadcrumbItems.push(
      <Anchor
        key="root"
        href={`/${orgName}/${repoName}?ref=${encodeURIComponent(gitRef)}`}
        onClick={(e: Event) => {
          e.preventDefault();
          router.push(`/${orgName}/${repoName}?ref=${encodeURIComponent(gitRef)}`);
        }}
      >
        {repoName}
      </Anchor>
    );
    parts.forEach((part, i) => {
      const partPath = parts.slice(0, i + 1).join('/');
      if (i === parts.length - 1) {
        breadcrumbItems.push(<Text key={partPath}>{part}</Text>);
      } else {
        breadcrumbItems.push(
          <Anchor
            key={partPath}
            href={`/${orgName}/${repoName}?ref=${encodeURIComponent(gitRef)}&path=${encodeURIComponent(partPath)}`}
            onClick={(e: Event) => {
              e.preventDefault();
              router.push(`/${orgName}/${repoName}?ref=${encodeURIComponent(gitRef)}&path=${encodeURIComponent(partPath)}`);
            }}
          >
            {part}
          </Anchor>
        );
      }
    });
  }

  return (
    <>
      <Modal opened={cloneModalOpened} onClose={closeCloneModal} title="Clone Repository" size="lg">
        <Text size="sm" fw={500} mb="xs">SSH</Text>
        <Group mb="md">
          <Code block style={{ flex: 1 }}>git clone {sshCloneUrl}</Code>
          <CopyButton value={`git clone ${sshCloneUrl}`}>
            {({ copied, copy }) => (
              <Tooltip label={copied ? 'Copied!' : 'Copy'}>
                <ActionIcon color={copied ? 'teal' : 'gray'} onClick={copy}>
                  {copied ? '✓' : '📋'}
                </ActionIcon>
              </Tooltip>
            )}
          </CopyButton>
        </Group>
        
        <Text size="sm" fw={500} mb="xs">HTTP</Text>
        <Group>
          <Code block style={{ flex: 1 }}>git clone {httpCloneUrl}</Code>
          <CopyButton value={`git clone ${httpCloneUrl}`}>
            {({ copied, copy }) => (
              <Tooltip label={copied ? 'Copied!' : 'Copy'}>
                <ActionIcon color={copied ? 'teal' : 'gray'} onClick={copy}>
                  {copied ? '✓' : '📋'}
                </ActionIcon>
              </Tooltip>
            )}
          </CopyButton>
        </Group>
      </Modal>

      <Card shadow="sm" padding="lg" radius="md" withBorder>
        <div class="border-b border-gray-200 pb-4 mb-4">
          <Group justify="space-between">
            <Group gap="xs">
              <Text fw={600} size="lg">📁 {repoName}</Text>
              {gitRef !== 'HEAD' && (
                <Badge color="blue" variant="light">
                  {gitRef.substring(0, 7)}
                </Badge>
              )}
              {repo.value?.forked_from && (
                <Badge color="gray" variant="light">
                  Forked from {repo.value.forked_from}
                </Badge>
              )}
            </Group>
            <Group gap="xs">
              <Button
                variant="outline"
                size="xs"
                onClick={openCloneModal}
              >
                📥 Clone
              </Button>
              <Button
                variant="outline"
                size="xs"
                onClick={() => router.push(`/${orgName}/${repoName}/new-file`)}
              >
                ➕ New File
              </Button>
              <Button
                variant="outline"
                size="xs"
                onClick={() => router.push(`/${orgName}/${repoName}/fork`)}
              >
                🍴 Fork
              </Button>
            </Group>
          </Group>
        </div>

        <Tabs value={activeTab.value} onChange={(value: string | null) => (activeTab.value = value || 'files')}>
          <Tabs.List>
            <Tabs.Tab value="files">📁 Files</Tabs.Tab>
            <Tabs.Tab value="commits">📝 Commits</Tabs.Tab>
          </Tabs.List>

          <Tabs.Panel value="files" pt="md">
            {currentPath && (
              <div class="bg-gray-50 px-4 py-3 border-b border-gray-200 mb-4 -mx-4 -mt-4">
                <Breadcrumbs>{breadcrumbItems}</Breadcrumbs>
              </div>
            )}

            {files.value.length === 0 && !currentPath ? (
              <div class="text-center py-8 text-gray-500">
                <Text size="lg" mb="md">No files in this repository</Text>
                <Button
                  onClick={() => router.push(`/${orgName}/${repoName}/new-file`)}
                >
                  Create your first file
                </Button>
              </div>
            ) : (
              <ul class="divide-y divide-gray-200">
                {currentPath && (
                  <li class="py-2 flex items-center gap-3">
                    <span class="w-5 text-center">📁</span>
                    <Anchor
                      href={`/${orgName}/${repoName}?ref=${encodeURIComponent(gitRef)}&path=${encodeURIComponent(currentPath.split('/').slice(0, -1).join('/'))}`}
                      onClick={(e: Event) => {
                        e.preventDefault();
                        router.push(`/${orgName}/${repoName}?ref=${encodeURIComponent(gitRef)}&path=${encodeURIComponent(currentPath.split('/').slice(0, -1).join('/'))}`);
                      }}
                    >
                      ..
                    </Anchor>
                  </li>
                )}
                {files.value.map((file) => (
                  <li key={file.path} class="py-2 flex items-center gap-3">
                    <span class="w-5 text-center">{file.type === 'dir' ? '📁' : '📄'}</span>
                    <Anchor
                      href={
                        file.type === 'dir'
                          ? `/${orgName}/${repoName}?ref=${encodeURIComponent(gitRef)}&path=${encodeURIComponent(file.path)}`
                          : `/${orgName}/${repoName}/blob/${file.path}?ref=${encodeURIComponent(gitRef)}`
                      }
                      onClick={(e: Event) => {
                        e.preventDefault();
                        if (file.type === 'dir') {
                          router.push(`/${orgName}/${repoName}?ref=${encodeURIComponent(gitRef)}&path=${encodeURIComponent(file.path)}`);
                        } else {
                          router.push(`/${orgName}/${repoName}/blob/${file.path}?ref=${encodeURIComponent(gitRef)}`);
                        }
                      }}
                    >
                      {file.name}
                    </Anchor>
                    {file.size !== null && (
                      <span class="ml-auto text-sm text-gray-500">{formatSize(file.size)}</span>
                    )}
                    {file.type === 'file' && (
                      <Button
                        variant="subtle"
                        size="xs"
                        onClick={() => router.push(`/${orgName}/${repoName}/edit/${file.path}?ref=${encodeURIComponent(gitRef)}`)}
                      >
                        Edit
                      </Button>
                    )}
                  </li>
                ))}
              </ul>
            )}
          </Tabs.Panel>

          <Tabs.Panel value="commits" pt="md">
            {commits.value.length === 0 ? (
              <div class="text-center py-8 text-gray-500">
                <Text size="lg">No commits yet</Text>
              </div>
            ) : (
              <ul class="divide-y divide-gray-200">
                {commits.value.map((commit) => (
                  <li key={commit.hash} class="py-3">
                    <div class="flex items-start gap-3">
                      <Anchor
                        href={`/${orgName}/${repoName}?ref=${encodeURIComponent(commit.hash)}`}
                        class="font-mono bg-gray-100 px-2 py-1 rounded text-sm text-blue-600 hover:bg-blue-50"
                        onClick={(e: Event) => {
                          e.preventDefault();
                          router.push(`/${orgName}/${repoName}?ref=${encodeURIComponent(commit.hash)}`);
                        }}
                      >
                        {commit.short_hash}
                      </Anchor>
                      <div>
                        <Text fw={500}>{commit.message}</Text>
                        <Text size="sm" c="dimmed">
                          {commit.author} committed on {formatDate(commit.date)}
                        </Text>
                      </div>
                    </div>
                  </li>
                ))}
              </ul>
            )}
          </Tabs.Panel>
        </Tabs>
      </Card>
    </>
  );
}
