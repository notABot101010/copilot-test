import * as Automerge from '@automerge/automerge';

// Server URLs for sync
const HTTP_SERVER_URL = import.meta.env.VITE_SYNC_SERVER_URL || 'http://localhost:4001';
const WS_SERVER_URL = (import.meta.env.VITE_SYNC_SERVER_URL || 'http://localhost:4001')
  .replace(/^http/, 'ws');

// Export getServerUrl for use in documentStore
export function getServerUrl(): string {
  return HTTP_SERVER_URL;
}

// BroadcastChannel name for document sync (same browser only)
const SYNC_CHANNEL_NAME = 'realtime-docs-sync';

// Reconnect delay in milliseconds
const RECONNECT_DELAY_MS = 2000;

// Message types for the BroadcastChannel
interface BroadcastSyncMessage {
  type: 'sync-update';
  documentId: string;
  changes: string; // base64 encoded Automerge document
}

// WebSocket message types (matching server)
interface WsIdentifyMessage {
  type: 'identify';
  client_id: string;
}

interface WsUpdateMessage {
  type: 'update';
  document: string;
}

interface WsSyncMessage {
  type: 'sync';
  document: string;
  sender_id: string;
}

interface WsConnectedMessage {
  type: 'connected';
  document: string;
}

interface WsErrorMessage {
  type: 'error';
  message: string;
}

type WsMessage = WsIdentifyMessage | WsUpdateMessage | WsSyncMessage | WsConnectedMessage | WsErrorMessage;

// Store callbacks for when updates are received
type UpdateCallback = (documentId: string, changes: Uint8Array) => void;

let channel: BroadcastChannel | null = null;
let updateCallback: UpdateCallback | null = null;
let currentDocumentId: string | null = null;
let ws: WebSocket | null = null;
let reconnectTimeoutId: number | null = null;
let clientId: string | null = null;

/**
 * Set or clear the sync update callback
 * This allows components to register/unregister their callback dynamically
 */
export function setSyncUpdateCallback(callback: UpdateCallback | null): void {
  updateCallback = callback;
}

// Generate a unique client ID using crypto.randomUUID if available
function generateClientId(): string {
  if (typeof crypto !== 'undefined' && crypto.randomUUID) {
    return crypto.randomUUID();
  }
  // Fallback for older browsers
  return Math.random().toString(36).substring(2, 15) + Date.now().toString(36);
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

/**
 * Initialize the sync manager
 * The update callback can be set later via setSyncUpdateCallback
 */
export function initSync(): void {
  if (channel) {
    // Already initialized
    return;
  }

  clientId = generateClientId();

  // Initialize BroadcastChannel for same-browser tab sync
  channel = new BroadcastChannel(SYNC_CHANNEL_NAME);

  channel.onmessage = (event: MessageEvent<BroadcastSyncMessage>) => {
    const { type, documentId, changes } = event.data;
    if (type === 'sync-update' && updateCallback) {
      const changesBytes = base64ToUint8Array(changes);
      updateCallback(documentId, changesBytes);
    }
  };
}

/**
 * Start WebSocket connection for real-time sync
 */
export function startServerSync(documentId: string): void {
  currentDocumentId = documentId;

  // Close any existing WebSocket connection
  closeWebSocket();

  // Connect via WebSocket
  connectWebSocket(documentId);
}

/**
 * Connect to WebSocket server for real-time updates
 */
function connectWebSocket(documentId: string): void {
  if (!clientId) {
    clientId = generateClientId();
  }

  const wsUrl = `${WS_SERVER_URL}/ws/documents/${documentId}`;

  try {
    ws = new WebSocket(wsUrl);

    ws.onopen = () => {
      console.debug('WebSocket connected for document:', documentId);

      // Identify ourselves to the server
      if (ws && clientId) {
        const identifyMsg: WsIdentifyMessage = {
          type: 'identify',
          client_id: clientId,
        };
        ws.send(JSON.stringify(identifyMsg));
      }
    };

    ws.onmessage = (event) => {
      try {
        const msg = JSON.parse(event.data) as WsMessage;
        handleWsMessage(msg);
      } catch (err) {
        console.debug('Failed to parse WebSocket message:', err);
      }
    };

    ws.onclose = () => {
      console.debug('WebSocket closed for document:', documentId);
      ws = null;

      // Reconnect if we still want to be connected to this document
      if (currentDocumentId === documentId) {
        scheduleReconnect(documentId);
      }
    };

    ws.onerror = (err) => {
      console.debug('WebSocket error:', err);
    };
  } catch (err) {
    console.debug('Failed to create WebSocket:', err);
    scheduleReconnect(documentId);
  }
}

/**
 * Handle incoming WebSocket messages
 */
function handleWsMessage(msg: WsMessage): void {
  switch (msg.type) {
    case 'connected':
      // Server sent the current document state
      if (updateCallback && currentDocumentId) {
        const binary = base64ToUint8Array(msg.document);
        updateCallback(currentDocumentId, binary);
      }
      break;

    case 'sync':
      // Another client made changes
      if (msg.sender_id !== clientId && updateCallback && currentDocumentId) {
        const binary = base64ToUint8Array(msg.document);
        updateCallback(currentDocumentId, binary);
      }
      break;

    case 'error':
      console.error('WebSocket error from server:', msg.message);
      break;
  }
}

/**
 * Schedule WebSocket reconnection
 */
function scheduleReconnect(documentId: string): void {
  if (reconnectTimeoutId !== null) {
    clearTimeout(reconnectTimeoutId);
  }

  reconnectTimeoutId = window.setTimeout(() => {
    reconnectTimeoutId = null;
    if (currentDocumentId === documentId) {
      connectWebSocket(documentId);
    }
  }, RECONNECT_DELAY_MS);
}

/**
 * Close WebSocket connection
 */
function closeWebSocket(): void {
  if (reconnectTimeoutId !== null) {
    clearTimeout(reconnectTimeoutId);
    reconnectTimeoutId = null;
  }

  if (ws) {
    ws.close();
    ws = null;
  }
}

/**
 * Stop server sync
 */
export function stopServerSync(): void {
  closeWebSocket();
  currentDocumentId = null;
}

/**
 * Broadcast Automerge changes to other tabs and server via WebSocket
 */
export function broadcastChanges<T>(documentId: string, oldDoc: Automerge.Doc<T>, newDoc: Automerge.Doc<T>): void {
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
    const message: BroadcastSyncMessage = {
      type: 'sync-update',
      documentId,
      changes: encodedBase64,
    };
    channel.postMessage(message);
  }

  // Send via WebSocket for real-time cross-browser sync
  sendViaWebSocket(encodedBase64);
}

/**
 * Send document update via WebSocket
 */
function sendViaWebSocket(documentBase64: string): void {
  if (ws && ws.readyState === WebSocket.OPEN) {
    const msg: WsUpdateMessage = {
      type: 'update',
      document: documentBase64,
    };
    ws.send(JSON.stringify(msg));
  } else {
    // WebSocket not connected - fall back to HTTP sync
    if (currentDocumentId) {
      syncToServerHttp(currentDocumentId, base64ToUint8Array(documentBase64));
    }
  }
}

/**
 * Push changes to the server via HTTP (fallback when WebSocket is not available)
 */
async function syncToServerHttp(documentId: string, binary: Uint8Array): Promise<void> {
  try {
    await fetch(`${HTTP_SERVER_URL}/api/documents/${documentId}/sync`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        document: uint8ArrayToBase64(binary),
      }),
    });
  } catch (err) {
    // Network error - ignore (changes are saved locally)
    console.debug('Server push failed (server may be offline):', err);
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
  }
}
