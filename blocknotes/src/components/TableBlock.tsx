import type { JSX } from 'preact';
import { ActionIcon, TextInput } from '@mantine/core';
import { useSignal } from '@preact/signals';
import { updateBlock } from '../store';
import { generateId } from '../utils';
import type { Block, TableData, TableRow } from '../types';

interface TableBlockProps {
  block: Block;
  pageId: string;
}

export function TableBlock({ block, pageId }: TableBlockProps) {
  const tableData: TableData = block.properties?.tableData || {
    columns: ['Column 1'],
    rows: [{ id: generateId(), cells: { 'Column 1': '' } }],
  };

  const updateTableData = (newData: TableData) => {
    updateBlock(pageId, block.id, {
      properties: { ...block.properties, tableData: newData },
    });
  };

  const addColumn = () => {
    const newColumnName = `Column ${tableData.columns.length + 1}`;
    const newColumns = [...tableData.columns, newColumnName];
    const newRows = tableData.rows.map((row) => ({
      ...row,
      cells: { ...row.cells, [newColumnName]: '' },
    }));
    updateTableData({ columns: newColumns, rows: newRows });
  };

  const removeColumn = (columnIndex: number) => {
    if (tableData.columns.length <= 1) return;
    const columnName = tableData.columns[columnIndex];
    const newColumns = tableData.columns.filter((_, i) => i !== columnIndex);
    const newRows = tableData.rows.map((row) => {
      const newCells = { ...row.cells };
      delete newCells[columnName];
      return { ...row, cells: newCells };
    });
    updateTableData({ columns: newColumns, rows: newRows });
  };

  const renameColumn = (columnIndex: number, newName: string) => {
    const oldName = tableData.columns[columnIndex];
    if (oldName === newName || tableData.columns.includes(newName)) return;
    
    const newColumns = tableData.columns.map((col, i) =>
      i === columnIndex ? newName : col
    );
    const newRows = tableData.rows.map((row) => {
      const newCells: Record<string, string> = {};
      Object.keys(row.cells).forEach((key) => {
        const newKey = key === oldName ? newName : key;
        newCells[newKey] = row.cells[key];
      });
      return { ...row, cells: newCells };
    });
    updateTableData({ columns: newColumns, rows: newRows });
  };

  const addRow = () => {
    const newRow: TableRow = {
      id: generateId(),
      cells: tableData.columns.reduce((acc, col) => {
        acc[col] = '';
        return acc;
      }, {} as Record<string, string>),
    };
    updateTableData({ ...tableData, rows: [...tableData.rows, newRow] });
  };

  const removeRow = (rowId: string) => {
    if (tableData.rows.length <= 1) return;
    updateTableData({
      ...tableData,
      rows: tableData.rows.filter((row) => row.id !== rowId),
    });
  };

  const updateCell = (rowId: string, columnName: string, value: string) => {
    const newRows = tableData.rows.map((row) =>
      row.id === rowId
        ? { ...row, cells: { ...row.cells, [columnName]: value } }
        : row
    );
    updateTableData({ ...tableData, rows: newRows });
  };

  return (
    <div className="overflow-x-auto rounded border border-zinc-700">
      <table className="w-full border-collapse">
        <thead>
          <tr className="bg-zinc-800">
            {tableData.columns.map((column, index) => (
              <TableHeaderCell
                key={column}
                column={column}
                columnIndex={index}
                onRename={(newName) => renameColumn(index, newName)}
                onRemove={() => removeColumn(index)}
                canRemove={tableData.columns.length > 1}
              />
            ))}
            <th className="border-l border-zinc-700 p-0">
              <ActionIcon
                variant="subtle"
                color="gray"
                className="m-1"
                onClick={addColumn}
                aria-label="Add column"
              >
                <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                  <path fillRule="evenodd" d="M10 3a1 1 0 011 1v5h5a1 1 0 110 2h-5v5a1 1 0 11-2 0v-5H4a1 1 0 110-2h5V4a1 1 0 011-1z" clipRule="evenodd" />
                </svg>
              </ActionIcon>
            </th>
          </tr>
        </thead>
        <tbody>
          {tableData.rows.map((row) => (
            <tr key={row.id} className="group/row">
              {tableData.columns.map((column) => (
                <td key={column} className="border-t border-zinc-700 p-0">
                  <input
                    type="text"
                    value={row.cells[column] || ''}
                    onChange={(e) =>
                      updateCell(row.id, column, (e.target as HTMLInputElement).value)
                    }
                    className="w-full bg-transparent px-2 py-1.5 text-sm outline-none"
                    placeholder="Empty"
                  />
                </td>
              ))}
              <td className="border-l border-t border-zinc-700 p-0">
                <ActionIcon
                  variant="subtle"
                  color="red"
                  className="m-1 opacity-0 group-hover/row:opacity-100"
                  onClick={() => removeRow(row.id)}
                  disabled={tableData.rows.length <= 1}
                  aria-label="Remove row"
                >
                  <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                    <path fillRule="evenodd" d="M3 10a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1z" clipRule="evenodd" />
                  </svg>
                </ActionIcon>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
      <button
        className="flex w-full items-center justify-center gap-1 border-t border-zinc-700 py-2 text-sm text-zinc-400 transition-colors hover:bg-zinc-800 hover:text-white"
        onClick={addRow}
      >
        <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
          <path fillRule="evenodd" d="M10 3a1 1 0 011 1v5h5a1 1 0 110 2h-5v5a1 1 0 11-2 0v-5H4a1 1 0 110-2h5V4a1 1 0 011-1z" clipRule="evenodd" />
        </svg>
        Add Row
      </button>
    </div>
  );
}

interface TableHeaderCellProps {
  column: string;
  columnIndex: number;
  onRename: (newName: string) => void;
  onRemove: () => void;
  canRemove: boolean;
}

function TableHeaderCell({
  column,
  onRename,
  onRemove,
  canRemove,
}: TableHeaderCellProps) {
  const isEditing = useSignal(false);
  const editValue = useSignal(column);

  const handleSave = () => {
    if (editValue.value.trim()) {
      onRename(editValue.value.trim());
    }
    isEditing.value = false;
  };

  return (
    <th className="group/header border-zinc-700 p-0 text-left font-medium [&:not(:first-child)]:border-l">
      <div className="flex items-center">
        {isEditing.value ? (
          <TextInput
            size="xs"
            value={editValue.value}
            onChange={(e: JSX.TargetedEvent<HTMLInputElement>) => (editValue.value = e.currentTarget.value)}
            onBlur={handleSave}
            onKeyDown={(e: JSX.TargetedKeyboardEvent<HTMLInputElement>) => {
              if (e.key === 'Enter') handleSave();
              if (e.key === 'Escape') {
                editValue.value = column;
                isEditing.value = false;
              }
            }}
            className="m-1 flex-1"
            autoFocus
          />
        ) : (
          <span
            className="flex-1 cursor-pointer px-2 py-1.5 text-sm"
            onDblClick={() => {
              editValue.value = column;
              isEditing.value = true;
            }}
          >
            {column}
          </span>
        )}
        {canRemove && (
          <ActionIcon
            variant="subtle"
            color="red"
            size="xs"
            className="mr-1 opacity-0 group-hover/header:opacity-100"
            onClick={onRemove}
            aria-label="Remove column"
          >
            <svg xmlns="http://www.w3.org/2000/svg" className="h-3 w-3" viewBox="0 0 20 20" fill="currentColor">
              <path fillRule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clipRule="evenodd" />
            </svg>
          </ActionIcon>
        )}
      </div>
    </th>
  );
}
