import { useSignal, useSignalEffect } from '@preact/signals';
import { Card, Text, TextInput, Button, Alert, Group, Select } from '@mantine/core';
import { useRoute, useRouter } from '@copilot-test/preact-router';
import { forkProject, listOrganizations, type Organization } from '../api';

export function ForkRepoPage() {
  const route = useRoute();
  const router = useRouter();
  const newName = useSignal('');
  const targetOrg = useSignal<string | null>(null);
  const orgs = useSignal<Organization[]>([]);
  const loading = useSignal(false);
  const loadingOrgs = useSignal(true);
  const error = useSignal<string | null>(null);

  const params = route.value.params;
  const orgName = params.org as string;
  const projectName = params.project as string;

  useSignalEffect(() => {
    loadOrgs();
  });

  async function loadOrgs() {
    try {
      loadingOrgs.value = true;
      const data = await listOrganizations();
      orgs.value = data;
      targetOrg.value = orgName; // Default to current org
      newName.value = `${projectName}-fork`;
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load organizations';
    } finally {
      loadingOrgs.value = false;
    }
  }

  async function handleSubmit(e: Event) {
    e.preventDefault();

    if (!newName.value.trim()) {
      error.value = 'New project name is required';
      return;
    }

    if (!targetOrg.value) {
      error.value = 'Target organization is required';
      return;
    }

    try {
      loading.value = true;
      error.value = null;
      const repo = await forkProject(orgName, projectName, newName.value.trim(), targetOrg.value);
      router.push(`/${repo.org_name}/${repo.project_name}`);
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to fork project';
      loading.value = false;
    }
  }

  return (
    <Card shadow="sm" padding="lg" radius="md" withBorder>
      <Text size="xl" fw={600} mb="xs">
        🍴 Fork this project
      </Text>
      <Text size="sm" c="dimmed" mb="lg">
        Create a copy of {orgName}/{projectName} under a new name
      </Text>

      {error.value && (
        <Alert color="red" title="Error" mb="md">
          {error.value}
        </Alert>
      )}

      <form onSubmit={handleSubmit}>
        <Select
          label="Target organization"
          placeholder="Select organization"
          data={orgs.value.map((org) => ({ value: org.name, label: org.display_name }))}
          value={targetOrg.value}
          onChange={(value: string | null) => (targetOrg.value = value)}
          disabled={loadingOrgs.value}
          mb="md"
        />

        <TextInput
          label="New project name"
          placeholder={`${projectName}-fork`}
          value={newName.value}
          onChange={(e: Event) => (newName.value = (e.target as HTMLInputElement).value)}
          required
          mb="lg"
        />

        <Group>
          <Button type="submit" loading={loading.value} color="green">
            Fork project
          </Button>
          <Button
            variant="outline"
            onClick={() => router.push(`/${orgName}/${projectName}`)}
            disabled={loading.value}
          >
            Cancel
          </Button>
        </Group>
      </form>
    </Card>
  );
}
