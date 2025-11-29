import { useSignal } from '@preact/signals';
import { Card, Text, TextInput, Button, Alert, Anchor } from '@mantine/core';
import { useRoute, useRouter } from '@copilot-test/preact-router';
import { forkRepo } from '../api';

export function ForkRepoPage() {
  const route = useRoute();
  const router = useRouter();
  const newName = useSignal('');
  const loading = useSignal(false);
  const error = useSignal<string | null>(null);

  const params = route.value.params;
  const repoName = params.name as string;

  async function handleSubmit(e: Event) {
    e.preventDefault();

    if (!newName.value.trim()) {
      error.value = 'New repository name is required';
      return;
    }

    try {
      loading.value = true;
      error.value = null;
      const repo = await forkRepo(repoName, newName.value.trim());
      router.push(`/repos/${encodeURIComponent(repo.name)}`);
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to fork repository';
      loading.value = false;
    }
  }

  return (
    <Card shadow="sm" padding="lg" radius="md" withBorder class="max-w-lg mx-auto">
      <div class="border-b border-gray-200 pb-4 mb-4">
        <div class="flex items-center gap-3 mb-2">
          <Anchor href={`/repos/${encodeURIComponent(repoName)}`} c="blue">
            {repoName}
          </Anchor>
          <span class="text-gray-400">/</span>
          <Text>Fork</Text>
        </div>
        <Text size="xl" fw={600}>
          Fork this repository
        </Text>
        <Text size="sm" c="dimmed" mt="xs">
          Create a copy of this repository under a new name
        </Text>
      </div>

      {error.value && (
        <Alert color="red" title="Error" mb="md">
          {error.value}
        </Alert>
      )}

      <form onSubmit={handleSubmit}>
        <TextInput
          label="New repository name"
          placeholder={`${repoName}-fork`}
          value={newName.value}
          onChange={(e: Event) => (newName.value = (e.target as HTMLInputElement).value)}
          required
          mb="lg"
        />

        <div class="flex gap-3">
          <Button type="submit" loading={loading.value} color="green">
            Fork repository
          </Button>
          <Button
            variant="outline"
            onClick={() => router.push(`/repos/${encodeURIComponent(repoName)}`)}
            disabled={loading.value}
          >
            Cancel
          </Button>
        </div>
      </form>
    </Card>
  );
}
