// API client for VectorDB

// Get base URL from environment or use current origin for API calls
const getBaseUrl = (): string => {
  // Check for environment variable first (set at build time via VITE_BASE_URL)
  if (typeof import.meta.env !== 'undefined' && import.meta.env.VITE_BASE_URL) {
    return import.meta.env.VITE_BASE_URL;
  }
  // In development, the Vite proxy handles /api routes
  return '';
};

const BASE_URL = getBaseUrl();

export interface Namespace {
  name: string;
  document_count: number;
  distance_metric: string;
  vector_dimensions: number | null;
}

export interface Document {
  id: string;
  vector?: number[];
  [key: string]: unknown;
}

export interface QueryResult {
  id: string;
  score: number;
  vector?: number[];
  attributes?: Record<string, unknown>;
}

export interface QueryResponse {
  results: QueryResult[];
  total_count: number;
}

export interface ApiKey {
  id: number;
  name: string;
  created_at: string;
  last_used_at: string | null;
}

export interface CreateApiKeyResponse {
  id: number;
  key: string;
  name: string;
}

// API Key header
let apiKey: string | null = null;

export function setApiKey(key: string | null) {
  apiKey = key;
}

export function getApiKey(): string | null {
  return apiKey;
}

function getHeaders(): HeadersInit {
  const headers: HeadersInit = {
    'Content-Type': 'application/json',
  };
  if (apiKey) {
    headers['Authorization'] = `Bearer ${apiKey}`;
  }
  return headers;
}

// ============ Namespace API ============

export async function listNamespaces(): Promise<Namespace[]> {
  const response = await fetch(`${BASE_URL}/api/namespaces`, {
    headers: getHeaders(),
  });
  if (!response.ok) {
    throw new Error(`Failed to list namespaces: ${response.statusText}`);
  }
  return response.json();
}

export async function getNamespace(name: string): Promise<Namespace> {
  const response = await fetch(`${BASE_URL}/api/namespaces/${encodeURIComponent(name)}`, {
    headers: getHeaders(),
  });
  if (!response.ok) {
    throw new Error(`Failed to get namespace: ${response.statusText}`);
  }
  return response.json();
}

export async function deleteNamespace(name: string): Promise<void> {
  const response = await fetch(`${BASE_URL}/api/namespaces/${encodeURIComponent(name)}`, {
    method: 'DELETE',
    headers: getHeaders(),
  });
  if (!response.ok) {
    throw new Error(`Failed to delete namespace: ${response.statusText}`);
  }
}

export async function upsertDocuments(
  namespace: string,
  documents: Document[],
  distanceMetric?: string
): Promise<{ status: string; document_count: number }> {
  const body: { documents: Document[]; distance_metric?: string } = { documents };
  if (distanceMetric) {
    body.distance_metric = distanceMetric;
  }
  
  const response = await fetch(`${BASE_URL}/api/namespaces/${encodeURIComponent(namespace)}`, {
    method: 'POST',
    headers: getHeaders(),
    body: JSON.stringify(body),
  });
  if (!response.ok) {
    const error = await response.text();
    throw new Error(`Failed to upsert documents: ${error}`);
  }
  return response.json();
}

// ============ Document API ============

export async function getDocuments(namespace: string): Promise<Document[]> {
  const response = await fetch(`${BASE_URL}/api/namespaces/${encodeURIComponent(namespace)}/documents`, {
    headers: getHeaders(),
  });
  if (!response.ok) {
    throw new Error(`Failed to get documents: ${response.statusText}`);
  }
  return response.json();
}

export async function getDocument(namespace: string, docId: string): Promise<Document> {
  const response = await fetch(
    `${BASE_URL}/api/namespaces/${encodeURIComponent(namespace)}/documents/${encodeURIComponent(docId)}`,
    { headers: getHeaders() }
  );
  if (!response.ok) {
    throw new Error(`Failed to get document: ${response.statusText}`);
  }
  return response.json();
}

export async function deleteDocuments(namespace: string, ids: string[]): Promise<{ status: string; deleted_count: number }> {
  const response = await fetch(`${BASE_URL}/api/namespaces/${encodeURIComponent(namespace)}/documents`, {
    method: 'DELETE',
    headers: getHeaders(),
    body: JSON.stringify({ ids }),
  });
  if (!response.ok) {
    throw new Error(`Failed to delete documents: ${response.statusText}`);
  }
  return response.json();
}

// ============ Query API ============

export interface QueryParams {
  rank_by: unknown;
  top_k?: number;
  filters?: unknown;
  include_attributes?: string[];
  include_vector?: boolean;
}

export async function queryNamespace(namespace: string, params: QueryParams): Promise<QueryResponse> {
  const response = await fetch(`${BASE_URL}/api/namespaces/${encodeURIComponent(namespace)}/query`, {
    method: 'POST',
    headers: getHeaders(),
    body: JSON.stringify(params),
  });
  if (!response.ok) {
    const error = await response.text();
    throw new Error(`Failed to query namespace: ${error}`);
  }
  return response.json();
}

// ============ API Key API ============

export async function listApiKeys(): Promise<ApiKey[]> {
  const response = await fetch(`${BASE_URL}/api/keys`, {
    headers: getHeaders(),
  });
  if (!response.ok) {
    throw new Error(`Failed to list API keys: ${response.statusText}`);
  }
  return response.json();
}

export async function createApiKey(name: string): Promise<CreateApiKeyResponse> {
  const response = await fetch(`${BASE_URL}/api/keys`, {
    method: 'POST',
    headers: getHeaders(),
    body: JSON.stringify({ name }),
  });
  if (!response.ok) {
    throw new Error(`Failed to create API key: ${response.statusText}`);
  }
  return response.json();
}

export async function deleteApiKey(id: number): Promise<void> {
  const response = await fetch(`${BASE_URL}/api/keys/${id}`, {
    method: 'DELETE',
    headers: getHeaders(),
  });
  if (!response.ok) {
    throw new Error(`Failed to delete API key: ${response.statusText}`);
  }
}
