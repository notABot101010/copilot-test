import { useEffect } from 'preact/hooks';
import { useSignal } from '@preact/signals';
import type { Media } from '../api';
import { loadMedia, mediaList, mediaLoading } from '../hooks/state';
import { MediaCard } from '../components/MediaCard';
import { UploadButton } from '../components/UploadButton';
import { VideoPlayer } from '../components/VideoPlayer';

export function VideosPage() {
  const playingVideo = useSignal<Media | null>(null);

  useEffect(() => {
    loadMedia('video');
  }, []);

  const videos = mediaList.value.filter(m => m.media_type === 'video');

  const handlePlayVideo = (media: Media) => {
    playingVideo.value = media;
  };

  return (
    <div class="p-6">
      {/* Header */}
      <div class="flex items-center justify-between mb-8">
        <div>
          <h1 class="text-3xl font-bold text-white">Videos</h1>
          <p class="text-neutral-400 mt-1">Your video library</p>
        </div>
        <UploadButton accept="video/*" mediaType="video" />
      </div>

      {/* Content */}
      {mediaLoading.value ? (
        <div class="flex items-center justify-center py-20">
          <div class="text-neutral-400">Loading...</div>
        </div>
      ) : videos.length === 0 ? (
        <div class="text-center py-20">
          <div class="text-6xl mb-4">🎬</div>
          <h2 class="text-xl font-medium text-white mb-2">No videos yet</h2>
          <p class="text-neutral-400">Upload your first video to get started</p>
        </div>
      ) : (
        <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4">
          {videos.map(video => (
            <MediaCard
              key={video.id}
              media={video}
              onPlay={handlePlayVideo}
            />
          ))}
        </div>
      )}

      {/* Video player modal */}
      {playingVideo.value && (
        <VideoPlayer
          media={playingVideo.value}
          onClose={() => playingVideo.value = null}
        />
      )}
    </div>
  );
}
