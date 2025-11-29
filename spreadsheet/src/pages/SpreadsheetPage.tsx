import { useEffect, useState, useCallback, useRef } from 'preact/hooks';
import { useRouter, useRoute } from '@copilot-test/preact-router';
import { Button, TextInput, Container, Group, ActionIcon, Text, Loader } from '@mantine/core';
import {
  currentSpreadsheet,
  loadSpreadsheet,
  updateCell,
  renameSpreadsheet,
} from '../store/spreadsheetStore';
import { getCellKey } from '../types/spreadsheet';

const NUM_ROWS = 50;
const NUM_COLS = 26;

function getColumnLabel(col: number): string {
  return String.fromCharCode(65 + col);
}

export function SpreadsheetPage() {
  const router = useRouter();
  const route = useRoute();
  const id = route.value.params.id;
  
  const [isLoading, setIsLoading] = useState(true);
  const [notFound, setNotFound] = useState(false);
  const [isEditingName, setIsEditingName] = useState(false);
  const [editName, setEditName] = useState('');
  const [selectedCell, setSelectedCell] = useState<{ row: number; col: number } | null>(null);
  const [editValue, setEditValue] = useState('');
  const editInputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (!id) {
      setNotFound(true);
      setIsLoading(false);
      return;
    }
    
    const success = loadSpreadsheet(id);
    if (!success) {
      setNotFound(true);
    }
    setIsLoading(false);
  }, [id]);

  const spreadsheet = currentSpreadsheet.value;

  const handleCellClick = useCallback((row: number, col: number) => {
    // Save previous cell if exists
    if (selectedCell) {
      const key = getCellKey(selectedCell.row, selectedCell.col);
      const currentValue = spreadsheet?.cells[key]?.value || '';
      if (editValue !== currentValue) {
        updateCell(selectedCell.row, selectedCell.col, editValue);
      }
    }
    
    setSelectedCell({ row, col });
    const key = getCellKey(row, col);
    setEditValue(spreadsheet?.cells[key]?.value || '');
  }, [selectedCell, editValue, spreadsheet]);

  const handleCellChange = useCallback((value: string) => {
    setEditValue(value);
  }, []);

  const handleCellBlur = useCallback(() => {
    if (selectedCell) {
      updateCell(selectedCell.row, selectedCell.col, editValue);
    }
  }, [selectedCell, editValue]);

  const handleCellKeyDown = useCallback((e: KeyboardEvent) => {
    if (!selectedCell) return;

    if (e.key === 'Enter') {
      updateCell(selectedCell.row, selectedCell.col, editValue);
      // Move to next row
      if (selectedCell.row < NUM_ROWS - 1) {
        const newRow = selectedCell.row + 1;
        setSelectedCell({ row: newRow, col: selectedCell.col });
        const key = getCellKey(newRow, selectedCell.col);
        setEditValue(spreadsheet?.cells[key]?.value || '');
      }
    } else if (e.key === 'Tab') {
      e.preventDefault();
      updateCell(selectedCell.row, selectedCell.col, editValue);
      // Move to next column
      if (selectedCell.col < NUM_COLS - 1) {
        const newCol = selectedCell.col + 1;
        setSelectedCell({ row: selectedCell.row, col: newCol });
        const key = getCellKey(selectedCell.row, newCol);
        setEditValue(spreadsheet?.cells[key]?.value || '');
      }
    } else if (e.key === 'Escape') {
      const key = getCellKey(selectedCell.row, selectedCell.col);
      setEditValue(spreadsheet?.cells[key]?.value || '');
      setSelectedCell(null);
    }
  }, [selectedCell, editValue, spreadsheet]);

  const handleNameEdit = () => {
    if (spreadsheet) {
      setEditName(spreadsheet.name);
      setIsEditingName(true);
    }
  };

  const handleNameSave = () => {
    if (editName.trim()) {
      renameSpreadsheet(editName.trim());
    }
    setIsEditingName(false);
  };

  const handleNameKeyDown = (e: KeyboardEvent) => {
    if (e.key === 'Enter') {
      handleNameSave();
    } else if (e.key === 'Escape') {
      setIsEditingName(false);
    }
  };

  const handleBack = () => {
    router.push('/');
  };

  if (isLoading) {
    return (
      <Container className="py-8 flex justify-center items-center h-screen">
        <Loader size="xl" />
      </Container>
    );
  }

  if (notFound || !spreadsheet) {
    return (
      <Container className="py-8">
        <Text size="xl" className="text-white mb-4">Spreadsheet not found</Text>
        <Button onClick={handleBack}>Back to Home</Button>
      </Container>
    );
  }

  return (
    <div className="h-screen flex flex-col bg-gray-900">
      {/* Header */}
      <div className="bg-gray-800 border-b border-gray-700 p-3 flex items-center gap-4">
        <ActionIcon onClick={handleBack} variant="subtle" size="lg">
          <span className="text-xl">←</span>
        </ActionIcon>
        
        {isEditingName ? (
          <TextInput
            value={editName}
            onChange={(e: Event) => setEditName((e.target as HTMLInputElement).value)}
            onBlur={handleNameSave}
            onKeyDown={handleNameKeyDown}
            autoFocus
            className="w-64"
          />
        ) : (
          <div
            onClick={handleNameEdit}
            className="text-white text-xl font-semibold cursor-pointer hover:bg-gray-700 px-2 py-1 rounded"
          >
            {spreadsheet.name}
          </div>
        )}
        
        <Group gap="sm" className="ml-auto">
          <Text size="sm" c="dimmed">
            ID: {spreadsheet.id.substring(0, 8)}...
          </Text>
        </Group>
      </div>

      {/* Formula Bar */}
      {selectedCell && (
        <div className="bg-gray-800 border-b border-gray-700 p-2 flex items-center gap-2">
          <span className="text-gray-400 font-mono w-12">
            {getColumnLabel(selectedCell.col)}{selectedCell.row + 1}
          </span>
          <TextInput
            ref={editInputRef}
            value={editValue}
            onChange={(e: Event) => handleCellChange((e.target as HTMLInputElement).value)}
            onBlur={handleCellBlur}
            onKeyDown={handleCellKeyDown}
            className="flex-1"
            placeholder="Enter value..."
          />
        </div>
      )}

      {/* Spreadsheet Grid */}
      <div className="flex-1 overflow-auto">
        <table className="border-collapse w-full">
          <thead>
            <tr className="sticky top-0 z-10">
              <th className="bg-gray-800 border border-gray-700 w-12 min-w-12 sticky left-0 z-20"></th>
              {Array.from({ length: NUM_COLS }, (_, col) => (
                <th
                  key={col}
                  className="bg-gray-800 border border-gray-700 text-gray-400 font-normal text-sm py-1 w-24 min-w-24"
                >
                  {getColumnLabel(col)}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {Array.from({ length: NUM_ROWS }, (_, row) => (
              <tr key={row}>
                <td className="bg-gray-800 border border-gray-700 text-gray-400 text-sm text-center sticky left-0 z-10">
                  {row + 1}
                </td>
                {Array.from({ length: NUM_COLS }, (_, col) => {
                  const key = getCellKey(row, col);
                  const cellValue = spreadsheet.cells[key]?.value || '';
                  const isSelected = selectedCell?.row === row && selectedCell?.col === col;
                  
                  return (
                    <td
                      key={col}
                      onClick={() => handleCellClick(row, col)}
                      className={`
                        border border-gray-700 text-white text-sm px-1 py-0.5 cursor-cell
                        ${isSelected ? 'bg-blue-900 border-blue-500 border-2' : 'bg-gray-900 hover:bg-gray-800'}
                      `}
                    >
                      {isSelected ? editValue : cellValue}
                    </td>
                  );
                })}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
