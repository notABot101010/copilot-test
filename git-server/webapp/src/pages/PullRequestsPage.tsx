import { useSignal, useSignalEffect } from '@preact/signals';
import { Card, Text, Loader, Alert, Badge, Button, Anchor, Group } from '@mantine/core';
import { useRoute, useRouter } from '@copilot-test/preact-router';
import { listPullRequests, type PullRequest, formatDate } from '../api';

export function PullRequestsPage() {
  const route = useRoute();
  const router = useRouter();
  const prs = useSignal<PullRequest[]>([]);
  const loading = useSignal(true);
  const error = useSignal<string | null>(null);

  const params = route.value.params;
  const orgName = params.org as string;
  const projectName = params.project as string;
  const repoName = params.name as string;

  useSignalEffect(() => {
    loadPRs();
  });

  async function loadPRs() {
    try {
      loading.value = true;
      error.value = null;
      const data = await listPullRequests(orgName, projectName, repoName);
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
      <Group justify="space-between" mb="lg" pb="md" style={{ borderBottom: '1px solid #e9ecef' }}>
        <Text size="xl" fw={600}>
          🔀 Pull Requests
        </Text>
        <Button
          onClick={() => router.push(`/${orgName}/${projectName}/${repoName}/pulls/new`)}
          color="green"
        >
          + New Pull Request
        </Button>
      </Group>

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
                    href={`/${orgName}/${projectName}/${repoName}/pulls/${pr.number}`}
                    class="font-semibold text-lg hover:underline"
                    onClick={(e: Event) => {
                      e.preventDefault();
                      router.push(`/${orgName}/${projectName}/${repoName}/pulls/${pr.number}`);
                    }}
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
