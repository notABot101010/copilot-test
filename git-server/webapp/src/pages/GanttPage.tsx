import { useSignal, useSignalEffect } from '@preact/signals';
import {
  Card,
  Text,
  Loader,
  Alert,
  Badge,
  Group,
  Select,
} from '@mantine/core';
import { useRoute } from '@copilot-test/preact-router';
import {
  listProjectIssues,
  type Issue,
} from '../api';

function parseDate(dateStr: string | null): Date | null {
  if (!dateStr) return null;
  const date = new Date(dateStr);
  if (isNaN(date.getTime())) return null;
  // Normalize to midnight for proper day alignment
  date.setHours(0, 0, 0, 0);
  return date;
}

function formatDateShort(date: Date): string {
  return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
}

function getDateRange(startDate: Date, endDate: Date): { start: Date; end: Date; days: number } {
  const days = Math.ceil((endDate.getTime() - startDate.getTime()) / (1000 * 60 * 60 * 24));
  return { start: startDate, end: endDate, days };
}

function getDayHeaders(start: Date, days: number): { date: Date; label: string }[] {
  const headers = [];
  for (let i = 0; i < days; i++) {
    const date = new Date(start);
    date.setDate(date.getDate() + i);
    headers.push({ date, label: formatDateShort(date) });
  }
  return headers;
}

interface GanttBarProps {
  issue: Issue;
  startDate: Date;
  totalDays: number;
  orgName: string;
  projectName: string;
}

function GanttBar({ issue, startDate, totalDays, orgName, projectName }: GanttBarProps) {
  const issueStart = parseDate(issue.start_date)!;
  const issueEnd = parseDate(issue.target_date)!;

  const dayWidth = 60; // pixels per day
  const startDay = Math.floor((issueStart.getTime() - startDate.getTime()) / (1000 * 60 * 60 * 24));
  const endDay = Math.floor((issueEnd.getTime() - startDate.getTime()) / (1000 * 60 * 60 * 24));

  const duration = Math.max(1, endDay - startDay + 1); // +1 to include both start and end days
  const leftPx = startDay * dayWidth;
  const widthPx = duration * dayWidth;

  const statusColor = issue.status === 'done' ? 'bg-green-500' :
                      issue.status === 'doing' ? 'bg-blue-500' : 'bg-gray-400';

  return (
    <a
      href={`/${orgName}/${projectName}/issues/${issue.number}`}
      class="block no-underline"
    >
      <div class="flex items-center border-b border-gray-100 hover:bg-gray-50">
        <div class="w-64 flex-shrink-0 px-3 py-2 border-r border-gray-200 sticky left-0 bg-white hover:bg-gray-50 z-10">
          <div class="flex items-center gap-2">
            <Text size="xs" c="dimmed">#{issue.number}</Text>
            <Badge size="xs" color={issue.state === 'open' ? 'green' : 'purple'} variant="light">
              {issue.state}
            </Badge>
          </div>
          <Text size="sm" fw={500} lineClamp={1}>
            {issue.title}
          </Text>
        </div>
        <div class="relative h-10" style={{ width: `${totalDays * 60}px` }}>
          <div
            class={`absolute top-2 h-6 ${statusColor} rounded opacity-80 hover:opacity-100 transition-opacity`}
            style={{ left: `${leftPx}px`, width: `${widthPx}px` }}
            title={`${issue.title}\nStatus: ${issue.status}\nStart: ${issue.start_date}\nTarget: ${issue.target_date}`}
          />
        </div>
      </div>
    </a>
  );
}

export function GanttPage() {
  const route = useRoute();
  const issues = useSignal<Issue[]>([]);
  const loading = useSignal(true);
  const error = useSignal<string | null>(null);
  const dayRange = useSignal<string>('30');

  const today = new Date();
  today.setHours(0, 0, 0, 0);

  const initialEnd = new Date(today);
  initialEnd.setDate(initialEnd.getDate() + 30);

  const startDate = useSignal<Date>(new Date(today));
  const endDate = useSignal<Date>(initialEnd);

  const scrollTimeout = useSignal<number | null>(null);
  const isLoadingMore = useSignal(false);

  const params = route.value.params;
  const orgName = params.org as string;
  const projectName = params.project as string;

  useSignalEffect(() => {
    loadIssues();
  });

  useSignalEffect(() => {
    // Update date range when dayRange changes
    const range = parseInt(dayRange.value);
    const newEnd = new Date(today);
    newEnd.setDate(newEnd.getDate() + range);
    startDate.value = new Date(today);
    endDate.value = newEnd;
  });

  function handleScroll(e: Event) {
    if (isLoadingMore.value) return;

    if (scrollTimeout.value) {
      clearTimeout(scrollTimeout.value);
    }

    scrollTimeout.value = setTimeout(() => {
      const target = e.target as HTMLDivElement;
      const scrollLeft = target.scrollLeft;
      const scrollWidth = target.scrollWidth;
      const clientWidth = target.clientWidth;

      const scrollThreshold = 300; // pixels from edge to trigger load

      // Scrolled near the left edge - load more past days
      if (scrollLeft < scrollThreshold) {
        isLoadingMore.value = true;
        const previousScrollLeft = scrollLeft;

        const newStart = new Date(startDate.value);
        newStart.setDate(newStart.getDate() - 30); // Add 30 days to the past
        startDate.value = newStart;

        // Restore scroll position after adding days to the left
        setTimeout(() => {
          const addedDays = 30;
          const addedWidth = addedDays * 60; // 60px per day
          target.scrollLeft = previousScrollLeft + addedWidth;
          isLoadingMore.value = false;
        }, 50);
      }

      // Scrolled near the right edge - load more future days
      if (scrollLeft + clientWidth > scrollWidth - scrollThreshold) {
        isLoadingMore.value = true;
        const newEnd = new Date(endDate.value);
        newEnd.setDate(newEnd.getDate() + 30); // Add 30 days to the future
        endDate.value = newEnd;

        setTimeout(() => {
          isLoadingMore.value = false;
        }, 50);
      }
    }, 100) as unknown as number;
  }

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

  const { start, days } = getDateRange(startDate.value, endDate.value);
  const dayHeaders = getDayHeaders(start, days);
  const todayDate = new Date();
  todayDate.setHours(0, 0, 0, 0);

  // Filter issues to only show those with both start_date and target_date
  const validIssues = issues.value.filter(issue => issue.start_date && issue.target_date);

  // Sort issues: open first, then by target date
  const sortedIssues = [...validIssues].sort((a, b) => {
    if (a.state !== b.state) return a.state === 'open' ? -1 : 1;
    const aTarget = parseDate(a.target_date)!;
    const bTarget = parseDate(b.target_date)!;
    return aTarget.getTime() - bTarget.getTime();
  });

  return (
    <div>
      <Group justify="space-between" mb="lg">
        <div class="flex items-center gap-4">
          <h1 class="text-2xl font-bold">Gantt Chart</h1>
          <Select
            value={dayRange.value}
            onChange={(value: string | null) => dayRange.value = value || '30'}
            data={[
              { value: '30', label: '30 Days' },
              { value: '90', label: '90 Days' }
            ]}
            w={120}
          />
        </div>
        <a href={`/${orgName}/${projectName}/issues/new`}>
          <button class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700">
            New Issue
          </button>
        </a>
      </Group>

      {sortedIssues.length === 0 ? (
        <Card shadow="sm" padding="lg" radius="md" withBorder>
          <Text c="dimmed" ta="center">No issues yet. Create your first issue to see the Gantt chart.</Text>
        </Card>
      ) : (
        <Card shadow="sm" padding="0" radius="md" withBorder>
          <div class="overflow-x-auto scrollbar-hide" onScroll={handleScroll}>
            <div class="min-w-fit">
              {/* Header */}
              <div class="flex border-b border-gray-200 bg-gray-50 sticky top-0">
                <div class="w-64 flex-shrink-0 px-3 py-2 border-r border-gray-200 font-medium sticky left-0 bg-gray-50 z-10">
                  Issue
                </div>
                <div class="flex">
                  {dayHeaders.map((header, idx) => {
                    const isToday = header.date.toDateString() === todayDate.toDateString();
                    const isWeekend = header.date.getDay() === 0 || header.date.getDay() === 6;
                    return (
                      <div
                        key={idx}
                        class={`w-[60px] flex-shrink-0 px-1 py-2 text-center text-xs border-r border-gray-100 ${isToday ? 'bg-blue-100' : isWeekend ? 'bg-gray-100' : ''}`}
                      >
                        {header.label}
                      </div>
                    );
                  })}
                </div>
              </div>

              {/* Issues */}
              {sortedIssues.map((issue) => (
                <GanttBar
                  key={issue.id}
                  issue={issue}
                  startDate={start}
                  totalDays={days}
                  orgName={orgName}
                  projectName={projectName}
                />
              ))}
            </div>
          </div>
        </Card>
      )}

      {/* Legend */}
      <div class="mt-4 flex gap-6 text-sm text-gray-600">
        <div class="flex items-center gap-2">
          <div class="w-4 h-4 rounded-full bg-gray-400"></div>
          <span>To Do</span>
        </div>
        <div class="flex items-center gap-2">
          <div class="w-4 h-4 rounded-full bg-blue-500"></div>
          <span>Doing</span>
        </div>
        <div class="flex items-center gap-2">
          <div class="w-4 h-4 rounded-full bg-green-500"></div>
          <span>Done</span>
        </div>
      </div>

      {/* Info about issues without dates */}
      {issues.value.filter(i => !i.start_date || !i.target_date).length > 0 && (
        <Text size="sm" c="dimmed" mt="md">
          Note: {issues.value.filter(i => !i.start_date || !i.target_date).length} issue(s) without start and/or target dates are not shown in the Gantt chart.
        </Text>
      )}
    </div>
  );
}
