import { useSignal, useSignalEffect, useComputed } from '@preact/signals';
import {
  Card,
  Text,
  Loader,
  Alert,
  Group,
  Button,
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

function isSameDay(date1: Date, date2: Date): boolean {
  return date1.getFullYear() === date2.getFullYear() &&
    date1.getMonth() === date2.getMonth() &&
    date1.getDate() === date2.getDate();
}

function getMonthDays(year: number, month: number): Date[] {
  const days: Date[] = [];
  const firstDay = new Date(year, month, 1);
  const lastDay = new Date(year, month + 1, 0);
  
  // Add padding for days before the first of the month
  const startPadding = firstDay.getDay();
  for (let i = startPadding - 1; i >= 0; i--) {
    const date = new Date(year, month, -i);
    days.push(date);
  }
  
  // Add days of the month
  for (let i = 1; i <= lastDay.getDate(); i++) {
    days.push(new Date(year, month, i));
  }
  
  // Add padding for days after the end of the month
  const endPadding = 42 - days.length; // 6 rows * 7 days
  for (let i = 1; i <= endPadding; i++) {
    days.push(new Date(year, month + 1, i));
  }
  
  return days;
}

function getMonthName(month: number): string {
  return new Date(2000, month, 1).toLocaleDateString('en-US', { month: 'long' });
}

interface CalendarDayProps {
  date: Date;
  issues: Issue[];
  isCurrentMonth: boolean;
  isToday: boolean;
  orgName: string;
  projectName: string;
}

function CalendarDay({ date, issues, isCurrentMonth, isToday, orgName, projectName }: CalendarDayProps) {
  return (
    <div 
      class={`min-h-[100px] p-1 border-r border-b border-gray-200 ${
        isCurrentMonth ? 'bg-white' : 'bg-gray-50'
      } ${isToday ? 'ring-2 ring-blue-500 ring-inset' : ''}`}
    >
      <div class={`text-sm mb-1 ${isCurrentMonth ? 'text-gray-900' : 'text-gray-400'} ${isToday ? 'font-bold text-blue-600' : ''}`}>
        {date.getDate()}
      </div>
      <div class="space-y-1 overflow-y-auto max-h-[80px]">
        {issues.map((issue) => (
          <a
            key={issue.id}
            href={`/${orgName}/${projectName}/issues/${issue.number}`}
            class="block no-underline"
          >
            <div 
              class={`text-xs px-1 py-0.5 rounded truncate ${
                issue.status === 'done' ? 'bg-green-100 text-green-800' :
                issue.status === 'doing' ? 'bg-blue-100 text-blue-800' :
                'bg-gray-100 text-gray-800'
              } hover:opacity-80`}
              title={issue.title}
            >
              #{issue.number} {issue.title}
            </div>
          </a>
        ))}
      </div>
    </div>
  );
}

export function CalendarPage() {
  const route = useRoute();
  const issues = useSignal<Issue[]>([]);
  const loading = useSignal(true);
  const error = useSignal<string | null>(null);
  
  const today = new Date();
  const currentYear = useSignal(today.getFullYear());
  const currentMonth = useSignal(today.getMonth());

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

  const calendarDays = useComputed(() => 
    getMonthDays(currentYear.value, currentMonth.value)
  );

  const issuesByDate = useComputed(() => {
    const map = new Map<string, Issue[]>();
    for (const issue of issues.value) {
      const startDate = parseDate(issue.start_date);
      const targetDate = parseDate(issue.target_date);

      if (targetDate) {
        if (startDate && startDate <= targetDate) {
          // Add issue to all days between start and target
          const current = new Date(startDate);
          while (current <= targetDate) {
            const key = current.toISOString().split('T')[0];
            if (!map.has(key)) {
              map.set(key, []);
            }
            map.get(key)!.push(issue);
            current.setDate(current.getDate() + 1);
          }
        } else {
          // No start date or start > target, just show on target date
          const key = targetDate.toISOString().split('T')[0];
          if (!map.has(key)) {
            map.set(key, []);
          }
          map.get(key)!.push(issue);
        }
      }
    }
    return map;
  });

  function prevMonth() {
    if (currentMonth.value === 0) {
      currentMonth.value = 11;
      currentYear.value -= 1;
    } else {
      currentMonth.value -= 1;
    }
  }

  function nextMonth() {
    if (currentMonth.value === 11) {
      currentMonth.value = 0;
      currentYear.value += 1;
    } else {
      currentMonth.value += 1;
    }
  }

  function goToToday() {
    currentYear.value = today.getFullYear();
    currentMonth.value = today.getMonth();
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

  const weekDays = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];

  return (
    <div>
      <Group justify="space-between" mb="lg">
        <h1 class="text-2xl font-bold">Calendar</h1>
        <a href={`/${orgName}/${projectName}/issues/new`}>
          <button class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700">
            New Issue
          </button>
        </a>
      </Group>

      <Card shadow="sm" padding="0" radius="md" withBorder>
        {/* Header */}
        <div class="flex items-center justify-between p-3 border-b border-gray-200 bg-gray-50">
          <Group>
            <Button variant="subtle" onClick={prevMonth} size="sm">
              &lt;
            </Button>
            <Button variant="subtle" onClick={goToToday} size="sm">
              Today
            </Button>
            <Button variant="subtle" onClick={nextMonth} size="sm">
              &gt;
            </Button>
          </Group>
          <Text fw={600} size="lg">
            {getMonthName(currentMonth.value)} {currentYear.value}
          </Text>
          <div class="w-[120px]"></div>
        </div>

        {/* Week days header */}
        <div class="grid grid-cols-7 border-b border-gray-200">
          {weekDays.map((day) => (
            <div key={day} class="p-2 text-center text-sm font-medium text-gray-600 border-r border-gray-200">
              {day}
            </div>
          ))}
        </div>

        {/* Calendar grid */}
        <div class="grid grid-cols-7">
          {calendarDays.value.map((date, idx) => {
            const dateKey = date.toISOString().split('T')[0];
            const dayIssues = issuesByDate.value.get(dateKey) || [];
            const isCurrentMonth = date.getMonth() === currentMonth.value;
            const isToday = isSameDay(date, today);
            
            return (
              <CalendarDay
                key={idx}
                date={date}
                issues={dayIssues}
                isCurrentMonth={isCurrentMonth}
                isToday={isToday}
                orgName={orgName}
                projectName={projectName}
              />
            );
          })}
        </div>
      </Card>

      {/* Legend */}
      <div class="mt-4 flex gap-6 text-sm text-gray-600">
        <div class="flex items-center gap-2">
          <div class="w-4 h-4 rounded bg-gray-100"></div>
          <span>To Do</span>
        </div>
        <div class="flex items-center gap-2">
          <div class="w-4 h-4 rounded bg-blue-100"></div>
          <span>Doing</span>
        </div>
        <div class="flex items-center gap-2">
          <div class="w-4 h-4 rounded bg-green-100"></div>
          <span>Done</span>
        </div>
      </div>

      {/* Info about issues without target dates */}
      {issues.value.filter(i => !i.target_date).length > 0 && (
        <Text size="sm" c="dimmed" mt="md">
          Note: {issues.value.filter(i => !i.target_date).length} issue(s) without target dates are not shown in the calendar.
        </Text>
      )}
    </div>
  );
}
