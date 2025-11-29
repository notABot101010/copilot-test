import { useSignal, useSignalEffect } from '@preact/signals';
import { Card, Text, Loader, Alert, Badge, Button, Group, SimpleGrid } from '@mantine/core';
import { useRoute, useRouter } from '@copilot-test/preact-router';
import { getOrganization, listRepos, type Organization, type RepoInfo } from '../api';

export function OrgPage() {
  const route = useRoute();
  const router = useRouter();
  const org = useSignal<Organization | null>(null);
  const repos = useSignal<RepoInfo[]>([]);
  const loading = useSignal(true);
  const error = useSignal<string | null>(null);

  const params = route.value.params;
  const orgName = params.org as string;

  useSignalEffect(() => {
    loadData();
  });

  async function loadData() {
    try {
      loading.value = true;
      error.value = null;
      const [orgData, reposData] = await Promise.all([
        getOrganization(orgName),
        listRepos(orgName),
      ]);
      org.value = orgData;
      repos.value = Array.isArray(reposData) ? reposData : [];
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load organization';
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

  if (!org.value) {
    return (
      <Alert color="red" title="Error">
        Organization not found
      </Alert>
    );
  }

  return (
    <div>
      <Card shadow="sm" padding="lg" radius="md" withBorder mb="lg">
        <Group justify="space-between">
          <div>
            <Group gap="sm">
              <Text size="xl" fw={700}>🏢 {org.value.display_name}</Text>
              <Badge color="blue" variant="light">@{org.value.name}</Badge>
            </Group>
            {org.value.description && (
              <Text size="sm" c="dimmed" mt="xs">
                {org.value.description}
              </Text>
            )}
          </div>
          <Group>
            <Button
              variant="outline"
              onClick={() => router.push(`/${orgName}/settings`)}
            >
              ⚙️ Settings
            </Button>
            <Button
              color="green"
              onClick={() => router.push(`/${orgName}/new`)}
            >
              + New Repository
            </Button>
          </Group>
        </Group>
      </Card>

      <Text size="lg" fw={600} mb="md">Repositories</Text>

      {repos.value.length === 0 ? (
        <Card shadow="sm" padding="xl" radius="md" withBorder>
          <div class="text-center py-8">
            <Text size="lg" fw={500} mb="xs">No repositories yet</Text>
            <Text size="sm" c="dimmed" mb="lg">
              Create your first repository in this organization
            </Text>
            <Button
              variant="filled"
              color="blue"
              onClick={() => router.push(`/${orgName}/new`)}
            >
              Create Repository
            </Button>
          </div>
        </Card>
      ) : (
        <SimpleGrid cols={{ base: 1, sm: 2 }}>
          {repos.value.map((repo) => (
            <Card
              key={repo.name}
              shadow="sm"
              padding="lg"
              radius="md"
              withBorder
              component="a"
              href={`/${orgName}/${repo.name}`}
              onClick={(e: Event) => {
                e.preventDefault();
                router.push(`/${orgName}/${repo.name}`);
              }}
              style={{ cursor: 'pointer' }}
            >
              <Group justify="space-between" mb="xs">
                <Text fw={600}>
                  📁 {repo.name}
                </Text>
                {repo.forked_from && (
                  <Badge color="gray" variant="light" size="sm">
                    Forked
                  </Badge>
                )}
              </Group>
              {repo.forked_from && (
                <Text size="xs" c="dimmed">
                  Forked from {repo.forked_from}
                </Text>
              )}
            </Card>
          ))}
        </SimpleGrid>
      )}
    </div>
  );
}
