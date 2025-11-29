import { useSignal } from '@preact/signals';
import { Card, Text, TextInput, Textarea, Button, Alert, Group } from '@mantine/core';
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
  const orgName = params.org as string;
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
      const issue = await createIssue(orgName, repoName, title.value.trim(), body.value.trim());
      router.push(`/${orgName}/${repoName}/issues/${issue.number}`);
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to create issue';
      loading.value = false;
    }
  }

  return (
    <Card shadow="sm" padding="lg" radius="md" withBorder>
      <Text size="xl" fw={600} mb="lg">
        🐛 Create a new issue
      </Text>

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
          label="Description (Markdown supported)"
          placeholder="Describe the issue..."
          value={body.value}
          onChange={(e: Event) => (body.value = (e.target as HTMLTextAreaElement).value)}
          minRows={6}
          mb="lg"
        />

        <Group>
          <Button type="submit" loading={loading.value} color="green">
            Create issue
          </Button>
          <Button
            variant="outline"
            onClick={() => router.push(`/${orgName}/${repoName}/issues`)}
            disabled={loading.value}
          >
            Cancel
          </Button>
        </Group>
      </form>
    </Card>
  );
}
