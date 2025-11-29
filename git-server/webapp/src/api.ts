// Types for API responses

export interface RepoInfo {
  name: string;
  path: string;
  forked_from?: string | null;
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

export interface Issue {
  id: number;
  repo_name: string;
  number: number;
  title: string;
  body: string;
  state: 'open' | 'closed';
  author: string;
  created_at: string;
  updated_at: string;
}

export interface IssueComment {
  id: number;
  issue_id: number;
  body: string;
  author: string;
  created_at: string;
}

export interface PullRequest {
  id: number;
  repo_name: string;
  number: number;
  title: string;
  body: string;
  state: 'open' | 'closed' | 'merged';
  source_repo: string;
  source_branch: string;
  target_branch: string;
  author: string;
  created_at: string;
  updated_at: string;
}

export interface PullRequestComment {
  id: number;
  pr_id: number;
  body: string;
  author: string;
  created_at: string;
}

export interface FileDiff {
  path: string;
  status: 'added' | 'modified' | 'deleted';
  additions: number;
  deletions: number;
  diff: string;
}

// Get API base URL from environment variable or default to relative path
const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || '';

// API helper function
async function api<T>(endpoint: string, options?: RequestInit): Promise<T> {
  const response = await fetch(API_BASE_URL + '/api' + endpoint, {
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

// Fork repository
export async function forkRepo(name: string, newName: string): Promise<RepoInfo> {
  return api<RepoInfo>(`/repos/${encodeURIComponent(name)}/fork`, {
    method: 'POST',
    body: JSON.stringify({ name: newName }),
  });
}

export async function getRepo(name: string): Promise<RepoInfo> {
  return api<RepoInfo>(`/repos/${encodeURIComponent(name)}`);
}

// Issue APIs
export async function listIssues(repoName: string): Promise<Issue[]> {
  return api<Issue[]>(`/repos/${encodeURIComponent(repoName)}/issues`);
}

export async function getIssue(repoName: string, issueNumber: number): Promise<Issue> {
  return api<Issue>(`/repos/${encodeURIComponent(repoName)}/issues/${issueNumber}`);
}

export async function createIssue(
  repoName: string,
  title: string,
  body: string
): Promise<Issue> {
  return api<Issue>(`/repos/${encodeURIComponent(repoName)}/issues`, {
    method: 'POST',
    body: JSON.stringify({ title, body }),
  });
}

export async function updateIssue(
  repoName: string,
  issueNumber: number,
  updates: { title?: string; body?: string; state?: 'open' | 'closed' }
): Promise<Issue> {
  return api<Issue>(`/repos/${encodeURIComponent(repoName)}/issues/${issueNumber}`, {
    method: 'PATCH',
    body: JSON.stringify(updates),
  });
}

export async function getIssueComments(
  repoName: string,
  issueNumber: number
): Promise<IssueComment[]> {
  return api<IssueComment[]>(
    `/repos/${encodeURIComponent(repoName)}/issues/${issueNumber}/comments`
  );
}

export async function createIssueComment(
  repoName: string,
  issueNumber: number,
  body: string
): Promise<IssueComment> {
  return api<IssueComment>(
    `/repos/${encodeURIComponent(repoName)}/issues/${issueNumber}/comments`,
    {
      method: 'POST',
      body: JSON.stringify({ body }),
    }
  );
}

// Pull Request APIs
export async function listPullRequests(repoName: string): Promise<PullRequest[]> {
  return api<PullRequest[]>(`/repos/${encodeURIComponent(repoName)}/pulls`);
}

export async function getPullRequest(repoName: string, prNumber: number): Promise<PullRequest> {
  return api<PullRequest>(`/repos/${encodeURIComponent(repoName)}/pulls/${prNumber}`);
}

export async function createPullRequest(
  repoName: string,
  title: string,
  body: string,
  sourceRepo: string,
  sourceBranch: string,
  targetBranch: string
): Promise<PullRequest> {
  return api<PullRequest>(`/repos/${encodeURIComponent(repoName)}/pulls`, {
    method: 'POST',
    body: JSON.stringify({
      title,
      body,
      source_repo: sourceRepo,
      source_branch: sourceBranch,
      target_branch: targetBranch,
    }),
  });
}

export async function updatePullRequest(
  repoName: string,
  prNumber: number,
  updates: { title?: string; body?: string; state?: 'open' | 'closed' | 'merged' }
): Promise<PullRequest> {
  return api<PullRequest>(`/repos/${encodeURIComponent(repoName)}/pulls/${prNumber}`, {
    method: 'PATCH',
    body: JSON.stringify(updates),
  });
}

export async function getPullRequestComments(
  repoName: string,
  prNumber: number
): Promise<PullRequestComment[]> {
  return api<PullRequestComment[]>(
    `/repos/${encodeURIComponent(repoName)}/pulls/${prNumber}/comments`
  );
}

export async function createPullRequestComment(
  repoName: string,
  prNumber: number,
  body: string
): Promise<PullRequestComment> {
  return api<PullRequestComment>(
    `/repos/${encodeURIComponent(repoName)}/pulls/${prNumber}/comments`,
    {
      method: 'POST',
      body: JSON.stringify({ body }),
    }
  );
}

export async function getPullRequestCommits(
  repoName: string,
  prNumber: number
): Promise<CommitInfo[]> {
  return api<CommitInfo[]>(`/repos/${encodeURIComponent(repoName)}/pulls/${prNumber}/commits`);
}

export async function getPullRequestFiles(
  repoName: string,
  prNumber: number
): Promise<FileDiff[]> {
  return api<FileDiff[]>(`/repos/${encodeURIComponent(repoName)}/pulls/${prNumber}/files`);
}

// Get repository branches
export async function getRepoBranches(repoName: string): Promise<string[]> {
  return api<string[]>(`/repos/${encodeURIComponent(repoName)}/branches`);
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
