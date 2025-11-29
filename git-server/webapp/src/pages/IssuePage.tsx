import { useSignal, useSignalEffect } from '@preact/signals';
import {
  Card,
  Text,
  Loader,
  Alert,
  Badge,
  Button,
  Textarea,
  Group,
  TypographyStylesProvider,
} from '@mantine/core';
import { useRoute } from '@copilot-test/preact-router';
import {
  getProjectIssue,
  getProjectIssueComments,
  createProjectIssueComment,
  updateProjectIssue,
  type Issue,
  type IssueComment,
  formatDate,
} from '../api';

// Simple markdown renderer (converts basic markdown to HTML)
function renderMarkdown(text: string): string {
  return text
    // Headers
    .replace(/^### (.*$)/gim, '<h3>$1</h3>')
    .replace(/^## (.*$)/gim, '<h2>$1</h2>')
    .replace(/^# (.*$)/gim, '<h1>$1</h1>')
    // Bold
    .replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>')
    // Italic
    .replace(/\*(.*?)\*/g, '<em>$1</em>')
    // Code blocks
    .replace(/```([\s\S]*?)```/g, '<pre><code>$1</code></pre>')
    // Inline code
    .replace(/`(.*?)`/g, '<code>$1</code>')
    // Links
    .replace(/\[(.*?)\]\((.*?)\)/g, '<a href="$2">$1</a>')
    // Line breaks
    .replace(/\n/g, '<br>');
}

export function IssuePage() {
  const route = useRoute();
  const issue = useSignal<Issue | null>(null);
  const comments = useSignal<IssueComment[]>([]);
  const loading = useSignal(true);
  const error = useSignal<string | null>(null);
  const newComment = useSignal('');
  const submitting = useSignal(false);

  const params = route.value.params;
  const orgName = params.org as string;
  const projectName = params.project as string;
  const issueNumber = parseInt(params.number as string, 10);

  useSignalEffect(() => {
    loadIssue();
  });

  async function loadIssue() {
    try {
      loading.value = true;
      error.value = null;
      const [issueData, commentsData] = await Promise.all([
        getProjectIssue(orgName, projectName, issueNumber),
        getProjectIssueComments(orgName, projectName, issueNumber),
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
      const comment = await createProjectIssueComment(orgName, projectName, issueNumber, newComment.value);
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
      const updated = await updateProjectIssue(orgName, projectName, issueNumber, { state: newState });
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
        <Group justify="space-between" mb="md" pb="md" style={{ borderBottom: '1px solid #e9ecef' }}>
          <div>
            <Text size="xl" fw={600}>
              {issue.value.title}
            </Text>
            <Group gap="xs" mt="xs">
              <Badge
                color={issue.value.state === 'open' ? 'green' : 'purple'}
                variant="filled"
              >
                {issue.value.state}
              </Badge>
              <Text size="sm" c="dimmed">
                #{issue.value.number} opened by {issue.value.author} on {formatDate(issue.value.created_at)}
              </Text>
            </Group>
          </div>
          <Button
            variant="outline"
            color={issue.value.state === 'open' ? 'red' : 'green'}
            onClick={handleToggleState}
          >
            {issue.value.state === 'open' ? 'Close issue' : 'Reopen issue'}
          </Button>
        </Group>

        {issue.value.body && (
          <TypographyStylesProvider>
            <div 
              class="bg-gray-50 p-4 rounded-lg"
              dangerouslySetInnerHTML={{ __html: renderMarkdown(issue.value.body) }}
            />
          </TypographyStylesProvider>
        )}
      </Card>

      {/* Comments */}
      <div class="space-y-4">
        {comments.value.map((comment) => (
          <Card key={comment.id} shadow="sm" padding="md" radius="md" withBorder>
            <Group gap="xs" mb="sm">
              <Text fw={500}>{comment.author}</Text>
              <Text size="sm" c="dimmed">
                commented on {formatDate(comment.created_at)}
              </Text>
            </Group>
            <TypographyStylesProvider>
              <div dangerouslySetInnerHTML={{ __html: renderMarkdown(comment.body) }} />
            </TypographyStylesProvider>
          </Card>
        ))}
      </div>

      {/* New comment form */}
      <Card shadow="sm" padding="lg" radius="md" withBorder>
        <Text fw={500} mb="md">
          Add a comment (supports Markdown)
        </Text>
        <form onSubmit={handleSubmitComment}>
          <Textarea
            placeholder="Leave a comment... (Markdown supported)"
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
