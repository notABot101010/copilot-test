import { useEffect } from 'preact/hooks';
import type { JSX } from 'preact';
import { useSignal } from '@preact/signals';
import { ActionIcon, TextInput, Badge, SegmentedControl, Select } from '@mantine/core';
import type { DragEndEvent, DragOverEvent, DragStartEvent } from '@dnd-kit/core';
import {
  DndContext,
  closestCorners,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
  DragOverlay,
} from '@dnd-kit/core';
import {
  SortableContext,
  sortableKeyboardCoordinates,
  useSortable,
  verticalListSortingStrategy,
} from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import {
  databases,
  createDatabase,
  addDatabaseRow,
  updateDatabaseRow,
  removeDatabaseRow,
  addDatabaseColumn,
  removeDatabaseColumn,
  updateDatabase,
} from '../store';
import { generateId } from '../utils';
import type { Block, Database, DatabaseColumn, DatabaseRow, DatabaseView } from '../types';

interface DatabaseBlockProps {
  block: Block;
  pageId: string;
}

export function DatabaseBlock({ block }: DatabaseBlockProps) {
  const database = useSignal<Database | null>(null);
  const currentViewId = useSignal<string | null>(null);
  const activeId = useSignal<string | null>(null);

  useEffect(() => {
    let db = databases.value.find((d) => d.id === block.properties?.databaseId);
    if (!db) {
      db = createDatabase('Database');
    }
    database.value = db;
    currentViewId.value = db.views[0]?.id || null;
  }, [block.properties?.databaseId]);

  useEffect(() => {
    const db = databases.value.find((d) => d.id === database.value?.id);
    if (db) {
      database.value = db;
    }
  }, [databases.value]);

  const sensors = useSensors(
    useSensor(PointerSensor),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
    })
  );

  if (!database.value) return null;

  const currentView = database.value.views.find((v) => v.id === currentViewId.value);

  const handleAddRow = (statusValue?: string) => {
    if (!database.value) return;
    const statusColumn = database.value.columns.find((c) => c.type === 'select');
    const properties: Record<string, string | number | boolean | string[] | Date | null> = {};
    if (statusColumn && statusValue) {
      properties[statusColumn.id] = statusValue;
    }
    addDatabaseRow(database.value.id, { properties });
  };

  const handleDragStart = (event: DragStartEvent) => {
    activeId.value = event.active.id as string;
  };

  const handleDragOver = (event: DragOverEvent) => {
    const { active, over } = event;
    if (!over || !database.value) return;

    const activeRow = database.value.rows.find((r) => r.id === active.id);
    if (!activeRow) return;

    const overId = over.id as string;
    const statusColumn = database.value.columns.find((c) => c.type === 'select');
    if (!statusColumn) return;

    // Check if dropping on a column container
    if (statusColumn.options?.some((opt) => opt.name === overId)) {
      updateDatabaseRow(database.value.id, activeRow.id, {
        [statusColumn.id]: overId,
      });
    }
  };

  const handleDragEnd = (_event: DragEndEvent) => {
    activeId.value = null;
    // Row reordering can be implemented here if needed
  };

  return (
    <div className="rounded border border-zinc-700 bg-zinc-800/50">
      <div className="flex items-center justify-between border-b border-zinc-700 px-3 py-2">
        <TextInput
          variant="unstyled"
          value={database.value.name}
          onChange={(e: JSX.TargetedEvent<HTMLInputElement>) => {
            if (database.value) {
              updateDatabase(database.value.id, { name: e.currentTarget.value });
            }
          }}
          className="font-semibold"
          size="sm"
        />
        <div className="flex items-center gap-2">
          <SegmentedControl
            size="xs"
            value={currentViewId.value || ''}
            onChange={(value: string) => (currentViewId.value = value)}
            data={database.value.views.map((v) => ({
              label: v.name,
              value: v.id,
            }))}
          />
        </div>
      </div>

      <DndContext
        sensors={sensors}
        collisionDetection={closestCorners}
        onDragStart={handleDragStart}
        onDragOver={handleDragOver}
        onDragEnd={handleDragEnd}
      >
        {currentView?.type === 'table' ? (
          <DatabaseTableView
            database={database.value}
            onAddRow={() => handleAddRow()}
          />
        ) : (
          <DatabaseKanbanView
            database={database.value}
            view={currentView!}
            activeId={activeId.value}
            onAddRow={handleAddRow}
          />
        )}
      </DndContext>
    </div>
  );
}

interface DatabaseTableViewProps {
  database: Database;
  onAddRow: () => void;
}

function DatabaseTableView({ database, onAddRow }: DatabaseTableViewProps) {
  const handleAddColumn = () => {
    const newColumn: DatabaseColumn = {
      id: generateId(),
      name: `Column ${database.columns.length + 1}`,
      type: 'text',
    };
    addDatabaseColumn(database.id, newColumn);
  };

  return (
    <div className="overflow-x-auto">
      <table className="w-full border-collapse">
        <thead>
          <tr className="bg-zinc-800">
            {database.columns.map((column) => (
              <th
                key={column.id}
                className="border-b border-zinc-700 px-3 py-2 text-left text-sm font-medium [&:not(:first-child)]:border-l"
              >
                <div className="flex items-center justify-between gap-2">
                  <span>{column.name}</span>
                  {database.columns.length > 1 && (
                    <ActionIcon
                      variant="subtle"
                      color="red"
                      size="xs"
                      onClick={() => removeDatabaseColumn(database.id, column.id)}
                    >
                      <svg xmlns="http://www.w3.org/2000/svg" className="h-3 w-3" viewBox="0 0 20 20" fill="currentColor">
                        <path fillRule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clipRule="evenodd" />
                      </svg>
                    </ActionIcon>
                  )}
                </div>
              </th>
            ))}
            <th className="border-b border-l border-zinc-700 p-0 w-10">
              <ActionIcon
                variant="subtle"
                color="gray"
                className="m-1"
                onClick={handleAddColumn}
              >
                <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                  <path fillRule="evenodd" d="M10 3a1 1 0 011 1v5h5a1 1 0 110 2h-5v5a1 1 0 11-2 0v-5H4a1 1 0 110-2h5V4a1 1 0 011-1z" clipRule="evenodd" />
                </svg>
              </ActionIcon>
            </th>
          </tr>
        </thead>
        <tbody>
          {database.rows.map((row) => (
            <tr key={row.id} className="group/row">
              {database.columns.map((column) => (
                <td
                  key={column.id}
                  className="border-t border-zinc-700 p-0 [&:not(:first-child)]:border-l"
                >
                  <DatabaseCell
                    database={database}
                    row={row}
                    column={column}
                  />
                </td>
              ))}
              <td className="border-l border-t border-zinc-700 p-0">
                <ActionIcon
                  variant="subtle"
                  color="red"
                  className="m-1 opacity-0 group-hover/row:opacity-100"
                  onClick={() => removeDatabaseRow(database.id, row.id)}
                >
                  <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                    <path fillRule="evenodd" d="M9 2a1 1 0 00-.894.553L7.382 4H4a1 1 0 000 2v10a2 2 0 002 2h8a2 2 0 002-2V6a1 1 0 100-2h-3.382l-.724-1.447A1 1 0 0011 2H9zM7 8a1 1 0 012 0v6a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v6a1 1 0 102 0V8a1 1 0 00-1-1z" clipRule="evenodd" />
                  </svg>
                </ActionIcon>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
      <button
        className="flex w-full items-center justify-center gap-1 border-t border-zinc-700 py-2 text-sm text-zinc-400 transition-colors hover:bg-zinc-800 hover:text-white"
        onClick={onAddRow}
      >
        <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
          <path fillRule="evenodd" d="M10 3a1 1 0 011 1v5h5a1 1 0 110 2h-5v5a1 1 0 11-2 0v-5H4a1 1 0 110-2h5V4a1 1 0 011-1z" clipRule="evenodd" />
        </svg>
        New
      </button>
    </div>
  );
}

interface DatabaseKanbanViewProps {
  database: Database;
  view: DatabaseView;
  activeId: string | null;
  onAddRow: (statusValue?: string) => void;
}

function DatabaseKanbanView({ database, view, activeId, onAddRow }: DatabaseKanbanViewProps) {
  const groupByColumn = database.columns.find((c) => c.id === view.groupBy);
  
  if (!groupByColumn || groupByColumn.type !== 'select') {
    return (
      <div className="p-4 text-center text-zinc-400">
        No select column found for Kanban view
      </div>
    );
  }

  const groups = groupByColumn.options || [];

  const getRowsForGroup = (groupName: string) => {
    return database.rows.filter(
      (row) => row.properties[groupByColumn.id] === groupName
    );
  };

  const activeRow = activeId ? database.rows.find((r) => r.id === activeId) : null;

  return (
    <div className="flex gap-4 overflow-x-auto p-4">
      {groups.map((group) => {
        const rowsInGroup = getRowsForGroup(group.name);
        return (
          <div
            key={group.id}
            className="flex min-w-[280px] flex-col rounded-lg bg-zinc-800"
            id={group.name}
          >
            <div className="flex items-center justify-between px-3 py-2">
              <Badge color={group.color} variant="light">
                {group.name}
              </Badge>
              <span className="text-xs text-zinc-500">{rowsInGroup.length}</span>
            </div>
            
            <SortableContext
              items={rowsInGroup.map((r) => r.id)}
              strategy={verticalListSortingStrategy}
            >
              <div className="flex-1 space-y-2 p-2">
                {rowsInGroup.map((row) => (
                  <KanbanCard
                    key={row.id}
                    database={database}
                    row={row}
                  />
                ))}
              </div>
            </SortableContext>

            <button
              className="flex items-center gap-1 px-3 py-2 text-sm text-zinc-400 transition-colors hover:text-white"
              onClick={() => onAddRow(group.name)}
            >
              <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                <path fillRule="evenodd" d="M10 3a1 1 0 011 1v5h5a1 1 0 110 2h-5v5a1 1 0 11-2 0v-5H4a1 1 0 110-2h5V4a1 1 0 011-1z" clipRule="evenodd" />
              </svg>
              Add
            </button>
          </div>
        );
      })}

      <DragOverlay>
        {activeRow ? (
          <div className="rounded border border-zinc-600 bg-zinc-700 p-3 shadow-lg">
            <KanbanCardContent database={database} row={activeRow} />
          </div>
        ) : null}
      </DragOverlay>
    </div>
  );
}

interface KanbanCardProps {
  database: Database;
  row: DatabaseRow;
}

function KanbanCard({ database, row }: KanbanCardProps) {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id: row.id });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : 1,
  };

  const handleKeyDown = (e: JSX.TargetedKeyboardEvent<HTMLDivElement>) => {
    if (listeners?.onKeyDown) {
      listeners.onKeyDown(e as unknown as KeyboardEvent);
    }
  };

  return (
    <div
      ref={setNodeRef}
      style={style}
      tabIndex={attributes.tabIndex}
      onKeyDown={handleKeyDown}
      className="cursor-grab rounded border border-zinc-700 bg-zinc-900 p-3 transition-colors hover:border-zinc-600"
    >
      <KanbanCardContent database={database} row={row} />
    </div>
  );
}

interface KanbanCardContentProps {
  database: Database;
  row: DatabaseRow;
}

function KanbanCardContent({ database, row }: KanbanCardContentProps) {
  const nameColumn = database.columns.find((c) => c.name === 'Name');
  const name = nameColumn ? (row.properties[nameColumn.id] as string) || 'Untitled' : 'Untitled';

  return (
    <div>
      <p className="text-sm font-medium">{name}</p>
    </div>
  );
}

interface DatabaseCellProps {
  database: Database;
  row: DatabaseRow;
  column: DatabaseColumn;
}

function DatabaseCell({ database, row, column }: DatabaseCellProps) {
  const value = row.properties[column.id];

  const handleChange = (newValue: string | boolean) => {
    updateDatabaseRow(database.id, row.id, { [column.id]: newValue });
  };

  switch (column.type) {
    case 'select':
      return (
        <Select
          variant="unstyled"
          value={(value as string) || ''}
          onChange={(val: string | null) => handleChange(val || '')}
          data={
            column.options?.map((opt) => ({
              value: opt.name,
              label: opt.name,
            })) || []
          }
          className="px-2"
          size="xs"
        />
      );

    case 'checkbox':
      return (
        <div className="flex items-center justify-center p-2">
          <input
            type="checkbox"
            checked={Boolean(value)}
            onChange={(e) => handleChange((e.target as HTMLInputElement).checked)}
            className="h-4 w-4 rounded border-zinc-600 bg-zinc-700"
          />
        </div>
      );

    default:
      return (
        <input
          type={column.type === 'number' ? 'number' : 'text'}
          value={(value as string) || ''}
          onChange={(e) => handleChange((e.target as HTMLInputElement).value)}
          className="w-full bg-transparent px-2 py-1.5 text-sm outline-none"
          placeholder="Empty"
        />
      );
  }
}
