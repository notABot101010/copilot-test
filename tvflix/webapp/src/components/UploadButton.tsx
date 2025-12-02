import { useSignal } from '@preact/signals';
import { uploadMedia } from '../hooks/state';

interface UploadButtonProps {
  accept?: string;
  mediaType?: 'video' | 'music' | 'photo';
}

export function UploadButton({ accept, mediaType }: UploadButtonProps) {
  const isUploading = useSignal(false);
  const uploadProgress = useSignal(0);
  const error = useSignal<string | null>(null);

  const acceptTypes = accept || (() => {
    switch (mediaType) {
      case 'video':
        return 'video/*';
      case 'music':
        return 'audio/*';
      case 'photo':
        return 'image/*';
      default:
        return 'video/*,audio/*,image/*';
    }
  })();

  const handleFileChange = async (event: Event) => {
    const target = event.target as HTMLInputElement;
    const files = target.files;
    if (!files || files.length === 0) return;

    isUploading.value = true;
    error.value = null;

    try {
      for (let index = 0; index < files.length; index++) {
        const file = files[index];
        uploadProgress.value = Math.round((index / files.length) * 100);
        await uploadMedia(file);
      }
      uploadProgress.value = 100;
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Upload failed';
    } finally {
      isUploading.value = false;
      uploadProgress.value = 0;
      target.value = '';
    }
  };

  return (
    <div class="relative">
      <input
        type="file"
        accept={acceptTypes}
        multiple
        onChange={handleFileChange}
        disabled={isUploading.value}
        class="absolute inset-0 w-full h-full opacity-0 cursor-pointer disabled:cursor-not-allowed"
      />
      <button
        disabled={isUploading.value}
        class="px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded-lg transition-colors flex items-center gap-2 disabled:opacity-50"
      >
        {isUploading.value ? (
          <>
            <span class="animate-spin">⏳</span>
            <span>Uploading {uploadProgress.value}%</span>
          </>
        ) : (
          <>
            <span>+</span>
            <span>Upload</span>
          </>
        )}
      </button>
      {error.value && (
        <p class="absolute top-full mt-1 text-sm text-red-500">{error.value}</p>
      )}
    </div>
  );
}
