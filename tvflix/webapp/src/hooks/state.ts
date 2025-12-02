import { signal, computed } from '@preact/signals';
import { api } from '../api';
import type { User, Media, Playlist, Album } from '../api';

// Auth state
export const user = signal<User | null>(null);
export const isAuthenticated = computed(() => user.value !== null);
export const authLoading = signal(true);

// Initialize auth state
export async function initAuth(): Promise<void> {
  const token = api.getToken();
  if (token) {
    try {
      const userData = await api.getCurrentUser();
      user.value = userData;
    } catch {
      api.setToken(null);
    }
  }
  authLoading.value = false;
}

export async function login(username: string, password: string): Promise<void> {
  const response = await api.login(username, password);
  api.setToken(response.token);
  user.value = response.user;
}

export async function register(username: string, password: string): Promise<void> {
  const response = await api.register(username, password);
  api.setToken(response.token);
  user.value = response.user;
}

export async function logout(): Promise<void> {
  await api.logout();
  user.value = null;
}

// Media state
export const mediaList = signal<Media[]>([]);
export const mediaLoading = signal(false);

export async function loadMedia(mediaType?: 'video' | 'music' | 'photo'): Promise<void> {
  mediaLoading.value = true;
  try {
    const media = await api.listMedia(mediaType);
    mediaList.value = media;
  } finally {
    mediaLoading.value = false;
  }
}

export async function uploadMedia(file: File, title?: string): Promise<Media> {
  const media = await api.uploadMedia(file, title);
  mediaList.value = [media, ...mediaList.value];
  return media;
}

export async function deleteMedia(id: number): Promise<void> {
  await api.deleteMedia(id);
  mediaList.value = mediaList.value.filter(m => m.id !== id);
}

// Playlist state
export const playlists = signal<Playlist[]>([]);
export const playlistsLoading = signal(false);

export async function loadPlaylists(): Promise<void> {
  playlistsLoading.value = true;
  try {
    const data = await api.listPlaylists();
    playlists.value = data;
  } finally {
    playlistsLoading.value = false;
  }
}

export async function createPlaylist(name: string): Promise<Playlist> {
  const playlist = await api.createPlaylist(name);
  playlists.value = [playlist, ...playlists.value];
  return playlist;
}

export async function deletePlaylist(id: number): Promise<void> {
  await api.deletePlaylist(id);
  playlists.value = playlists.value.filter(p => p.id !== id);
}

// Album state
export const albums = signal<Album[]>([]);
export const albumsLoading = signal(false);

export async function loadAlbums(): Promise<void> {
  albumsLoading.value = true;
  try {
    const data = await api.listAlbums();
    albums.value = data;
  } finally {
    albumsLoading.value = false;
  }
}

export async function createAlbum(name: string): Promise<Album> {
  const album = await api.createAlbum(name);
  albums.value = [album, ...albums.value];
  return album;
}

export async function deleteAlbum(id: number): Promise<void> {
  await api.deleteAlbum(id);
  albums.value = albums.value.filter(a => a.id !== id);
}

// Music player state
export const currentTrack = signal<Media | null>(null);
export const isPlaying = signal(false);
export const currentPlaylist = signal<Media[]>([]);
export const currentTrackIndex = signal(0);

export function playTrack(track: Media, playlist?: Media[]): void {
  currentTrack.value = track;
  isPlaying.value = true;
  if (playlist) {
    currentPlaylist.value = playlist;
    currentTrackIndex.value = playlist.findIndex(t => t.id === track.id);
  } else {
    currentPlaylist.value = [track];
    currentTrackIndex.value = 0;
  }
}

export function playNext(): void {
  const playlist = currentPlaylist.value;
  const idx = currentTrackIndex.value;
  if (idx < playlist.length - 1) {
    currentTrackIndex.value = idx + 1;
    currentTrack.value = playlist[idx + 1];
  }
}

export function playPrevious(): void {
  const idx = currentTrackIndex.value;
  if (idx > 0) {
    currentTrackIndex.value = idx - 1;
    currentTrack.value = currentPlaylist.value[idx - 1];
  }
}

export function togglePlay(): void {
  isPlaying.value = !isPlaying.value;
}
