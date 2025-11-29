import { useSignal, useSignalEffect } from '@preact/signals';
import { Card, Text, Loader, Alert, Badge, Button, Group, SimpleGrid } from '@mantine/core';
import { useRoute, useRouter } from '@copilot-test/preact-router';
import { getOrganization, listProjects, type Organization, type Project } from '../api';

export function OrgPage() {
  const route = useRoute();
  const router = useRouter();
  const org = useSignal<Organization | null>(null);
  const projects = useSignal<Project[]>([]);
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
      const [orgData, projectsData] = await Promise.all([
        getOrganization(orgName),
        listProjects(orgName),
      ]);
      org.value = orgData;
      projects.value = Array.isArray(projectsData) ? projectsData : [];
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
              onClick={() => router.push(`/${orgName}/projects/new`)}
            >
              + New Project
            </Button>
          </Group>
        </Group>
      </Card>

      <Text size="lg" fw={600} mb="md">Projects</Text>

      {projects.value.length === 0 ? (
        <Card shadow="sm" padding="xl" radius="md" withBorder>
          <div class="text-center py-8">
            <Text size="lg" fw={500} mb="xs">No projects yet</Text>
            <Text size="sm" c="dimmed" mb="lg">
              Create your first project in this organization
            </Text>
            <Button
              variant="filled"
              color="blue"
              onClick={() => router.push(`/${orgName}/projects/new`)}
            >
              Create Project
            </Button>
          </div>
        </Card>
      ) : (
        <SimpleGrid cols={{ base: 1, sm: 2 }}>
          {projects.value.map((project) => (
            <Card
              key={project.name}
              shadow="sm"
              padding="lg"
              radius="md"
              withBorder
              component="a"
              href={`/${orgName}/${project.name}`}
              onClick={(e: Event) => {
                e.preventDefault();
                router.push(`/${orgName}/${project.name}`);
              }}
              style={{ cursor: 'pointer' }}
            >
              <Group justify="space-between" mb="xs">
                <Text fw={600}>
                  📦 {project.display_name}
                </Text>
                <Badge color="gray" variant="light" size="sm">
                  @{project.name}
                </Badge>
              </Group>
              {project.description && (
                <Text size="xs" c="dimmed">
                  {project.description}
                </Text>
              )}
            </Card>
          ))}
        </SimpleGrid>
      )}
    </div>
  );
}
