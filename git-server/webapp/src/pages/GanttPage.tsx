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
  type Issue,
} from '../api';

function parseDate(dateStr: string | null): Date | null {
  if (!dateStr) return null;
  const date = new Date(dateStr);
  return isNaN(date.getTime()) ? null : date;
}

function formatDateShort(date: Date): string {
  return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
}

function getDateRange(issues: Issue[]): { start: Date; end: Date; days: number } {
  const today = new Date();
  today.setHours(0, 0, 0, 0);
  
  let minDate = new Date(today);
  let maxDate = new Date(today);
  maxDate.setDate(maxDate.getDate() + 30); // Default to 30 days ahead
  
  for (const issue of issues) {
    const startDate = parseDate(issue.start_date) || new Date(issue.created_at);
    const targetDate = parseDate(issue.target_date);
    
    if (startDate < minDate) minDate = new Date(startDate);
    if (targetDate && targetDate > maxDate) maxDate = new Date(targetDate);
    if (targetDate && targetDate < minDate) minDate = new Date(targetDate);
  }
  
  // Add some padding
  minDate.setDate(minDate.getDate() - 2);
  maxDate.setDate(maxDate.getDate() + 2);
  
  const days = Math.ceil((maxDate.getTime() - minDate.getTime()) / (1000 * 60 * 60 * 24));
  
  return { start: minDate, end: maxDate, days: Math.max(days, 14) };
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
  const issueStart = parseDate(issue.start_date) || new Date(issue.created_at);
  const targetDate = parseDate(issue.target_date);
  
  const startDay = Math.max(0, Math.floor((issueStart.getTime() - startDate.getTime()) / (1000 * 60 * 60 * 24)));
  
  let endDay: number;
  if (targetDate) {
    endDay = Math.floor((targetDate.getTime() - startDate.getTime()) / (1000 * 60 * 60 * 24));
  } else {
    endDay = startDay + 7; // Default to 7 days duration
  }
  
  const duration = Math.max(1, endDay - startDay);
  const leftPercent = (startDay / totalDays) * 100;
  const widthPercent = (duration / totalDays) * 100;
  
  const statusColor = issue.status === 'done' ? 'bg-green-500' : 
                      issue.status === 'doing' ? 'bg-blue-500' : 'bg-gray-400';
  
  return (
    <a
      href={`/${orgName}/${projectName}/issues/${issue.number}`}
      class="block no-underline"
    >
      <div class="flex items-center border-b border-gray-100 hover:bg-gray-50">
        <div class="w-64 flex-shrink-0 px-3 py-2 border-r border-gray-200">
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
        <div class="flex-1 relative h-10">
          <div
            class={`absolute top-2 h-6 ${statusColor} rounded-full opacity-80 hover:opacity-100 transition-opacity`}
            style={{ left: `${leftPercent}%`, width: `${Math.max(widthPercent, 2)}%` }}
            title={`${issue.title}\nStatus: ${issue.status}\n${targetDate ? `Target: ${issue.target_date}` : 'No target date'}`}
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

  const { start, days } = getDateRange(issues.value);
  const dayHeaders = getDayHeaders(start, days);
  const today = new Date();
  today.setHours(0, 0, 0, 0);

  // Sort issues: open first, then by target date
  const sortedIssues = [...issues.value].sort((a, b) => {
    if (a.state !== b.state) return a.state === 'open' ? -1 : 1;
    const aTarget = parseDate(a.target_date);
    const bTarget = parseDate(b.target_date);
    if (!aTarget && !bTarget) return 0;
    if (!aTarget) return 1;
    if (!bTarget) return -1;
    return aTarget.getTime() - bTarget.getTime();
  });

  return (
    <div>
      <Group justify="space-between" mb="lg">
        <h1 class="text-2xl font-bold">Gantt Chart</h1>
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
        <Card shadow="sm" padding="0" radius="md" withBorder class="overflow-x-auto">
          {/* Header */}
          <div class="flex border-b border-gray-200 bg-gray-50 sticky top-0">
            <div class="w-64 flex-shrink-0 px-3 py-2 border-r border-gray-200 font-medium">
              Issue
            </div>
            <div class="flex-1 flex">
              {dayHeaders.map((header, idx) => {
                const isToday = header.date.toDateString() === today.toDateString();
                const isWeekend = header.date.getDay() === 0 || header.date.getDay() === 6;
                return (
                  <div 
                    key={idx} 
                    class={`flex-1 min-w-[40px] px-1 py-2 text-center text-xs border-r border-gray-100 ${isToday ? 'bg-blue-100' : isWeekend ? 'bg-gray-100' : ''}`}
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
    </div>
  );
}
