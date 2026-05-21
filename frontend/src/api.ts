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
  comments: IssueComment[]
  createdAt: string
  updatedAt: string
}

type UserResponse = { id: number; username: string; token: string }

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(`${apiBaseSignal.value}${path}`, {
    ...init,
    headers: {
      'Content-Type': 'application/json',
      ...authHeaderSignal.value,
      ...(init?.headers ?? {}),
    },
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

export function listIssues(projectId: number) {
  return request<Issue[]>(`/api/projects/${projectId}/issues`)
}

export function createIssue(projectId: number, title: string, description: string) {
  return request<Issue>(`/api/projects/${projectId}/issues`, {
    method: 'POST',
    body: JSON.stringify({ title, description }),
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
