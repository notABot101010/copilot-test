import { authHeaderSignal, apiBaseSignal, sessionSignal, type Session } from './state'

export type Organization = { id: number; name: string; ownerId: number }
export type Project = { id: number; name: string; orgId: number; repoPath: string }
export type IssueComment = { id: number; authorId: number; body: string; createdAt: string }
export type Issue = {
  id: number
  projectId: number
  title: string
  description: string
  status: 'open' | 'closed'
  tags: string[]
  comments: IssueComment[]
  createdAt: string
  updatedAt: string
}
export type RepoBranch = { name: string; isDefault: boolean }
export type RepoEntry = { name: string; path: string; type: 'dir' | 'file' }
export type RepoFile = { branch: string; path: string; content: string }
export type MergeRequestComment = { id: number; authorId: number; body: string; createdAt: string }
export type MergeRequest = {
  id: number
  projectId: number
  authorId: number
  title: string
  description: string
  sourceBranch: string
  targetBranch: string
  status: 'open' | 'closed' | 'merged'
  comments: MergeRequestComment[]
  createdAt: string
  updatedAt: string
}

type UserResponse = { id: number; username: string; token: string }

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const headers = new Headers(init?.headers)
  headers.set('Content-Type', 'application/json')
  if (authHeaderSignal.value.Authorization) {
    headers.set('Authorization', authHeaderSignal.value.Authorization)
  }
  const response = await fetch(`${apiBaseSignal.value}${path}`, {
    ...init,
    headers,
  })
  if (!response.ok) {
    const text = await response.text()
    throw new Error(text || 'request failed')
  }
  if (response.status === 204) return undefined as T
  return (await response.json()) as T
}

export async function createUser(username: string): Promise<Session> {
  const data = await request<UserResponse>('/api/users', {
    method: 'POST',
    body: JSON.stringify({ username }),
  })
  const session = { userId: data.id, username: data.username, token: data.token }
  sessionSignal.value = session
  return session
}

export function addSSHKey(userId: number, key: string) {
  return request(`/api/users/${userId}/ssh-keys`, {
    method: 'POST',
    body: JSON.stringify({ key }),
  })
}

export function listOrganizations() {
  return request<Organization[]>('/api/orgs')
}

export function createOrganization(name: string) {
  return request<Organization>('/api/orgs', { method: 'POST', body: JSON.stringify({ name }) })
}

export function listProjects() {
  return request<Project[]>('/api/projects')
}

export function createProject(orgId: number, name: string) {
  return request<Project>(`/api/orgs/${orgId}/projects`, {
    method: 'POST',
    body: JSON.stringify({ name }),
  })
}

export function listRepoBranches(projectId: number) {
  return request<RepoBranch[]>(`/api/projects/${projectId}/repo/branches`)
}

export function listRepoTree(projectId: number, branch: string, path = '') {
  const query = new URLSearchParams({ branch })
  if (path) query.set('path', path)
  return request<RepoEntry[]>(`/api/projects/${projectId}/repo/tree?${query.toString()}`)
}

export function getRepoFile(projectId: number, branch: string, path: string) {
  const query = new URLSearchParams({ branch, path })
  return request<RepoFile>(`/api/projects/${projectId}/repo/file?${query.toString()}`)
}

export function saveRepoFile(projectId: number, payload: { branch: string; path: string; content: string }) {
  return request<RepoFile>(`/api/projects/${projectId}/repo/file`, {
    method: 'PUT',
    body: JSON.stringify(payload),
  })
}

export function deleteRepoFile(projectId: number, payload: { branch: string; path: string }) {
  return request<void>(`/api/projects/${projectId}/repo/file`, {
    method: 'DELETE',
    body: JSON.stringify(payload),
  })
}

export function listIssues(projectId: number) {
  return request<Issue[]>(`/api/projects/${projectId}/issues`)
}

export function createIssue(projectId: number, title: string, description: string, tags: string[] = []) {
  return request<Issue>(`/api/projects/${projectId}/issues`, {
    method: 'POST',
    body: JSON.stringify({ title, description, tags }),
  })
}

export function updateIssue(projectId: number, issueId: number, payload: Partial<Issue>) {
  return request<Issue>(`/api/projects/${projectId}/issues/${issueId}`, {
    method: 'PATCH',
    body: JSON.stringify(payload),
  })
}

export function addIssueComment(projectId: number, issueId: number, body: string) {
  return request<IssueComment>(`/api/projects/${projectId}/issues/${issueId}/comments`, {
    method: 'POST',
    body: JSON.stringify({ body }),
  })
}

export function listMergeRequests(projectId: number) {
  return request<MergeRequest[]>(`/api/projects/${projectId}/merge-requests`)
}

export function getMergeRequest(projectId: number, mergeRequestId: number) {
  return request<MergeRequest>(`/api/projects/${projectId}/merge-requests/${mergeRequestId}`)
}

export function createMergeRequest(
  projectId: number,
  title: string,
  description: string,
  sourceBranch: string,
  targetBranch: string,
) {
  return request<MergeRequest>(`/api/projects/${projectId}/merge-requests`, {
    method: 'POST',
    body: JSON.stringify({ title, description, sourceBranch, targetBranch }),
  })
}

export function getMergeRequestDiff(projectId: number, mergeRequestId: number) {
  return request<{ diff: string }>(`/api/projects/${projectId}/merge-requests/${mergeRequestId}/diff`)
}

export function addMergeRequestComment(projectId: number, mergeRequestId: number, body: string) {
  return request<MergeRequestComment>(`/api/projects/${projectId}/merge-requests/${mergeRequestId}/comments`, {
    method: 'POST',
    body: JSON.stringify({ body }),
  })
}
