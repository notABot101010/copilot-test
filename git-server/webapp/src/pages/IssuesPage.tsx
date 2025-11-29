import { useSignal, useSignalEffect } from '@preact/signals';
import { Card, Text, Loader, Alert, Badge, Button, Anchor, Group } from '@mantine/core';
import { useRoute, useRouter } from '@copilot-test/preact-router';
import { listProjectIssues, type Issue, formatDate } from '../api';

export function IssuesPage() {
  const route = useRoute();
  const router = useRouter();
  const issues = useSignal<Issue[]>([]);
  const loading = useSignal(true);
  const error = useSignal<string | null>(null);

  const params = route.value.params;
  const orgName = params.org as string;
  const projectName = params.project as string;

  useSignalEffect(() => {
    loadIssues();
  });

  async function loadIssues() {
    try {
      loading.value = true;
      error.value = null;
      const data = await listProjectIssues(orgName, projectName);
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
      <Group justify="space-between" mb="lg" pb="md" style={{ borderBottom: '1px solid #e9ecef' }}>
        <Text size="xl" fw={600}>
          🐛 Issues
        </Text>
        <Button
          onClick={() => router.push(`/${orgName}/${projectName}/issues/new`)}
          color="green"
        >
          + New Issue
        </Button>
      </Group>

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
                    href={`/${orgName}/${projectName}/issues/${issue.number}`}
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
