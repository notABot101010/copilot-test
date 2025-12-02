import { useEffect } from 'preact/hooks';
import { useSignal } from '@preact/signals';
import type { PlaylistWithMedia } from '../api';
import { api } from '../api';
import {
  loadMedia,
  mediaList,
  mediaLoading,
  loadPlaylists,
  playlists,
  createPlaylist,
  deletePlaylist,
  playTrack,
} from '../hooks/state';
import { MediaCard } from '../components/MediaCard';
import { UploadButton } from '../components/UploadButton';
import { MusicPlayer } from '../components/MusicPlayer';

export function MusicPage() {
  const showPlaylists = useSignal(false);
  const selectedPlaylist = useSignal<PlaylistWithMedia | null>(null);
  const newPlaylistName = useSignal('');
  const isCreatingPlaylist = useSignal(false);

  useEffect(() => {
    loadMedia('music');
    loadPlaylists();
  }, []);

  const music = mediaList.value.filter(m => m.media_type === 'music');

  const handleCreatePlaylist = async (event: Event) => {
    event.preventDefault();
    if (!newPlaylistName.value.trim()) return;

    isCreatingPlaylist.value = true;
    try {
      await createPlaylist(newPlaylistName.value);
      newPlaylistName.value = '';
    } finally {
      isCreatingPlaylist.value = false;
    }
  };

  const handleLoadPlaylist = async (id: number) => {
    const playlist = await api.getPlaylist(id);
    selectedPlaylist.value = playlist;
  };

  const handlePlayAllMusic = () => {
    if (music.length > 0) {
      playTrack(music[0], music);
    }
  };

  return (
    <div class="p-6 pb-32">
      {/* Header */}
      <div class="flex items-center justify-between mb-8">
        <div>
          <h1 class="text-3xl font-bold text-white">Music</h1>
          <p class="text-neutral-400 mt-1">Your music library</p>
        </div>
        <div class="flex items-center gap-4">
          <button
            onClick={() => showPlaylists.value = !showPlaylists.value}
            class={`px-4 py-2 rounded-lg transition-colors ${
              showPlaylists.value
                ? 'bg-red-600 text-white'
                : 'bg-neutral-700 text-neutral-300 hover:bg-neutral-600'
            }`}
          >
            Playlists
          </button>
          {music.length > 0 && (
            <button
              onClick={handlePlayAllMusic}
              class="px-4 py-2 bg-neutral-700 hover:bg-neutral-600 text-white rounded-lg transition-colors"
            >
              ▶ Play All
            </button>
          )}
          <UploadButton accept="audio/*" mediaType="music" />
        </div>
      </div>

      {/* Playlists sidebar */}
      {showPlaylists.value && (
        <div class="mb-8 bg-neutral-800 rounded-lg p-4">
          <h2 class="text-lg font-medium text-white mb-4">Playlists</h2>

          {/* Create playlist form */}
          <form onSubmit={handleCreatePlaylist} class="flex gap-2 mb-4">
            <input
              type="text"
              value={newPlaylistName.value}
              onInput={(event) => newPlaylistName.value = (event.target as HTMLInputElement).value}
              placeholder="New playlist name"
              class="flex-1 px-3 py-2 bg-neutral-700 rounded text-white placeholder-neutral-400 focus:outline-none focus:ring-2 focus:ring-red-600"
            />
            <button
              type="submit"
              disabled={isCreatingPlaylist.value}
              class="px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded transition-colors disabled:opacity-50"
            >
              Create
            </button>
          </form>

          {/* Playlist list */}
          <div class="space-y-2">
            {playlists.value.map(playlist => (
              <div
                key={playlist.id}
                class="flex items-center justify-between p-3 bg-neutral-700 rounded hover:bg-neutral-600 cursor-pointer"
                onClick={() => handleLoadPlaylist(playlist.id)}
              >
                <span class="text-white">{playlist.name}</span>
                <button
                  onClick={(event) => {
                    event.stopPropagation();
                    deletePlaylist(playlist.id);
                  }}
                  class="text-neutral-400 hover:text-red-500"
                >
                  ×
                </button>
              </div>
            ))}
            {playlists.value.length === 0 && (
              <p class="text-neutral-400 text-sm">No playlists yet</p>
            )}
          </div>

          {/* Selected playlist */}
          {selectedPlaylist.value && (
            <div class="mt-4 pt-4 border-t border-neutral-700">
              <div class="flex items-center justify-between mb-4">
                <h3 class="text-lg font-medium text-white">{selectedPlaylist.value.name}</h3>
                <button
                  onClick={() => {
                    if (selectedPlaylist.value && selectedPlaylist.value.items.length > 0) {
                      playTrack(selectedPlaylist.value.items[0], selectedPlaylist.value.items);
                    }
                  }}
                  class="px-3 py-1 bg-red-600 hover:bg-red-700 text-white rounded text-sm"
                >
                  ▶ Play
                </button>
              </div>
              <div class="space-y-2">
                {selectedPlaylist.value.items.map((item, index) => (
                  <div
                    key={item.id}
                    class="flex items-center gap-3 p-2 bg-neutral-700 rounded hover:bg-neutral-600 cursor-pointer"
                    onClick={() => playTrack(item, selectedPlaylist.value?.items)}
                  >
                    <span class="text-neutral-400 w-6 text-right">{index + 1}</span>
                    <span class="text-white truncate">{item.title}</span>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      )}

      {/* Content */}
      {mediaLoading.value ? (
        <div class="flex items-center justify-center py-20">
          <div class="text-neutral-400">Loading...</div>
        </div>
      ) : music.length === 0 ? (
        <div class="text-center py-20">
          <div class="text-6xl mb-4">🎵</div>
          <h2 class="text-xl font-medium text-white mb-2">No music yet</h2>
          <p class="text-neutral-400">Upload your first track to get started</p>
        </div>
      ) : (
        <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4">
          {music.map(track => (
            <MediaCard
              key={track.id}
              media={track}
              playlist={music}
            />
          ))}
        </div>
      )}

      {/* Music player */}
      <MusicPlayer />
    </div>
  );
}
