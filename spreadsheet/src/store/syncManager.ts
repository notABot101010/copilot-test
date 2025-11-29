import * as Automerge from '@automerge/automerge';

// BroadcastChannel name for spreadsheet sync
const SYNC_CHANNEL_NAME = 'spreadsheet-sync';

// Message types for the sync channel
interface SyncMessage {
  type: 'sync-update';
  spreadsheetId: string;
  changes: string; // base64 encoded Automerge changes
}

// Store callbacks for when updates are received
type UpdateCallback = (spreadsheetId: string, changes: Uint8Array) => void;

let channel: BroadcastChannel | null = null;
let updateCallback: UpdateCallback | null = null;

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
 * Initialize the sync manager with a callback for handling incoming updates
 */
export function initSync(onUpdate: UpdateCallback): void {
  if (channel) {
    // Already initialized
    return;
  }

  updateCallback = onUpdate;
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
 * Broadcast Automerge changes to other tabs
 */
export function broadcastChanges<T>(spreadsheetId: string, oldDoc: Automerge.Doc<T>, newDoc: Automerge.Doc<T>): void {
  if (!channel) {
    return;
  }

  // Get the changes between old and new documents
  const changes = Automerge.getChanges(oldDoc, newDoc);
  if (changes.length === 0) {
    return;
  }

  // Encode changes as base64 for transmission
  // We encode each change separately and send as a single binary
  const encoded = Automerge.save(newDoc);
  
  const message: SyncMessage = {
    type: 'sync-update',
    spreadsheetId,
    changes: uint8ArrayToBase64(encoded),
  };

  channel.postMessage(message);
}

/**
 * Close the sync channel
 */
export function closeSync(): void {
  if (channel) {
    channel.close();
    channel = null;
    updateCallback = null;
  }
}
