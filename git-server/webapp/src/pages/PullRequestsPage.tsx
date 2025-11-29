import { useSignal, useSignalEffect } from '@preact/signals';
import { Card, Text, Loader, Alert, Badge, Button, Anchor } from '@mantine/core';
import { useRoute } from '@copilot-test/preact-router';
import { listPullRequests, type PullRequest, formatDate } from '../api';

export function PullRequestsPage() {
  const route = useRoute();
  const prs = useSignal<PullRequest[]>([]);
  const loading = useSignal(true);
  const error = useSignal<string | null>(null);

  const params = route.value.params;
  const repoName = params.name as string;

  useSignalEffect(() => {
    loadPRs();
  });

  async function loadPRs() {
    try {
      loading.value = true;
      error.value = null;
      const data = await listPullRequests(repoName);
      prs.value = data;
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load pull requests';
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

  const getStateBadgeColor = (state: string) => {
    switch (state) {
      case 'open':
        return 'green';
      case 'merged':
        return 'purple';
      case 'closed':
        return 'red';
      default:
        return 'gray';
    }
  };

  return (
    <Card shadow="sm" padding="lg" radius="md" withBorder>
      <div class="border-b border-gray-200 pb-4 mb-4 flex justify-between items-center">
        <div class="flex items-center gap-3">
          <Anchor href={`/repos/${encodeURIComponent(repoName)}`} c="blue">
            {repoName}
          </Anchor>
          <span class="text-gray-400">/</span>
          <Text size="xl" fw={600}>
            Pull Requests
          </Text>
        </div>
        <Button
          component="a"
          href={`/repos/${encodeURIComponent(repoName)}/pulls/new`}
          color="green"
        >
          New Pull Request
        </Button>
      </div>

      {prs.value.length === 0 ? (
        <div class="text-center py-8 text-gray-500">
          <Text size="lg">No pull requests found</Text>
          <Text size="sm" c="dimmed" mt="sm">
            Create a pull request to propose changes
          </Text>
        </div>
      ) : (
        <ul class="divide-y divide-gray-200">
          {prs.value.map((pr) => (
            <li key={pr.id} class="py-4">
              <div class="flex items-start gap-3">
                <Badge color={getStateBadgeColor(pr.state)} variant="filled">
                  {pr.state}
                </Badge>
                <div class="flex-1">
                  <Anchor
                    href={`/repos/${encodeURIComponent(repoName)}/pulls/${pr.number}`}
                    class="font-semibold text-lg hover:underline"
                  >
                    {pr.title}
                  </Anchor>
                  <Text size="sm" c="dimmed" mt="xs">
                    #{pr.number} opened by {pr.author} on {formatDate(pr.created_at)}
                  </Text>
                  <Text size="sm" c="dimmed">
                    {pr.source_repo}:{pr.source_branch} → {pr.target_branch}
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
