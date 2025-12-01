import { signal, Signal } from '@preact/signals';
import * as Automerge from '@automerge/automerge';
import { getServerUrl, initSync, startServerSync, stopServerSync, broadcastChanges, setSyncUpdateCallback } from './syncManager';

// API endpoint
const API_URL = getServerUrl();

// Document type for Automerge
export interface DocumentSchema {
  id: string;
  title: string;
  content: string;
  createdAt: number;
  updatedAt: number;
}

// Document info for list view
export interface DocumentInfo {
  id: string;
  title: string;
  createdAt: number;
  updatedAt: number;
}

// Document state interface for local signals
export interface DocumentState {
  document: Signal<Automerge.Doc<DocumentSchema> | null>;
  documentId: Signal<string | null>;
  isLoading: Signal<boolean>;
}

// Helper functions for base64 encoding/decoding
function base64ToUint8Array(base64: string): Uint8Array {
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes;
}

// Initialize sync manager (just set up the broadcast channel, no callback yet)
initSync();

/**
 * Create local document state signals for a component.
 * These signals are scoped to the component and will be cleaned up when it unmounts.
 */
export function createDocumentState(): DocumentState {
  return {
    document: signal<Automerge.Doc<DocumentSchema> | null>(null),
    documentId: signal<string | null>(null),
    isLoading: signal(false),
  };
}

/**
 * Load the list of documents from the server
 */
export async function fetchDocumentList(): Promise<DocumentInfo[]> {
  try {
    const response = await fetch(`${API_URL}/api/documents`);
    if (!response.ok) {
      throw new Error('Failed to load documents');
    }
    const data = await response.json();
    return data.documents;
  } catch (err) {
    console.error('Failed to load documents:', err);
    return [];
  }
}

/**
 * Create a new document
 */
export async function createDocument(title: string): Promise<string | null> {
  try {
    const response = await fetch(`${API_URL}/api/documents`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ title }),
    });
    
    if (!response.ok) {
      throw new Error('Failed to create document');
    }
    
    const data = await response.json();
    return data.id;
  } catch (err) {
    console.error('Failed to create document:', err);
    return null;
  }
}

/**
 * Delete a document
 */
export async function deleteDocument(id: string): Promise<boolean> {
  try {
    const response = await fetch(`${API_URL}/api/documents/${id}`, {
      method: 'DELETE',
    });
    
    if (!response.ok) {
      throw new Error('Failed to delete document');
    }
    
    return true;
  } catch (err) {
    console.error('Failed to delete document:', err);
    return false;
  }
}

/**
 * Load a document from the server with local state
 */
export async function loadDocument(id: string, state: DocumentState): Promise<boolean> {
  state.isLoading.value = true;
  state.documentId.value = id;
  
  // Set up sync callback for this document state
  setSyncUpdateCallback((documentId: string, binary: Uint8Array) => {
    if (state.documentId.value === documentId && state.document.value) {
      try {
        const remoteDoc = Automerge.load<DocumentSchema>(binary);
        const mergedDoc = Automerge.merge(state.document.value, remoteDoc);
        state.document.value = mergedDoc;
      } catch (err) {
        console.debug('Failed to merge remote document:', err);
      }
    }
  });
  
  try {
    const response = await fetch(`${API_URL}/api/documents/${id}`);
    if (!response.ok) {
      throw new Error('Failed to load document');
    }
    
    const data = await response.json();
    const binary = base64ToUint8Array(data.document);
    const doc = Automerge.load<DocumentSchema>(binary);
    
    state.document.value = doc;
    
    // Start real-time sync
    startServerSync(id);
    
    return true;
  } catch (err) {
    console.error('Failed to load document:', err);
    state.document.value = null;
    return false;
  } finally {
    state.isLoading.value = false;
  }
}

/**
 * Update the document content with local state
 */
export function updateDocumentContent(newContent: string, state: DocumentState): void {
  if (!state.document.value || !state.documentId.value) return;
  
  const oldDoc = state.document.value;
  const currentContent = String(oldDoc.content || '');
  
  // Skip if content hasn't changed
  if (currentContent === newContent) return;
  
  const newDoc = Automerge.change(oldDoc, doc => {
    doc.content = newContent;
    doc.updatedAt = Date.now();
  });
  
  state.document.value = newDoc;
  
  // Broadcast changes to other clients
  broadcastChanges(state.documentId.value, oldDoc, newDoc);
}

/**
 * Update the document title with local state
 */
export function updateDocumentTitle(newTitle: string, state: DocumentState): void {
  if (!state.document.value || !state.documentId.value) return;
  
  const oldDoc = state.document.value;
  
  const newDoc = Automerge.change(oldDoc, doc => {
    doc.title = newTitle;
    doc.updatedAt = Date.now();
  });
  
  state.document.value = newDoc;
  
  // Broadcast changes to other clients
  broadcastChanges(state.documentId.value, oldDoc, newDoc);
}

/**
 * Close document and clean up local state
 */
export function closeDocument(state: DocumentState): void {
  stopServerSync();
  setSyncUpdateCallback(null);
  state.document.value = null;
  state.documentId.value = null;
}
