import { useSignal } from '@preact/signals';
import { useEffect, useRef } from 'preact/hooks';
import type { Media } from '../api';
import { api } from '../api';

interface VideoPlayerProps {
  media: Media;
  onClose: () => void;
}

export function VideoPlayer({ media, onClose }: VideoPlayerProps) {
  const videoRef = useRef<HTMLVideoElement>(null);
  const isFullscreen = useSignal(false);
  const showControls = useSignal(true);
  const hideTimeout = useRef<number | null>(null);

  useEffect(() => {
    const video = videoRef.current;
    if (video) {
      video.play().catch(() => {});
    }

    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        onClose();
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, []);

  const handleMouseMove = () => {
    showControls.value = true;
    if (hideTimeout.current) {
      clearTimeout(hideTimeout.current);
    }
    hideTimeout.current = window.setTimeout(() => {
      showControls.value = false;
    }, 3000);
  };

  const toggleFullscreen = () => {
    const container = videoRef.current?.parentElement;
    if (!container) return;

    if (isFullscreen.value) {
      document.exitFullscreen?.();
    } else {
      container.requestFullscreen?.();
    }
    isFullscreen.value = !isFullscreen.value;
  };

  return (
    <div
      class="fixed inset-0 z-50 bg-black flex items-center justify-center"
      onMouseMove={handleMouseMove}
    >
      {/* Close button */}
      <button
        onClick={onClose}
        class={`absolute top-4 right-4 z-10 w-10 h-10 rounded-full bg-black/50 text-white text-xl flex items-center justify-center hover:bg-black/70 transition-all ${
          showControls.value ? 'opacity-100' : 'opacity-0'
        }`}
      >
        ×
      </button>

      {/* Video */}
      <video
        ref={videoRef}
        src={api.getStreamUrl(media.id)}
        controls
        class="max-w-full max-h-full"
        onDblClick={toggleFullscreen}
      />

      {/* Title */}
      <div
        class={`absolute bottom-20 left-4 right-4 transition-opacity ${
          showControls.value ? 'opacity-100' : 'opacity-0'
        }`}
      >
        <h2 class="text-2xl font-bold text-white">{media.title}</h2>
      </div>
    </div>
  );
}
