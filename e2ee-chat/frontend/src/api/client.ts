const API_BASE = '/api';

export interface RegisterResponse {
  success: boolean;
  username: string;
}

export interface LoginResponse {
  success: boolean;
  encrypted_identity_key: string;
  identity_public_key: string;
}

export interface SendMessageResponse {
  success: boolean;
  message_id: number;
}

export interface Message {
  id: number;
  from_user: string;
  to_user: string;
  encrypted_content: string;
  ephemeral_public_key: string;
  sender_identity_key: string | null;
  sender_signature: string | null;
  message_number: number;
  previous_chain_length: number;
  created_at: string;
}

export interface UserKeysResponse {
  identity_public_key: string;
  prekey_signature: string;
}

export async function register(
  username: string,
  password: string,
  encryptedIdentityKey: string,
  identityPublicKey: string,
  prekeySignature: string
): Promise<RegisterResponse> {
  const response = await fetch(`${API_BASE}/auth/register`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      username,
      password,
      encrypted_identity_key: encryptedIdentityKey,
      identity_public_key: identityPublicKey,
      prekey_signature: prekeySignature,
    }),
  });

  if (!response.ok) {
    const error = await response.json();
    throw new Error(error.error || 'Registration failed');
  }

  return response.json();
}

export async function login(username: string, password: string): Promise<LoginResponse> {
  const response = await fetch(`${API_BASE}/auth/login`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ username, password }),
  });

  if (!response.ok) {
    const error = await response.json();
    throw new Error(error.error || 'Login failed');
  }

  return response.json();
}

export async function sendMessage(
  fromUser: string,
  toUser: string,
  encryptedContent: string,
  ephemeralPublicKey: string,
  senderIdentityKey: string | null,
  senderSignature: string | null,
  messageNumber: number,
  previousChainLength: number
): Promise<SendMessageResponse> {
  const response = await fetch(`${API_BASE}/messages/send`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      from_user: fromUser,
      to_user: toUser,
      encrypted_content: encryptedContent,
      ephemeral_public_key: ephemeralPublicKey,
      sender_identity_key: senderIdentityKey,
      sender_signature: senderSignature,
      message_number: messageNumber,
      previous_chain_length: previousChainLength,
    }),
  });

  if (!response.ok) {
    const error = await response.json();
    throw new Error(error.error || 'Failed to send message');
  }

  return response.json();
}

export async function pollMessages(username: string): Promise<Message[]> {
  const response = await fetch(`${API_BASE}/messages/poll?username=${encodeURIComponent(username)}`);

  if (!response.ok) {
    throw new Error('Failed to poll messages');
  }

  return response.json();
}

export async function getMessages(username: string, currentUser: string): Promise<Message[]> {
  const response = await fetch(
    `${API_BASE}/messages/${encodeURIComponent(username)}?current_user=${encodeURIComponent(currentUser)}`
  );

  if (!response.ok) {
    throw new Error('Failed to get messages');
  }

  return response.json();
}

export async function getUserKeys(username: string): Promise<UserKeysResponse> {
  const response = await fetch(`${API_BASE}/users/${encodeURIComponent(username)}/keys`);

  if (!response.ok) {
    const error = await response.json();
    throw new Error(error.error || 'Failed to get user keys');
  }

  return response.json();
}
