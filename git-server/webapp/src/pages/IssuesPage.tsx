import { useSignal, useSignalEffect } from '@preact/signals';
import { Card, Text, Loader, Alert, Badge, Button, Anchor } from '@mantine/core';
import { useRoute } from '@copilot-test/preact-router';
import { listIssues, type Issue, formatDate } from '../api';

export function IssuesPage() {
  const route = useRoute();
  const issues = useSignal<Issue[]>([]);
  const loading = useSignal(true);
  const error = useSignal<string | null>(null);

  const params = route.value.params;
  const repoName = params.name as string;

  useSignalEffect(() => {
    loadIssues();
  });

  async function loadIssues() {
    try {
      loading.value = true;
      error.value = null;
      const data = await listIssues(repoName);
      issues.value = data;
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load issues';
    } finally {
      loading.value = false;
    }
  }

  if (loading.value) {
    return (
      <div class="flex justify-center py-12">
        <Loader size="lg" />
      </div>
    );
  }

  if (error.value) {
    return (
      <Alert color="red" title="Error">
        {error.value}
      </Alert>
    );
  }

  return (
    <Card shadow="sm" padding="lg" radius="md" withBorder>
      <div class="border-b border-gray-200 pb-4 mb-4 flex justify-between items-center">
        <div class="flex items-center gap-3">
          <Anchor href={`/repos/${encodeURIComponent(repoName)}`} c="blue">
            {repoName}
          </Anchor>
          <span class="text-gray-400">/</span>
          <Text size="xl" fw={600}>
            Issues
          </Text>
        </div>
        <Button
          component="a"
          href={`/repos/${encodeURIComponent(repoName)}/issues/new`}
          color="green"
        >
          New Issue
        </Button>
      </div>

      {issues.value.length === 0 ? (
        <div class="text-center py-8 text-gray-500">
          <Text size="lg">No issues found</Text>
          <Text size="sm" c="dimmed" mt="sm">
            Create a new issue to start a conversation
          </Text>
        </div>
      ) : (
        <ul class="divide-y divide-gray-200">
          {issues.value.map((issue) => (
            <li key={issue.id} class="py-4">
              <div class="flex items-start gap-3">
                <Badge
                  color={issue.state === 'open' ? 'green' : 'purple'}
                  variant="filled"
                >
                  {issue.state}
                </Badge>
                <div class="flex-1">
                  <Anchor
                    href={`/repos/${encodeURIComponent(repoName)}/issues/${issue.number}`}
                    class="font-semibold text-lg hover:underline"
                  >
                    {issue.title}
                  </Anchor>
                  <Text size="sm" c="dimmed" mt="xs">
                    #{issue.number} opened by {issue.author} on {formatDate(issue.created_at)}
                  </Text>
                </div>
              </div>
            </li>
          ))}
        </ul>
      )}
    </Card>
  );
}
