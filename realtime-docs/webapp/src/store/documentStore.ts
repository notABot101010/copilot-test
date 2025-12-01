import { signal } from '@preact/signals';
import * as Automerge from '@automerge/automerge';
import { getServerUrl, initSync, startServerSync, stopServerSync, broadcastChanges } from './syncManager';

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

// Global signals
export const documentList = signal<DocumentInfo[]>([]);
export const currentDocument = signal<Automerge.Doc<DocumentSchema> | null>(null);
export const currentDocumentId = signal<string | null>(null);
export const isLoadingList = signal(false);
export const isLoadingDocument = signal(false);

// Helper functions for base64 encoding/decoding
function base64ToUint8Array(base64: string): Uint8Array {
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes;
}

// Initialize sync manager
initSync((documentId: string, binary: Uint8Array) => {
  if (currentDocumentId.value === documentId && currentDocument.value) {
    try {
      const remoteDoc = Automerge.load<DocumentSchema>(binary);
      const mergedDoc = Automerge.merge(currentDocument.value, remoteDoc);
      currentDocument.value = mergedDoc;
    } catch (err) {
      console.debug('Failed to merge remote document:', err);
    }
  }
});

/**
 * Load the list of documents from the server
 */
export async function loadDocumentList(): Promise<void> {
  isLoadingList.value = true;
  try {
    const response = await fetch(`${API_URL}/api/documents`);
    if (!response.ok) {
      throw new Error('Failed to load documents');
    }
    const data = await response.json();
    documentList.value = data.documents;
  } catch (err) {
    console.error('Failed to load documents:', err);
  } finally {
    isLoadingList.value = false;
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
    
    // Refresh the document list
    await loadDocumentList();
    
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
    
    // Update the local list
    documentList.value = documentList.value.filter(doc => doc.id !== id);
    
    return true;
  } catch (err) {
    console.error('Failed to delete document:', err);
    return false;
  }
}

/**
 * Load a document from the server
 */
export async function loadDocument(id: string): Promise<boolean> {
  isLoadingDocument.value = true;
  currentDocumentId.value = id;
  
  try {
    const response = await fetch(`${API_URL}/api/documents/${id}`);
    if (!response.ok) {
      throw new Error('Failed to load document');
    }
    
    const data = await response.json();
    const binary = base64ToUint8Array(data.document);
    const doc = Automerge.load<DocumentSchema>(binary);
    
    currentDocument.value = doc;
    
    // Start real-time sync
    startServerSync(id);
    
    return true;
  } catch (err) {
    console.error('Failed to load document:', err);
    currentDocument.value = null;
    return false;
  } finally {
    isLoadingDocument.value = false;
  }
}

/**
 * Update the document content
 */
export function updateDocumentContent(newContent: string): void {
  if (!currentDocument.value || !currentDocumentId.value) return;
  
  const oldDoc = currentDocument.value;
  const currentContent = String(oldDoc.content || '');
  
  // Skip if content hasn't changed
  if (currentContent === newContent) return;
  
  const newDoc = Automerge.change(oldDoc, doc => {
    doc.content = newContent;
    doc.updatedAt = Date.now();
  });
  
  currentDocument.value = newDoc;
  
  // Broadcast changes to other clients
  broadcastChanges(currentDocumentId.value, oldDoc, newDoc);
}

/**
 * Update the document title
 */
export function updateDocumentTitle(newTitle: string): void {
  if (!currentDocument.value || !currentDocumentId.value) return;
  
  const oldDoc = currentDocument.value;
  
  const newDoc = Automerge.change(oldDoc, doc => {
    doc.title = newTitle;
    doc.updatedAt = Date.now();
  });
  
  currentDocument.value = newDoc;
  
  // Broadcast changes to other clients
  broadcastChanges(currentDocumentId.value, oldDoc, newDoc);
}

/**
 * Stop editing and disconnect from real-time sync
 */
export function closeDocument(): void {
  stopServerSync();
  currentDocument.value = null;
  currentDocumentId.value = null;
}

/**
 * Get the current document content as a string
 */
export function getDocumentContent(): string {
  if (!currentDocument.value) return '';
  return String(currentDocument.value.content || '');
}

/**
 * Get the current document title
 */
export function getDocumentTitle(): string {
  if (!currentDocument.value) return '';
  return currentDocument.value.title;
}
