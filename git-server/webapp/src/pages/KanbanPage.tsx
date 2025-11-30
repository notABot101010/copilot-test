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
  DndContext,
  DragOverlay,
  useDroppable,
  useDraggable,
  PointerSensor,
  useSensor,
  useSensors,
  type DragEndEvent,
  type DragStartEvent,
} from '@dnd-kit/core';
import { CSS } from '@dnd-kit/utilities';
import {
  listProjectIssues,
  updateProjectIssue,
  type Issue,
} from '../api';

type IssueStatus = 'todo' | 'doing' | 'done';

interface DraggableIssueCardProps {
  issue: Issue;
  orgName: string;
  projectName: string;
}

function DraggableIssueCard({ issue, orgName, projectName }: DraggableIssueCardProps) {
  const { attributes, listeners, setNodeRef, transform, isDragging } = useDraggable({
    id: `issue-${issue.number}`,
    data: { issue },
  });

  const style = {
    transform: CSS.Translate.toString(transform),
    opacity: isDragging ? 0.5 : 1,
  };

  // Type cast is needed because @dnd-kit uses React event types while Preact has different types.
  // The handlers are compatible at runtime.
  const pointerDownHandler = listeners?.onPointerDown as unknown as (e: PointerEvent) => void;
  const keyDownHandler = listeners?.onKeyDown as unknown as (e: KeyboardEvent) => void;

  return (
    <div
      ref={setNodeRef}
      style={style}
      role="button"
      tabIndex={attributes.tabIndex}
      aria-disabled={attributes['aria-disabled']}
      aria-pressed={attributes['aria-pressed']}
      aria-roledescription={attributes['aria-roledescription']}
      aria-describedby={attributes['aria-describedby']}
      onPointerDown={pointerDownHandler}
      onKeyDown={keyDownHandler}
    >
      <a
        href={`/${orgName}/${projectName}/issues/${issue.number}`}
        class="block no-underline"
        onClick={(e: MouseEvent) => {
          // Prevent navigation when dragging
          if (isDragging) {
            e.preventDefault();
          }
        }}
      >
        <Card
          shadow="sm"
          padding="sm"
          radius="md"
          withBorder
          class="cursor-grab hover:shadow-md transition-shadow active:cursor-grabbing"
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
    </div>
  );
}

interface IssueCardOverlayProps {
  issue: Issue;
}

function IssueCardOverlay({ issue }: IssueCardOverlayProps) {
  return (
    <Card
      shadow="lg"
      padding="sm"
      radius="md"
      withBorder
      class="cursor-grabbing"
      style={{ width: '250px' }}
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
  );
}

interface KanbanColumnProps {
  title: string;
  status: IssueStatus;
  issues: Issue[];
  orgName: string;
  projectName: string;
}

function KanbanColumn({ title, status, issues, orgName, projectName }: KanbanColumnProps) {
  const { setNodeRef, isOver } = useDroppable({
    id: status,
  });

  const bgColor = status === 'todo' ? 'bg-gray-100' : status === 'doing' ? 'bg-blue-50' : 'bg-green-50';
  const headerColor = status === 'todo' ? 'gray' : status === 'doing' ? 'blue' : 'green';

  return (
    <div
      ref={setNodeRef}
      class={`flex-1 min-w-[280px] ${bgColor} rounded-lg p-3 transition-all ${isOver ? 'ring-2 ring-blue-400 ring-offset-2' : ''}`}
    >
      <Group justify="space-between" mb="md">
        <Text fw={600} size="lg">{title}</Text>
        <Badge color={headerColor} variant="light">{issues.length}</Badge>
      </Group>
      <div class="space-y-2">
        {issues.map((issue) => (
          <DraggableIssueCard
            key={issue.id}
            issue={issue}
            orgName={orgName}
            projectName={projectName}
          />
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
  const activeIssue = useSignal<Issue | null>(null);

  const params = route.value.params;
  const orgName = params.org as string;
  const projectName = params.project as string;

  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: {
        // Require 8px of movement before starting drag.
        // This allows clicking on links within the draggable cards.
        distance: 8,
      },
    })
  );

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

  function handleDragStart(event: DragStartEvent) {
    const { active } = event;
    const issue = active.data.current?.issue as Issue | undefined;
    if (issue) {
      activeIssue.value = issue;
    }
  }

  async function handleDragEnd(event: DragEndEvent) {
    const { active, over } = event;
    activeIssue.value = null;

    if (!over) return;

    const newStatus = over.id as IssueStatus;
    const issue = active.data.current?.issue as Issue | undefined;

    if (!issue) return;

    // Store the original status for potential rollback
    const originalStatus = issue.status;

    // Only update if the status actually changed
    if (originalStatus === newStatus) return;

    try {
      // Optimistically update the UI
      issues.value = issues.value.map(i =>
        i.number === issue.number ? { ...i, status: newStatus } : i
      );

      // Make the API call
      await updateProjectIssue(orgName, projectName, issue.number, { status: newStatus });
    } catch (err) {
      // Revert on error - restore the original status
      issues.value = issues.value.map(i =>
        i.number === issue.number ? { ...i, status: originalStatus } : i
      );
      error.value = err instanceof Error ? err.message : 'Failed to update issue';
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

      <DndContext
        sensors={sensors}
        onDragStart={handleDragStart}
        onDragEnd={handleDragEnd}
      >
        <div class="flex gap-4 overflow-x-auto pb-4">
          <KanbanColumn
            title="To Do"
            status="todo"
            issues={todoIssues}
            orgName={orgName}
            projectName={projectName}
          />
          <KanbanColumn
            title="Doing"
            status="doing"
            issues={doingIssues}
            orgName={orgName}
            projectName={projectName}
          />
          <KanbanColumn
            title="Done"
            status="done"
            issues={doneIssues}
            orgName={orgName}
            projectName={projectName}
          />
        </div>

        <DragOverlay>
          {activeIssue.value ? (
            <IssueCardOverlay issue={activeIssue.value} />
          ) : null}
        </DragOverlay>
      </DndContext>
    </div>
  );
}
