import type { Workspace, Contact, Conversation, Message, Analytics } from '../types';

const API_BASE = '/api';

async function handleResponse<T>(response: Response): Promise<T> {
  if (!response.ok) {
    const text = await response.text();
    throw new Error(`API Error: ${response.status} ${text}`);
  }
  return response.json();
}

// Workspaces
export async function listWorkspaces(): Promise<Workspace[]> {
  const response = await fetch(`${API_BASE}/workspaces`);
  return handleResponse<Workspace[]>(response);
}

export async function createWorkspace(name: string): Promise<Workspace> {
  const response = await fetch(`${API_BASE}/workspaces`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ name }),
  });
  return handleResponse<Workspace>(response);
}

export async function getWorkspace(workspaceId: string): Promise<Workspace> {
  const response = await fetch(`${API_BASE}/workspaces/${workspaceId}`);
  return handleResponse<Workspace>(response);
}

// Contacts
export async function listContacts(workspaceId: string): Promise<Contact[]> {
  const response = await fetch(`${API_BASE}/workspaces/${workspaceId}/contacts`);
  return handleResponse<Contact[]>(response);
}

export async function getContact(workspaceId: string, contactId: string): Promise<Contact> {
  const response = await fetch(`${API_BASE}/workspaces/${workspaceId}/contacts/${contactId}`);
  return handleResponse<Contact>(response);
}

export async function updateContact(
  workspaceId: string,
  contactId: string,
  data: { name?: string; email?: string }
): Promise<Contact> {
  const response = await fetch(`${API_BASE}/workspaces/${workspaceId}/contacts/${contactId}`, {
    method: 'PATCH',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  });
  return handleResponse<Contact>(response);
}

export async function getContactConversations(
  workspaceId: string,
  contactId: string
): Promise<Conversation[]> {
  const response = await fetch(
    `${API_BASE}/workspaces/${workspaceId}/contacts/${contactId}/conversations`
  );
  return handleResponse<Conversation[]>(response);
}

// Conversations
export async function listConversations(workspaceId: string): Promise<Conversation[]> {
  const response = await fetch(`${API_BASE}/workspaces/${workspaceId}/conversations`);
  return handleResponse<Conversation[]>(response);
}

export async function getConversation(
  workspaceId: string,
  conversationId: string
): Promise<Conversation> {
  const response = await fetch(
    `${API_BASE}/workspaces/${workspaceId}/conversations/${conversationId}`
  );
  return handleResponse<Conversation>(response);
}

export async function updateConversation(
  workspaceId: string,
  conversationId: string,
  status: string
): Promise<Conversation> {
  const response = await fetch(
    `${API_BASE}/workspaces/${workspaceId}/conversations/${conversationId}`,
    {
      method: 'PATCH',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ status }),
    }
  );
  return handleResponse<Conversation>(response);
}

// Messages
export async function listMessages(
  workspaceId: string,
  conversationId: string
): Promise<Message[]> {
  const response = await fetch(
    `${API_BASE}/workspaces/${workspaceId}/conversations/${conversationId}/messages`
  );
  return handleResponse<Message[]>(response);
}

export async function sendMessage(
  workspaceId: string,
  conversationId: string,
  content: string,
  senderType: 'agent' | 'visitor' = 'agent',
  senderId: string = 'agent-1'
): Promise<Message> {
  const response = await fetch(
    `${API_BASE}/workspaces/${workspaceId}/conversations/${conversationId}/messages`,
    {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        sender_type: senderType,
        sender_id: senderId,
        content,
      }),
    }
  );
  return handleResponse<Message>(response);
}

// Analytics
export async function getAnalytics(workspaceId: string, days: number = 30): Promise<Analytics> {
  const response = await fetch(
    `${API_BASE}/workspaces/${workspaceId}/analytics?days=${days}`
  );
  return handleResponse<Analytics>(response);
}

// WebSocket connection for real-time updates
export function createWebSocket(workspaceId: string): WebSocket {
  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  const host = window.location.host;
  return new WebSocket(`${protocol}//${host}/ws/workspaces/${workspaceId}`);
}
