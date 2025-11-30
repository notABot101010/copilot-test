import { useSignal, useSignalEffect } from '@preact/signals';
import {
  Card,
  Text,
  Loader,
  Alert,
  Badge,
  Group,
} from '@mantine/core';
import { useRoute } from '@copilot-test/preact-router';
import {
  listProjectIssues,
  updateProjectIssue,
  type Issue,
} from '../api';

interface KanbanColumnProps {
  title: string;
  status: 'todo' | 'doing' | 'done';
  issues: Issue[];
  orgName: string;
  projectName: string;
  onDrop: (issueNumber: number, newStatus: 'todo' | 'doing' | 'done') => void;
  onDragOver: (e: DragEvent) => void;
}

function KanbanColumn({ title, status, issues, orgName, projectName, onDrop, onDragOver }: KanbanColumnProps) {
  const handleDrop = (e: DragEvent) => {
    e.preventDefault();
    const issueNumber = parseInt(e.dataTransfer?.getData('issueNumber') || '0', 10);
    if (issueNumber) {
      onDrop(issueNumber, status);
    }
  };

  const handleDragStart = (e: DragEvent, issueNumber: number) => {
    e.dataTransfer?.setData('issueNumber', String(issueNumber));
  };

  const bgColor = status === 'todo' ? 'bg-gray-100' : status === 'doing' ? 'bg-blue-50' : 'bg-green-50';
  const headerColor = status === 'todo' ? 'gray' : status === 'doing' ? 'blue' : 'green';

  return (
    <div 
      class={`flex-1 min-w-[280px] ${bgColor} rounded-lg p-3`}
      onDrop={handleDrop}
      onDragOver={onDragOver}
    >
      <Group justify="space-between" mb="md">
        <Text fw={600} size="lg">{title}</Text>
        <Badge color={headerColor} variant="light">{issues.length}</Badge>
      </Group>
      <div class="space-y-2">
        {issues.map((issue) => (
          <a
            key={issue.id}
            href={`/${orgName}/${projectName}/issues/${issue.number}`}
            class="block no-underline"
          >
            <Card
              shadow="sm"
              padding="sm"
              radius="md"
              withBorder
              draggable
              onDragStart={(e: DragEvent) => handleDragStart(e, issue.number)}
              class="cursor-pointer hover:shadow-md transition-shadow"
            >
              <Group justify="space-between" mb="xs">
                <Text size="xs" c="dimmed">#{issue.number}</Text>
                <Badge
                  size="xs"
                  color={issue.state === 'open' ? 'green' : 'purple'}
                  variant="light"
                >
                  {issue.state}
                </Badge>
              </Group>
              <Text size="sm" fw={500} lineClamp={2}>
                {issue.title}
              </Text>
              {issue.due_date && (
                <Text size="xs" c="dimmed" mt="xs">
                  Due: {issue.due_date}
                </Text>
              )}
            </Card>
          </a>
        ))}
        {issues.length === 0 && (
          <Text size="sm" c="dimmed" ta="center" py="md">
            No issues
          </Text>
        )}
      </div>
    </div>
  );
}

export function KanbanPage() {
  const route = useRoute();
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
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to load issues';
    } finally {
      loading.value = false;
    }
  }

  async function handleDrop(issueNumber: number, newStatus: 'todo' | 'doing' | 'done') {
    try {
      await updateProjectIssue(orgName, projectName, issueNumber, { status: newStatus });
      // Update local state
      issues.value = issues.value.map(issue => 
        issue.number === issueNumber ? { ...issue, status: newStatus } : issue
      );
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to update issue';
    }
  }

  function handleDragOver(e: DragEvent) {
    e.preventDefault();
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

  const todoIssues = issues.value.filter(i => i.status === 'todo');
  const doingIssues = issues.value.filter(i => i.status === 'doing');
  const doneIssues = issues.value.filter(i => i.status === 'done');

  return (
    <div>
      <Group justify="space-between" mb="lg">
        <h1 class="text-2xl font-bold">Kanban Board</h1>
        <a href={`/${orgName}/${projectName}/issues/new`}>
          <button class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700">
            New Issue
          </button>
        </a>
      </Group>
      
      <div class="flex gap-4 overflow-x-auto pb-4">
        <KanbanColumn
          title="To Do"
          status="todo"
          issues={todoIssues}
          orgName={orgName}
          projectName={projectName}
          onDrop={handleDrop}
          onDragOver={handleDragOver}
        />
        <KanbanColumn
          title="Doing"
          status="doing"
          issues={doingIssues}
          orgName={orgName}
          projectName={projectName}
          onDrop={handleDrop}
          onDragOver={handleDragOver}
        />
        <KanbanColumn
          title="Done"
          status="done"
          issues={doneIssues}
          orgName={orgName}
          projectName={projectName}
          onDrop={handleDrop}
          onDragOver={handleDragOver}
        />
      </div>
    </div>
  );
}
