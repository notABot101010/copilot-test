import { signal, computed } from '@preact/signals';
import * as Automerge from '@automerge/automerge';
import type { SpreadsheetData, SpreadsheetListItem } from '../types/spreadsheet';
import { getCellKey } from '../types/spreadsheet';
import type { ChartData, ChartPosition, ChartSize, ChartDataRange, ChartType } from '../types/chart';
import { generateChartId, DEFAULT_CHART_WIDTH, DEFAULT_CHART_HEIGHT } from '../types/chart';
import { initSync, broadcastChanges, startServerSync, stopServerSync, getServerUrl } from './syncManager';

// Automerge document type
interface AutomergeSpreadsheet {
  id: string;
  name: string;
  cells: Record<string, { value: string }>;
  charts: Record<string, ChartData>;
  createdAt: number;
  updatedAt: number;
}

// Store for spreadsheet list
export const spreadsheetList = signal<SpreadsheetListItem[]>([]);

// Loading state for the spreadsheet list
export const isLoadingList = signal<boolean>(false);

// Current active spreadsheet document
export const currentSpreadsheetDoc = signal<Automerge.Doc<AutomergeSpreadsheet> | null>(null);

// Undo/Redo history configuration
// Maximum number of undo steps to keep in memory. Each step stores a full document snapshot.
// Higher values use more memory but allow more undo history.
const MAX_HISTORY = 100;
const undoStack: Automerge.Doc<AutomergeSpreadsheet>[] = [];
const redoStack: Automerge.Doc<AutomergeSpreadsheet>[] = [];

// Signals to track undo/redo availability for UI reactivity
export const canUndoSignal = signal<boolean>(false);
export const canRedoSignal = signal<boolean>(false);

// Track the document state before the current edit session started
// This is used to push to undo stack when the first live update happens
let preEditDoc: Automerge.Doc<AutomergeSpreadsheet> | null = null;

// Initialize sync manager for cross-tab and cross-browser communication
initSync(
  // Handle incoming sync from other tabs/browsers
  (spreadsheetId: string, changes: Uint8Array) => {
    handleRemoteSync(spreadsheetId, changes);
  }
);

// Handle remote sync updates from other tabs
function handleRemoteSync(spreadsheetId: string, remoteBinary: Uint8Array): void {
  const doc = currentSpreadsheetDoc.value;
  // Note: String() cast is required because Automerge returns CRDT wrapper objects
  if (!doc || String(doc.id) !== spreadsheetId) return;
  
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
      updateSpreadsheetListItemInternal();
      // Clear pre-edit state as remote changes invalidate the edit context
      preEditDoc = null;
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
  // Update signals for UI reactivity
  canUndoSignal.value = undoStack.length > 0;
  canRedoSignal.value = false;
}

/**
 * Get the latest document from the signal and clone it to avoid outdated document errors.
 * Returns null if no document is available.
 */
function getLatestClonedDoc(): Automerge.Doc<AutomergeSpreadsheet> | null {
  const doc = currentSpreadsheetDoc.value;
  if (!doc) return null;
  return Automerge.clone(doc);
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
  // Update signals for UI reactivity
  canUndoSignal.value = undoStack.length > 0;
  canRedoSignal.value = redoStack.length > 0;
  saveAndBroadcast(doc, previousDoc);
}

export function redo(): void {
  if (redoStack.length === 0) return;
  const doc = currentSpreadsheetDoc.value;
  if (!doc) return;
  
  const nextDoc = redoStack.pop()!;
  undoStack.push(doc);
  currentSpreadsheetDoc.value = nextDoc;
  // Update signals for UI reactivity
  canUndoSignal.value = undoStack.length > 0;
  canRedoSignal.value = redoStack.length > 0;
  saveAndBroadcast(doc, nextDoc);
}

// Derived computed value for current spreadsheet data
// Note: String() and Number() casts are required because Automerge returns
// CRDT wrapper objects, not primitive JavaScript values
export const currentSpreadsheet = computed<SpreadsheetData | null>(() => {
  const doc = currentSpreadsheetDoc.value;
  if (!doc) return null;
  return {
    id: String(doc.id),
    name: String(doc.name),
    cells: doc.cells,
    charts: doc.charts || {},
    createdAt: Number(doc.createdAt),
    updatedAt: Number(doc.updatedAt),
  };
});

// Helper function for base64 decoding
function base64ToUint8Array(base64: string): Uint8Array {
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes;
}

// Load spreadsheet list from server
export async function loadSpreadsheetList(): Promise<void> {
  isLoadingList.value = true;
  try {
    const response = await fetch(`${getServerUrl()}/api/spreadsheets`);
    if (response.ok) {
      const data = await response.json();
      spreadsheetList.value = data.spreadsheets.map((s: { id: string; name: string; createdAt: number; updatedAt: number }) => ({
        id: s.id,
        name: s.name,
        createdAt: s.createdAt,
        updatedAt: s.updatedAt,
      }));
    }
  } catch {
    // Server may be offline - keep existing list
    console.debug('Failed to load spreadsheet list from server');
  } finally {
    isLoadingList.value = false;
  }
}

// Create a new spreadsheet via server API
export async function createSpreadsheet(name: string): Promise<string | null> {
  try {
    const response = await fetch(`${getServerUrl()}/api/spreadsheets`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ name }),
    });
    
    if (response.ok) {
      const data = await response.json();
      // Add to local list
      spreadsheetList.value = [
        { id: data.id, name: data.name, createdAt: data.createdAt, updatedAt: data.updatedAt },
        ...spreadsheetList.value,
      ];
      return data.id;
    }
  } catch {
    console.debug('Failed to create spreadsheet on server');
  }
  return null;
}

// Load a spreadsheet by ID from server
export async function loadSpreadsheet(id: string): Promise<boolean> {
  // Stop any existing server sync
  stopServerSync();
  
  // Reset undo/redo state for the new document
  undoStack.length = 0;
  redoStack.length = 0;
  preEditDoc = null;
  canUndoSignal.value = false;
  canRedoSignal.value = false;
  
  try {
    const response = await fetch(`${getServerUrl()}/api/spreadsheets/${id}`);
    if (!response.ok) {
      currentSpreadsheetDoc.value = null;
      return false;
    }
    
    const data = await response.json();
    const binary = base64ToUint8Array(data.document);
    const doc = Automerge.load<AutomergeSpreadsheet>(binary);
    currentSpreadsheetDoc.value = doc;
    
    // Start server sync for this spreadsheet
    startServerSync(id);
    
    return true;
  } catch {
    currentSpreadsheetDoc.value = null;
    return false;
  }
}

// Delete a spreadsheet via server API
export async function deleteSpreadsheet(id: string): Promise<boolean> {
  try {
    const response = await fetch(`${getServerUrl()}/api/spreadsheets/${id}`, {
      method: 'DELETE',
    });
    
    if (response.ok || response.status === 204) {
      spreadsheetList.value = spreadsheetList.value.filter((s) => s.id !== id);
      
      if (currentSpreadsheetDoc.value?.id === id) {
        currentSpreadsheetDoc.value = null;
      }
      return true;
    }
  } catch {
    console.debug('Failed to delete spreadsheet on server');
  }
  return false;
}

// Update a cell value
export function updateCell(row: number, col: number, value: string): void {
  // Always get the latest doc reference to avoid race conditions
  let doc = currentSpreadsheetDoc.value;
  if (!doc) return;
  
  const key = getCellKey(row, col);
  const currentValue = doc.cells[key]?.value || '';
  
  // If there was a live edit session, push the pre-edit state to undo stack
  if (preEditDoc !== null) {
    pushToUndoStack(preEditDoc);
    preEditDoc = null;
    // If the value is the same as what was already set by live updates, just return
    if (currentValue === value) return;
  } else {
    // No live edit session - check if value actually changed
    if (currentValue === value) return;
    // Save to undo stack before making changes
    pushToUndoStack(doc);
  }
  
  // Get latest cloned doc to avoid outdated document errors
  const clonedDoc = getLatestClonedDoc();
  if (!clonedDoc) return;
  
  const newDoc = Automerge.change(clonedDoc, `Update cell ${row}:${col}`, (d) => {
    if (!d.cells[key]) {
      d.cells[key] = { value: '' };
    }
    d.cells[key].value = value;
    d.updatedAt = Date.now();
  });
  
  currentSpreadsheetDoc.value = newDoc;
  saveAndBroadcast(clonedDoc, newDoc);
}

// Update a cell value in real-time (without adding to undo stack on each keystroke)
// This is used for propagating changes as the user types
// On the first call of an edit session, it saves the pre-edit state for undo
export function updateCellLive(row: number, col: number, value: string): void {
  // Always get the latest doc reference to avoid race conditions
  const doc = currentSpreadsheetDoc.value;
  if (!doc) return;
  
  // Check if value actually changed
  const key = getCellKey(row, col);
  const currentValue = doc.cells[key]?.value || '';
  if (currentValue === value) return;
  
  // If this is the first change in an edit session, save the pre-edit state
  if (preEditDoc === null) {
    preEditDoc = doc;
  }
  
  // Get latest cloned doc to avoid outdated document errors
  const clonedDoc = getLatestClonedDoc();
  if (!clonedDoc) return;
  
  const newDoc = Automerge.change(clonedDoc, `Live update cell ${row}:${col}`, (d) => {
    if (!d.cells[key]) {
      d.cells[key] = { value: '' };
    }
    d.cells[key].value = value;
    d.updatedAt = Date.now();
  });
  
  currentSpreadsheetDoc.value = newDoc;
  saveAndBroadcast(clonedDoc, newDoc);
}

// Cancel the current edit session and restore the pre-edit state
// This is called when the user cancels an edit (e.g., presses Escape)
export function cancelEdit(): void {
  if (preEditDoc !== null) {
    const doc = currentSpreadsheetDoc.value;
    currentSpreadsheetDoc.value = preEditDoc;
    if (doc) {
      saveAndBroadcast(doc, preEditDoc);
    }
    preEditDoc = null;
  }
}

// Update multiple cells at once
export function updateMultipleCells(updates: { row: number; col: number; value: string }[]): void {
  const initialDoc = currentSpreadsheetDoc.value;
  if (!initialDoc || updates.length === 0) return;
  
  // Check if any values actually changed
  const changedUpdates = updates.filter(({ row, col, value }) => {
    const key = getCellKey(row, col);
    const currentValue = initialDoc.cells[key]?.value || '';
    return currentValue !== value;
  });
  
  if (changedUpdates.length === 0) return;
  
  // Save to undo stack before making changes
  pushToUndoStack(initialDoc);
  
  // Get latest cloned doc to avoid outdated document errors
  const clonedDoc = getLatestClonedDoc();
  if (!clonedDoc) return;
  
  const newDoc = Automerge.change(clonedDoc, `Update ${changedUpdates.length} cells`, (d) => {
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
  saveAndBroadcast(clonedDoc, newDoc);
}

// Rename spreadsheet
export function renameSpreadsheet(name: string): void {
  // Get latest cloned doc to avoid outdated document errors
  const clonedDoc = getLatestClonedDoc();
  if (!clonedDoc) return;
  
  const newDoc = Automerge.change(clonedDoc, 'Rename spreadsheet', (d) => {
    d.name = name;
    d.updatedAt = Date.now();
  });
  
  currentSpreadsheetDoc.value = newDoc;
  saveAndBroadcast(clonedDoc, newDoc);
}

// Update the list item with current spreadsheet info (internal, no broadcast)
function updateSpreadsheetListItemInternal(): void {
  const doc = currentSpreadsheetDoc.value;
  if (!doc) return;
  
  spreadsheetList.value = spreadsheetList.value.map((item) =>
    item.id === String(doc.id)
      ? { ...item, name: doc.name, updatedAt: doc.updatedAt }
      : item
  );
}

// Save and broadcast changes to other tabs (server sync handles persistence)
function saveAndBroadcast(oldDoc: Automerge.Doc<AutomergeSpreadsheet>, newDoc: Automerge.Doc<AutomergeSpreadsheet>): void {
  updateSpreadsheetListItemInternal();
  // Note: String() cast is required because Automerge returns CRDT wrapper objects
  broadcastChanges(String(newDoc.id), oldDoc, newDoc);
}

// Get the Automerge binary for current spreadsheet
export function getSpreadsheetBinary(): Uint8Array | null {
  const doc = currentSpreadsheetDoc.value;
  if (!doc) return null;
  return Automerge.save(doc);
}

// Merge remote changes into the spreadsheet
export function mergeRemoteChanges(id: string, remoteBinary: Uint8Array): void {
  const doc = currentSpreadsheetDoc.value;
  // Note: String() cast is required because Automerge returns CRDT wrapper objects
  if (!doc || String(doc.id) !== id) return;
  
  const remoteDoc = Automerge.load<AutomergeSpreadsheet>(remoteBinary);
  const mergedDoc = Automerge.merge(doc, remoteDoc);
  
  currentSpreadsheetDoc.value = mergedDoc;
  updateSpreadsheetListItemInternal();
}

// ===== Chart Operations =====

// Create a new chart
export function createChart(
  type: ChartType,
  title: string,
  dataRange: ChartDataRange,
  position?: ChartPosition
): string | null {
  const clonedDoc = getLatestClonedDoc();
  if (!clonedDoc) return null;
  
  const chartId = generateChartId();
  const now = Date.now();
  
  const chart: ChartData = {
    id: chartId,
    type,
    title,
    position: position || { x: 100, y: 100 },
    size: { width: DEFAULT_CHART_WIDTH, height: DEFAULT_CHART_HEIGHT },
    dataRange,
    createdAt: now,
    updatedAt: now,
  };
  
  // Save to undo stack before making changes
  const doc = currentSpreadsheetDoc.value;
  if (doc) {
    pushToUndoStack(doc);
  }
  
  const newDoc = Automerge.change(clonedDoc, `Create chart ${title}`, (d) => {
    if (!d.charts) {
      d.charts = {};
    }
    d.charts[chartId] = chart;
    d.updatedAt = now;
  });
  
  currentSpreadsheetDoc.value = newDoc;
  saveAndBroadcast(clonedDoc, newDoc);
  
  return chartId;
}

// Update chart position
export function updateChartPosition(chartId: string, position: ChartPosition): void {
  const clonedDoc = getLatestClonedDoc();
  if (!clonedDoc) return;
  
  if (!clonedDoc.charts || !clonedDoc.charts[chartId]) return;
  
  const newDoc = Automerge.change(clonedDoc, `Move chart`, (d) => {
    if (d.charts && d.charts[chartId]) {
      d.charts[chartId].position = position;
      d.charts[chartId].updatedAt = Date.now();
      d.updatedAt = Date.now();
    }
  });
  
  currentSpreadsheetDoc.value = newDoc;
  saveAndBroadcast(clonedDoc, newDoc);
}

// Update chart size
export function updateChartSize(chartId: string, size: ChartSize): void {
  const clonedDoc = getLatestClonedDoc();
  if (!clonedDoc) return;
  
  if (!clonedDoc.charts || !clonedDoc.charts[chartId]) return;
  
  const newDoc = Automerge.change(clonedDoc, `Resize chart`, (d) => {
    if (d.charts && d.charts[chartId]) {
      d.charts[chartId].size = size;
      d.charts[chartId].updatedAt = Date.now();
      d.updatedAt = Date.now();
    }
  });
  
  currentSpreadsheetDoc.value = newDoc;
  saveAndBroadcast(clonedDoc, newDoc);
}

// Update chart data range
export function updateChartDataRange(chartId: string, dataRange: ChartDataRange): void {
  const clonedDoc = getLatestClonedDoc();
  if (!clonedDoc) return;
  
  if (!clonedDoc.charts || !clonedDoc.charts[chartId]) return;
  
  // Save to undo stack before making changes
  const doc = currentSpreadsheetDoc.value;
  if (doc) {
    pushToUndoStack(doc);
  }
  
  const newDoc = Automerge.change(clonedDoc, `Update chart data range`, (d) => {
    if (d.charts && d.charts[chartId]) {
      d.charts[chartId].dataRange = dataRange;
      d.charts[chartId].updatedAt = Date.now();
      d.updatedAt = Date.now();
    }
  });
  
  currentSpreadsheetDoc.value = newDoc;
  saveAndBroadcast(clonedDoc, newDoc);
}

// Update chart title
export function updateChartTitle(chartId: string, title: string): void {
  const clonedDoc = getLatestClonedDoc();
  if (!clonedDoc) return;
  
  if (!clonedDoc.charts || !clonedDoc.charts[chartId]) return;
  
  const newDoc = Automerge.change(clonedDoc, `Update chart title`, (d) => {
    if (d.charts && d.charts[chartId]) {
      d.charts[chartId].title = title;
      d.charts[chartId].updatedAt = Date.now();
      d.updatedAt = Date.now();
    }
  });
  
  currentSpreadsheetDoc.value = newDoc;
  saveAndBroadcast(clonedDoc, newDoc);
}

// Delete a chart
export function deleteChart(chartId: string): void {
  const clonedDoc = getLatestClonedDoc();
  if (!clonedDoc) return;
  
  if (!clonedDoc.charts || !clonedDoc.charts[chartId]) return;
  
  // Save to undo stack before making changes
  const doc = currentSpreadsheetDoc.value;
  if (doc) {
    pushToUndoStack(doc);
  }
  
  const newDoc = Automerge.change(clonedDoc, `Delete chart`, (d) => {
    if (d.charts && d.charts[chartId]) {
      delete d.charts[chartId];
      d.updatedAt = Date.now();
    }
  });
  
  currentSpreadsheetDoc.value = newDoc;
  saveAndBroadcast(clonedDoc, newDoc);
}
