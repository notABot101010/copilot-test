import { getCellKey, parseCellKey } from '../types/spreadsheet';
import type { CellData } from '../types/spreadsheet';

/**
 * Parse CSV content into a 2D array of strings.
 * Handles quoted fields, escaped quotes, and multiline values.
 */
export function parseCSV(csvContent: string): string[][] {
  const rows: string[][] = [];
  let currentRow: string[] = [];
  let currentField = '';
  let insideQuotes = false;
  let i = 0;

  while (i < csvContent.length) {
    const char = csvContent[i];
    const nextChar = csvContent[i + 1];

    if (insideQuotes) {
      if (char === '"') {
        if (nextChar === '"') {
          // Escaped quote
          currentField += '"';
          i += 2;
        } else {
          // End of quoted field
          insideQuotes = false;
          i++;
        }
      } else {
        currentField += char;
        i++;
      }
    } else {
      if (char === '"' && currentField === '') {
        // Start of quoted field
        insideQuotes = true;
        i++;
      } else if (char === ',') {
        // Field separator
        currentRow.push(currentField);
        currentField = '';
        i++;
      } else if (char === '\r' && nextChar === '\n') {
        // Windows line ending
        currentRow.push(currentField);
        rows.push(currentRow);
        currentRow = [];
        currentField = '';
        i += 2;
      } else if (char === '\n' || char === '\r') {
        // Unix or old Mac line ending
        currentRow.push(currentField);
        rows.push(currentRow);
        currentRow = [];
        currentField = '';
        i++;
      } else {
        currentField += char;
        i++;
      }
    }
  }

  // Handle the last field and row
  if (currentField !== '' || currentRow.length > 0) {
    currentRow.push(currentField);
    rows.push(currentRow);
  }

  return rows;
}

/**
 * Convert cells data to CSV format.
 * Finds the bounds of the data and exports all cells within those bounds.
 */
export function generateCSV(cells: Record<string, CellData>): string {
  const cellKeys = Object.keys(cells).filter(key => cells[key]?.value);
  
  if (cellKeys.length === 0) {
    return '';
  }

  // Find bounds
  let maxRow = 0;
  let maxCol = 0;

  for (const key of cellKeys) {
    const { row, col } = parseCellKey(key);
    maxRow = Math.max(maxRow, row);
    maxCol = Math.max(maxCol, col);
  }

  const rows: string[] = [];

  for (let row = 0; row <= maxRow; row++) {
    const rowValues: string[] = [];
    for (let col = 0; col <= maxCol; col++) {
      const key = getCellKey(row, col);
      const value = cells[key]?.value || '';
      rowValues.push(escapeCSVField(value));
    }
    rows.push(rowValues.join(','));
  }

  return rows.join('\n');
}

/**
 * Escape a field value for CSV format.
 * Wraps in quotes if the value contains commas, quotes, or newlines.
 */
function escapeCSVField(value: string): string {
  const needsQuotes = value.includes(',') || value.includes('"') || value.includes('\n') || value.includes('\r');
  
  if (needsQuotes) {
    // Escape quotes by doubling them
    const escaped = value.replace(/"/g, '""');
    return `"${escaped}"`;
  }
  
  return value;
}

/**
 * Import CSV data into cell updates.
 * Returns an array of cell updates to apply.
 */
export function importCSVToCells(csvContent: string): { row: number; col: number; value: string }[] {
  const parsedRows = parseCSV(csvContent);
  const updates: { row: number; col: number; value: string }[] = [];

  for (let row = 0; row < parsedRows.length; row++) {
    const rowData = parsedRows[row];
    for (let col = 0; col < rowData.length; col++) {
      const value = rowData[col];
      // Include all values, even empty ones, to properly clear existing cells
      updates.push({ row, col, value });
    }
  }

  return updates;
}

/**
 * Download a CSV file with the given content.
 */
export function downloadCSV(content: string, filename: string): void {
  const blob = new Blob([content], { type: 'text/csv;charset=utf-8;' });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = filename;
  link.style.display = 'none';
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
  URL.revokeObjectURL(url);
}

/**
 * Open a file picker dialog to select a CSV file.
 * Returns the file content as a string, or null if cancelled.
 */
export function openCSVFilePicker(): Promise<string | null> {
  return new Promise((resolve) => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.csv,text/csv';
    input.style.display = 'none';
    
    input.onchange = async (event) => {
      const target = event.target as HTMLInputElement;
      const file = target.files?.[0];
      
      if (file) {
        const content = await file.text();
        resolve(content);
      } else {
        resolve(null);
      }
      
      document.body.removeChild(input);
    };
    
    input.oncancel = () => {
      resolve(null);
      document.body.removeChild(input);
    };
    
    document.body.appendChild(input);
    input.click();
  });
}
