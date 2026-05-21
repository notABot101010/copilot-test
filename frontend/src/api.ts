import { authHeaderSignal, apiBaseSignal, sessionSignal, type Session } from './state'

export type User = { id: number; username: string; sshKeys?: string[] }
export type Organization = { id: number; name: string; ownerId: number }
export type OrganizationMember = { userId: number; role: 'owner' | 'admin' | 'developer' | 'viewer' }
export type Project = {
  id: number
  name: string
  orgId: number
  repoPath: string
  description: string
  defaultBranch: string
  archived: boolean
}
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
export type RepoTag = { name: string; target: string; createdAt: string }
export type RepoEntry = { name: string; path: string; type: 'dir' | 'file' }
export type RepoFile = { branch: string; path: string; content: string }
export type RepoCommit = {
  hash: string
  shortHash: string
  authorName: string
  authorEmail: string
  subject: string
  body: string
  parents: string[]
  authoredAt: string
}
export type RepoCommitDetails = RepoCommit & { diff: string }
export type RepoBlameLine = {
  lineNumber: number
  commitHash: string
  authorName: string
  authorEmail: string
  summary: string
  committedAt: string
  content: string
}
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
  mergeable: boolean
  hasConflicts: boolean
  alreadyMerged: boolean
  mergedBy?: number
  mergedAt?: string
  mergedCommitId?: string
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

export function listUsers() {
  return request<User[]>('/api/users')
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

export function listOrganizationMembers(orgId: number) {
  return request<OrganizationMember[]>(`/api/orgs/${orgId}/members`)
}

export function addOrganizationMember(orgId: number, userId: number, role: OrganizationMember['role']) {
  return request<OrganizationMember>(`/api/orgs/${orgId}/members`, {
    method: 'POST',
    body: JSON.stringify({ userId, role }),
  })
}

export function updateOrganizationMember(orgId: number, userId: number, role: OrganizationMember['role']) {
  return request<OrganizationMember>(`/api/orgs/${orgId}/members/${userId}`, {
    method: 'PATCH',
    body: JSON.stringify({ role }),
  })
}

export function removeOrganizationMember(orgId: number, userId: number) {
  return request<void>(`/api/orgs/${orgId}/members/${userId}`, {
    method: 'DELETE',
  })
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

export function getProjectSettings(projectId: number) {
  return request<Project>(`/api/projects/${projectId}/settings`)
}

export function updateProjectSettings(
  projectId: number,
  payload: Partial<Pick<Project, 'description' | 'defaultBranch' | 'archived'>>,
) {
  return request<Project>(`/api/projects/${projectId}/settings`, {
    method: 'PATCH',
    body: JSON.stringify(payload),
  })
}

export function listRepoBranches(projectId: number) {
  return request<RepoBranch[]>(`/api/projects/${projectId}/repo/branches`)
}

export function createRepoBranch(projectId: number, name: string, sourceBranch: string) {
  return request<RepoBranch>(`/api/projects/${projectId}/repo/branches`, {
    method: 'POST',
    body: JSON.stringify({ name, sourceBranch }),
  })
}

export function deleteRepoBranch(projectId: number, branchName: string) {
  return request<void>(`/api/projects/${projectId}/repo/branches/${encodeURIComponent(branchName)}`, {
    method: 'DELETE',
  })
}

export function listRepoTags(projectId: number) {
  return request<RepoTag[]>(`/api/projects/${projectId}/repo/tags`)
}

export function createRepoTag(projectId: number, name: string, target: string) {
  return request<RepoTag>(`/api/projects/${projectId}/repo/tags`, {
    method: 'POST',
    body: JSON.stringify({ name, target }),
  })
}

export function listRepoCommits(projectId: number, branch: string, path = '', limit = 20) {
  const query = new URLSearchParams({ branch, limit: String(limit) })
  if (path) query.set('path', path)
  return request<RepoCommit[]>(`/api/projects/${projectId}/repo/commits?${query.toString()}`)
}

export function getRepoCommit(projectId: number, commitHash: string) {
  return request<RepoCommitDetails>(`/api/projects/${projectId}/repo/commits/${encodeURIComponent(commitHash)}`)
}

export function getRepoBlame(projectId: number, branch: string, path: string) {
  const query = new URLSearchParams({ branch, path })
  return request<RepoBlameLine[]>(`/api/projects/${projectId}/repo/blame?${query.toString()}`)
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

export function getMergeRequestMergeStatus(projectId: number, mergeRequestId: number) {
  return request<{ mergeable: boolean; hasConflicts: boolean; alreadyMerged: boolean }>(
    `/api/projects/${projectId}/merge-requests/${mergeRequestId}/merge-status`,
  )
}

export function mergeMergeRequest(projectId: number, mergeRequestId: number) {
  return request<MergeRequest>(`/api/projects/${projectId}/merge-requests/${mergeRequestId}/merge`, {
    method: 'POST',
  })
}

export function addMergeRequestComment(projectId: number, mergeRequestId: number, body: string) {
  return request<MergeRequestComment>(`/api/projects/${projectId}/merge-requests/${mergeRequestId}/comments`, {
    method: 'POST',
    body: JSON.stringify({ body }),
  })
}
