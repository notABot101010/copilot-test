// Types for API responses

export interface Organization {
  id: number;
  name: string;
  display_name: string;
  description: string;
  created_at: string;
}

export interface Project {
  id: number;
  name: string;
  org_name: string;
  display_name: string;
  description: string;
  created_at: string;
}

export interface RepoInfo {
  name: string;
  org_name: string;
  project_name: string;
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
  status: 'todo' | 'doing' | 'done';
  start_date: string | null;
  target_date: string | null;
  author: string;
  created_at: string;
  updated_at: string;
}

export interface Tag {
  id: number;
  repo_name: string;
  name: string;
  color: string;
}

export interface IssueComment {
  id: number;
  issue_id: number;
  body: string;
  author: string;
  created_at: string;
  updated_at: string;
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

// Organization APIs
export async function listOrganizations(): Promise<Organization[]> {
  return api<Organization[]>('/orgs');
}

export async function getOrganization(name: string): Promise<Organization> {
  return api<Organization>(`/orgs/${encodeURIComponent(name)}`);
}

export async function createOrganization(
  name: string,
  displayName: string,
  description: string = ''
): Promise<Organization> {
  return api<Organization>('/orgs', {
    method: 'POST',
    body: JSON.stringify({ name, display_name: displayName, description }),
  });
}

export async function updateOrganization(
  name: string,
  updates: { display_name?: string; description?: string }
): Promise<Organization> {
  return api<Organization>(`/orgs/${encodeURIComponent(name)}`, {
    method: 'PATCH',
    body: JSON.stringify(updates),
  });
}

// Project APIs (project now creates a git repo automatically)
export async function listProjects(orgName: string): Promise<Project[]> {
  return api<Project[]>(`/orgs/${encodeURIComponent(orgName)}/projects`);
}

export async function getProject(orgName: string, projectName: string): Promise<Project> {
  return api<Project>(`/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}`);
}

export async function createProject(
  orgName: string,
  name: string,
  displayName: string,
  description: string = ''
): Promise<Project> {
  return api<Project>(`/orgs/${encodeURIComponent(orgName)}/projects`, {
    method: 'POST',
    body: JSON.stringify({ name, display_name: displayName, description }),
  });
}

export async function updateProject(
  orgName: string,
  projectName: string,
  updates: { display_name?: string; description?: string }
): Promise<Project> {
  return api<Project>(`/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}`, {
    method: 'PATCH',
    body: JSON.stringify(updates),
  });
}

// Project Git Operations (project = repository)

export async function getProjectTree(
  orgName: string,
  projectName: string,
  ref: string = 'HEAD',
  path: string = ''
): Promise<FileEntry[]> {
  const params = new URLSearchParams();
  if (ref !== 'HEAD') params.set('ref', ref);
  if (path) params.set('path', path);
  const query = params.toString() ? '?' + params.toString() : '';
  return api<FileEntry[]>(`/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}/tree${query}`);
}

export async function getProjectCommits(orgName: string, projectName: string): Promise<CommitInfo[]> {
  return api<CommitInfo[]>(`/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}/commits`);
}

export async function getProjectBlob(
  orgName: string,
  projectName: string,
  path: string,
  ref: string = 'HEAD'
): Promise<string> {
  const params = new URLSearchParams();
  params.set('path', path);
  if (ref !== 'HEAD') params.set('ref', ref);
  return api<string>(`/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}/blob?${params.toString()}`);
}

export async function updateProjectFile(
  orgName: string,
  projectName: string,
  path: string,
  content: string,
  message: string
): Promise<void> {
  await api(`/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}/files`, {
    method: 'POST',
    body: JSON.stringify({ path, content, message }),
  });
}

export async function deleteProjectFile(
  orgName: string,
  projectName: string,
  path: string,
  message: string
): Promise<void> {
  await api(`/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}/files`, {
    method: 'DELETE',
    body: JSON.stringify({ path, message }),
  });
}

export async function getProjectBranches(orgName: string, projectName: string): Promise<string[]> {
  return api<string[]>(`/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}/branches`);
}

// Fork project
export async function forkProject(
  orgName: string,
  projectName: string,
  newName: string,
  targetOrg?: string
): Promise<RepoInfo> {
  return api<RepoInfo>(`/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}/fork`, {
    method: 'POST',
    body: JSON.stringify({ name: newName, target_org: targetOrg }),
  });
}

// Project Issue APIs
export async function listProjectIssues(orgName: string, projectName: string): Promise<Issue[]> {
  return api<Issue[]>(`/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}/issues`);
}

export async function getProjectIssue(orgName: string, projectName: string, issueNumber: number): Promise<Issue> {
  return api<Issue>(`/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}/issues/${issueNumber}`);
}

export async function createProjectIssue(
  orgName: string,
  projectName: string,
  title: string,
  body: string,
  startDate?: string,
  targetDate?: string
): Promise<Issue> {
  return api<Issue>(`/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}/issues`, {
    method: 'POST',
    body: JSON.stringify({ title, body, start_date: startDate, target_date: targetDate }),
  });
}

export async function updateProjectIssue(
  orgName: string,
  projectName: string,
  issueNumber: number,
  updates: { title?: string; body?: string; state?: 'open' | 'closed'; status?: 'todo' | 'doing' | 'done'; start_date?: string | null; target_date?: string | null }
): Promise<Issue> {
  return api<Issue>(`/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}/issues/${issueNumber}`, {
    method: 'PATCH',
    body: JSON.stringify(updates),
  });
}

export async function getProjectIssueComments(
  orgName: string,
  projectName: string,
  issueNumber: number
): Promise<IssueComment[]> {
  return api<IssueComment[]>(
    `/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}/issues/${issueNumber}/comments`
  );
}

export async function createProjectIssueComment(
  orgName: string,
  projectName: string,
  issueNumber: number,
  body: string
): Promise<IssueComment> {
  return api<IssueComment>(
    `/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}/issues/${issueNumber}/comments`,
    {
      method: 'POST',
      body: JSON.stringify({ body }),
    }
  );
}

export async function updateProjectIssueComment(
  orgName: string,
  projectName: string,
  issueNumber: number,
  commentId: number,
  body: string
): Promise<IssueComment> {
  return api<IssueComment>(
    `/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}/issues/${issueNumber}/comments/${commentId}`,
    {
      method: 'PATCH',
      body: JSON.stringify({ body }),
    }
  );
}

// Project Pull Request APIs
export async function listProjectPullRequests(orgName: string, projectName: string): Promise<PullRequest[]> {
  return api<PullRequest[]>(`/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}/pulls`);
}

export async function getProjectPullRequest(orgName: string, projectName: string, prNumber: number): Promise<PullRequest> {
  return api<PullRequest>(`/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}/pulls/${prNumber}`);
}

export async function createProjectPullRequest(
  orgName: string,
  projectName: string,
  title: string,
  body: string,
  sourceRepo: string,
  sourceBranch: string,
  targetBranch: string
): Promise<PullRequest> {
  return api<PullRequest>(`/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}/pulls`, {
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

export async function updateProjectPullRequest(
  orgName: string,
  projectName: string,
  prNumber: number,
  updates: { title?: string; body?: string; state?: 'open' | 'closed' | 'merged' }
): Promise<PullRequest> {
  return api<PullRequest>(`/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}/pulls/${prNumber}`, {
    method: 'PATCH',
    body: JSON.stringify(updates),
  });
}

export async function getProjectPullRequestComments(
  orgName: string,
  projectName: string,
  prNumber: number
): Promise<PullRequestComment[]> {
  return api<PullRequestComment[]>(
    `/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}/pulls/${prNumber}/comments`
  );
}

export async function createProjectPullRequestComment(
  orgName: string,
  projectName: string,
  prNumber: number,
  body: string
): Promise<PullRequestComment> {
  return api<PullRequestComment>(
    `/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}/pulls/${prNumber}/comments`,
    {
      method: 'POST',
      body: JSON.stringify({ body }),
    }
  );
}

export async function getProjectPullRequestCommits(
  orgName: string,
  projectName: string,
  prNumber: number
): Promise<CommitInfo[]> {
  return api<CommitInfo[]>(`/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}/pulls/${prNumber}/commits`);
}

export async function getProjectPullRequestFiles(
  orgName: string,
  projectName: string,
  prNumber: number
): Promise<FileDiff[]> {
  return api<FileDiff[]>(`/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}/pulls/${prNumber}/files`);
}

// Legacy Repository APIs (kept for backward compatibility)
export async function listRepos(orgName: string, projectName: string): Promise<RepoInfo[]> {
  return api<RepoInfo[]>(`/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}/repos`);
}

export async function createRepo(orgName: string, projectName: string, name: string): Promise<RepoInfo> {
  return api<RepoInfo>(`/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}/repos`, {
    method: 'POST',
    body: JSON.stringify({ name }),
  });
}

export async function getRepo(orgName: string, projectName: string, name: string): Promise<RepoInfo> {
  return api<RepoInfo>(`/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}/repos/${encodeURIComponent(name)}`);
}

export async function getRepoTree(
  orgName: string,
  projectName: string,
  name: string,
  ref: string = 'HEAD',
  path: string = ''
): Promise<FileEntry[]> {
  const params = new URLSearchParams();
  if (ref !== 'HEAD') params.set('ref', ref);
  if (path) params.set('path', path);
  const query = params.toString() ? '?' + params.toString() : '';
  return api<FileEntry[]>(`/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}/repos/${encodeURIComponent(name)}/tree${query}`);
}

export async function getRepoCommits(orgName: string, projectName: string, name: string): Promise<CommitInfo[]> {
  return api<CommitInfo[]>(`/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}/repos/${encodeURIComponent(name)}/commits`);
}

export async function getBlob(
  orgName: string,
  projectName: string,
  name: string,
  path: string,
  ref: string = 'HEAD'
): Promise<string> {
  const params = new URLSearchParams();
  params.set('path', path);
  if (ref !== 'HEAD') params.set('ref', ref);
  return api<string>(`/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}/repos/${encodeURIComponent(name)}/blob?${params.toString()}`);
}

export async function updateFile(
  orgName: string,
  projectName: string,
  repoName: string,
  path: string,
  content: string,
  message: string
): Promise<void> {
  await api(`/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}/repos/${encodeURIComponent(repoName)}/files`, {
    method: 'POST',
    body: JSON.stringify({ path, content, message }),
  });
}

export async function deleteFile(
  orgName: string,
  projectName: string,
  repoName: string,
  path: string,
  message: string
): Promise<void> {
  await api(`/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}/repos/${encodeURIComponent(repoName)}/files`, {
    method: 'DELETE',
    body: JSON.stringify({ path, message }),
  });
}

// Fork repository
export async function forkRepo(
  orgName: string,
  projectName: string,
  name: string,
  newName: string,
  targetOrg?: string,
  targetProject?: string
): Promise<RepoInfo> {
  return api<RepoInfo>(`/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}/repos/${encodeURIComponent(name)}/fork`, {
    method: 'POST',
    body: JSON.stringify({ name: newName, target_org: targetOrg, target_project: targetProject }),
  });
}

// Get repository branches
export async function getRepoBranches(orgName: string, projectName: string, repoName: string): Promise<string[]> {
  return api<string[]>(`/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}/repos/${encodeURIComponent(repoName)}/branches`);
}

// Issue APIs
export async function listIssues(orgName: string, projectName: string, repoName: string): Promise<Issue[]> {
  return api<Issue[]>(`/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}/repos/${encodeURIComponent(repoName)}/issues`);
}

export async function getIssue(orgName: string, projectName: string, repoName: string, issueNumber: number): Promise<Issue> {
  return api<Issue>(`/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}/repos/${encodeURIComponent(repoName)}/issues/${issueNumber}`);
}

export async function createIssue(
  orgName: string,
  projectName: string,
  repoName: string,
  title: string,
  body: string
): Promise<Issue> {
  return api<Issue>(`/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}/repos/${encodeURIComponent(repoName)}/issues`, {
    method: 'POST',
    body: JSON.stringify({ title, body }),
  });
}

export async function updateIssue(
  orgName: string,
  projectName: string,
  repoName: string,
  issueNumber: number,
  updates: { title?: string; body?: string; state?: 'open' | 'closed' }
): Promise<Issue> {
  return api<Issue>(`/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}/repos/${encodeURIComponent(repoName)}/issues/${issueNumber}`, {
    method: 'PATCH',
    body: JSON.stringify(updates),
  });
}

export async function getIssueComments(
  orgName: string,
  projectName: string,
  repoName: string,
  issueNumber: number
): Promise<IssueComment[]> {
  return api<IssueComment[]>(
    `/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}/repos/${encodeURIComponent(repoName)}/issues/${issueNumber}/comments`
  );
}

export async function createIssueComment(
  orgName: string,
  projectName: string,
  repoName: string,
  issueNumber: number,
  body: string
): Promise<IssueComment> {
  return api<IssueComment>(
    `/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}/repos/${encodeURIComponent(repoName)}/issues/${issueNumber}/comments`,
    {
      method: 'POST',
      body: JSON.stringify({ body }),
    }
  );
}

// Pull Request APIs
export async function listPullRequests(orgName: string, projectName: string, repoName: string): Promise<PullRequest[]> {
  return api<PullRequest[]>(`/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}/repos/${encodeURIComponent(repoName)}/pulls`);
}

export async function getPullRequest(orgName: string, projectName: string, repoName: string, prNumber: number): Promise<PullRequest> {
  return api<PullRequest>(`/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}/repos/${encodeURIComponent(repoName)}/pulls/${prNumber}`);
}

export async function createPullRequest(
  orgName: string,
  projectName: string,
  repoName: string,
  title: string,
  body: string,
  sourceRepo: string,
  sourceBranch: string,
  targetBranch: string
): Promise<PullRequest> {
  return api<PullRequest>(`/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}/repos/${encodeURIComponent(repoName)}/pulls`, {
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
  orgName: string,
  projectName: string,
  repoName: string,
  prNumber: number,
  updates: { title?: string; body?: string; state?: 'open' | 'closed' | 'merged' }
): Promise<PullRequest> {
  return api<PullRequest>(`/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}/repos/${encodeURIComponent(repoName)}/pulls/${prNumber}`, {
    method: 'PATCH',
    body: JSON.stringify(updates),
  });
}

export async function getPullRequestComments(
  orgName: string,
  projectName: string,
  repoName: string,
  prNumber: number
): Promise<PullRequestComment[]> {
  return api<PullRequestComment[]>(
    `/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}/repos/${encodeURIComponent(repoName)}/pulls/${prNumber}/comments`
  );
}

export async function createPullRequestComment(
  orgName: string,
  projectName: string,
  repoName: string,
  prNumber: number,
  body: string
): Promise<PullRequestComment> {
  return api<PullRequestComment>(
    `/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}/repos/${encodeURIComponent(repoName)}/pulls/${prNumber}/comments`,
    {
      method: 'POST',
      body: JSON.stringify({ body }),
    }
  );
}

export async function getPullRequestCommits(
  orgName: string,
  projectName: string,
  repoName: string,
  prNumber: number
): Promise<CommitInfo[]> {
  return api<CommitInfo[]>(`/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}/repos/${encodeURIComponent(repoName)}/pulls/${prNumber}/commits`);
}

export async function getPullRequestFiles(
  orgName: string,
  projectName: string,
  repoName: string,
  prNumber: number
): Promise<FileDiff[]> {
  return api<FileDiff[]>(`/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}/repos/${encodeURIComponent(repoName)}/pulls/${prNumber}/files`);
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

// Tag APIs
export async function listProjectTags(orgName: string, projectName: string): Promise<Tag[]> {
  return api<Tag[]>(`/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}/tags`);
}

export async function createProjectTag(
  orgName: string,
  projectName: string,
  name: string,
  color: string = '#6b7280'
): Promise<Tag> {
  return api<Tag>(`/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}/tags`, {
    method: 'POST',
    body: JSON.stringify({ name, color }),
  });
}

export async function updateProjectTag(
  orgName: string,
  projectName: string,
  tagId: number,
  updates: { name?: string; color?: string }
): Promise<Tag> {
  return api<Tag>(`/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}/tags/${tagId}`, {
    method: 'PATCH',
    body: JSON.stringify(updates),
  });
}

export async function deleteProjectTag(
  orgName: string,
  projectName: string,
  tagId: number
): Promise<void> {
  await api(`/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}/tags/${tagId}`, {
    method: 'DELETE',
  });
}

export async function getIssueTags(
  orgName: string,
  projectName: string,
  issueNumber: number
): Promise<Tag[]> {
  return api<Tag[]>(
    `/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}/issues/${issueNumber}/tags`
  );
}

export async function addTagToIssue(
  orgName: string,
  projectName: string,
  issueNumber: number,
  tagId: number
): Promise<void> {
  await api(
    `/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}/issues/${issueNumber}/tags`,
    {
      method: 'POST',
      body: JSON.stringify({ tag_id: tagId }),
    }
  );
}

export async function removeTagFromIssue(
  orgName: string,
  projectName: string,
  issueNumber: number,
  tagId: number
): Promise<void> {
  await api(
    `/orgs/${encodeURIComponent(orgName)}/projects/${encodeURIComponent(projectName)}/issues/${issueNumber}/tags/${tagId}`,
    {
      method: 'DELETE',
    }
  );
}
