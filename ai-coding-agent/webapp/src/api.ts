import type { Session, Message, PromptTemplate, SteerCommand } from './types';

const API_BASE = '/api';

export async function createSession(name?: string): Promise<Session> {
  const response = await fetch(`${API_BASE}/sessions`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ name }),
  });
  if (!response.ok) throw new Error('Failed to create session');
  return response.json();
}

export async function listSessions(): Promise<Session[]> {
  const response = await fetch(`${API_BASE}/sessions`);
  if (!response.ok) throw new Error('Failed to list sessions');
  return response.json();
}

export async function getSession(id: string): Promise<Session> {
  const response = await fetch(`${API_BASE}/sessions/${id}`);
  if (!response.ok) throw new Error('Failed to get session');
  return response.json();
}

export async function sendMessage(sessionId: string, content: string): Promise<Message> {
  const response = await fetch(`${API_BASE}/sessions/${sessionId}/messages`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ content }),
  });
  if (!response.ok) throw new Error('Failed to send message');
  return response.json();
}

export async function getMessages(sessionId: string): Promise<Message[]> {
  const response = await fetch(`${API_BASE}/sessions/${sessionId}/messages`);
  if (!response.ok) throw new Error('Failed to get messages');
  return response.json();
}

export async function steerSession(sessionId: string, command: SteerCommand): Promise<void> {
  const response = await fetch(`${API_BASE}/sessions/${sessionId}/steer`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ command }),
  });
  if (!response.ok) throw new Error('Failed to steer session');
}

export async function listTemplates(): Promise<PromptTemplate[]> {
  const response = await fetch(`${API_BASE}/templates`);
  if (!response.ok) throw new Error('Failed to list templates');
  return response.json();
}

export async function updateTemplate(id: string, systemPrompt: string): Promise<PromptTemplate> {
  const response = await fetch(`${API_BASE}/templates/${id}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ system_prompt: systemPrompt }),
  });
  if (!response.ok) throw new Error('Failed to update template');
  return response.json();
}

export function createSessionStream(sessionId: string): WebSocket {
  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  const host = window.location.host;
  return new WebSocket(`${protocol}//${host}/api/sessions/${sessionId}/stream`);
}
