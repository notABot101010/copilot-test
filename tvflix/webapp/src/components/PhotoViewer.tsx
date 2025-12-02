import { useSignal } from '@preact/signals';
import type { Media } from '../api';
import { api } from '../api';

interface PhotoViewerProps {
  media: Media;
  onClose: () => void;
  allPhotos?: Media[];
}

export function PhotoViewer({ media, onClose, allPhotos = [] }: PhotoViewerProps) {
  const currentIndex = useSignal(allPhotos.findIndex(p => p.id === media.id) || 0);

  const currentPhoto = allPhotos[currentIndex.value] || media;

  const goNext = () => {
    if (currentIndex.value < allPhotos.length - 1) {
      currentIndex.value++;
    }
  };

  const goPrevious = () => {
    if (currentIndex.value > 0) {
      currentIndex.value--;
    }
  };

  const handleKeyDown = (event: KeyboardEvent) => {
    if (event.key === 'Escape') {
      onClose();
    } else if (event.key === 'ArrowRight') {
      goNext();
    } else if (event.key === 'ArrowLeft') {
      goPrevious();
    }
  };

  return (
    <div
      class="fixed inset-0 z-50 bg-black/95 flex items-center justify-center"
      onKeyDown={handleKeyDown}
      tabIndex={0}
    >
      {/* Close button */}
      <button
        onClick={onClose}
        class="absolute top-4 right-4 z-10 w-10 h-10 rounded-full bg-black/50 text-white text-xl flex items-center justify-center hover:bg-black/70"
      >
        ×
      </button>

      {/* Previous button */}
      {currentIndex.value > 0 && (
        <button
          onClick={goPrevious}
          class="absolute left-4 top-1/2 -translate-y-1/2 w-12 h-12 rounded-full bg-black/50 text-white text-2xl flex items-center justify-center hover:bg-black/70"
        >
          ‹
        </button>
      )}

      {/* Image */}
      <img
        src={api.getStreamUrl(currentPhoto.id)}
        alt={currentPhoto.title}
        class="max-w-full max-h-full object-contain"
      />

      {/* Next button */}
      {currentIndex.value < allPhotos.length - 1 && (
        <button
          onClick={goNext}
          class="absolute right-4 top-1/2 -translate-y-1/2 w-12 h-12 rounded-full bg-black/50 text-white text-2xl flex items-center justify-center hover:bg-black/70"
        >
          ›
        </button>
      )}

      {/* Info */}
      <div class="absolute bottom-4 left-4 right-4 text-center">
        <h2 class="text-xl font-bold text-white">{currentPhoto.title}</h2>
        {allPhotos.length > 1 && (
          <p class="text-neutral-400 mt-1">
            {currentIndex.value + 1} / {allPhotos.length}
          </p>
        )}
      </div>
    </div>
  );
}
