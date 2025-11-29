import * as Automerge from '@automerge/automerge';

// Server URL for CRDT sync
const SYNC_SERVER_URL = import.meta.env.VITE_SYNC_SERVER_URL || 'http://localhost:3001';

// BroadcastChannel name for spreadsheet sync (same browser only)
const SYNC_CHANNEL_NAME = 'spreadsheet-sync';

// Sync interval in milliseconds (poll server every 2 seconds)
const SYNC_INTERVAL_MS = 2000;

// Message types for the sync channel
interface SyncMessage {
  type: 'sync-update';
  spreadsheetId: string;
  changes: string; // base64 encoded Automerge document
}

// Server sync response
interface SyncResponse {
  document: string; // base64 encoded Automerge document
  updated: boolean;
}

// Store callbacks for when updates are received
type UpdateCallback = (spreadsheetId: string, changes: Uint8Array) => void;
type GetDocCallback = () => { id: string; binary: Uint8Array } | null;

let channel: BroadcastChannel | null = null;
let updateCallback: UpdateCallback | null = null;
let getDocCallback: GetDocCallback | null = null;
let syncIntervalId: number | null = null;
let currentSpreadsheetId: string | null = null;

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

/**
 * Initialize the sync manager with callbacks
 * @param onUpdate - Called when remote changes are received
 * @param getDoc - Called to get the current document for syncing
 */
export function initSync(onUpdate: UpdateCallback, getDoc?: GetDocCallback): void {
  if (channel) {
    // Already initialized
    return;
  }

  updateCallback = onUpdate;
  getDocCallback = getDoc || null;
  
  // Initialize BroadcastChannel for same-browser tab sync
  channel = new BroadcastChannel(SYNC_CHANNEL_NAME);

  channel.onmessage = (event: MessageEvent<SyncMessage>) => {
    const { type, spreadsheetId, changes } = event.data;
    if (type === 'sync-update' && updateCallback) {
      const changesBytes = base64ToUint8Array(changes);
      updateCallback(spreadsheetId, changesBytes);
    }
  };
}

/**
 * Start periodic sync with the server for a specific spreadsheet
 */
export function startServerSync(spreadsheetId: string): void {
  currentSpreadsheetId = spreadsheetId;
  
  // Clear any existing sync interval
  if (syncIntervalId !== null) {
    clearInterval(syncIntervalId);
  }
  
  // Start periodic sync
  syncIntervalId = window.setInterval(() => {
    syncWithServer();
  }, SYNC_INTERVAL_MS);
  
  // Do an immediate sync
  syncWithServer();
}

/**
 * Stop server sync
 */
export function stopServerSync(): void {
  if (syncIntervalId !== null) {
    clearInterval(syncIntervalId);
    syncIntervalId = null;
  }
  currentSpreadsheetId = null;
}

/**
 * Sync current document with the server
 */
async function syncWithServer(): Promise<void> {
  if (!currentSpreadsheetId || !getDocCallback || !updateCallback) {
    return;
  }
  
  const docInfo = getDocCallback();
  if (!docInfo || docInfo.id !== currentSpreadsheetId) {
    return;
  }
  
  try {
    const response = await fetch(`${SYNC_SERVER_URL}/api/spreadsheets/${currentSpreadsheetId}/sync`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        document: uint8ArrayToBase64(docInfo.binary),
      }),
    });
    
    if (!response.ok) {
      // Server might not have this document yet - the sync endpoint will create it
      // so 404 shouldn't happen, but if it does, the document will be created on next sync
      console.error('Sync failed:', response.status);
      return;
    }
    
    const data: SyncResponse = await response.json();
    
    if (data.updated) {
      // Server has newer changes - update our document
      const serverBinary = base64ToUint8Array(data.document);
      updateCallback(currentSpreadsheetId, serverBinary);
    }
  } catch (error) {
    // Network error - ignore and retry on next interval
    console.debug('Server sync failed (server may be offline):', error);
  }
}

/**
 * Broadcast Automerge changes to other tabs and sync with server
 */
export function broadcastChanges<T>(spreadsheetId: string, oldDoc: Automerge.Doc<T>, newDoc: Automerge.Doc<T>): void {
  // Get the changes between old and new documents
  const changes = Automerge.getChanges(oldDoc, newDoc);
  if (changes.length === 0) {
    return;
  }

  // Encode the new document as base64 for transmission
  const encoded = Automerge.save(newDoc);
  const encodedBase64 = uint8ArrayToBase64(encoded);
  
  // Broadcast to other tabs in the same browser (faster)
  if (channel) {
    const message: SyncMessage = {
      type: 'sync-update',
      spreadsheetId,
      changes: encodedBase64,
    };
    channel.postMessage(message);
  }
  
  // Also sync with server for cross-browser support
  syncToServer(spreadsheetId, encoded);
}

/**
 * Push changes to the server
 */
async function syncToServer(spreadsheetId: string, binary: Uint8Array): Promise<void> {
  try {
    await fetch(`${SYNC_SERVER_URL}/api/spreadsheets/${spreadsheetId}/sync`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        document: uint8ArrayToBase64(binary),
      }),
    });
  } catch (error) {
    // Network error - ignore (changes are saved locally)
    console.debug('Server push failed (server may be offline):', error);
  }
}

/**
 * Close the sync channel and stop server sync
 */
export function closeSync(): void {
  stopServerSync();
  
  if (channel) {
    channel.close();
    channel = null;
    updateCallback = null;
    getDocCallback = null;
  }
}
