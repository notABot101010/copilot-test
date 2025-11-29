import { useSignal, useSignalEffect } from '@preact/signals';
import { Card, Text, TextInput, Textarea, Button, Alert, Group, Loader } from '@mantine/core';
import { useRoute, useRouter } from '@copilot-test/preact-router';
import { getProject, updateProject, type Project } from '../api';

export function ProjectSettingsPage() {
  const route = useRoute();
  const router = useRouter();
  const project = useSignal<Project | null>(null);
  const displayName = useSignal('');
  const description = useSignal('');
  const loading = useSignal(true);
  const saving = useSignal(false);
  const error = useSignal<string | null>(null);
  const success = useSignal(false);

  const params = route.value.params;
  const orgName = params.org as string;
  const projectName = params.project as string;

  useSignalEffect(() => {
    loadProject();
  });

  async function loadProject() {
    try {
      loading.value = true;
      error.value = null;
      const projectData = await getProject(orgName, projectName);
      project.value = projectData;
      displayName.value = projectData.display_name;
      description.value = projectData.description;
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load project';
    } finally {
      loading.value = false;
    }
  }

  async function handleSubmit(e: Event) {
    e.preventDefault();
    
    try {
      saving.value = true;
      error.value = null;
      success.value = false;
      await updateProject(orgName, projectName, {
        display_name: displayName.value.trim(),
        description: description.value.trim(),
      });
      success.value = true;
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to update project';
    } finally {
      saving.value = false;
    }
  }

  if (loading.value) {
    return (
      <div class="flex justify-center py-12">
        <Loader size="lg" />
      </div>
    );
  }

  if (!project.value) {
    return (
      <Alert color="red" title="Error">
        Project not found
      </Alert>
    );
  }

  return (
    <Card shadow="sm" padding="lg" radius="md" withBorder>
      <Text size="xl" fw={600} mb="lg">
        ⚙️ Project Settings
      </Text>

      {error.value && (
        <Alert color="red" title="Error" mb="md">
          {error.value}
        </Alert>
      )}

      {success.value && (
        <Alert color="green" title="Success" mb="md">
          Project settings updated successfully
        </Alert>
      )}

      <form onSubmit={handleSubmit}>
        <TextInput
          label="Organization"
          description="The organization this project belongs to"
          value={orgName}
          disabled
          mb="md"
        />

        <TextInput
          label="Project slug"
          description="This cannot be changed"
          value={projectName}
          disabled
          mb="md"
        />

        <TextInput
          label="Display name"
          description="Human-readable name for your project"
          placeholder="My Project"
          value={displayName.value}
          onChange={(e: Event) => (displayName.value = (e.target as HTMLInputElement).value)}
          mb="md"
        />

        <Textarea
          label="Description"
          description="Optional description of your project"
          placeholder="What does your project do?"
          value={description.value}
          onChange={(e: Event) => (description.value = (e.target as HTMLTextAreaElement).value)}
          mb="lg"
        />

        <Group>
          <Button type="submit" loading={saving.value} color="blue">
            Save Changes
          </Button>
          <Button
            variant="outline"
            onClick={() => router.push(`/${orgName}/${projectName}`)}
            disabled={saving.value}
          >
            Back to Project
          </Button>
        </Group>
      </form>
    </Card>
  );
}
