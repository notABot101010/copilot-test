import { useEffect, useRef } from 'preact/hooks';
import { Signal } from '@preact/signals';

interface VideoProps {
  stream: Signal<MediaStream | null>;
  muted?: boolean;
  label: string;
}

export default function Video({ stream, muted = false, label }: VideoProps) {
  const videoRef = useRef<HTMLVideoElement>(null);

  useEffect(() => {
    if (videoRef.current && stream.value) {
      videoRef.current.srcObject = stream.value;
    }
  }, [stream.value]);

  return (
    <div class="relative bg-gray-900 rounded-lg overflow-hidden">
      <video
        ref={videoRef}
        autoPlay
        playsInline
        muted={muted}
        class="w-full h-full object-cover"
      />
      <div class="absolute bottom-2 left-2 bg-black/50 text-white text-sm px-2 py-1 rounded">
        {label}
      </div>
      {!stream.value && (
        <div class="absolute inset-0 flex items-center justify-center bg-gray-800 text-gray-400">
          <p>Waiting for video...</p>
        </div>
      )}
    </div>
  );
}
