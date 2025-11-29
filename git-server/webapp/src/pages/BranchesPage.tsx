import { useSignal, useSignalEffect } from '@preact/signals';
import { Card, Text, Loader, Alert, Badge, Group, Anchor } from '@mantine/core';
import { useRoute, useRouter } from '@copilot-test/preact-router';
import { getRepoBranches } from '../api';

export function BranchesPage() {
  const route = useRoute();
  const router = useRouter();
  const branches = useSignal<string[]>([]);
  const loading = useSignal(true);
  const error = useSignal<string | null>(null);

  const params = route.value.params;
  const orgName = params.org as string;
  const projectName = params.project as string;
  const repoName = params.name as string;

  useSignalEffect(() => {
    loadBranches();
  });

  async function loadBranches() {
    try {
      loading.value = true;
      error.value = null;
      const data = await getRepoBranches(orgName, projectName, repoName);
      branches.value = Array.isArray(data) ? data : [];
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load branches';
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
      <Text size="xl" fw={600} mb="lg">
        🌿 Branches
      </Text>

      {branches.value.length === 0 ? (
        <div class="text-center py-8 text-gray-500">
          <Text size="lg">No branches yet</Text>
          <Text size="sm" c="dimmed" mt="sm">
            Create your first commit to see branches
          </Text>
        </div>
      ) : (
        <ul class="divide-y divide-gray-200">
          {branches.value.map((branch) => (
            <li key={branch} class="py-3">
              <Group justify="space-between">
                <Anchor
                  href={`/${orgName}/${projectName}/${repoName}?ref=${encodeURIComponent(branch)}`}
                  onClick={(e: Event) => {
                    e.preventDefault();
                    router.push(`/${orgName}/${projectName}/${repoName}?ref=${encodeURIComponent(branch)}`);
                  }}
                >
                  <Group gap="xs">
                    <span>🌿</span>
                    <Text fw={500}>{branch}</Text>
                  </Group>
                </Anchor>
                {branch === 'main' && (
                  <Badge color="green" variant="light">
                    default
                  </Badge>
                )}
              </Group>
            </li>
          ))}
        </ul>
      )}
    </Card>
  );
}
