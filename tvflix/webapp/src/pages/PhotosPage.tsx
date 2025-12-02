import { useEffect } from 'preact/hooks';
import { useSignal } from '@preact/signals';
import type { Media, AlbumWithMedia } from '../api';
import { api } from '../api';
import {
  loadMedia,
  mediaList,
  mediaLoading,
  loadAlbums,
  albums,
  createAlbum,
  deleteAlbum,
} from '../hooks/state';
import { UploadButton } from '../components/UploadButton';
import { PhotoViewer } from '../components/PhotoViewer';

export function PhotosPage() {
  const showAlbums = useSignal(false);
  const selectedAlbum = useSignal<AlbumWithMedia | null>(null);
  const viewingPhoto = useSignal<Media | null>(null);
  const newAlbumName = useSignal('');
  const isCreatingAlbum = useSignal(false);

  useEffect(() => {
    loadMedia('photo');
    loadAlbums();
  }, []);

  const photos = mediaList.value.filter(m => m.media_type === 'photo');

  const handleCreateAlbum = async (event: Event) => {
    event.preventDefault();
    if (!newAlbumName.value.trim()) return;

    isCreatingAlbum.value = true;
    try {
      await createAlbum(newAlbumName.value);
      newAlbumName.value = '';
    } finally {
      isCreatingAlbum.value = false;
    }
  };

  const handleLoadAlbum = async (id: number) => {
    const album = await api.getAlbum(id);
    selectedAlbum.value = album;
  };

  const handleViewPhoto = (photo: Media) => {
    viewingPhoto.value = photo;
  };

  return (
    <div class="p-6">
      {/* Header */}
      <div class="flex items-center justify-between mb-8">
        <div>
          <h1 class="text-3xl font-bold text-white">Photos</h1>
          <p class="text-neutral-400 mt-1">Your photo library</p>
        </div>
        <div class="flex items-center gap-4">
          <button
            onClick={() => showAlbums.value = !showAlbums.value}
            class={`px-4 py-2 rounded-lg transition-colors ${
              showAlbums.value
                ? 'bg-red-600 text-white'
                : 'bg-neutral-700 text-neutral-300 hover:bg-neutral-600'
            }`}
          >
            Albums
          </button>
          <UploadButton accept="image/*" mediaType="photo" />
        </div>
      </div>

      {/* Albums sidebar */}
      {showAlbums.value && (
        <div class="mb-8 bg-neutral-800 rounded-lg p-4">
          <h2 class="text-lg font-medium text-white mb-4">Albums</h2>

          {/* Create album form */}
          <form onSubmit={handleCreateAlbum} class="flex gap-2 mb-4">
            <input
              type="text"
              value={newAlbumName.value}
              onInput={(event) => newAlbumName.value = (event.target as HTMLInputElement).value}
              placeholder="New album name"
              class="flex-1 px-3 py-2 bg-neutral-700 rounded text-white placeholder-neutral-400 focus:outline-none focus:ring-2 focus:ring-red-600"
            />
            <button
              type="submit"
              disabled={isCreatingAlbum.value}
              class="px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded transition-colors disabled:opacity-50"
            >
              Create
            </button>
          </form>

          {/* Album list */}
          <div class="space-y-2">
            {albums.value.map(album => (
              <div
                key={album.id}
                class="flex items-center justify-between p-3 bg-neutral-700 rounded hover:bg-neutral-600 cursor-pointer"
                onClick={() => handleLoadAlbum(album.id)}
              >
                <span class="text-white">{album.name}</span>
                <button
                  onClick={(event) => {
                    event.stopPropagation();
                    deleteAlbum(album.id);
                  }}
                  class="text-neutral-400 hover:text-red-500"
                >
                  ×
                </button>
              </div>
            ))}
            {albums.value.length === 0 && (
              <p class="text-neutral-400 text-sm">No albums yet</p>
            )}
          </div>

          {/* Selected album */}
          {selectedAlbum.value && (
            <div class="mt-4 pt-4 border-t border-neutral-700">
              <h3 class="text-lg font-medium text-white mb-4">{selectedAlbum.value.name}</h3>
              <div class="grid grid-cols-4 gap-2">
                {selectedAlbum.value.items.map(item => (
                  <div
                    key={item.id}
                    class="aspect-square bg-neutral-700 rounded overflow-hidden cursor-pointer"
                    onClick={() => viewingPhoto.value = item}
                  >
                    <img
                      src={api.getStreamUrl(item.id)}
                      alt={item.title}
                      class="w-full h-full object-cover"
                    />
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
      ) : photos.length === 0 ? (
        <div class="text-center py-20">
          <div class="text-6xl mb-4">📷</div>
          <h2 class="text-xl font-medium text-white mb-2">No photos yet</h2>
          <p class="text-neutral-400">Upload your first photo to get started</p>
        </div>
      ) : (
        <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4">
          {photos.map(photo => (
            <div
              key={photo.id}
              class="aspect-square bg-neutral-800 rounded-lg overflow-hidden cursor-pointer hover:ring-2 hover:ring-red-600 transition-all"
              onClick={() => handleViewPhoto(photo)}
            >
              <img
                src={api.getStreamUrl(photo.id)}
                alt={photo.title}
                class="w-full h-full object-cover"
              />
            </div>
          ))}
        </div>
      )}

      {/* Photo viewer modal */}
      {viewingPhoto.value && (
        <PhotoViewer
          media={viewingPhoto.value}
          allPhotos={photos}
          onClose={() => viewingPhoto.value = null}
        />
      )}
    </div>
  );
}
