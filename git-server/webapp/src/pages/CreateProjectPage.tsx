import { useSignal } from '@preact/signals';
import { Card, TextInput, Textarea, Button, Alert, Title } from '@mantine/core';
import { useRoute, useRouter } from '@copilot-test/preact-router';
import { createProject } from '../api';

export function CreateProjectPage() {
  const route = useRoute();
  const router = useRouter();
  const name = useSignal('');
  const displayName = useSignal('');
  const description = useSignal('');
  const error = useSignal<string | null>(null);
  const loading = useSignal(false);

  const params = route.value.params;
  const orgName = params.org as string;

  async function handleSubmit(e: Event) {
    e.preventDefault();
    
    if (!name.value.trim()) {
      error.value = 'Project name is required';
      return;
    }

    try {
      loading.value = true;
      error.value = null;
      await createProject(
        orgName,
        name.value.trim(),
        displayName.value.trim() || name.value.trim(),
        description.value.trim()
      );
      router.push(`/${orgName}/${name.value.trim()}`);
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to create project';
      loading.value = false;
    }
  }

  return (
    <Card shadow="sm" padding="lg" radius="md" withBorder>
      <Title order={2} mb="lg">Create New Project</Title>

      {error.value && (
        <Alert color="red" title="Error" mb="md">
          {error.value}
        </Alert>
      )}

      <form onSubmit={handleSubmit}>
        <TextInput
          label="Project Name"
          placeholder="my-project"
          description="Unique identifier for this project (lowercase, no spaces)"
          value={name.value}
          onChange={(e: Event) => (name.value = (e.target as HTMLInputElement).value)}
          required
          mb="md"
        />

        <TextInput
          label="Display Name"
          placeholder="My Awesome Project"
          description="Human-readable name (optional)"
          value={displayName.value}
          onChange={(e: Event) => (displayName.value = (e.target as HTMLInputElement).value)}
          mb="md"
        />

        <Textarea
          label="Description"
          placeholder="A short description of this project"
          value={description.value}
          onChange={(e: Event) => (description.value = (e.target as HTMLTextAreaElement).value)}
          mb="lg"
        />

        <Button type="submit" loading={loading.value}>
          Create Project
        </Button>
      </form>
    </Card>
  );
}
