import { useSignal } from '@preact/signals';
import { useEffect } from 'preact/hooks';
import type { Session } from '../types';
import { listSessions, createSession } from '../api';
import { SessionCard } from '../components/SessionCard';

export function SessionsPage() {
  const sessions = useSignal<Session[]>([]);
  const loading = useSignal(true);
  const creating = useSignal(false);
  const newSessionName = useSignal('');

  useEffect(() => {
    loadSessions();
  }, []);

  const loadSessions = async () => {
    loading.value = true;
    try {
      sessions.value = await listSessions();
    } catch (err) {
      console.error('Failed to load sessions:', err);
    } finally {
      loading.value = false;
    }
  };

  const handleCreate = async () => {
    creating.value = true;
    try {
      const session = await createSession(newSessionName.value || undefined);
      sessions.value = [session, ...sessions.value];
      newSessionName.value = '';
      window.location.href = `/session/${session.id}`;
    } catch (err) {
      console.error('Failed to create session:', err);
    } finally {
      creating.value = false;
    }
  };

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-2xl font-bold text-gray-900">Sessions</h2>
      </div>

      <div className="bg-white rounded-lg border border-gray-200 p-4 mb-6">
        <h3 className="font-medium text-gray-900 mb-3">Create New Session</h3>
        <div className="flex gap-3">
          <input
            type="text"
            value={newSessionName.value}
            onInput={(e) => newSessionName.value = (e.target as HTMLInputElement).value}
            placeholder="Session name (optional)"
            className="flex-1 px-3 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          <button
            onClick={handleCreate}
            disabled={creating.value}
            className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50"
          >
            {creating.value ? 'Creating...' : 'Create Session'}
          </button>
        </div>
      </div>

      {loading.value ? (
        <div className="text-center py-8 text-gray-500">Loading sessions...</div>
      ) : sessions.value.length === 0 ? (
        <div className="text-center py-8 text-gray-500">
          No sessions yet. Create one to get started!
        </div>
      ) : (
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
          {sessions.value.map((session) => (
            <SessionCard key={session.id} session={session} />
          ))}
        </div>
      )}
    </div>
  );
}
