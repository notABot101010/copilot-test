import { useSignal, useSignalEffect } from '@preact/signals';
import {
  Card,
  Text,
  Loader,
  Alert,
  Badge,
  Button,
  Textarea,
  Anchor,
  Tabs,
} from '@mantine/core';
import { useRoute } from '@copilot-test/preact-router';
import {
  getPullRequest,
  getPullRequestComments,
  getPullRequestCommits,
  getPullRequestFiles,
  createPullRequestComment,
  updatePullRequest,
  type PullRequest,
  type PullRequestComment,
  type CommitInfo,
  type FileDiff,
  formatDate,
} from '../api';

export function PullRequestPage() {
  const route = useRoute();
  const pr = useSignal<PullRequest | null>(null);
  const comments = useSignal<PullRequestComment[]>([]);
  const commits = useSignal<CommitInfo[]>([]);
  const files = useSignal<FileDiff[]>([]);
  const loading = useSignal(true);
  const error = useSignal<string | null>(null);
  const newComment = useSignal('');
  const submitting = useSignal(false);
  const activeTab = useSignal<string>('conversation');

  const params = route.value.params;
  const repoName = params.name as string;
  const prNumber = parseInt(params.number as string, 10);

  useSignalEffect(() => {
    loadPR();
  });

  async function loadPR() {
    try {
      loading.value = true;
      error.value = null;
      const [prData, commentsData, commitsData, filesData] = await Promise.all([
        getPullRequest(repoName, prNumber),
        getPullRequestComments(repoName, prNumber),
        getPullRequestCommits(repoName, prNumber),
        getPullRequestFiles(repoName, prNumber),
      ]);
      pr.value = prData;
      comments.value = commentsData;
      commits.value = commitsData;
      files.value = filesData;
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load pull request';
    } finally {
      loading.value = false;
    }
  }

  async function handleSubmitComment(e: Event) {
    e.preventDefault();
    if (!newComment.value.trim()) return;

    try {
      submitting.value = true;
      const comment = await createPullRequestComment(repoName, prNumber, newComment.value);
      comments.value = [...comments.value, comment];
      newComment.value = '';
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to add comment';
    } finally {
      submitting.value = false;
    }
  }

  async function handleUpdateState(newState: 'open' | 'closed' | 'merged') {
    if (!pr.value) return;

    try {
      const updated = await updatePullRequest(repoName, prNumber, { state: newState });
      pr.value = updated;
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to update pull request';
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

  if (!pr.value) {
    return (
      <Alert color="red" title="Error">
        Pull request not found
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
    <div class="space-y-4">
      <Card shadow="sm" padding="lg" radius="md" withBorder>
        <div class="border-b border-gray-200 pb-4 mb-4">
          <div class="flex items-center gap-3 mb-2">
            <Anchor href={`/repos/${encodeURIComponent(repoName)}`} c="blue">
              {repoName}
            </Anchor>
            <span class="text-gray-400">/</span>
            <Anchor href={`/repos/${encodeURIComponent(repoName)}/pulls`} c="blue">
              Pull Requests
            </Anchor>
            <span class="text-gray-400">/</span>
            <Text>#{pr.value.number}</Text>
          </div>
          <div class="flex items-start justify-between">
            <div>
              <Text size="xl" fw={600}>
                {pr.value.title}
              </Text>
              <div class="flex items-center gap-2 mt-2">
                <Badge color={getStateBadgeColor(pr.value.state)} variant="filled">
                  {pr.value.state}
                </Badge>
                <Text size="sm" c="dimmed">
                  opened by {pr.value.author} on {formatDate(pr.value.created_at)}
                </Text>
              </div>
              <Text size="sm" c="dimmed" mt="xs">
                {pr.value.source_repo}:{pr.value.source_branch} → {pr.value.target_branch}
              </Text>
            </div>
            <div class="flex gap-2">
              {pr.value.state === 'open' && (
                <>
                  <Button
                    variant="filled"
                    color="purple"
                    onClick={() => handleUpdateState('merged')}
                  >
                    Merge
                  </Button>
                  <Button
                    variant="outline"
                    color="red"
                    onClick={() => handleUpdateState('closed')}
                  >
                    Close
                  </Button>
                </>
              )}
              {pr.value.state === 'closed' && (
                <Button
                  variant="outline"
                  color="green"
                  onClick={() => handleUpdateState('open')}
                >
                  Reopen
                </Button>
              )}
            </div>
          </div>
        </div>

        <Tabs
          value={activeTab.value}
          onChange={(value: string | null) => (activeTab.value = value || 'conversation')}
        >
          <Tabs.List>
            <Tabs.Tab value="conversation">💬 Conversation</Tabs.Tab>
            <Tabs.Tab value="commits">📝 Commits ({commits.value.length})</Tabs.Tab>
            <Tabs.Tab value="files">📁 Files changed ({files.value.length})</Tabs.Tab>
          </Tabs.List>

          <Tabs.Panel value="conversation" pt="md">
            {/* PR description */}
            {pr.value.body && (
              <div class="bg-gray-50 p-4 rounded-lg mb-4">
                <Text style={{ whiteSpace: 'pre-wrap' }}>{pr.value.body}</Text>
              </div>
            )}

            {/* Comments */}
            <div class="space-y-4 mb-4">
              {comments.value.map((comment) => (
                <Card key={comment.id} shadow="xs" padding="md" radius="md" withBorder>
                  <div class="flex items-center gap-2 mb-2">
                    <Text fw={500}>{comment.author}</Text>
                    <Text size="sm" c="dimmed">
                      commented on {formatDate(comment.created_at)}
                    </Text>
                  </div>
                  <Text style={{ whiteSpace: 'pre-wrap' }}>{comment.body}</Text>
                </Card>
              ))}
            </div>

            {/* New comment form */}
            <Card shadow="xs" padding="md" radius="md" withBorder>
              <Text fw={500} mb="md">
                Add a comment
              </Text>
              <form onSubmit={handleSubmitComment}>
                <Textarea
                  placeholder="Leave a comment..."
                  value={newComment.value}
                  onChange={(e: Event) =>
                    (newComment.value = (e.target as HTMLTextAreaElement).value)
                  }
                  minRows={4}
                  mb="md"
                />
                <Button type="submit" loading={submitting.value} color="green">
                  Comment
                </Button>
              </form>
            </Card>
          </Tabs.Panel>

          <Tabs.Panel value="commits" pt="md">
            {commits.value.length === 0 ? (
              <div class="text-center py-8 text-gray-500">
                <Text size="lg">No commits found</Text>
              </div>
            ) : (
              <ul class="divide-y divide-gray-200">
                {commits.value.map((commit) => (
                  <li key={commit.hash} class="py-3">
                    <div class="flex items-start gap-3">
                      <span class="font-mono bg-gray-100 px-2 py-1 rounded text-sm text-blue-600">
                        {commit.short_hash}
                      </span>
                      <div>
                        <Text fw={500}>{commit.message}</Text>
                        <Text size="sm" c="dimmed">
                          {commit.author} committed on {formatDate(commit.date)}
                        </Text>
                      </div>
                    </div>
                  </li>
                ))}
              </ul>
            )}
          </Tabs.Panel>

          <Tabs.Panel value="files" pt="md">
            {files.value.length === 0 ? (
              <div class="text-center py-8 text-gray-500">
                <Text size="lg">No files changed</Text>
              </div>
            ) : (
              <div class="space-y-4">
                {files.value.map((file) => (
                  <Card key={file.path} shadow="xs" padding="md" radius="md" withBorder>
                    <div class="flex items-center gap-2 mb-3">
                      <Badge
                        color={
                          file.status === 'added'
                            ? 'green'
                            : file.status === 'deleted'
                              ? 'red'
                              : 'yellow'
                        }
                        variant="light"
                      >
                        {file.status}
                      </Badge>
                      <Text fw={500} class="font-mono">
                        {file.path}
                      </Text>
                      <Text size="sm" c="dimmed" class="ml-auto">
                        <span class="text-green-600">+{file.additions}</span>{' '}
                        <span class="text-red-600">-{file.deletions}</span>
                      </Text>
                    </div>
                    <pre class="bg-gray-50 p-3 rounded overflow-x-auto text-sm font-mono whitespace-pre-wrap">
                      {file.diff}
                    </pre>
                  </Card>
                ))}
              </div>
            )}
          </Tabs.Panel>
        </Tabs>
      </Card>
    </div>
  );
}
