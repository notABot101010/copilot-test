import { useSignal } from '@preact/signals';
import { useNavigation } from '@copilot-test/preact-router';
import { createGroup } from '../services/api';
import { createMlsGroup } from '../services/mls';
import { currentUser } from '../app';
import { useEffect } from 'preact/hooks';

export default function CreateChannel() {
  const name = useSignal('');
  const error = useSignal('');
  const loading = useSignal(false);
  const { push } = useNavigation();

  useEffect(() => {
    if (!currentUser.value) {
      push('/login');
    }
  }, []);

  const handleSubmit = async (event: Event) => {
    event.preventDefault();
    if (!currentUser.value) return;

    if (name.value.trim().length < 1) {
      error.value = 'Channel name is required';
      return;
    }

    loading.value = true;
    error.value = '';

    try {
      const response = await createGroup(currentUser.value.username, name.value.trim(), true);
      if (response.success) {
        // Create MLS group state locally
        await createMlsGroup(response.group_id);
        push(`/groups/${response.group_id}`);
      }
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to create channel';
    } finally {
      loading.value = false;
    }
  };

  return (
    <div class="min-h-screen bg-gray-100">
      <header class="bg-blue-600 text-white shadow">
        <div class="max-w-4xl mx-auto px-4 py-4 flex items-center gap-4">
          <a href="/channels" class="hover:bg-blue-700 p-2 rounded">
            ← Back
          </a>
          <h1 class="text-xl font-bold">Create Channel</h1>
        </div>
      </header>

      <main class="max-w-md mx-auto px-4 py-8">
        <form onSubmit={handleSubmit} class="bg-white rounded-lg shadow p-6">
          {error.value && (
            <div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded mb-4">
              {error.value}
            </div>
          )}

          <div class="mb-4">
            <label htmlFor="name" class="block text-sm font-medium text-gray-700 mb-1">
              Channel Name
            </label>
            <input
              id="name"
              type="text"
              value={name.value}
              onInput={(e) => (name.value = (e.target as HTMLInputElement).value)}
              placeholder="Enter channel name"
              class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>

          <p class="text-sm text-gray-500 mb-4">
            Channels are broadcast-only groups where only admins can post messages. 
            Other users can subscribe and read messages.
          </p>

          <button
            type="submit"
            disabled={loading.value}
            class="w-full py-2 px-4 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50"
          >
            {loading.value ? 'Creating...' : 'Create Channel'}
          </button>
        </form>
      </main>
    </div>
  );
}
