import { useSignal } from '@preact/signals';
import { Card, Text, TextInput, Button, Alert } from '@mantine/core';
import { useRouter } from '@copilot-test/preact-router';
import { createRepo } from '../api';

export function CreateRepoPage() {
  const router = useRouter();
  const name = useSignal('');
  const loading = useSignal(false);
  const error = useSignal<string | null>(null);

  async function handleSubmit(e: Event) {
    e.preventDefault();

    if (!name.value.trim()) {
      error.value = 'Repository name is required';
      return;
    }

    try {
      loading.value = true;
      error.value = null;
      await createRepo(name.value.trim());
      router.push(`/repos/${encodeURIComponent(name.value.trim())}`);
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to create repository';
      loading.value = false;
    }
  }

  return (
    <Card shadow="sm" padding="lg" radius="md" withBorder class="max-w-lg mx-auto">
      <div class="border-b border-gray-200 pb-4 mb-4">
        <Text size="xl" fw={600}>
          Create a new repository
        </Text>
      </div>

      {error.value && (
        <Alert color="red" title="Error" mb="md">
          {error.value}
        </Alert>
      )}

      <form onSubmit={handleSubmit}>
        <TextInput
          label="Repository name"
          placeholder="my-project"
          value={name.value}
          onChange={(e: Event) => (name.value = (e.target as HTMLInputElement).value)}
          required
          mb="lg"
        />

        <div class="flex gap-3">
          <Button type="submit" loading={loading.value} color="green">
            Create repository
          </Button>
          <Button
            variant="outline"
            onClick={() => router.push('/')}
            disabled={loading.value}
          >
            Cancel
          </Button>
        </div>
      </form>
    </Card>
  );
}
