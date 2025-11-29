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
} from '@mantine/core';
import { useRoute } from '@copilot-test/preact-router';
import {
  getIssue,
  getIssueComments,
  createIssueComment,
  updateIssue,
  type Issue,
  type IssueComment,
  formatDate,
} from '../api';

export function IssuePage() {
  const route = useRoute();
  const issue = useSignal<Issue | null>(null);
  const comments = useSignal<IssueComment[]>([]);
  const loading = useSignal(true);
  const error = useSignal<string | null>(null);
  const newComment = useSignal('');
  const submitting = useSignal(false);

  const params = route.value.params;
  const repoName = params.name as string;
  const issueNumber = parseInt(params.number as string, 10);

  useSignalEffect(() => {
    loadIssue();
  });

  async function loadIssue() {
    try {
      loading.value = true;
      error.value = null;
      const [issueData, commentsData] = await Promise.all([
        getIssue(repoName, issueNumber),
        getIssueComments(repoName, issueNumber),
      ]);
      issue.value = issueData;
      comments.value = commentsData;
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load issue';
    } finally {
      loading.value = false;
    }
  }

  async function handleSubmitComment(e: Event) {
    e.preventDefault();
    if (!newComment.value.trim()) return;

    try {
      submitting.value = true;
      const comment = await createIssueComment(repoName, issueNumber, newComment.value);
      comments.value = [...comments.value, comment];
      newComment.value = '';
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to add comment';
    } finally {
      submitting.value = false;
    }
  }

  async function handleToggleState() {
    if (!issue.value) return;

    try {
      const newState = issue.value.state === 'open' ? 'closed' : 'open';
      const updated = await updateIssue(repoName, issueNumber, { state: newState });
      issue.value = updated;
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to update issue';
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

  if (!issue.value) {
    return (
      <Alert color="red" title="Error">
        Issue not found
      </Alert>
    );
  }

  return (
    <div class="space-y-4">
      <Card shadow="sm" padding="lg" radius="md" withBorder>
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
            <Text>#{issue.value.number}</Text>
          </div>
          <div class="flex items-start justify-between">
            <div>
              <Text size="xl" fw={600}>
                {issue.value.title}
              </Text>
              <div class="flex items-center gap-2 mt-2">
                <Badge
                  color={issue.value.state === 'open' ? 'green' : 'purple'}
                  variant="filled"
                >
                  {issue.value.state}
                </Badge>
                <Text size="sm" c="dimmed">
                  opened by {issue.value.author} on {formatDate(issue.value.created_at)}
                </Text>
              </div>
            </div>
            <Button
              variant="outline"
              color={issue.value.state === 'open' ? 'red' : 'green'}
              onClick={handleToggleState}
            >
              {issue.value.state === 'open' ? 'Close issue' : 'Reopen issue'}
            </Button>
          </div>
        </div>

        {issue.value.body && (
          <div class="bg-gray-50 p-4 rounded-lg mb-4">
            <Text style={{ whiteSpace: 'pre-wrap' }}>{issue.value.body}</Text>
          </div>
        )}
      </Card>

      {/* Comments */}
      <div class="space-y-4">
        {comments.value.map((comment) => (
          <Card key={comment.id} shadow="sm" padding="md" radius="md" withBorder>
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
      <Card shadow="sm" padding="lg" radius="md" withBorder>
        <Text fw={500} mb="md">
          Add a comment
        </Text>
        <form onSubmit={handleSubmitComment}>
          <Textarea
            placeholder="Leave a comment..."
            value={newComment.value}
            onChange={(e: Event) => (newComment.value = (e.target as HTMLTextAreaElement).value)}
            minRows={4}
            mb="md"
          />
          <Button type="submit" loading={submitting.value} color="green">
            Comment
          </Button>
        </form>
      </Card>
    </div>
  );
}
