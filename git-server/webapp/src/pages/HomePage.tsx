import { useSignal, useSignalEffect } from '@preact/signals';
import { Card, Text, Loader, Alert } from '@mantine/core';
import { listRepos, type RepoInfo } from '../api';

export function HomePage() {
  const repos = useSignal<RepoInfo[]>([]);
  const loading = useSignal(true);
  const error = useSignal<string | null>(null);

  useSignalEffect(() => {
    loadRepos();
  });

  async function loadRepos() {
    try {
      loading.value = true;
      error.value = null;
      const data = await listRepos();
      repos.value = Array.isArray(data) ? data : [];
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load repositories';
      repos.value = [];
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

  return (
    <Card shadow="sm" padding="lg" radius="md" withBorder>
      <div class="border-b border-gray-200 pb-4 mb-4">
        <Text size="xl" fw={600}>
          Repositories
        </Text>
      </div>

      {repos.value.length === 0 ? (
        <div class="text-center py-8 text-gray-500">
          <Text size="lg">No repositories found</Text>
          <Text size="sm" c="dimmed" mt="sm">
            Create a new repository to get started
          </Text>
        </div>
      ) : (
        <ul class="divide-y divide-gray-200">
          {repos.value.map((repo) => (
            <li key={repo.name} class="py-4">
              <a
                href={`/repos/${encodeURIComponent(repo.name)}`}
                class="text-blue-600 font-semibold hover:underline"
              >
                {repo.name}
              </a>
              <Text size="sm" c="dimmed" mt="xs">
                {repo.path}
              </Text>
            </li>
          ))}
        </ul>
      )}
    </Card>
  );
}
