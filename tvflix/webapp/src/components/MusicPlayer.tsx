import { useSignal, useComputed } from '@preact/signals';
import { useEffect, useRef } from 'preact/hooks';
import { currentTrack, isPlaying, playNext, playPrevious, togglePlay, currentPlaylist, currentTrackIndex } from '../hooks/state';
import { api } from '../api';

export function MusicPlayer() {
  const audioRef = useRef<HTMLAudioElement>(null);
  const progress = useSignal(0);
  const duration = useSignal(0);
  const volume = useSignal(1);

  const track = currentTrack.value;
  const playing = isPlaying.value;
  const playlist = currentPlaylist.value;
  const trackIndex = currentTrackIndex.value;

  const hasNext = useComputed(() => trackIndex < playlist.length - 1);
  const hasPrevious = useComputed(() => trackIndex > 0);

  useEffect(() => {
    const audio = audioRef.current;
    if (!audio) return;

    if (playing && track) {
      audio.play().catch(() => {
        isPlaying.value = false;
      });
    } else {
      audio.pause();
    }
  }, [playing, track]);

  useEffect(() => {
    const audio = audioRef.current;
    if (!audio || !track) return;

    audio.src = api.getStreamUrl(track.id);
    audio.load();
    if (playing) {
      audio.play().catch(() => {
        isPlaying.value = false;
      });
    }
  }, [track?.id]);

  const handleTimeUpdate = () => {
    const audio = audioRef.current;
    if (audio) {
      progress.value = audio.currentTime;
      duration.value = audio.duration || 0;
    }
  };

  const handleEnded = () => {
    if (hasNext.value) {
      playNext();
    } else {
      isPlaying.value = false;
    }
  };

  const handleSeek = (event: Event) => {
    const audio = audioRef.current;
    const target = event.target as HTMLInputElement;
    if (audio) {
      audio.currentTime = parseFloat(target.value);
    }
  };

  const handleVolumeChange = (event: Event) => {
    const audio = audioRef.current;
    const target = event.target as HTMLInputElement;
    const newVolume = parseFloat(target.value);
    volume.value = newVolume;
    if (audio) {
      audio.volume = newVolume;
    }
  };

  const formatTime = (seconds: number): string => {
    if (isNaN(seconds)) return '0:00';
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  };

  if (!track) return null;

  return (
    <div class="fixed bottom-0 left-0 right-0 bg-neutral-900 border-t border-neutral-800 p-4 z-50">
      <audio
        ref={audioRef}
        onTimeUpdate={handleTimeUpdate}
        onEnded={handleEnded}
      />

      <div class="max-w-4xl mx-auto flex items-center gap-4">
        {/* Track info */}
        <div class="flex items-center gap-3 min-w-0 flex-1">
          <div class="w-12 h-12 rounded bg-neutral-700 flex items-center justify-center text-xl shrink-0">
            🎵
          </div>
          <div class="min-w-0">
            <p class="font-medium truncate">{track.title}</p>
            <p class="text-sm text-neutral-400 truncate">{track.filename}</p>
          </div>
        </div>

        {/* Controls */}
        <div class="flex flex-col items-center gap-2 flex-1">
          <div class="flex items-center gap-4">
            <button
              onClick={playPrevious}
              disabled={!hasPrevious.value}
              class="p-2 text-xl text-neutral-400 hover:text-white disabled:opacity-30 disabled:cursor-not-allowed"
            >
              ⏮
            </button>
            <button
              onClick={togglePlay}
              class="w-12 h-12 rounded-full bg-white text-neutral-900 flex items-center justify-center text-xl hover:scale-105 transition-transform"
            >
              {playing ? '⏸' : '▶'}
            </button>
            <button
              onClick={playNext}
              disabled={!hasNext.value}
              class="p-2 text-xl text-neutral-400 hover:text-white disabled:opacity-30 disabled:cursor-not-allowed"
            >
              ⏭
            </button>
          </div>

          {/* Progress bar */}
          <div class="flex items-center gap-2 w-full max-w-md">
            <span class="text-xs text-neutral-400 w-10 text-right">
              {formatTime(progress.value)}
            </span>
            <input
              type="range"
              min="0"
              max={duration.value || 0}
              value={progress.value}
              onInput={handleSeek}
              class="flex-1 h-1 accent-red-600"
            />
            <span class="text-xs text-neutral-400 w-10">
              {formatTime(duration.value)}
            </span>
          </div>
        </div>

        {/* Volume */}
        <div class="flex items-center gap-2 flex-1 justify-end">
          <span class="text-lg">🔊</span>
          <input
            type="range"
            min="0"
            max="1"
            step="0.01"
            value={volume.value}
            onInput={handleVolumeChange}
            class="w-24 h-1 accent-red-600"
          />
        </div>
      </div>
    </div>
  );
}
