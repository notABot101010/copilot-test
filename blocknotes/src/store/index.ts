import { signal, computed } from '@preact/signals';
import type { Page, Block, Database, DatabaseColumn, DatabaseRow, BlockType, DatabaseView, DatabaseCellValue } from '../types';
import { generateId } from '../utils';

// Pages store
export const pages = signal<Page[]>([]);
export const currentPageId = signal<string | null>(null);

export const currentPage = computed(() => {
  if (!currentPageId.value) return null;
  return pages.value.find((p) => p.id === currentPageId.value) || null;
});

// Database store
export const databases = signal<Database[]>([]);

// Command palette state
export const isCommandPaletteOpen = signal(false);
export const commandPalettePosition = signal({ x: 0, y: 0 });
export const commandPaletteBlockIndex = signal<number | null>(null);

// Sidebar state
export const isSidebarOpen = signal(true);

// Block focus state
export const focusedBlockId = signal<string | null>(null);

export function setFocusedBlock(blockId: string | null): void {
  focusedBlockId.value = blockId;
}

export function getPreviousBlock(pageId: string, blockId: string): Block | null {
  const page = pages.value.find((p) => p.id === pageId);
  if (!page) return null;
  const index = page.blocks.findIndex((b) => b.id === blockId);
  if (index <= 0) return null;
  return page.blocks[index - 1];
}

export function getNextBlock(pageId: string, blockId: string): Block | null {
  const page = pages.value.find((p) => p.id === pageId);
  if (!page) return null;
  const index = page.blocks.findIndex((b) => b.id === blockId);
  if (index < 0 || index >= page.blocks.length - 1) return null;
  return page.blocks[index + 1];
}

// Page actions
export function createPage(title: string = 'Untitled', parentId?: string): Page {
  const now = Date.now();
  const newPage: Page = {
    id: generateId(),
    title,
    blocks: [],
    parentId,
    children: [],
    createdAt: now,
    updatedAt: now,
  };
  pages.value = [...pages.value, newPage];
  
  if (parentId) {
    const parent = pages.value.find(p => p.id === parentId);
    if (parent) {
      parent.children = [...(parent.children || []), newPage.id];
      pages.value = [...pages.value];
    }
  }
  
  return newPage;
}

export function updatePage(pageId: string, updates: Partial<Page>): void {
  pages.value = pages.value.map((page) =>
    page.id === pageId ? { ...page, ...updates, updatedAt: Date.now() } : page
  );
}

export function deletePage(pageId: string): void {
  pages.value = pages.value.filter((page) => page.id !== pageId);
  if (currentPageId.value === pageId) {
    currentPageId.value = pages.value[0]?.id || null;
  }
}

export function setCurrentPage(pageId: string): void {
  currentPageId.value = pageId;
}

// Block actions
export function createBlock(pageId: string, type: BlockType, index?: number): Block {
  const now = Date.now();
  const newBlock: Block = {
    id: generateId(),
    type,
    content: '',
    createdAt: now,
    updatedAt: now,
  };
  
  if (type === 'table') {
    newBlock.properties = {
      tableData: {
        columns: ['Column 1', 'Column 2', 'Column 3'],
        rows: [
          { id: generateId(), cells: { 'Column 1': '', 'Column 2': '', 'Column 3': '' } },
          { id: generateId(), cells: { 'Column 1': '', 'Column 2': '', 'Column 3': '' } },
        ],
      },
    };
  }

  pages.value = pages.value.map((page) => {
    if (page.id !== pageId) return page;
    const blocks = [...page.blocks];
    if (index !== undefined && index >= 0) {
      blocks.splice(index + 1, 0, newBlock);
    } else {
      blocks.push(newBlock);
    }
    return { ...page, blocks, updatedAt: Date.now() };
  });

  return newBlock;
}

export function updateBlock(pageId: string, blockId: string, updates: Partial<Block>): void {
  pages.value = pages.value.map((page) => {
    if (page.id !== pageId) return page;
    return {
      ...page,
      blocks: page.blocks.map((block) =>
        block.id === blockId ? { ...block, ...updates, updatedAt: Date.now() } : block
      ),
      updatedAt: Date.now(),
    };
  });
}

export function deleteBlock(pageId: string, blockId: string): void {
  pages.value = pages.value.map((page) => {
    if (page.id !== pageId) return page;
    return {
      ...page,
      blocks: page.blocks.filter((block) => block.id !== blockId),
      updatedAt: Date.now(),
    };
  });
}

export function reorderBlocks(pageId: string, oldIndex: number, newIndex: number): void {
  pages.value = pages.value.map((page) => {
    if (page.id !== pageId) return page;
    const blocks = [...page.blocks];
    const [removed] = blocks.splice(oldIndex, 1);
    blocks.splice(newIndex, 0, removed);
    return { ...page, blocks, updatedAt: Date.now() };
  });
}

// Database actions
export function createDatabase(name: string = 'Untitled Database'): Database {
  const now = Date.now();
  const statusColumn: DatabaseColumn = {
    id: generateId(),
    name: 'Status',
    type: 'select',
    options: [
      { id: generateId(), name: 'Not Started', color: 'gray' },
      { id: generateId(), name: 'In Progress', color: 'blue' },
      { id: generateId(), name: 'Done', color: 'green' },
    ],
  };
  
  const newDatabase: Database = {
    id: generateId(),
    name,
    columns: [
      { id: generateId(), name: 'Name', type: 'text' },
      statusColumn,
    ],
    rows: [],
    views: [
      { id: generateId(), name: 'Table View', type: 'table' },
      { id: generateId(), name: 'Kanban', type: 'kanban', groupBy: statusColumn.id },
    ],
    createdAt: now,
    updatedAt: now,
  };
  databases.value = [...databases.value, newDatabase];
  return newDatabase;
}

export function updateDatabase(databaseId: string, updates: Partial<Database>): void {
  databases.value = databases.value.map((db) =>
    db.id === databaseId ? { ...db, ...updates, updatedAt: Date.now() } : db
  );
}

export function addDatabaseColumn(databaseId: string, column: DatabaseColumn): void {
  databases.value = databases.value.map((db) => {
    if (db.id !== databaseId) return db;
    return {
      ...db,
      columns: [...db.columns, column],
      updatedAt: Date.now(),
    };
  });
}

export function removeDatabaseColumn(databaseId: string, columnId: string): void {
  databases.value = databases.value.map((db) => {
    if (db.id !== databaseId) return db;
    return {
      ...db,
      columns: db.columns.filter((col) => col.id !== columnId),
      rows: db.rows.map((row) => {
        const newProperties = { ...row.properties };
        delete newProperties[columnId];
        return { ...row, properties: newProperties };
      }),
      updatedAt: Date.now(),
    };
  });
}

export function addDatabaseRow(databaseId: string, row?: Partial<DatabaseRow>): DatabaseRow {
  const now = Date.now();
  const database = databases.value.find((db) => db.id === databaseId);
  const defaultProperties: Record<string, DatabaseCellValue> = {};
  
  database?.columns.forEach((col) => {
    if (col.type === 'checkbox') {
      defaultProperties[col.id] = false;
    } else if (col.type === 'select' && col.options?.length) {
      defaultProperties[col.id] = col.options[0].name;
    } else {
      defaultProperties[col.id] = '';
    }
  });

  const newRow: DatabaseRow = {
    id: generateId(),
    properties: { ...defaultProperties, ...row?.properties },
    createdAt: now,
    updatedAt: now,
  };

  databases.value = databases.value.map((db) => {
    if (db.id !== databaseId) return db;
    return {
      ...db,
      rows: [...db.rows, newRow],
      updatedAt: Date.now(),
    };
  });

  return newRow;
}

export function updateDatabaseRow(
  databaseId: string,
  rowId: string,
  updates: Record<string, DatabaseCellValue>
): void {
  databases.value = databases.value.map((db) => {
    if (db.id !== databaseId) return db;
    return {
      ...db,
      rows: db.rows.map((row) =>
        row.id === rowId
          ? { ...row, properties: { ...row.properties, ...updates }, updatedAt: Date.now() }
          : row
      ),
      updatedAt: Date.now(),
    };
  });
}

export function removeDatabaseRow(databaseId: string, rowId: string): void {
  databases.value = databases.value.map((db) => {
    if (db.id !== databaseId) return db;
    return {
      ...db,
      rows: db.rows.filter((row) => row.id !== rowId),
      updatedAt: Date.now(),
    };
  });
}

export function addDatabaseView(databaseId: string, view: DatabaseView): void {
  databases.value = databases.value.map((db) => {
    if (db.id !== databaseId) return db;
    return {
      ...db,
      views: [...db.views, view],
      updatedAt: Date.now(),
    };
  });
}

// Command palette actions
export function openCommandPalette(x: number, y: number, blockIndex: number | null): void {
  commandPalettePosition.value = { x, y };
  commandPaletteBlockIndex.value = blockIndex;
  isCommandPaletteOpen.value = true;
}

export function closeCommandPalette(): void {
  isCommandPaletteOpen.value = false;
  commandPaletteBlockIndex.value = null;
}

// Sidebar actions
export function toggleSidebar(): void {
  isSidebarOpen.value = !isSidebarOpen.value;
}

// Initialize with a default page
export function initializeStore(): void {
  if (pages.value.length === 0) {
    const page = createPage('Getting Started');
    // Set initial BlockNote content
    updatePage(page.id, {
      blocknoteContent: [
        {
          type: 'heading',
          props: { level: 1 },
          content: [{ type: 'text', text: 'Welcome to Notes', styles: {} }],
        },
        {
          type: 'paragraph',
          content: [{ type: 'text', text: 'Start typing or press "/" to see available commands.', styles: {} }],
        },
      ],
    });
    setCurrentPage(page.id);
  }
}
