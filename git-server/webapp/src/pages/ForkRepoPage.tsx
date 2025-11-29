import { useSignal, useSignalEffect } from '@preact/signals';
import { Card, Text, TextInput, Button, Alert, Group, Select } from '@mantine/core';
import { useRoute, useRouter } from '@copilot-test/preact-router';
import { forkRepo, listOrganizations, listProjects, type Organization, type Project } from '../api';

export function ForkRepoPage() {
  const route = useRoute();
  const router = useRouter();
  const newName = useSignal('');
  const targetOrg = useSignal<string | null>(null);
  const targetProject = useSignal<string | null>(null);
  const orgs = useSignal<Organization[]>([]);
  const projects = useSignal<Project[]>([]);
  const loading = useSignal(false);
  const loadingOrgs = useSignal(true);
  const error = useSignal<string | null>(null);

  const params = route.value.params;
  const orgName = params.org as string;
  const projectName = params.project as string;
  const repoName = params.name as string;

  useSignalEffect(() => {
    loadOrgs();
  });

  async function loadOrgs() {
    try {
      loadingOrgs.value = true;
      const data = await listOrganizations();
      orgs.value = data;
      targetOrg.value = orgName; // Default to current org
      newName.value = `${repoName}-fork`;
      // Load projects for the default org
      await loadProjectsForOrg(orgName);
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load organizations';
    } finally {
      loadingOrgs.value = false;
    }
  }

  async function loadProjectsForOrg(org: string) {
    try {
      const data = await listProjects(org);
      projects.value = data;
      if (org === orgName) {
        targetProject.value = projectName; // Default to current project
      } else if (data.length > 0) {
        targetProject.value = data[0].name;
      }
    } catch (e) {
      projects.value = [];
    }
  }

  async function handleOrgChange(value: string | null) {
    targetOrg.value = value;
    if (value) {
      await loadProjectsForOrg(value);
    }
  }

  async function handleSubmit(e: Event) {
    e.preventDefault();

    if (!newName.value.trim()) {
      error.value = 'New repository name is required';
      return;
    }

    if (!targetOrg.value || !targetProject.value) {
      error.value = 'Target organization and project are required';
      return;
    }

    try {
      loading.value = true;
      error.value = null;
      const repo = await forkRepo(orgName, projectName, repoName, newName.value.trim(), targetOrg.value, targetProject.value);
      router.push(`/${repo.org_name}/${repo.project_name}/${repo.name}`);
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to fork repository';
      loading.value = false;
    }
  }

  return (
    <Card shadow="sm" padding="lg" radius="md" withBorder>
      <Text size="xl" fw={600} mb="xs">
        🍴 Fork this repository
      </Text>
      <Text size="sm" c="dimmed" mb="lg">
        Create a copy of {orgName}/{projectName}/{repoName} under a new name
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
          onChange={handleOrgChange}
          disabled={loadingOrgs.value}
          mb="md"
        />

        <Select
          label="Target project"
          placeholder="Select project"
          data={projects.value.map((proj) => ({ value: proj.name, label: proj.display_name }))}
          value={targetProject.value}
          onChange={(value: string | null) => (targetProject.value = value)}
          disabled={loadingOrgs.value || projects.value.length === 0}
          mb="md"
        />

        <TextInput
          label="New repository name"
          placeholder={`${repoName}-fork`}
          value={newName.value}
          onChange={(e: Event) => (newName.value = (e.target as HTMLInputElement).value)}
          required
          mb="lg"
        />

        <Group>
          <Button type="submit" loading={loading.value} color="green">
            Fork repository
          </Button>
          <Button
            variant="outline"
            onClick={() => router.push(`/${orgName}/${projectName}/${repoName}`)}
            disabled={loading.value}
          >
            Cancel
          </Button>
        </Group>
      </form>
    </Card>
  );
}
