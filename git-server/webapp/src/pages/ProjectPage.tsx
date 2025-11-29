import { useSignal, useSignalEffect } from '@preact/signals';
import { Card, Text, Loader, Alert, Button, Group, Title } from '@mantine/core';
import { useRoute, useRouter } from '@copilot-test/preact-router';
import { getProject, listRepos, type Project, type RepoInfo } from '../api';

export function ProjectPage() {
  const route = useRoute();
  const router = useRouter();
  const project = useSignal<Project | null>(null);
  const repos = useSignal<RepoInfo[]>([]);
  const loading = useSignal(true);
  const error = useSignal<string | null>(null);

  const params = route.value.params;
  const orgName = params.org as string;
  const projectName = params.project as string;

  useSignalEffect(() => {
    loadData();
  });

  async function loadData() {
    try {
      loading.value = true;
      error.value = null;
      const [projectData, reposData] = await Promise.all([
        getProject(orgName, projectName),
        listRepos(orgName, projectName),
      ]);
      project.value = projectData;
      repos.value = reposData;
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load project';
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
    <div class="space-y-6">
      <Card shadow="sm" padding="lg" radius="md" withBorder>
        <Group justify="space-between" mb="md">
          <div>
            <Title order={2}>📦 {project.value?.display_name}</Title>
            {project.value?.description && (
              <Text c="dimmed" size="sm" mt="xs">
                {project.value.description}
              </Text>
            )}
          </div>
          <Button
            variant="filled"
            onClick={() => router.push(`/${orgName}/${projectName}/new-repo`)}
          >
            ➕ New Repository
          </Button>
        </Group>
      </Card>

      <Card shadow="sm" padding="lg" radius="md" withBorder>
        <Title order={3} mb="md">Repositories</Title>
        
        {repos.value.length === 0 ? (
          <div class="text-center py-8">
            <Text c="dimmed" mb="md">No repositories in this project yet.</Text>
            <Button onClick={() => router.push(`/${orgName}/${projectName}/new-repo`)}>
              Create your first repository
            </Button>
          </div>
        ) : (
          <ul class="divide-y divide-gray-200">
            {repos.value.map((repo) => (
              <li key={repo.name} class="py-3">
                <Group justify="space-between">
                  <div>
                    <Text
                      component="a"
                      href={`/${orgName}/${projectName}/${repo.name}`}
                      fw={500}
                      c="blue"
                      style={{ cursor: 'pointer' }}
                      onClick={(e: Event) => {
                        e.preventDefault();
                        router.push(`/${orgName}/${projectName}/${repo.name}`);
                      }}
                    >
                      📁 {repo.name}
                    </Text>
                    {repo.forked_from && (
                      <Text size="xs" c="dimmed">
                        Forked from {repo.forked_from}
                      </Text>
                    )}
                  </div>
                  <Group gap="xs">
                    <Button
                      variant="subtle"
                      size="xs"
                      onClick={() => router.push(`/${orgName}/${projectName}/${repo.name}/issues`)}
                    >
                      Issues
                    </Button>
                    <Button
                      variant="subtle"
                      size="xs"
                      onClick={() => router.push(`/${orgName}/${projectName}/${repo.name}/pulls`)}
                    >
                      Pull Requests
                    </Button>
                  </Group>
                </Group>
              </li>
            ))}
          </ul>
        )}
      </Card>
    </div>
  );
}
