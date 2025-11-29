import { signal, computed } from '@preact/signals';
import * as Automerge from '@automerge/automerge';
import type { SpreadsheetData, SpreadsheetListItem } from '../types/spreadsheet';
import { getCellKey } from '../types/spreadsheet';
import { initSync, broadcastChanges } from './syncManager';

// Automerge document type
interface AutomergeSpreadsheet {
  id: string;
  name: string;
  cells: Record<string, { value: string }>;
  createdAt: number;
  updatedAt: number;
}

// Store for spreadsheet list
export const spreadsheetList = signal<SpreadsheetListItem[]>([]);

// Current active spreadsheet document
export const currentSpreadsheetDoc = signal<Automerge.Doc<AutomergeSpreadsheet> | null>(null);

// Undo/Redo history configuration
// Maximum number of undo steps to keep in memory. Each step stores a full document snapshot.
// Higher values use more memory but allow more undo history.
const MAX_HISTORY = 100;
const undoStack: Automerge.Doc<AutomergeSpreadsheet>[] = [];
const redoStack: Automerge.Doc<AutomergeSpreadsheet>[] = [];

// Initialize sync manager for cross-tab communication
initSync((spreadsheetId: string, changes: Uint8Array) => {
  // Handle incoming sync from other tabs
  handleRemoteSync(spreadsheetId, changes);
});

// Handle remote sync updates from other tabs
function handleRemoteSync(spreadsheetId: string, remoteBinary: Uint8Array): void {
  const doc = currentSpreadsheetDoc.value;
  if (!doc || doc.id !== spreadsheetId) return;
  
  try {
    const remoteDoc = Automerge.load<AutomergeSpreadsheet>(remoteBinary);
    const mergedDoc = Automerge.merge(doc, remoteDoc);
    
    // Check if there are any changes by comparing the heads
    const localHeads = Automerge.getHeads(doc);
    const mergedHeads = Automerge.getHeads(mergedDoc);
    
    // If heads are different, we have new changes
    const hasChanges = localHeads.length !== mergedHeads.length || 
      localHeads.some((h, i) => h !== mergedHeads[i]);
    
    if (hasChanges) {
      currentSpreadsheetDoc.value = mergedDoc;
      saveCurrentSpreadsheetInternal();
      updateSpreadsheetListItemInternal();
    }
  } catch {
    // Ignore invalid remote data
  }
}

function pushToUndoStack(doc: Automerge.Doc<AutomergeSpreadsheet>): void {
  undoStack.push(doc);
  if (undoStack.length > MAX_HISTORY) {
    undoStack.shift();
  }
  // Clear redo stack on new action
  redoStack.length = 0;
}

export function canUndo(): boolean {
  return undoStack.length > 0;
}

export function canRedo(): boolean {
  return redoStack.length > 0;
}

export function undo(): void {
  if (undoStack.length === 0) return;
  const doc = currentSpreadsheetDoc.value;
  if (!doc) return;
  
  const previousDoc = undoStack.pop()!;
  redoStack.push(doc);
  currentSpreadsheetDoc.value = previousDoc;
  saveAndBroadcast(doc, previousDoc);
}

export function redo(): void {
  if (redoStack.length === 0) return;
  const doc = currentSpreadsheetDoc.value;
  if (!doc) return;
  
  const nextDoc = redoStack.pop()!;
  undoStack.push(doc);
  currentSpreadsheetDoc.value = nextDoc;
  saveAndBroadcast(doc, nextDoc);
}

// Derived computed value for current spreadsheet data
export const currentSpreadsheet = computed<SpreadsheetData | null>(() => {
  const doc = currentSpreadsheetDoc.value;
  if (!doc) return null;
  return {
    id: doc.id,
    name: doc.name,
    cells: doc.cells,
    createdAt: doc.createdAt,
    updatedAt: doc.updatedAt,
  };
});

// Local storage key prefix
const STORAGE_KEY_PREFIX = 'spreadsheet_';
const SPREADSHEET_LIST_KEY = 'spreadsheet_list';

// Generate a unique ID
function generateId(): string {
  return Math.random().toString(36).substring(2, 15) + Date.now().toString(36);
}

// Load spreadsheet list from localStorage
export function loadSpreadsheetList(): void {
  const storedList = localStorage.getItem(SPREADSHEET_LIST_KEY);
  if (storedList) {
    try {
      spreadsheetList.value = JSON.parse(storedList);
    } catch {
      spreadsheetList.value = [];
    }
  }
}

// Save spreadsheet list to localStorage
function saveSpreadsheetList(): void {
  localStorage.setItem(SPREADSHEET_LIST_KEY, JSON.stringify(spreadsheetList.value));
}

// Create a new spreadsheet
export function createSpreadsheet(name: string): string {
  const id = generateId();
  const now = Date.now();
  
  // Create a new Automerge document
  let doc = Automerge.init<AutomergeSpreadsheet>();
  doc = Automerge.change(doc, 'Initialize spreadsheet', (d) => {
    d.id = id;
    d.name = name;
    d.cells = {};
    d.createdAt = now;
    d.updatedAt = now;
  });
  
  // Save document to localStorage
  const binary = Automerge.save(doc);
  localStorage.setItem(STORAGE_KEY_PREFIX + id, uint8ArrayToBase64(binary));
  
  // Add to list
  spreadsheetList.value = [
    ...spreadsheetList.value,
    { id, name, createdAt: now, updatedAt: now },
  ];
  saveSpreadsheetList();
  
  return id;
}

// Load a spreadsheet by ID
export function loadSpreadsheet(id: string): boolean {
  const stored = localStorage.getItem(STORAGE_KEY_PREFIX + id);
  if (!stored) {
    currentSpreadsheetDoc.value = null;
    return false;
  }
  
  try {
    const binary = base64ToUint8Array(stored);
    const doc = Automerge.load<AutomergeSpreadsheet>(binary);
    currentSpreadsheetDoc.value = doc;
    return true;
  } catch {
    currentSpreadsheetDoc.value = null;
    return false;
  }
}

// Update a cell value
export function updateCell(row: number, col: number, value: string): void {
  const doc = currentSpreadsheetDoc.value;
  if (!doc) return;
  
  // Check if value actually changed
  const key = getCellKey(row, col);
  const currentValue = doc.cells[key]?.value || '';
  if (currentValue === value) return;
  
  // Save to undo stack before making changes
  pushToUndoStack(doc);
  
  const newDoc = Automerge.change(doc, `Update cell ${row}:${col}`, (d) => {
    if (!d.cells[key]) {
      d.cells[key] = { value: '' };
    }
    d.cells[key].value = value;
    d.updatedAt = Date.now();
  });
  
  currentSpreadsheetDoc.value = newDoc;
  saveAndBroadcast(doc, newDoc);
}

// Update multiple cells at once
export function updateMultipleCells(updates: { row: number; col: number; value: string }[]): void {
  const doc = currentSpreadsheetDoc.value;
  if (!doc || updates.length === 0) return;
  
  // Check if any values actually changed
  const changedUpdates = updates.filter(({ row, col, value }) => {
    const key = getCellKey(row, col);
    const currentValue = doc.cells[key]?.value || '';
    return currentValue !== value;
  });
  
  if (changedUpdates.length === 0) return;
  
  // Save to undo stack before making changes
  pushToUndoStack(doc);
  
  const newDoc = Automerge.change(doc, `Update ${changedUpdates.length} cells`, (d) => {
    for (const { row, col, value } of changedUpdates) {
      const key = getCellKey(row, col);
      if (!d.cells[key]) {
        d.cells[key] = { value: '' };
      }
      d.cells[key].value = value;
    }
    d.updatedAt = Date.now();
  });
  
  currentSpreadsheetDoc.value = newDoc;
  saveAndBroadcast(doc, newDoc);
}

// Rename spreadsheet
export function renameSpreadsheet(name: string): void {
  const doc = currentSpreadsheetDoc.value;
  if (!doc) return;
  
  const newDoc = Automerge.change(doc, 'Rename spreadsheet', (d) => {
    d.name = name;
    d.updatedAt = Date.now();
  });
  
  currentSpreadsheetDoc.value = newDoc;
  saveAndBroadcast(doc, newDoc);
}

// Save current spreadsheet to localStorage (internal, no broadcast)
function saveCurrentSpreadsheetInternal(): void {
  const doc = currentSpreadsheetDoc.value;
  if (!doc) return;
  
  const binary = Automerge.save(doc);
  localStorage.setItem(STORAGE_KEY_PREFIX + doc.id, uint8ArrayToBase64(binary));
}

// Update the list item with current spreadsheet info (internal, no broadcast)
function updateSpreadsheetListItemInternal(): void {
  const doc = currentSpreadsheetDoc.value;
  if (!doc) return;
  
  spreadsheetList.value = spreadsheetList.value.map((item) =>
    item.id === doc.id
      ? { ...item, name: doc.name, updatedAt: doc.updatedAt }
      : item
  );
  saveSpreadsheetList();
}

// Save and broadcast changes to other tabs
function saveAndBroadcast(oldDoc: Automerge.Doc<AutomergeSpreadsheet>, newDoc: Automerge.Doc<AutomergeSpreadsheet>): void {
  saveCurrentSpreadsheetInternal();
  updateSpreadsheetListItemInternal();
  broadcastChanges(newDoc.id, oldDoc, newDoc);
}

// Delete a spreadsheet
export function deleteSpreadsheet(id: string): void {
  localStorage.removeItem(STORAGE_KEY_PREFIX + id);
  spreadsheetList.value = spreadsheetList.value.filter((s) => s.id !== id);
  saveSpreadsheetList();
  
  if (currentSpreadsheetDoc.value?.id === id) {
    currentSpreadsheetDoc.value = null;
  }
}

// Get the Automerge binary for sync
export function getSpreadsheetBinary(id: string): Uint8Array | null {
  const stored = localStorage.getItem(STORAGE_KEY_PREFIX + id);
  if (!stored) return null;
  return base64ToUint8Array(stored);
}

// Merge remote changes into the spreadsheet
export function mergeRemoteChanges(id: string, remoteBinary: Uint8Array): void {
  const doc = currentSpreadsheetDoc.value;
  if (!doc || doc.id !== id) return;
  
  const remoteDoc = Automerge.load<AutomergeSpreadsheet>(remoteBinary);
  const mergedDoc = Automerge.merge(doc, remoteDoc);
  
  currentSpreadsheetDoc.value = mergedDoc;
  saveCurrentSpreadsheetInternal();
  updateSpreadsheetListItemInternal();
}

// Helper functions for base64 encoding/decoding
function uint8ArrayToBase64(bytes: Uint8Array): string {
  let binary = '';
  for (let i = 0; i < bytes.length; i++) {
    binary += String.fromCharCode(bytes[i]);
  }
  return btoa(binary);
}

function base64ToUint8Array(base64: string): Uint8Array {
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes;
}
