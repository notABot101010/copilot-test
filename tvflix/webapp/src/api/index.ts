// API types and client for TVflix

export interface User {
  id: number;
  username: string;
}

export interface AuthResponse {
  token: string;
  user: User;
}

export interface Media {
  id: number;
  user_id: number;
  title: string;
  media_type: 'video' | 'music' | 'photo';
  filename: string;
  storage_path: string;
  thumbnail_path: string | null;
  content_type: string;
  size: number;
  duration: number | null;
  created_at: string;
}

export interface Playlist {
  id: number;
  user_id: number;
  name: string;
  created_at: string;
}

export interface PlaylistWithMedia extends Playlist {
  items: Media[];
}

export interface Album {
  id: number;
  user_id: number;
  name: string;
  created_at: string;
}

export interface AlbumWithMedia extends Album {
  items: Media[];
}

class ApiClient {
  private baseUrl = '/api';
  private token: string | null = null;

  setToken(token: string | null) {
    this.token = token;
    if (token) {
      localStorage.setItem('tvflix_token', token);
    } else {
      localStorage.removeItem('tvflix_token');
    }
  }

  getToken(): string | null {
    if (!this.token) {
      this.token = localStorage.getItem('tvflix_token');
    }
    return this.token;
  }

  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    const token = this.getToken();
    const headers: Record<string, string> = {
      ...options.headers as Record<string, string>,
    };

    if (token) {
      headers['Authorization'] = `Bearer ${token}`;
    }

    if (!(options.body instanceof FormData)) {
      headers['Content-Type'] = 'application/json';
    }

    const response = await fetch(`${this.baseUrl}${endpoint}`, {
      ...options,
      headers,
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({ error: 'Request failed' }));
      throw new Error(errorData.error || `HTTP ${response.status}`);
    }

    if (response.status === 204) {
      return undefined as T;
    }

    return response.json();
  }

  // Auth
  async register(username: string, password: string): Promise<AuthResponse> {
    return this.request('/auth/register', {
      method: 'POST',
      body: JSON.stringify({ username, password }),
    });
  }

  async login(username: string, password: string): Promise<AuthResponse> {
    return this.request('/auth/login', {
      method: 'POST',
      body: JSON.stringify({ username, password }),
    });
  }

  async logout(): Promise<void> {
    await this.request('/auth/logout', { method: 'POST' });
    this.setToken(null);
  }

  async getCurrentUser(): Promise<User> {
    return this.request('/auth/me');
  }

  // Media
  async listMedia(mediaType?: 'video' | 'music' | 'photo'): Promise<Media[]> {
    const params = mediaType ? `?type=${mediaType}` : '';
    return this.request(`/media${params}`);
  }

  async uploadMedia(file: File, title?: string): Promise<Media> {
    const formData = new FormData();
    formData.append('file', file);
    if (title) {
      formData.append('title', title);
    }

    return this.request('/media', {
      method: 'POST',
      body: formData,
    });
  }

  async getMedia(id: number): Promise<Media> {
    return this.request(`/media/${id}`);
  }

  async deleteMedia(id: number): Promise<void> {
    return this.request(`/media/${id}`, { method: 'DELETE' });
  }

  getStreamUrl(id: number): string {
    return `${this.baseUrl}/media/${id}/stream`;
  }

  getThumbnailUrl(id: number): string {
    return `${this.baseUrl}/media/${id}/thumbnail`;
  }

  // Playlists
  async listPlaylists(): Promise<Playlist[]> {
    return this.request('/playlists');
  }

  async createPlaylist(name: string): Promise<Playlist> {
    return this.request('/playlists', {
      method: 'POST',
      body: JSON.stringify({ name }),
    });
  }

  async getPlaylist(id: number): Promise<PlaylistWithMedia> {
    return this.request(`/playlists/${id}`);
  }

  async deletePlaylist(id: number): Promise<void> {
    return this.request(`/playlists/${id}`, { method: 'DELETE' });
  }

  async addToPlaylist(playlistId: number, mediaId: number): Promise<void> {
    return this.request(`/playlists/${playlistId}/items`, {
      method: 'POST',
      body: JSON.stringify({ media_id: mediaId }),
    });
  }

  async removeFromPlaylist(playlistId: number, mediaId: number): Promise<void> {
    return this.request(`/playlists/${playlistId}/items/${mediaId}`, {
      method: 'DELETE',
    });
  }

  // Albums
  async listAlbums(): Promise<Album[]> {
    return this.request('/albums');
  }

  async createAlbum(name: string): Promise<Album> {
    return this.request('/albums', {
      method: 'POST',
      body: JSON.stringify({ name }),
    });
  }

  async getAlbum(id: number): Promise<AlbumWithMedia> {
    return this.request(`/albums/${id}`);
  }

  async deleteAlbum(id: number): Promise<void> {
    return this.request(`/albums/${id}`, { method: 'DELETE' });
  }

  async addToAlbum(albumId: number, mediaId: number): Promise<void> {
    return this.request(`/albums/${albumId}/items`, {
      method: 'POST',
      body: JSON.stringify({ media_id: mediaId }),
    });
  }

  async removeFromAlbum(albumId: number, mediaId: number): Promise<void> {
    return this.request(`/albums/${albumId}/items/${mediaId}`, {
      method: 'DELETE',
    });
  }
}

export const api = new ApiClient();
