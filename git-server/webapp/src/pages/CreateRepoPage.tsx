import { useSignal } from '@preact/signals';
import { Card, Text, TextInput, Button, Alert, Group } from '@mantine/core';
import { useRoute, useRouter } from '@copilot-test/preact-router';
import { createRepo } from '../api';

export function CreateRepoPage() {
  const route = useRoute();
  const router = useRouter();
  const name = useSignal('');
  const loading = useSignal(false);
  const error = useSignal<string | null>(null);

  const params = route.value.params;
  const orgName = params.org as string;

  async function handleSubmit(e: Event) {
    e.preventDefault();

    if (!name.value.trim()) {
      error.value = 'Repository name is required';
      return;
    }

    try {
      loading.value = true;
      error.value = null;
      await createRepo(orgName, name.value.trim());
      router.push(`/${orgName}/${encodeURIComponent(name.value.trim())}`);
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to create repository';
      loading.value = false;
    }
  }

  return (
    <Card shadow="sm" padding="lg" radius="md" withBorder>
      <Text size="xl" fw={600} mb="lg">
        Create a new repository in {orgName}
      </Text>

      {error.value && (
        <Alert color="red" title="Error" mb="md">
          {error.value}
        </Alert>
      )}

      <form onSubmit={handleSubmit}>
        <TextInput
          label="Repository name"
          description="Use lowercase letters, numbers, and hyphens"
          placeholder="my-project"
          value={name.value}
          onChange={(e: Event) => (name.value = (e.target as HTMLInputElement).value.toLowerCase().replace(/[^a-z0-9-_.]/g, '-'))}
          required
          mb="lg"
        />

        <Group>
          <Button type="submit" loading={loading.value} color="green">
            Create repository
          </Button>
          <Button
            variant="outline"
            onClick={() => router.push(`/${orgName}`)}
            disabled={loading.value}
          >
            Cancel
          </Button>
        </Group>
      </form>
    </Card>
  );
}
