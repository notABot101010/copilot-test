import { useSignal } from '@preact/signals';
import { useEffect } from 'preact/hooks';
import Video from './Video';
import { createWebRTCService } from '../services/webrtc';

interface CallProps {
  roomId: string;
  onEnd: () => void;
}

export default function Call({ roomId, onEnd }: CallProps) {
  const webrtc = useSignal(createWebRTCService());
  const isConnecting = useSignal(true);

  useEffect(() => {
    const service = webrtc.value;
    
    async function initCall() {
      try {
        // Determine if we're the initiator or joiner
        // The URL path determines this - if we navigated here, we're joining
        const isInitiator = window.location.pathname === '/';
        await service.connect(roomId, isInitiator);
        isConnecting.value = false;
      } catch (err) {
        console.error('Failed to initialize call:', err);
      }
    }

    initCall();

    return () => {
      service.disconnect();
    };
  }, [roomId]);

  function handleEndCall() {
    webrtc.value.disconnect();
    onEnd();
  }

  function handleToggleMute() {
    const stream = webrtc.value.localStream.value;
    if (stream) {
      const audioTracks = stream.getAudioTracks();
      audioTracks.forEach(track => {
        track.enabled = !track.enabled;
      });
    }
  }

  function handleToggleVideo() {
    const stream = webrtc.value.localStream.value;
    if (stream) {
      const videoTracks = stream.getVideoTracks();
      videoTracks.forEach(track => {
        track.enabled = !track.enabled;
      });
    }
  }

  const service = webrtc.value;
  const connectionStatus = service.connectionState.value;
  const hasRemoteStream = service.remoteStream.value !== null;

  return (
    <div class="min-h-screen bg-gray-900 flex flex-col">
      {/* Status bar */}
      <div class="bg-gray-800 p-3 flex items-center justify-between">
        <div class="flex items-center gap-3">
          <div class={`w-3 h-3 rounded-full ${
            connectionStatus === 'connected' ? 'bg-green-500' :
            connectionStatus === 'connecting' ? 'bg-yellow-500 animate-pulse' :
            'bg-red-500'
          }`} />
          <span class="text-white text-sm">
            {connectionStatus === 'connected' ? 'Connected (Encrypted)' :
             connectionStatus === 'connecting' ? 'Connecting...' :
             service.peerJoined.value ? 'Peer joined, establishing connection...' :
             'Waiting for peer...'}
          </span>
        </div>
        {service.error.value && (
          <span class="text-red-400 text-sm">{service.error.value}</span>
        )}
      </div>

      {/* Video grid */}
      <div class="flex-1 p-4 grid grid-cols-1 md:grid-cols-2 gap-4">
        <div class="aspect-video">
          <Video stream={service.remoteStream} label="Remote" />
        </div>
        <div class="aspect-video">
          <Video stream={service.localStream} muted={true} label="You" />
        </div>
      </div>

      {/* Controls */}
      <div class="bg-gray-800 p-4">
        <div class="flex items-center justify-center gap-4">
          <button
            onClick={handleToggleMute}
            class="p-4 rounded-full bg-gray-700 hover:bg-gray-600 text-white transition-colors"
            title="Toggle microphone"
          >
            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11a7 7 0 01-7 7m0 0a7 7 0 01-7-7m7 7v4m0 0H8m4 0h4m-4-8a3 3 0 01-3-3V5a3 3 0 116 0v6a3 3 0 01-3 3z" />
            </svg>
          </button>
          
          <button
            onClick={handleEndCall}
            class="p-4 rounded-full bg-red-500 hover:bg-red-600 text-white transition-colors"
            title="End call"
          >
            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 8l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2M3 5a2 2 0 012-2h3.28a1 1 0 01.948.684l1.498 4.493a1 1 0 01-.502 1.21l-2.257 1.13a11.042 11.042 0 005.516 5.516l1.13-2.257a1 1 0 011.21-.502l4.493 1.498a1 1 0 01.684.949V19a2 2 0 01-2 2h-1C9.716 21 3 14.284 3 6V5z" />
            </svg>
          </button>
          
          <button
            onClick={handleToggleVideo}
            class="p-4 rounded-full bg-gray-700 hover:bg-gray-600 text-white transition-colors"
            title="Toggle camera"
          >
            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 10l4.553-2.276A1 1 0 0121 8.618v6.764a1 1 0 01-1.447.894L15 14M5 18h8a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z" />
            </svg>
          </button>
        </div>
      </div>
    </div>
  );
}
