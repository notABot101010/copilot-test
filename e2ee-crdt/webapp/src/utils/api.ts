export interface Document {
  id: string;
  encrypted_data: string;
  created_at: number;
}

export interface DocumentOperation {
  document_id: string;
  encrypted_operation: string;
  timestamp: number;
}

const API_BASE = '/api';

export async function listDocuments(): Promise<Document[]> {
  const response = await fetch(`${API_BASE}/documents`);
  if (!response.ok) {
    throw new Error('Failed to fetch documents');
  }
  return await response.json();
}

export async function createDocument(encryptedData: string): Promise<{ id: string }> {
  const response = await fetch(`${API_BASE}/documents`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ encrypted_data: encryptedData }),
  });
  if (!response.ok) {
    throw new Error('Failed to create document');
  }
  return await response.json();
}

export async function getDocument(id: string): Promise<Document> {
  const response = await fetch(`${API_BASE}/documents/${id}`);
  if (!response.ok) {
    throw new Error('Failed to fetch document');
  }
  return await response.json();
}

export async function updateDocument(
  id: string,
  encryptedData: string,
  encryptedOperation: string
): Promise<void> {
  const response = await fetch(`${API_BASE}/documents/${id}`, {
    method: 'PUT',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      encrypted_data: encryptedData,
      encrypted_operation: encryptedOperation,
    }),
  });
  if (!response.ok) {
    throw new Error('Failed to update document');
  }
}

export async function getOperations(id: string): Promise<DocumentOperation[]> {
  const response = await fetch(`${API_BASE}/documents/${id}/operations`);
  if (!response.ok) {
    throw new Error('Failed to fetch operations');
  }
  return await response.json();
}
