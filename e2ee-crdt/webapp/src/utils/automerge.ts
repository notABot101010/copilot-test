import * as Automerge from '@automerge/automerge';
import { encrypt, decrypt, generateDocumentKey, exportKey, importKey } from './crypto';

export interface TextDocument extends Record<string, unknown> {
  title: string;
  content: string;
}

export interface EncryptedDocument {
  id: string;
  encrypted_data: string;
  created_at: number;
}

export interface DocumentMetadata {
  id: string;
  key: CryptoKey;
  doc: Automerge.Doc<TextDocument>;
}

// Create a new Automerge document
export function createDocument(title: string = 'Untitled'): Automerge.Doc<TextDocument> {
  return Automerge.from<TextDocument>({
    title,
    content: '',
  });
}

// Update document title
export function updateTitle(
  doc: Automerge.Doc<TextDocument>,
  title: string
): Automerge.Doc<TextDocument> {
  return Automerge.change(doc, (d) => {
    d.title = title;
  });
}

// Update document content
export function updateContent(
  doc: Automerge.Doc<TextDocument>,
  content: string
): Automerge.Doc<TextDocument> {
  return Automerge.change(doc, (d) => {
    d.content = content;
  });
}

// Encrypt an Automerge document
export async function encryptDocument(
  doc: Automerge.Doc<TextDocument>,
  key: CryptoKey
): Promise<{ encrypted: string; iv: string }> {
  const binary = Automerge.save(doc);
  return await encrypt(binary, key);
}

// Decrypt an Automerge document
export async function decryptDocument(
  encryptedData: string,
  ivData: string,
  key: CryptoKey
): Promise<Automerge.Doc<TextDocument>> {
  const decrypted = await decrypt(encryptedData, ivData, key);
  return Automerge.load<TextDocument>(decrypted);
}

// Encrypt a change/operation
export async function encryptChange(
  changes: Uint8Array,
  key: CryptoKey
): Promise<{ encrypted: string; iv: string }> {
  return await encrypt(changes, key);
}

// Decrypt a change/operation
export async function decryptChange(
  encryptedData: string,
  ivData: string,
  key: CryptoKey
): Promise<Uint8Array> {
  return await decrypt(encryptedData, ivData, key);
}

// Apply changes to a document
export function applyChanges(
  doc: Automerge.Doc<TextDocument>,
  changes: Uint8Array
): Automerge.Doc<TextDocument> {
  const [newDoc] = Automerge.applyChanges(doc, [changes]);
  return newDoc;
}

// Get changes between two document states
export function getChanges(
  oldDoc: Automerge.Doc<TextDocument>,
  newDoc: Automerge.Doc<TextDocument>
): Uint8Array[] {
  return Automerge.getChanges(oldDoc, newDoc);
}

// Merge two documents
export function mergeDocuments(
  doc1: Automerge.Doc<TextDocument>,
  doc2: Automerge.Doc<TextDocument>
): Automerge.Doc<TextDocument> {
  return Automerge.merge(doc1, doc2);
}

// Create encrypted document for storage
export async function prepareDocumentForStorage(
  doc: Automerge.Doc<TextDocument>
): Promise<{ encryptedData: string; key: string; iv: string }> {
  const key = await generateDocumentKey();
  const { encrypted, iv } = await encryptDocument(doc, key);
  const exportedKey = await exportKey(key);

  return {
    encryptedData: encrypted,
    key: exportedKey,
    iv,
  };
}

// Load document from storage
export async function loadDocumentFromStorage(
  encryptedData: string,
  ivData: string,
  keyData: string
): Promise<{ doc: Automerge.Doc<TextDocument>; key: CryptoKey }> {
  const key = await importKey(keyData);
  const doc = await decryptDocument(encryptedData, ivData, key);

  return { doc, key };
}
