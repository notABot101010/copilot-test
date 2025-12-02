import { useSignal } from '@preact/signals';
import type { Media } from '../api';
import { api } from '../api';
import { deleteMedia as deleteMediaState, playTrack, currentTrack, isPlaying } from '../hooks/state';

interface MediaCardProps {
  media: Media;
  onPlay?: (media: Media) => void;
  playlist?: Media[];
}

export function MediaCard({ media, onPlay, playlist }: MediaCardProps) {
  const isDeleting = useSignal(false);

  const handleDelete = async (event: Event) => {
    event.preventDefault();
    event.stopPropagation();
    if (confirm(`Delete "${media.title}"?`)) {
      isDeleting.value = true;
      try {
        await deleteMediaState(media.id);
      } finally {
        isDeleting.value = false;
      }
    }
  };

  const handlePlay = (event: Event) => {
    event.preventDefault();
    if (onPlay) {
      onPlay(media);
    } else if (media.media_type === 'music') {
      playTrack(media, playlist);
    }
  };

  const isCurrentlyPlaying = currentTrack.value?.id === media.id && isPlaying.value;

  return (
    <div class="group relative bg-neutral-800 rounded-lg overflow-hidden hover:ring-2 hover:ring-red-600 transition-all cursor-pointer">
      {/* Thumbnail */}
      <div class="aspect-video bg-neutral-700 relative" onClick={handlePlay}>
        {media.thumbnail_path ? (
          <img
            src={api.getThumbnailUrl(media.id)}
            alt={media.title}
            class="w-full h-full object-cover"
          />
        ) : (
          <div class="w-full h-full flex items-center justify-center text-4xl">
            {media.media_type === 'video' && '🎬'}
            {media.media_type === 'music' && '🎵'}
            {media.media_type === 'photo' && '📷'}
          </div>
        )}

        {/* Play overlay */}
        <div class="absolute inset-0 bg-black/50 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center">
          <div class="w-14 h-14 rounded-full bg-white/90 flex items-center justify-center">
            <span class="text-2xl text-neutral-900">
              {isCurrentlyPlaying ? '⏸' : '▶'}
            </span>
          </div>
        </div>
      </div>

      {/* Info */}
      <div class="p-3">
        <h3 class="font-medium truncate">{media.title}</h3>
        <p class="text-sm text-neutral-400 truncate">{media.filename}</p>
      </div>

      {/* Delete button */}
      <button
        onClick={handleDelete}
        disabled={isDeleting.value}
        class="absolute top-2 right-2 w-8 h-8 rounded-full bg-black/70 text-white opacity-0 group-hover:opacity-100 transition-opacity hover:bg-red-600 flex items-center justify-center"
      >
        {isDeleting.value ? '...' : '×'}
      </button>
    </div>
  );
}
