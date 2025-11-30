import type { Document } from './types';

export async function listDocuments(): Promise<Document[]> {
  const response = await fetch('/api/documents');
  if (!response.ok) {
    throw new Error('Failed to fetch documents');
  }
  return response.json();
}

export async function createDocument(
  title: string,
  docType: 'document' | 'presentation'
): Promise<Document> {
  const response = await fetch('/api/documents', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      title,
      doc_type: docType,
    }),
  });

  if (!response.ok) {
    throw new Error('Failed to create document');
  }

  return response.json();
}

export async function deleteDocument(id: string): Promise<void> {
  const response = await fetch(`/api/documents/${id}`, {
    method: 'DELETE',
  });

  if (!response.ok) {
    throw new Error('Failed to delete document');
  }
}
