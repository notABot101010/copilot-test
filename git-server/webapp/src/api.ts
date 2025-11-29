// Types for API responses

export interface RepoInfo {
  name: string;
  path: string;
}

export interface CommitInfo {
  hash: string;
  short_hash: string;
  author: string;
  date: string;
  message: string;
}

export interface FileEntry {
  name: string;
  path: string;
  type: 'file' | 'dir';
  size: number | null;
}

// API helper function
async function api<T>(endpoint: string, options?: RequestInit): Promise<T> {
  const response = await fetch('/api' + endpoint, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      ...options?.headers,
    },
  });
  if (!response.ok) {
    const text = await response.text();
    throw new Error(text || `API error: ${response.status}`);
  }
  const contentType = response.headers.get('content-type');
  if (contentType && contentType.includes('application/json')) {
    return response.json() as Promise<T>;
  }
  return response.text() as unknown as T;
}

// Repository APIs
export async function listRepos(): Promise<RepoInfo[]> {
  return api<RepoInfo[]>('/repos');
}

export async function createRepo(name: string): Promise<RepoInfo> {
  return api<RepoInfo>('/repos', {
    method: 'POST',
    body: JSON.stringify({ name }),
  });
}

export async function getRepoTree(
  name: string,
  ref: string = 'HEAD',
  path: string = ''
): Promise<FileEntry[]> {
  const params = new URLSearchParams();
  if (ref !== 'HEAD') params.set('ref', ref);
  if (path) params.set('path', path);
  const query = params.toString() ? '?' + params.toString() : '';
  return api<FileEntry[]>(`/repos/${encodeURIComponent(name)}/tree${query}`);
}

export async function getRepoCommits(name: string): Promise<CommitInfo[]> {
  return api<CommitInfo[]>(`/repos/${encodeURIComponent(name)}/commits`);
}

export async function getBlob(
  name: string,
  path: string,
  ref: string = 'HEAD'
): Promise<string> {
  const params = new URLSearchParams();
  params.set('path', path);
  if (ref !== 'HEAD') params.set('ref', ref);
  return api<string>(`/repos/${encodeURIComponent(name)}/blob?${params.toString()}`);
}

export async function updateFile(
  repoName: string,
  path: string,
  content: string,
  message: string
): Promise<void> {
  await api(`/repos/${encodeURIComponent(repoName)}/files`, {
    method: 'POST',
    body: JSON.stringify({ path, content, message }),
  });
}

// Utility functions
export function formatSize(bytes: number): string {
  if (bytes < 1024) return bytes + ' B';
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
  return (bytes / (1024 * 1024)).toFixed(1) + ' MB';
}

export function formatDate(dateString: string): string {
  const date = new Date(dateString);
  return date.toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  });
}
