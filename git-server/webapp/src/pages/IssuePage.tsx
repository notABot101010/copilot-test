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
  Select,
  TextInput,
} from '@mantine/core';
import { useRoute } from '@copilot-test/preact-router';
import {
  getProjectIssue,
  getProjectIssueComments,
  createProjectIssueComment,
  updateProjectIssue,
  updateProjectIssueComment,
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
  const editingCommentId = useSignal<number | null>(null);
  const editingCommentBody = useSignal('');

  useSignalEffect(() => {
    // Access route.value inside the effect to track signal changes
    const params = route.value.params;
    const orgName = params.org as string;
    const projectName = params.project as string;
    const issueNumber = parseInt(params.number as string, 10);

    if (!orgName || !projectName || isNaN(issueNumber)) {
      return;
    }

    loadIssue(orgName, projectName, issueNumber);
  });

  async function loadIssue(orgName: string, projectName: string, issueNumber: number) {
    try {
      loading.value = true;
      error.value = null;
      const [issueData, commentsData] = await Promise.all([
        getProjectIssue(orgName, projectName, issueNumber),
        getProjectIssueComments(orgName, projectName, issueNumber),
      ]);
      issue.value = issueData;
      comments.value = commentsData;
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to load issue';
    } finally {
      loading.value = false;
    }
  }

  // Get current route params for rendering and event handlers
  const params = route.value.params;
  const orgName = params.org as string;
  const projectName = params.project as string;
  const issueNumber = parseInt(params.number as string, 10);

  async function handleSubmitComment(e: Event) {
    e.preventDefault();
    if (!newComment.value.trim()) return;

    try {
      submitting.value = true;
      const comment = await createProjectIssueComment(orgName, projectName, issueNumber, newComment.value);
      comments.value = [...comments.value, comment];
      newComment.value = '';
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to add comment';
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
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to update issue';
    }
  }

  async function handleStatusChange(newStatus: string) {
    if (!issue.value || !newStatus) return;

    try {
      const updated = await updateProjectIssue(orgName, projectName, issueNumber, { 
        status: newStatus as 'todo' | 'doing' | 'done' 
      });
      issue.value = updated;
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to update status';
    }
  }

  async function handleStartDateChange(e: Event) {
    if (!issue.value) return;
    const newDate = (e.target as HTMLInputElement).value;

    try {
      const updated = await updateProjectIssue(orgName, projectName, issueNumber, { 
        start_date: newDate || null
      });
      issue.value = updated;
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to update start date';
    }
  }

  async function handleTargetDateChange(e: Event) {
    if (!issue.value) return;
    const newDate = (e.target as HTMLInputElement).value;

    try {
      const updated = await updateProjectIssue(orgName, projectName, issueNumber, { 
        target_date: newDate || null
      });
      issue.value = updated;
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to update target date';
    }
  }

  function startEditingComment(comment: IssueComment) {
    editingCommentId.value = comment.id;
    editingCommentBody.value = comment.body;
  }

  function cancelEditingComment() {
    editingCommentId.value = null;
    editingCommentBody.value = '';
  }

  async function saveEditedComment(commentId: number) {
    if (!editingCommentBody.value.trim()) return;

    try {
      submitting.value = true;
      const updated = await updateProjectIssueComment(
        orgName, 
        projectName, 
        issueNumber, 
        commentId, 
        editingCommentBody.value
      );
      comments.value = comments.value.map(c => c.id === commentId ? updated : c);
      editingCommentId.value = null;
      editingCommentBody.value = '';
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to update comment';
    } finally {
      submitting.value = false;
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

  const statusColor = issue.value.status === 'done' ? 'green' : 
                      issue.value.status === 'doing' ? 'blue' : 'gray';

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
              <Badge color={statusColor} variant="light">
                {issue.value.status}
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

        {/* Status and Due Date controls */}
        <Group mb="md">
          <Select
            label="Status"
            value={issue.value.status}
            onChange={(value: string | null) => value && handleStatusChange(value)}
            data={[
              { value: 'todo', label: 'To Do' },
              { value: 'doing', label: 'Doing' },
              { value: 'done', label: 'Done' },
            ]}
            style={{ width: 150 }}
          />
          <TextInput
            label="Start Date"
            type="date"
            value={issue.value.start_date || ''}
            onChange={handleStartDateChange}
            style={{ width: 180 }}
          />
          <TextInput
            label="Target Date"
            type="date"
            value={issue.value.target_date || ''}
            onChange={handleTargetDateChange}
            style={{ width: 180 }}
          />
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
            <Group justify="space-between" mb="sm">
              <Group gap="xs">
                <Text fw={500}>{comment.author}</Text>
                <Text size="sm" c="dimmed">
                  commented on {formatDate(comment.created_at)}
                  {comment.updated_at !== comment.created_at && ' (edited)'}
                </Text>
              </Group>
              {editingCommentId.value !== comment.id && (
                <Button 
                  variant="subtle" 
                  size="xs" 
                  onClick={() => startEditingComment(comment)}
                >
                  Edit
                </Button>
              )}
            </Group>
            {editingCommentId.value === comment.id ? (
              <div>
                <Textarea
                  value={editingCommentBody.value}
                  onChange={(e: Event) => (editingCommentBody.value = (e.target as HTMLTextAreaElement).value)}
                  minRows={3}
                  mb="sm"
                />
                <Group>
                  <Button 
                    size="xs" 
                    color="green" 
                    onClick={() => saveEditedComment(comment.id)}
                    loading={submitting.value}
                  >
                    Save
                  </Button>
                  <Button 
                    size="xs" 
                    variant="subtle" 
                    onClick={cancelEditingComment}
                  >
                    Cancel
                  </Button>
                </Group>
              </div>
            ) : (
              <TypographyStylesProvider>
                <div dangerouslySetInnerHTML={{ __html: renderMarkdown(comment.body) }} />
              </TypographyStylesProvider>
            )}
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
