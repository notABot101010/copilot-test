/**
 * API service for encrypted chat server communication
 */

const API_BASE = '/api';

export interface RegisterRequest {
  username: string;
  identity_public_key: string;
  salt: string;
  encrypted_identity_private_key: string;
  identity_key_iv: string;
}

export interface RegisterResponse {
  username: string;
  created_at: number;
}

export interface LoginRequest {
  username: string;
}

export interface LoginResponse {
  username: string;
  identity_public_key: string;
  salt: string;
  encrypted_identity_private_key: string;
  identity_key_iv: string;
}

export interface UploadPreKeyBundleRequest {
  signed_prekey_public: string;
  signed_prekey_signature: string;
  encrypted_signed_prekey_private: string;
  signed_prekey_iv: string;
  one_time_prekeys: string[];
  encrypted_one_time_prekey_privates: string[];
  one_time_prekey_ivs: string[];
}

export interface PreKeyBundleResponse {
  identity_public_key: string;
  signed_prekey_public: string;
  signed_prekey_signature: string;
  one_time_prekey?: string;
}

export interface MyPreKeysResponse {
  encrypted_signed_prekey_private: string;
  signed_prekey_iv: string;
}

export interface SendMessageRequest {
  recipient_username: string;
  sealed_sender_envelope: string;
}

export interface SendMessageResponse {
  id: string;
  created_at: number;
}

export interface EncryptedMessage {
  id: string;
  sealed_sender_envelope: string;
  created_at: number;
}

export interface PollMessagesResponse {
  messages: EncryptedMessage[];
  timestamp: number;
}

export interface AckMessagesRequest {
  message_ids: string[];
}

class ApiError extends Error {
  status: number;
  
  constructor(status: number, message: string) {
    super(message);
    this.name = 'ApiError';
    this.status = status;
  }
}

async function handleResponse<T>(response: Response): Promise<T> {
  if (!response.ok) {
    const text = await response.text().catch(() => 'Unknown error');
    throw new ApiError(response.status, text);
  }
  
  if (response.status === 204 || response.headers.get('content-length') === '0') {
    return undefined as T;
  }
  
  return response.json();
}

export const api = {
  async register(data: RegisterRequest): Promise<RegisterResponse> {
    const response = await fetch(`${API_BASE}/register`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data)
    });
    return handleResponse<RegisterResponse>(response);
  },
  
  async login(data: LoginRequest): Promise<LoginResponse> {
    const response = await fetch(`${API_BASE}/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data)
    });
    return handleResponse<LoginResponse>(response);
  },
  
  async listUsers(): Promise<string[]> {
    const response = await fetch(`${API_BASE}/users`);
    return handleResponse<string[]>(response);
  },
  
  async uploadPreKeyBundle(username: string, data: UploadPreKeyBundleRequest): Promise<void> {
    const response = await fetch(`${API_BASE}/users/${encodeURIComponent(username)}/prekeys`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data)
    });
    await handleResponse<void>(response);
  },
  
  async getPreKeyBundle(username: string): Promise<PreKeyBundleResponse> {
    const response = await fetch(`${API_BASE}/users/${encodeURIComponent(username)}/prekeys`);
    return handleResponse<PreKeyBundleResponse>(response);
  },
  
  async getMyPreKeys(username: string): Promise<MyPreKeysResponse> {
    const response = await fetch(`${API_BASE}/users/${encodeURIComponent(username)}/myprekeys`);
    return handleResponse<MyPreKeysResponse>(response);
  },
  
  async sendMessage(senderUsername: string, data: SendMessageRequest): Promise<SendMessageResponse> {
    const response = await fetch(`${API_BASE}/users/${encodeURIComponent(senderUsername)}/messages`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data)
    });
    return handleResponse<SendMessageResponse>(response);
  },
  
  async pollMessages(username: string, timeoutSecs: number = 25): Promise<PollMessagesResponse> {
    const response = await fetch(
      `${API_BASE}/users/${encodeURIComponent(username)}/messages/poll?timeout_secs=${timeoutSecs}`
    );
    return handleResponse<PollMessagesResponse>(response);
  },
  
  async ackMessages(username: string, data: AckMessagesRequest): Promise<void> {
    const response = await fetch(`${API_BASE}/users/${encodeURIComponent(username)}/messages/ack`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data)
    });
    await handleResponse<void>(response);
  }
};

export { ApiError };
