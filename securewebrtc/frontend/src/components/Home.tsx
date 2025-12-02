import { useSignal } from '@preact/signals';
import { useEffect } from 'preact/hooks';
import { generateIdentityKeys, exportPublicKey } from '../crypto/keys';

interface HomeProps {
  onStartCall: (roomId: string) => void;
}

export default function Home({ onStartCall }: HomeProps) {
  const isGenerating = useSignal(false);
  const shareUrl = useSignal<string | null>(null);
  const roomId = useSignal<string | null>(null);
  const copied = useSignal(false);

  async function generateAndShare() {
    isGenerating.value = true;
    try {
      const keyPair = await generateIdentityKeys();
      const pubKeyString = await exportPublicKey(keyPair.publicKey);
      roomId.value = pubKeyString;
      
      // Use the public key as room ID
      const url = `${window.location.origin}/call/${pubKeyString}`;
      shareUrl.value = url;
    } catch (err) {
      console.error('Failed to generate keys:', err);
    } finally {
      isGenerating.value = false;
    }
  }

  async function copyToClipboard() {
    if (shareUrl.value) {
      await navigator.clipboard.writeText(shareUrl.value);
      copied.value = true;
      setTimeout(() => {
        copied.value = false;
      }, 2000);
    }
  }

  function handleStartCall() {
    if (roomId.value) {
      onStartCall(roomId.value);
    }
  }

  useEffect(() => {
    // Generate keys on component mount
    generateAndShare();
  }, []);

  return (
    <div class="min-h-screen bg-gray-100 flex items-center justify-center p-4">
      <div class="max-w-lg w-full bg-white rounded-xl shadow-lg p-8">
        <div class="text-center mb-8">
          <h1 class="text-3xl font-bold text-gray-800 mb-2">Secure WebRTC Call</h1>
          <p class="text-gray-600">End-to-end encrypted video calls</p>
        </div>

        {isGenerating.value ? (
          <div class="flex flex-col items-center py-8">
            <div class="w-12 h-12 border-4 border-blue-500 border-t-transparent rounded-full animate-spin mb-4" />
            <p class="text-gray-600">Generating secure identity...</p>
          </div>
        ) : (
          <div class="space-y-6">
            <div class="bg-gray-50 rounded-lg p-4">
              <p class="text-sm text-gray-500 mb-2">Share this URL to invite someone to call:</p>
              <div class="flex gap-2">
                <input
                  type="text"
                  value={shareUrl.value || ''}
                  readOnly
                  class="flex-1 px-3 py-2 bg-white border border-gray-300 rounded-lg text-sm font-mono truncate"
                />
                <button
                  onClick={copyToClipboard}
                  class="px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors min-w-20"
                >
                  {copied.value ? 'Copied!' : 'Copy'}
                </button>
              </div>
            </div>

            <button
              onClick={handleStartCall}
              class="w-full py-3 bg-green-500 text-white rounded-lg hover:bg-green-600 transition-colors font-medium text-lg"
            >
              Start Call
            </button>

            <div class="border-t border-gray-200 pt-6">
              <div class="flex items-center justify-center text-gray-500 text-sm">
                <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
                </svg>
                <span>Your call will be end-to-end encrypted</span>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
