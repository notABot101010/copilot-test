import { useEffect, useState, useCallback, useRef, useMemo } from 'preact/hooks';
import { useRouter, useRoute } from '@copilot-test/preact-router';
import { Button, TextInput, Group, ActionIcon, Text, Loader, Tooltip } from '@mantine/core';
import {
  currentSpreadsheet,
  loadSpreadsheet,
  updateCell,
  renameSpreadsheet,
  updateMultipleCells,
  undo,
  redo,
  canUndo,
  canRedo,
} from '../store/spreadsheetStore';
import { getCellKey } from '../types/spreadsheet';
import { getComputedValue, indexToColumn } from '../utils/formulaEngine';

// Virtual scrolling configuration
const CELL_HEIGHT = 28;
const ROW_HEADER_WIDTH = 50;
const DEFAULT_COLUMN_WIDTH = 100;
const OVERSCAN = 5;

// Infinite grid size
const TOTAL_ROWS = 10000;
const TOTAL_COLS = 702; // A to ZZ

interface Selection {
  start: { row: number; col: number };
  end: { row: number; col: number };
}

interface ColumnWidths {
  [col: number]: number;
}

function getColumnLabel(col: number): string {
  return indexToColumn(col);
}

function normalizeSelection(sel: Selection): Selection {
  return {
    start: {
      row: Math.min(sel.start.row, sel.end.row),
      col: Math.min(sel.start.col, sel.end.col),
    },
    end: {
      row: Math.max(sel.start.row, sel.end.row),
      col: Math.max(sel.start.col, sel.end.col),
    },
  };
}

function isCellInSelection(row: number, col: number, selection: Selection | null): boolean {
  if (!selection) return false;
  const norm = normalizeSelection(selection);
  return row >= norm.start.row && row <= norm.end.row && col >= norm.start.col && col <= norm.end.col;
}

interface SelectionBorders {
  top: boolean;
  right: boolean;
  bottom: boolean;
  left: boolean;
}

function getSelectionBorders(row: number, col: number, selection: Selection | null): SelectionBorders | null {
  if (!selection) return null;
  const norm = normalizeSelection(selection);
  
  // Check if cell is in selection
  if (row < norm.start.row || row > norm.end.row || col < norm.start.col || col > norm.end.col) {
    return null;
  }
  
  return {
    top: row === norm.start.row,
    right: col === norm.end.col,
    bottom: row === norm.end.row,
    left: col === norm.start.col,
  };
}

export function SpreadsheetPage() {
  const router = useRouter();
  const route = useRoute();
  const id = route.value.params.id;
  
  const [isLoading, setIsLoading] = useState(true);
  const [notFound, setNotFound] = useState(false);
  const [isEditingName, setIsEditingName] = useState(false);
  const [editName, setEditName] = useState('');
  
  // Selection state
  const [selection, setSelection] = useState<Selection | null>(null);
  const [isSelecting, setIsSelecting] = useState(false);
  
  // Inline editing state
  const [editingCell, setEditingCell] = useState<{ row: number; col: number } | null>(null);
  const [editValue, setEditValue] = useState('');
  
  // Column widths
  const [columnWidths, setColumnWidths] = useState<ColumnWidths>({});
  const [resizingColumn, setResizingColumn] = useState<number | null>(null);
  const resizeStartX = useRef(0);
  const resizeStartWidth = useRef(0);
  
  // Virtual scrolling state
  const containerRef = useRef<HTMLDivElement>(null);
  const [scrollTop, setScrollTop] = useState(0);
  const [scrollLeft, setScrollLeft] = useState(0);
  const [containerHeight, setContainerHeight] = useState(600);
  const [containerWidth, setContainerWidth] = useState(800);
  
  // Formula bar ref
  const formulaInputRef = useRef<HTMLInputElement>(null);
  const cellInputRef = useRef<HTMLInputElement>(null);

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

  // Update container dimensions
  useEffect(() => {
    const updateDimensions = () => {
      if (containerRef.current) {
        setContainerHeight(containerRef.current.clientHeight);
        setContainerWidth(containerRef.current.clientWidth);
      }
    };
    
    updateDimensions();
    window.addEventListener('resize', updateDimensions);
    return () => window.removeEventListener('resize', updateDimensions);
  }, []);

  const spreadsheet = currentSpreadsheet.value;

  // Calculate visible rows
  const visibleRowStart = Math.max(0, Math.floor(scrollTop / CELL_HEIGHT) - OVERSCAN);
  const visibleRowEnd = Math.min(
    TOTAL_ROWS,
    Math.ceil((scrollTop + containerHeight) / CELL_HEIGHT) + OVERSCAN
  );

  // Calculate column positions incrementally - only recalculate changed positions
  const columnPositions = useMemo(() => {
    const positions: number[] = [0];
    let currentX = 0;
    // Only calculate positions up to visible area + buffer for performance
    const maxNeededCol = Math.min(TOTAL_COLS, Math.ceil((scrollLeft + containerWidth) / DEFAULT_COLUMN_WIDTH) + OVERSCAN * 10);
    for (let col = 0; col < maxNeededCol; col++) {
      currentX += columnWidths[col] || DEFAULT_COLUMN_WIDTH;
      positions.push(currentX);
    }
    // Add remaining positions with default width
    for (let col = maxNeededCol; col <= TOTAL_COLS; col++) {
      positions.push(currentX + (col - maxNeededCol + 1) * DEFAULT_COLUMN_WIDTH);
    }
    return positions;
  }, [columnWidths, scrollLeft, containerWidth]);

  const totalWidth = columnPositions[TOTAL_COLS] || TOTAL_COLS * DEFAULT_COLUMN_WIDTH;
  const totalHeight = TOTAL_ROWS * CELL_HEIGHT;

  // Binary search for finding the visible column start
  const visibleColStart = useMemo(() => {
    const targetX = Math.max(0, scrollLeft - DEFAULT_COLUMN_WIDTH * OVERSCAN);
    let low = 0;
    let high = columnPositions.length - 1;
    while (low < high) {
      const mid = Math.floor((low + high) / 2);
      if (columnPositions[mid] < targetX) {
        low = mid + 1;
      } else {
        high = mid;
      }
    }
    return Math.max(0, low - 1);
  }, [scrollLeft, columnPositions]);

  const visibleColEnd = useMemo(() => {
    const targetX = scrollLeft + containerWidth + DEFAULT_COLUMN_WIDTH * OVERSCAN;
    let low = visibleColStart;
    let high = columnPositions.length - 1;
    while (low < high) {
      const mid = Math.floor((low + high) / 2);
      if (columnPositions[mid] <= targetX) {
        low = mid + 1;
      } else {
        high = mid;
      }
    }
    return Math.min(TOTAL_COLS, low);
  }, [scrollLeft, containerWidth, visibleColStart, columnPositions]);

  // Handle scroll
  const handleScroll = useCallback((e: Event) => {
    const target = e.target as HTMLDivElement;
    setScrollTop(target.scrollTop);
    setScrollLeft(target.scrollLeft);
  }, []);

  // Cell click handler
  const handleCellMouseDown = useCallback((row: number, col: number, e: MouseEvent) => {
    e.preventDefault();
    
    if (editingCell) {
      // Save current edit
      updateCell(editingCell.row, editingCell.col, editValue);
      setEditingCell(null);
    }
    
    if (e.shiftKey && selection) {
      // Extend selection
      setSelection({
        start: selection.start,
        end: { row, col },
      });
    } else {
      // Start new selection
      setSelection({
        start: { row, col },
        end: { row, col },
      });
      setIsSelecting(true);
    }
  }, [editingCell, editValue, selection]);

  // Mouse move for selection
  const handleCellMouseEnter = useCallback((row: number, col: number) => {
    if (isSelecting && selection) {
      setSelection({
        start: selection.start,
        end: { row, col },
      });
    }
  }, [isSelecting, selection]);

  // Mouse up
  useEffect(() => {
    const handleMouseUp = () => {
      setIsSelecting(false);
    };
    
    window.addEventListener('mouseup', handleMouseUp);
    return () => window.removeEventListener('mouseup', handleMouseUp);
  }, []);

  // Double click to edit
  const handleCellDoubleClick = useCallback((row: number, col: number) => {
    setEditingCell({ row, col });
    const key = getCellKey(row, col);
    setEditValue(spreadsheet?.cells[key]?.value || '');
    setSelection({ start: { row, col }, end: { row, col } });
    setTimeout(() => cellInputRef.current?.focus(), 0);
  }, [spreadsheet]);

  // Handle typing to start editing
  const handleKeyDown = useCallback((e: KeyboardEvent) => {
    if (!selection || editingCell) return;
    
    // Arrow key navigation
    if (e.key === 'ArrowUp' || e.key === 'ArrowDown' || e.key === 'ArrowLeft' || e.key === 'ArrowRight') {
      e.preventDefault();
      const newRow = selection.end.row + (e.key === 'ArrowDown' ? 1 : e.key === 'ArrowUp' ? -1 : 0);
      const newCol = selection.end.col + (e.key === 'ArrowRight' ? 1 : e.key === 'ArrowLeft' ? -1 : 0);
      
      if (newRow >= 0 && newRow < TOTAL_ROWS && newCol >= 0 && newCol < TOTAL_COLS) {
        if (e.shiftKey) {
          setSelection({ start: selection.start, end: { row: newRow, col: newCol } });
        } else {
          setSelection({ start: { row: newRow, col: newCol }, end: { row: newRow, col: newCol } });
        }
        
        // Scroll into view if needed
        const targetTop = newRow * CELL_HEIGHT;
        const targetLeft = columnPositions[newCol];
        const targetRight = columnPositions[newCol + 1];
        
        if (containerRef.current) {
          if (targetTop < scrollTop) {
            containerRef.current.scrollTop = targetTop;
          } else if (targetTop + CELL_HEIGHT > scrollTop + containerHeight - 50) {
            containerRef.current.scrollTop = targetTop - containerHeight + CELL_HEIGHT + 50;
          }
          
          if (targetLeft < scrollLeft + ROW_HEADER_WIDTH) {
            containerRef.current.scrollLeft = targetLeft - ROW_HEADER_WIDTH;
          } else if (targetRight > scrollLeft + containerWidth - 20) {
            containerRef.current.scrollLeft = targetRight - containerWidth + 20;
          }
        }
      }
      return;
    }
    
    // Tab navigation
    if (e.key === 'Tab') {
      e.preventDefault();
      const direction = e.shiftKey ? -1 : 1;
      const newCol = selection.end.col + direction;
      if (newCol >= 0 && newCol < TOTAL_COLS) {
        setSelection({ start: { row: selection.end.row, col: newCol }, end: { row: selection.end.row, col: newCol } });
      }
      return;
    }
    
    // Enter to move down or start editing
    if (e.key === 'Enter') {
      e.preventDefault();
      if (e.shiftKey) {
        const newRow = selection.end.row - 1;
        if (newRow >= 0) {
          setSelection({ start: { row: newRow, col: selection.end.col }, end: { row: newRow, col: selection.end.col } });
        }
      } else {
        const newRow = selection.end.row + 1;
        if (newRow < TOTAL_ROWS) {
          setSelection({ start: { row: newRow, col: selection.end.col }, end: { row: newRow, col: selection.end.col } });
        }
      }
      return;
    }
    
    // Delete/Backspace to clear cells
    if (e.key === 'Delete' || e.key === 'Backspace') {
      e.preventDefault();
      const norm = normalizeSelection(selection);
      const updates: { row: number; col: number; value: string }[] = [];
      for (let r = norm.start.row; r <= norm.end.row; r++) {
        for (let c = norm.start.col; c <= norm.end.col; c++) {
          updates.push({ row: r, col: c, value: '' });
        }
      }
      updateMultipleCells(updates);
      return;
    }
    
    // Undo/Redo
    if ((e.ctrlKey || e.metaKey) && e.key === 'z') {
      e.preventDefault();
      if (e.shiftKey) {
        redo();
      } else {
        undo();
      }
      return;
    }
    
    if ((e.ctrlKey || e.metaKey) && e.key === 'y') {
      e.preventDefault();
      redo();
      return;
    }
    
    // Start typing to edit
    if (e.key.length === 1 && !e.ctrlKey && !e.metaKey && !e.altKey) {
      setEditingCell({ row: selection.end.row, col: selection.end.col });
      setEditValue(e.key);
      setTimeout(() => cellInputRef.current?.focus(), 0);
    }
    
    // F2 to edit
    if (e.key === 'F2') {
      e.preventDefault();
      const key = getCellKey(selection.end.row, selection.end.col);
      setEditingCell({ row: selection.end.row, col: selection.end.col });
      setEditValue(spreadsheet?.cells[key]?.value || '');
      setTimeout(() => cellInputRef.current?.focus(), 0);
    }
  }, [selection, editingCell, spreadsheet, scrollTop, scrollLeft, containerHeight, containerWidth, columnPositions]);

  // Inline edit key handler
  const handleEditKeyDown = useCallback((e: KeyboardEvent) => {
    if (!editingCell) return;
    
    if (e.key === 'Enter') {
      e.preventDefault();
      updateCell(editingCell.row, editingCell.col, editValue);
      setEditingCell(null);
      
      // Move to next row
      const newRow = editingCell.row + 1;
      if (newRow < TOTAL_ROWS) {
        setSelection({ start: { row: newRow, col: editingCell.col }, end: { row: newRow, col: editingCell.col } });
      }
    } else if (e.key === 'Tab') {
      e.preventDefault();
      updateCell(editingCell.row, editingCell.col, editValue);
      setEditingCell(null);
      
      // Move to next column
      const newCol = e.shiftKey ? editingCell.col - 1 : editingCell.col + 1;
      if (newCol >= 0 && newCol < TOTAL_COLS) {
        setSelection({ start: { row: editingCell.row, col: newCol }, end: { row: editingCell.row, col: newCol } });
      }
    } else if (e.key === 'Escape') {
      setEditingCell(null);
    }
  }, [editingCell, editValue]);

  // Global keyboard handler
  useEffect(() => {
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [handleKeyDown]);

  // Copy/Paste handlers
  useEffect(() => {
    const handleCopy = (e: ClipboardEvent) => {
      if (!selection || editingCell) return;
      
      const norm = normalizeSelection(selection);
      const lines: string[] = [];
      
      for (let r = norm.start.row; r <= norm.end.row; r++) {
        const rowValues: string[] = [];
        for (let c = norm.start.col; c <= norm.end.col; c++) {
          const key = getCellKey(r, c);
          rowValues.push(spreadsheet?.cells[key]?.value || '');
        }
        lines.push(rowValues.join('\t'));
      }
      
      e.clipboardData?.setData('text/plain', lines.join('\n'));
      e.preventDefault();
    };
    
    const handlePaste = async (e: ClipboardEvent) => {
      if (!selection || editingCell) return;
      
      const text = e.clipboardData?.getData('text/plain');
      if (!text) return;
      
      e.preventDefault();
      
      const lines = text.split('\n');
      const updates: { row: number; col: number; value: string }[] = [];
      
      for (let r = 0; r < lines.length; r++) {
        const values = lines[r].split('\t');
        for (let c = 0; c < values.length; c++) {
          const targetRow = selection.start.row + r;
          const targetCol = selection.start.col + c;
          if (targetRow < TOTAL_ROWS && targetCol < TOTAL_COLS) {
            updates.push({ row: targetRow, col: targetCol, value: values[c] });
          }
        }
      }
      
      updateMultipleCells(updates);
    };
    
    document.addEventListener('copy', handleCopy);
    document.addEventListener('paste', handlePaste);
    
    return () => {
      document.removeEventListener('copy', handleCopy);
      document.removeEventListener('paste', handlePaste);
    };
  }, [selection, editingCell, spreadsheet]);

  // Column resize handlers
  const handleColumnResizeStart = useCallback((col: number, e: MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setResizingColumn(col);
    resizeStartX.current = e.clientX;
    resizeStartWidth.current = columnWidths[col] || DEFAULT_COLUMN_WIDTH;
  }, [columnWidths]);

  useEffect(() => {
    if (resizingColumn === null) return;
    
    const handleMouseMove = (e: MouseEvent) => {
      const diff = e.clientX - resizeStartX.current;
      const newWidth = Math.max(40, resizeStartWidth.current + diff);
      setColumnWidths(prev => ({ ...prev, [resizingColumn]: newWidth }));
    };
    
    const handleMouseUp = () => {
      setResizingColumn(null);
    };
    
    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('mouseup', handleMouseUp);
    
    return () => {
      window.removeEventListener('mousemove', handleMouseMove);
      window.removeEventListener('mouseup', handleMouseUp);
    };
  }, [resizingColumn]);

  // Header handlers
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
    e.stopPropagation();
    if (e.key === 'Enter') {
      handleNameSave();
    } else if (e.key === 'Escape') {
      setIsEditingName(false);
    }
  };

  const handleBack = () => {
    router.push('/');
  };

  // Formula bar change
  const handleFormulaChange = useCallback((value: string) => {
    setEditValue(value);
    if (selection && !editingCell) {
      setEditingCell({ row: selection.end.row, col: selection.end.col });
    }
  }, [selection, editingCell]);

  const handleFormulaBlur = useCallback(() => {
    if (editingCell) {
      updateCell(editingCell.row, editingCell.col, editValue);
      setEditingCell(null);
    }
  }, [editingCell, editValue]);

  const handleFormulaKeyDown = useCallback((e: KeyboardEvent) => {
    e.stopPropagation();
    if (e.key === 'Enter') {
      e.preventDefault();
      if (editingCell) {
        updateCell(editingCell.row, editingCell.col, editValue);
        setEditingCell(null);
      }
    } else if (e.key === 'Escape') {
      setEditingCell(null);
    }
  }, [editingCell, editValue]);

  // Select all in column/row
  const handleColumnHeaderClick = useCallback((col: number) => {
    setSelection({
      start: { row: 0, col },
      end: { row: TOTAL_ROWS - 1, col },
    });
  }, []);

  const handleRowHeaderClick = useCallback((row: number) => {
    setSelection({
      start: { row, col: 0 },
      end: { row, col: TOTAL_COLS - 1 },
    });
  }, []);

  if (isLoading) {
    return (
      <div className="py-8 flex justify-center items-center h-screen">
        <Loader size="xl" />
      </div>
    );
  }

  if (notFound || !spreadsheet) {
    return (
      <div className="py-8 px-4">
        <Text size="xl" className="text-white mb-4">Spreadsheet not found</Text>
        <Button onClick={handleBack}>Back to Home</Button>
      </div>
    );
  }

  const selectedCellKey = selection ? getCellKey(selection.end.row, selection.end.col) : null;
  const selectedCellValue = selectedCellKey ? spreadsheet.cells[selectedCellKey]?.value || '' : '';
  const displayEditValue = editingCell ? editValue : selectedCellValue;

  return (
    <div className="h-screen flex flex-col bg-gray-900 select-none">
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
            onClick={(e: MouseEvent) => e.stopPropagation()}
          />
        ) : (
          <div
            onClick={handleNameEdit}
            className="text-white text-xl font-semibold cursor-pointer hover:bg-gray-700 px-2 py-1 rounded"
          >
            {spreadsheet.name}
          </div>
        )}
        
        {/* Toolbar */}
        <Group gap="xs" className="ml-4">
          <Tooltip label="Undo (Ctrl+Z)">
            <ActionIcon
              variant="subtle"
              disabled={!canUndo()}
              onClick={() => undo()}
            >
              <span className="text-lg">↶</span>
            </ActionIcon>
          </Tooltip>
          <Tooltip label="Redo (Ctrl+Y)">
            <ActionIcon
              variant="subtle"
              disabled={!canRedo()}
              onClick={() => redo()}
            >
              <span className="text-lg">↷</span>
            </ActionIcon>
          </Tooltip>
        </Group>
        
        <Group gap="sm" className="ml-auto">
          <Text size="sm" c="dimmed">
            ID: {spreadsheet.id.substring(0, 8)}...
          </Text>
        </Group>
      </div>

      {/* Formula Bar */}
      <div className="bg-gray-800 border-b border-gray-700 p-2 flex items-center gap-2">
        <span className="text-gray-400 font-mono w-16 text-sm">
          {selection ? `${getColumnLabel(selection.end.col)}${selection.end.row + 1}` : ''}
        </span>
        <span className="text-gray-600">|</span>
        <span className="text-gray-400 text-sm w-8">fx</span>
        <TextInput
          ref={formulaInputRef}
          value={displayEditValue}
          onChange={(e: Event) => handleFormulaChange((e.target as HTMLInputElement).value)}
          onBlur={handleFormulaBlur}
          onKeyDown={handleFormulaKeyDown}
          className="flex-1"
          placeholder="Enter value or formula (e.g., =SUM(A1:A10))"
          styles={{ input: { fontFamily: 'monospace' } }}
        />
      </div>

      {/* Spreadsheet Grid */}
      <div 
        ref={containerRef}
        className="flex-1 overflow-auto relative"
        onScroll={handleScroll}
      >
        {/* Virtual content */}
        <div style={{ width: totalWidth + ROW_HEADER_WIDTH, height: totalHeight + CELL_HEIGHT, position: 'relative' }}>
          {/* Column headers */}
          <div 
            className="sticky top-0 z-20 flex" 
            style={{ height: CELL_HEIGHT, marginLeft: ROW_HEADER_WIDTH }}
          >
            {Array.from({ length: visibleColEnd - visibleColStart }, (_, i) => {
              const col = visibleColStart + i;
              const width = columnWidths[col] || DEFAULT_COLUMN_WIDTH;
              const left = columnPositions[col];
              
              return (
                <div
                  key={col}
                  className="bg-gray-800 border border-gray-700 text-gray-400 font-normal text-sm flex items-center justify-center relative cursor-pointer hover:bg-gray-700"
                  style={{
                    position: 'absolute',
                    left: left,
                    width: width,
                    height: CELL_HEIGHT,
                  }}
                  onClick={() => handleColumnHeaderClick(col)}
                >
                  {getColumnLabel(col)}
                  {/* Resize handle */}
                  <div
                    className="absolute right-0 top-0 bottom-0 w-1 cursor-col-resize hover:bg-blue-500"
                    onMouseDown={(e) => handleColumnResizeStart(col, e)}
                  />
                </div>
              );
            })}
          </div>

          {/* Row headers */}
          <div 
            className="sticky left-0 z-10" 
            style={{ 
              position: 'absolute',
              top: CELL_HEIGHT,
              width: ROW_HEADER_WIDTH,
            }}
          >
            {Array.from({ length: visibleRowEnd - visibleRowStart }, (_, i) => {
              const row = visibleRowStart + i;
              return (
                <div
                  key={row}
                  className="bg-gray-800 border border-gray-700 text-gray-400 text-sm flex items-center justify-center cursor-pointer hover:bg-gray-700"
                  style={{
                    position: 'absolute',
                    top: row * CELL_HEIGHT,
                    width: ROW_HEADER_WIDTH,
                    height: CELL_HEIGHT,
                  }}
                  onClick={() => handleRowHeaderClick(row)}
                >
                  {row + 1}
                </div>
              );
            })}
          </div>

          {/* Corner header */}
          <div
            className="sticky top-0 left-0 z-30 bg-gray-800 border border-gray-700"
            style={{ width: ROW_HEADER_WIDTH, height: CELL_HEIGHT, position: 'absolute' }}
          />

          {/* Cells */}
          <div 
            style={{ 
              position: 'absolute',
              top: CELL_HEIGHT,
              left: ROW_HEADER_WIDTH,
            }}
          >
            {Array.from({ length: visibleRowEnd - visibleRowStart }, (_, ri) => {
              const row = visibleRowStart + ri;
              
              return Array.from({ length: visibleColEnd - visibleColStart }, (_, ci) => {
                const col = visibleColStart + ci;
                const key = getCellKey(row, col);
                const cellData = spreadsheet.cells[key];
                const rawValue = cellData?.value || '';
                const displayValue = getComputedValue(row, col, spreadsheet.cells);
                const isSelected = isCellInSelection(row, col, selection);
                const isActiveCell = selection?.end.row === row && selection?.end.col === col;
                const isEditing = editingCell?.row === row && editingCell?.col === col;
                const isFormula = rawValue.startsWith('=');
                const width = columnWidths[col] || DEFAULT_COLUMN_WIDTH;
                const left = columnPositions[col];
                const selectionBorders = getSelectionBorders(row, col, selection);
                
                // Build border styles for selection (only outer edges)
                const borderStyle: Record<string, string> = {};
                if (isActiveCell) {
                  borderStyle.border = '2px solid rgb(59, 130, 246)'; // blue-500
                } else if (selectionBorders) {
                  const borderColor = 'rgb(96, 165, 250)'; // blue-400
                  borderStyle.borderTop = selectionBorders.top ? `2px solid ${borderColor}` : '1px solid rgb(55, 65, 81)';
                  borderStyle.borderRight = selectionBorders.right ? `2px solid ${borderColor}` : '1px solid rgb(55, 65, 81)';
                  borderStyle.borderBottom = selectionBorders.bottom ? `2px solid ${borderColor}` : '1px solid rgb(55, 65, 81)';
                  borderStyle.borderLeft = selectionBorders.left ? `2px solid ${borderColor}` : '1px solid rgb(55, 65, 81)';
                }
                
                return (
                  <div
                    key={`${row}:${col}`}
                    className={`
                      absolute text-white text-sm px-1 flex items-center
                      ${isActiveCell 
                        ? 'bg-blue-900/50' 
                        : isSelected 
                          ? 'bg-blue-900/30' 
                          : 'border border-gray-700 bg-gray-900 hover:bg-gray-800'
                      }
                      ${isFormula && !isEditing ? 'text-green-400' : ''}
                    `}
                    style={{
                      top: row * CELL_HEIGHT,
                      left: left,
                      width: width,
                      height: CELL_HEIGHT,
                      cursor: resizingColumn !== null ? 'col-resize' : 'cell',
                      ...borderStyle,
                    }}
                    onMouseDown={(e) => handleCellMouseDown(row, col, e)}
                    onMouseEnter={() => handleCellMouseEnter(row, col)}
                    onDblClick={() => handleCellDoubleClick(row, col)}
                  >
                    {isEditing ? (
                      <input
                        ref={cellInputRef}
                        type="text"
                        value={editValue}
                        onChange={(e: Event) => setEditValue((e.target as HTMLInputElement).value)}
                        onKeyDown={handleEditKeyDown}
                        onBlur={() => {
                          updateCell(editingCell.row, editingCell.col, editValue);
                          setEditingCell(null);
                        }}
                        className="w-full h-full bg-transparent outline-none text-white font-mono"
                        autoFocus
                        onClick={(e) => e.stopPropagation()}
                      />
                    ) : (
                      <span className="truncate">{displayValue}</span>
                    )}
                  </div>
                );
              });
            })}
          </div>
        </div>
      </div>

      {/* Status Bar */}
      <div className="bg-gray-800 border-t border-gray-700 px-4 py-1 flex items-center justify-between text-xs text-gray-400">
        <span>
          {selection 
            ? `Selected: ${getColumnLabel(selection.start.col)}${selection.start.row + 1}` +
              (selection.start.row !== selection.end.row || selection.start.col !== selection.end.col
                ? `:${getColumnLabel(selection.end.col)}${selection.end.row + 1}`
                : '')
            : 'Ready'
          }
        </span>
        <span>
          Formulas: =SUM, =AVERAGE, =MIN, =MAX, =COUNT, =IF, =ROUND, etc.
        </span>
      </div>
    </div>
  );
}
