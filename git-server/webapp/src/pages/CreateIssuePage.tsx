import { useSignal } from '@preact/signals';
import { Card, Text, TextInput, Textarea, Button, Alert, Anchor } from '@mantine/core';
import { useRoute, useRouter } from '@copilot-test/preact-router';
import { createIssue } from '../api';

export function CreateIssuePage() {
  const route = useRoute();
  const router = useRouter();
  const title = useSignal('');
  const body = useSignal('');
  const loading = useSignal(false);
  const error = useSignal<string | null>(null);

  const params = route.value.params;
  const repoName = params.name as string;

  async function handleSubmit(e: Event) {
    e.preventDefault();

    if (!title.value.trim()) {
      error.value = 'Issue title is required';
      return;
    }

    try {
      loading.value = true;
      error.value = null;
      const issue = await createIssue(repoName, title.value.trim(), body.value.trim());
      router.push(`/repos/${encodeURIComponent(repoName)}/issues/${issue.number}`);
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to create issue';
      loading.value = false;
    }
  }

  return (
    <Card shadow="sm" padding="lg" radius="md" withBorder class="max-w-2xl mx-auto">
      <div class="border-b border-gray-200 pb-4 mb-4">
        <div class="flex items-center gap-3 mb-2">
          <Anchor href={`/repos/${encodeURIComponent(repoName)}`} c="blue">
            {repoName}
          </Anchor>
          <span class="text-gray-400">/</span>
          <Anchor href={`/repos/${encodeURIComponent(repoName)}/issues`} c="blue">
            Issues
          </Anchor>
          <span class="text-gray-400">/</span>
          <Text>New</Text>
        </div>
        <Text size="xl" fw={600}>
          Create a new issue
        </Text>
      </div>

      {error.value && (
        <Alert color="red" title="Error" mb="md">
          {error.value}
        </Alert>
      )}

      <form onSubmit={handleSubmit}>
        <TextInput
          label="Title"
          placeholder="Issue title"
          value={title.value}
          onChange={(e: Event) => (title.value = (e.target as HTMLInputElement).value)}
          required
          mb="lg"
        />

        <Textarea
          label="Description"
          placeholder="Describe the issue..."
          value={body.value}
          onChange={(e: Event) => (body.value = (e.target as HTMLTextAreaElement).value)}
          minRows={6}
          mb="lg"
        />

        <div class="flex gap-3">
          <Button type="submit" loading={loading.value} color="green">
            Create issue
          </Button>
          <Button
            variant="outline"
            onClick={() => router.push(`/repos/${encodeURIComponent(repoName)}/issues`)}
            disabled={loading.value}
          >
            Cancel
          </Button>
        </div>
      </form>
    </Card>
  );
}
