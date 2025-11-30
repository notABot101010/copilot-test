import { useSignal } from '@preact/signals';
import { Card, Text, TextInput, Textarea, Button, Alert, Group } from '@mantine/core';
import { useRoute, useRouter } from '@copilot-test/preact-router';
import { createProjectIssue } from '../api';

export function CreateIssuePage() {
  const route = useRoute();
  const router = useRouter();
  const title = useSignal('');
  const body = useSignal('');
  const startDate = useSignal('');
  const targetDate = useSignal('');
  const loading = useSignal(false);
  const error = useSignal<string | null>(null);

  const params = route.value.params;
  const orgName = params.org as string;
  const projectName = params.project as string;

  async function handleSubmit(e: Event) {
    e.preventDefault();

    if (!title.value.trim()) {
      error.value = 'Issue title is required';
      return;
    }

    try {
      loading.value = true;
      error.value = null;
      const issue = await createProjectIssue(
        orgName, 
        projectName, 
        title.value.trim(), 
        body.value.trim(),
        startDate.value || undefined,
        targetDate.value || undefined
      );
      router.push(`/${orgName}/${projectName}/issues/${issue.number}`);
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to create issue';
      loading.value = false;
    }
  }

  return (
    <Card shadow="sm" padding="lg" radius="md" withBorder>
      <Text size="xl" fw={600} mb="lg">
        Create a new issue
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

        <Group mb="lg">
          <TextInput
            label="Start Date"
            type="date"
            placeholder="YYYY-MM-DD"
            value={startDate.value}
            onChange={(e: Event) => (startDate.value = (e.target as HTMLInputElement).value)}
          />
          <TextInput
            label="Target Date"
            type="date"
            placeholder="YYYY-MM-DD"
            value={targetDate.value}
            onChange={(e: Event) => (targetDate.value = (e.target as HTMLInputElement).value)}
          />
        </Group>

        <Group>
          <Button type="submit" loading={loading.value} color="green">
            Create issue
          </Button>
          <Button
            variant="outline"
            onClick={() => router.push(`/${orgName}/${projectName}/issues`)}
            disabled={loading.value}
          >
            Cancel
          </Button>
        </Group>
      </form>
    </Card>
  );
}
