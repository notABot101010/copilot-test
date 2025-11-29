import { useSignal, useSignalEffect } from '@preact/signals';
import { Card, Text, Loader, Alert, Tabs, Badge, Breadcrumbs, Anchor, Button } from '@mantine/core';
import { useRoute } from '@copilot-test/preact-router';
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
  const repo = useSignal<RepoInfo | null>(null);
  const files = useSignal<FileEntry[]>([]);
  const commits = useSignal<CommitInfo[]>([]);
  const loading = useSignal(true);
  const error = useSignal<string | null>(null);
  const activeTab = useSignal<string>('files');

  // Get query params
  const params = route.value.params;
  const query = route.value.query as { ref?: string; path?: string };
  const repoName = params.name as string;
  const gitRef = (query.ref as string) || 'HEAD';
  const currentPath = (query.path as string) || '';

  useSignalEffect(() => {
    loadData();
  });

  async function loadData() {
    try {
      loading.value = true;
      error.value = null;
      const [repoData, filesData, commitsData] = await Promise.all([
        getRepo(repoName),
        getRepoTree(repoName, gitRef, currentPath),
        getRepoCommits(repoName),
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
        href={`/repos/${encodeURIComponent(repoName)}?ref=${encodeURIComponent(gitRef)}`}
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
            href={`/repos/${encodeURIComponent(repoName)}?ref=${encodeURIComponent(
              gitRef
            )}&path=${encodeURIComponent(partPath)}`}
          >
            {part}
          </Anchor>
        );
      }
    });
  }

  return (
    <Card shadow="sm" padding="lg" radius="md" withBorder>
      <div class="border-b border-gray-200 pb-4 mb-4">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-3">
            <Anchor href="/" c="blue">
              Repositories
            </Anchor>
            <span class="text-gray-400">/</span>
            <Text fw={600}>{repoName}</Text>
            {gitRef !== 'HEAD' && (
              <Badge color="blue" variant="light">
                {gitRef.substring(0, 7)}
              </Badge>
            )}
            {repo.value?.forked_from && (
              <Badge color="gray" variant="light">
                Forked from{' '}
                <Anchor href={`/repos/${encodeURIComponent(repo.value.forked_from)}`}>
                  {repo.value.forked_from}
                </Anchor>
              </Badge>
            )}
          </div>
          <div class="flex gap-2">
            <Button
              variant="outline"
              size="sm"
              component="a"
              href={`/repos/${encodeURIComponent(repoName)}/issues`}
            >
              🐛 Issues
            </Button>
            <Button
              variant="outline"
              size="sm"
              component="a"
              href={`/repos/${encodeURIComponent(repoName)}/pulls`}
            >
              🔀 Pull Requests
            </Button>
            <Button
              variant="outline"
              size="sm"
              component="a"
              href={`/repos/${encodeURIComponent(repoName)}/fork`}
            >
              🍴 Fork
            </Button>
          </div>
        </div>
      </div>

      <Tabs value={activeTab.value} onChange={(value: string | null) => (activeTab.value = value || 'files')}>
        <Tabs.List>
          <Tabs.Tab value="files">📁 Files</Tabs.Tab>
          <Tabs.Tab value="commits">📝 Commits</Tabs.Tab>
        </Tabs.List>

        <Tabs.Panel value="files" pt="md">
          {currentPath && (
            <div class="bg-gray-50 px-4 py-3 border-b border-gray-200 mb-0 -mx-4 -mt-4">
              <Breadcrumbs>{breadcrumbItems}</Breadcrumbs>
            </div>
          )}

          {files.value.length === 0 && !currentPath ? (
            <div class="text-center py-8 text-gray-500">
              <Text size="lg">No files in this repository</Text>
            </div>
          ) : (
            <ul class="divide-y divide-gray-200">
              {currentPath && (
                <li class="py-2 flex items-center gap-3">
                  <span class="w-5 text-center">📁</span>
                  <Anchor
                    href={`/repos/${encodeURIComponent(repoName)}?ref=${encodeURIComponent(
                      gitRef
                    )}&path=${encodeURIComponent(currentPath.split('/').slice(0, -1).join('/'))}`}
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
                        ? `/repos/${encodeURIComponent(repoName)}?ref=${encodeURIComponent(
                            gitRef
                          )}&path=${encodeURIComponent(file.path)}`
                        : `/repos/${encodeURIComponent(repoName)}/blob/${encodeURIComponent(
                            file.path
                          )}?ref=${encodeURIComponent(gitRef)}`
                    }
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
                      component="a"
                      href={`/repos/${encodeURIComponent(repoName)}/edit/${encodeURIComponent(
                        file.path
                      )}?ref=${encodeURIComponent(gitRef)}`}
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
                      href={`/repos/${encodeURIComponent(repoName)}?ref=${encodeURIComponent(
                        commit.hash
                      )}`}
                      class="font-mono bg-gray-100 px-2 py-1 rounded text-sm text-blue-600 hover:bg-blue-50"
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
  );
}
