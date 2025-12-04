// Block types
export type BlockType =
  | 'text'
  | 'heading1'
  | 'heading2'
  | 'heading3'
  | 'bulletList'
  | 'numberedList'
  | 'todoList'
  | 'image'
  | 'table'
  | 'pageLink'
  | 'divider'
  | 'quote'
  | 'database';

export interface Block {
  id: string;
  type: BlockType;
  content: string;
  properties?: BlockProperties;
  createdAt: number;
  updatedAt: number;
}

export interface BlockProperties {
  // For headings
  level?: 1 | 2 | 3;
  // For images
  imageUrl?: string;
  imageAlt?: string;
  // For todo lists
  checked?: boolean;
  // For tables
  tableData?: TableData;
  // For page links
  linkedPageId?: string;
  // For databases
  databaseId?: string;
}

export interface TableData {
  columns: string[];
  rows: TableRow[];
}

export interface TableRow {
  id: string;
  cells: Record<string, string>;
}

// Page types
export interface Page {
  id: string;
  title: string;
  icon?: string;
  blocks: Block[];
  parentId?: string;
  children?: string[];
  createdAt: number;
  updatedAt: number;
}

// Database types
export type DatabaseViewType = 'table' | 'kanban';

export interface DatabaseColumn {
  id: string;
  name: string;
  type: 'text' | 'number' | 'select' | 'multiSelect' | 'date' | 'checkbox';
  options?: SelectOption[];
}

export interface SelectOption {
  id: string;
  name: string;
  color: string;
}

export interface DatabaseRow {
  id: string;
  properties: Record<string, DatabaseCellValue>;
  createdAt: number;
  updatedAt: number;
}

export type DatabaseCellValue = string | number | boolean | string[] | Date | null;

export interface Database {
  id: string;
  name: string;
  columns: DatabaseColumn[];
  rows: DatabaseRow[];
  views: DatabaseView[];
  createdAt: number;
  updatedAt: number;
}

export interface DatabaseView {
  id: string;
  name: string;
  type: DatabaseViewType;
  groupBy?: string; // Column id for kanban grouping
}

// Command palette types
export interface Command {
  id: string;
  name: string;
  description: string;
  icon: string;
  blockType: BlockType;
}
