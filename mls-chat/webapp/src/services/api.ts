// API service for MLS Chat

const API_BASE = '/api';

async function request<T>(
  path: string,
  options: RequestInit = {}
): Promise<T> {
  const response = await fetch(`${API_BASE}${path}`, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      ...options.headers,
    },
  });

  if (!response.ok) {
    const error = await response.json().catch(() => ({ error: 'Unknown error' }));
    throw new Error(error.error || `HTTP error ${response.status}`);
  }

  return response.json();
}

// Auth
export interface AuthResponse {
  success: boolean;
  user_id?: number;
  username: string;
  error?: string;
}

export async function register(username: string, password: string): Promise<AuthResponse> {
  return request<AuthResponse>('/auth/register', {
    method: 'POST',
    body: JSON.stringify({ username, password }),
  });
}

export async function login(username: string, password: string): Promise<AuthResponse> {
  return request<AuthResponse>('/auth/login', {
    method: 'POST',
    body: JSON.stringify({ username, password }),
  });
}

// Key packages
export async function uploadKeyPackages(username: string, keyPackages: string[]): Promise<void> {
  await request('/key-packages', {
    method: 'POST',
    body: JSON.stringify({ username, key_packages: keyPackages }),
  });
}

export interface KeyPackageResponse {
  key_package: string;
}

export async function getKeyPackage(username: string): Promise<KeyPackageResponse> {
  return request<KeyPackageResponse>(`/key-packages/${encodeURIComponent(username)}`);
}

// Groups
export interface GroupInfo {
  group_id: string;
  name: string;
  is_channel: boolean;
  is_admin: boolean;
  member_count: number;
}

export interface CreateGroupResponse {
  success: boolean;
  group_id: string;
}

export async function createGroup(
  username: string,
  name: string,
  isChannel: boolean
): Promise<CreateGroupResponse> {
  return request<CreateGroupResponse>('/groups', {
    method: 'POST',
    body: JSON.stringify({ username, name, is_channel: isChannel }),
  });
}

export async function listGroups(username: string): Promise<GroupInfo[]> {
  return request<GroupInfo[]>(`/groups?username=${encodeURIComponent(username)}`);
}

export async function getGroup(groupId: string, username: string): Promise<GroupInfo> {
  return request<GroupInfo>(
    `/groups/${encodeURIComponent(groupId)}?username=${encodeURIComponent(username)}`
  );
}

export async function inviteMember(
  groupId: string,
  username: string,
  inviteUsername: string,
  welcomeData: string,
  commitData: string
): Promise<void> {
  await request(`/groups/${encodeURIComponent(groupId)}/invite`, {
    method: 'POST',
    body: JSON.stringify({
      username,
      invite_username: inviteUsername,
      welcome_data: welcomeData,
      commit_data: commitData,
    }),
  });
}

export async function joinGroup(groupId: string, username: string): Promise<void> {
  await request(`/groups/${encodeURIComponent(groupId)}/join`, {
    method: 'POST',
    body: JSON.stringify({ username }),
  });
}

export interface GroupMessage {
  id: number;
  message_type: string;
  message_data: string;
  sender_name?: string;
  created_at: string;
}

export async function sendMessage(
  groupId: string,
  username: string,
  messageData: string,
  messageType: string
): Promise<void> {
  await request(`/groups/${encodeURIComponent(groupId)}/message`, {
    method: 'POST',
    body: JSON.stringify({
      username,
      message_data: messageData,
      message_type: messageType,
    }),
  });
}

export async function getMessages(
  groupId: string,
  username: string,
  sinceId?: number
): Promise<GroupMessage[]> {
  let url = `/groups/${encodeURIComponent(groupId)}/messages?username=${encodeURIComponent(username)}`;
  if (sinceId !== undefined) {
    url += `&since_id=${sinceId}`;
  }
  return request<GroupMessage[]>(url);
}

// Channels
export async function listChannels(): Promise<GroupInfo[]> {
  return request<GroupInfo[]>('/channels');
}

export async function subscribeChannel(groupId: string, username: string): Promise<void> {
  await request(`/channels/${encodeURIComponent(groupId)}/subscribe`, {
    method: 'POST',
    body: JSON.stringify({ username }),
  });
}

// Welcomes
export interface PendingWelcome {
  id: number;
  group_id: string;
  group_name: string;
  welcome_data: string;
  group_info_data?: string;
  inviter_name: string;
}

export async function getPendingWelcomes(username: string): Promise<PendingWelcome[]> {
  return request<PendingWelcome[]>(`/welcomes?username=${encodeURIComponent(username)}`);
}

export async function deleteWelcome(welcomeId: number): Promise<void> {
  await request(`/welcomes/${welcomeId}`, { method: 'DELETE' });
}

// Long polling
export interface PendingMessage {
  id: number;
  group_id: string;
  message_type: string;
  message_data: string;
  sender_name?: string;
}

export interface PollResponse {
  welcomes: PendingWelcome[];
  messages: PendingMessage[];
}

export async function poll(username: string): Promise<PollResponse> {
  return request<PollResponse>(`/poll?username=${encodeURIComponent(username)}`);
}

// Users
export interface UserInfo {
  id: number;
  username: string;
}

export async function listUsers(exclude: string): Promise<UserInfo[]> {
  return request<UserInfo[]>(`/users?exclude=${encodeURIComponent(exclude)}`);
}
