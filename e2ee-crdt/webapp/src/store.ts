import { signal, computed } from '@preact/signals';
import * as Automerge from '@automerge/automerge';
import type { TextDocument, DocumentMetadata } from './utils/automerge';
import {
  createDocument,
  updateTitle,
  updateContent,
  prepareDocumentForStorage,
  loadDocumentFromStorage,
  encryptChange,
  applyChanges,
  getChanges,
} from './utils/automerge';
import {
  listDocuments as apiListDocuments,
  createDocument as apiCreateDocument,
  getDocument as apiGetDocument,
  updateDocument as apiUpdateDocument,
} from './utils/api';
import { createWebSocketClient, type WsOperationMessage, type WsDocumentCreatedMessage } from './utils/websocket';

export interface DocumentListItem {
  id: string;
  title: string;
  created_at: number;
  key: string;
  iv: string;
}

const documents = signal<Map<string, DocumentMetadata>>(new Map());
const documentList = signal<DocumentListItem[]>([]);
const currentDocumentId = signal<string | null>(null);
const loading = signal(false);
const error = signal<string | null>(null);

const wsClient = createWebSocketClient();

// Computed values
export const currentDocument = computed(() => {
  const id = currentDocumentId.value;
  if (!id) return null;
  return documents.value.get(id) || null;
});

export const documentListItems = computed(() => documentList.value);
export const isLoading = computed(() => loading.value);
export const errorMessage = computed(() => error.value);

// Store document keys in memory (in production, use proper key management)
const documentKeys = new Map<string, { key: string; iv: string }>();

// Initialize WebSocket
wsClient.connect();
wsClient.onOperation(async (message: WsOperationMessage) => {
  console.log('[WebSocket] Received operation for document:', message.document_id);

  const docMeta = documents.value.get(message.document_id);
  if (!docMeta) {
    console.log('[WebSocket] Document not loaded locally, ignoring operation');
    return;
  }

  const keyData = documentKeys.get(message.document_id);
  if (!keyData) {
    console.log('[WebSocket] No key found for document, ignoring operation');
    return;
  }

  try {
    // Parse the encrypted operation
    const parts = message.encrypted_operation.split(':');
    if (parts.length !== 2) {
      console.error('[WebSocket] Invalid operation format');
      return;
    }

    const [encryptedData, ivData] = parts;
    const { decryptChange } = await import('./utils/automerge');
    const changes = await decryptChange(encryptedData, ivData, docMeta.key);

    // Apply changes to document
    const newDoc = applyChanges(docMeta.doc, changes);
    console.log('[WebSocket] Applied remote changes, new doc:', { title: newDoc.title, content: newDoc.content });

    // Update document in store
    const newDocuments = new Map(documents.value);
    newDocuments.set(message.document_id, {
      ...docMeta,
      doc: newDoc,
    });
    documents.value = newDocuments;
    console.log('[WebSocket] Updated store with new document');
  } catch (err) {
    console.error('Failed to apply remote operation:', err);
  }
});

wsClient.onDocumentCreated(async (message: WsDocumentCreatedMessage) => {
  try {
    // Parse encrypted data to extract IV and key
    const parts = message.encrypted_data.split(':');
    if (parts.length < 3) return;

    const [encryptedData, ivData, keyData] = parts;

    const { loadDocumentFromStorage } = await import('./utils/automerge');
    const { doc: automergeDoc } = await loadDocumentFromStorage(
      encryptedData,
      ivData,
      keyData
    );

    // Add to document list
    const newItem = {
      id: message.id,
      title: automergeDoc.title,
      created_at: message.created_at,
      key: keyData,
      iv: ivData,
    };

    documentList.value = [newItem, ...documentList.value];
    documentKeys.set(message.id, { key: keyData, iv: ivData });

    console.log('New document added:', message.id);
  } catch (err) {
    console.error('Failed to process new document:', err);
  }
});

// Actions
export async function loadDocuments(): Promise<void> {
  loading.value = true;
  error.value = null;

  try {
    const docs = await apiListDocuments();

    const items: DocumentListItem[] = [];
    for (const doc of docs) {
      // Parse encrypted data to extract IV and key
      const parts = doc.encrypted_data.split(':');
      if (parts.length < 3) continue;

      const [encryptedData, ivData, keyData] = parts;

      try {
        const { doc: automergeDoc } = await loadDocumentFromStorage(
          encryptedData,
          ivData,
          keyData
        );

        items.push({
          id: doc.id,
          title: automergeDoc.title,
          created_at: doc.created_at,
          key: keyData,
          iv: ivData,
        });

        documentKeys.set(doc.id, { key: keyData, iv: ivData });
      } catch (err) {
        console.error(`Failed to decrypt document ${doc.id}:`, err);
      }
    }

    documentList.value = items;
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Failed to load documents';
  } finally {
    loading.value = false;
  }
}

export async function createNewDocument(title: string = 'Untitled'): Promise<string> {
  loading.value = true;
  error.value = null;

  try {
    const doc = createDocument(title);
    const { encryptedData, key, iv } = await prepareDocumentForStorage(doc);

    // Store with format: encryptedData:iv:key
    const combinedData = `${encryptedData}:${iv}:${key}`;
    const result = await apiCreateDocument(combinedData);

    const cryptoModule = await import('./utils/crypto');
    const cryptoKey = await cryptoModule.importKey(key);

    // Add to store
    const newDocuments = new Map(documents.value);
    newDocuments.set(result.id, {
      id: result.id,
      key: cryptoKey,
      doc,
    });
    documents.value = newDocuments;

    // Add to list
    documentList.value = [
      {
        id: result.id,
        title,
        created_at: Date.now(),
        key,
        iv,
      },
      ...documentList.value,
    ];

    documentKeys.set(result.id, { key, iv });

    return result.id;
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Failed to create document';
    throw err;
  } finally {
    loading.value = false;
  }
}

export async function loadDocument(id: string): Promise<void> {
  loading.value = true;
  error.value = null;

  try {
    // Check if already loaded
    if (documents.value.has(id)) {
      currentDocumentId.value = id;
      wsClient.subscribe(id);
      return;
    }

    const doc = await apiGetDocument(id);
    const parts = doc.encrypted_data.split(':');
    if (parts.length < 3) {
      throw new Error('Invalid encrypted data format');
    }

    const [encryptedData, ivData, keyData] = parts;
    const { doc: automergeDoc, key } = await loadDocumentFromStorage(
      encryptedData,
      ivData,
      keyData
    );

    // Add to store
    const newDocuments = new Map(documents.value);
    newDocuments.set(id, {
      id,
      key,
      doc: automergeDoc,
    });
    documents.value = newDocuments;

    documentKeys.set(id, { key: keyData, iv: ivData });
    currentDocumentId.value = id;

    wsClient.subscribe(id);
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Failed to load document';
  } finally {
    loading.value = false;
  }
}

export function selectDocument(id: string | null): void {
  currentDocumentId.value = id;
  if (id) {
    wsClient.subscribe(id);
  }
}

export async function updateDocumentTitle(id: string, title: string): Promise<void> {
  const docMeta = documents.value.get(id);
  if (!docMeta) return;

  const oldDoc = docMeta.doc;
  const newDoc = updateTitle(docMeta.doc, title);

  // Update local state
  const newDocuments = new Map(documents.value);
  newDocuments.set(id, { ...docMeta, doc: newDoc });
  documents.value = newDocuments;

  // Update list
  documentList.value = documentList.value.map((item) =>
    item.id === id ? { ...item, title } : item
  );

  await syncDocument(id, oldDoc, newDoc);
}

export async function updateDocumentContent(id: string, content: string): Promise<void> {
  const docMeta = documents.value.get(id);
  if (!docMeta) return;

  const oldDoc = docMeta.doc;
  const newDoc = updateContent(docMeta.doc, content);

  // Update local state
  const newDocuments = new Map(documents.value);
  newDocuments.set(id, { ...docMeta, doc: newDoc });
  documents.value = newDocuments;

  await syncDocument(id, oldDoc, newDoc);
}

async function syncDocument(
  id: string,
  oldDoc: Automerge.Doc<TextDocument>,
  newDoc: Automerge.Doc<TextDocument>
): Promise<void> {
  const docMeta = documents.value.get(id);
  if (!docMeta) return;

  const keyData = documentKeys.get(id);
  if (!keyData) return;

  try {
    // Get changes
    const changes = getChanges(oldDoc, newDoc);
    if (changes.length === 0) {
      console.log('[Sync] No changes to sync');
      return;
    }

    console.log('[Sync] Syncing document:', id, 'changes:', changes.length);

    // Encrypt changes
    const change = changes[0];
    const { encrypted, iv } = await encryptChange(change, docMeta.key);
    const encryptedOperation = `${encrypted}:${iv}`;

    // Encrypt full document
    const { encryptDocument } = await import('./utils/automerge');
    const { encrypted: encryptedDoc, iv: newIv } = await encryptDocument(newDoc, docMeta.key);
    const combinedData = `${encryptedDoc}:${newIv}:${keyData.key}`;

    // Update stored IV with the new one
    documentKeys.set(id, { key: keyData.key, iv: newIv });

    // Update document list item with new IV
    documentList.value = documentList.value.map((item) =>
      item.id === id ? { ...item, iv: newIv } : item
    );

    // Update server
    console.log('[Sync] Sending update to server');
    await apiUpdateDocument(id, combinedData, encryptedOperation);
    console.log('[Sync] Update sent successfully');
  } catch (err) {
    console.error('Failed to sync document:', err);
    error.value = err instanceof Error ? err.message : 'Failed to sync document';
  }
}

// Cleanup
export function cleanup(): void {
  wsClient.disconnect();
}
