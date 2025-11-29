import { useSignal, useSignalEffect } from '@preact/signals';
import { Card, Text, Loader, Alert, Button, Group, SimpleGrid, Badge } from '@mantine/core';
import { useRouter } from '@copilot-test/preact-router';
import { listOrganizations, type Organization } from '../api';

export function OrganizationsPage() {
  const router = useRouter();
  const orgs = useSignal<Organization[]>([]);
  const loading = useSignal(true);
  const error = useSignal<string | null>(null);

  useSignalEffect(() => {
    loadOrgs();
  });

  async function loadOrgs() {
    try {
      loading.value = true;
      error.value = null;
      const data = await listOrganizations();
      orgs.value = Array.isArray(data) ? data : [];
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load organizations';
      orgs.value = [];
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
    <div>
      <Group justify="space-between" mb="lg">
        <div>
          <Text size="xl" fw={700}>Organizations</Text>
          <Text size="sm" c="dimmed">Manage your organizations and repositories</Text>
        </div>
        <Button
          color="green"
          onClick={() => router.push('/new-org')}
        >
          + New Organization
        </Button>
      </Group>

      {orgs.value.length === 0 ? (
        <Card shadow="sm" padding="xl" radius="md" withBorder>
          <div class="text-center py-8">
            <Text size="lg" fw={500} mb="xs">No organizations yet</Text>
            <Text size="sm" c="dimmed" mb="lg">
              Create your first organization to get started
            </Text>
            <Button
              variant="filled"
              color="blue"
              onClick={() => router.push('/new-org')}
            >
              Create Organization
            </Button>
          </div>
        </Card>
      ) : (
        <SimpleGrid cols={{ base: 1, sm: 2, lg: 3 }}>
          {orgs.value.map((org) => (
            <Card
              key={org.name}
              shadow="sm"
              padding="lg"
              radius="md"
              withBorder
              component="a"
              href={`/${org.name}`}
              onClick={(e: Event) => {
                e.preventDefault();
                router.push(`/${org.name}`);
              }}
              style={{ cursor: 'pointer' }}
            >
              <Group justify="space-between" mb="xs">
                <Text fw={600} size="lg">
                  🏢 {org.display_name}
                </Text>
                <Badge color="blue" variant="light">
                  @{org.name}
                </Badge>
              </Group>
              {org.description && (
                <Text size="sm" c="dimmed" lineClamp={2}>
                  {org.description}
                </Text>
              )}
            </Card>
          ))}
        </SimpleGrid>
      )}
    </div>
  );
}
