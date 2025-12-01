import type { ChartData } from './chart';

export interface CellData {
  value: string;
}

export interface SpreadsheetData {
  id: string;
  name: string;
  cells: Record<string, CellData>; // Key is "row:col" format, e.g., "0:0", "1:2"
  charts: Record<string, ChartData>; // Key is chart id
  createdAt: number;
  updatedAt: number;
}

export interface SpreadsheetListItem {
  id: string;
  name: string;
  createdAt: number;
  updatedAt: number;
}

export function getCellKey(row: number, col: number): string {
  return `${row}:${col}`;
}

export function parseCellKey(key: string): { row: number; col: number } {
  const [row, col] = key.split(':').map(Number);
  return { row, col };
}
