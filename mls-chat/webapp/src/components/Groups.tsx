import { useSignal } from '@preact/signals';
import { useEffect } from 'preact/hooks';
import { useNavigation } from '@copilot-test/preact-router';
import { listGroups, getPendingWelcomes, joinGroup, deleteWelcome, poll, type GroupInfo, type PendingWelcome } from '../services/api';
import { currentUser, setCurrentUser } from '../app';
import { processWelcome, loadAllGroupStates } from '../services/mls';

export default function Groups() {
  const groups = useSignal<GroupInfo[]>([]);
  const welcomes = useSignal<PendingWelcome[]>([]);
  const loading = useSignal(true);
  const error = useSignal('');
  const { push } = useNavigation();

  useEffect(() => {
    if (!currentUser.value) {
      push('/login');
      return;
    }

    loadAllGroupStates();
    loadData();
    startPolling();
  }, []);

  const loadData = async () => {
    if (!currentUser.value) return;

    try {
      const [groupList, welcomeList] = await Promise.all([
        listGroups(currentUser.value.username),
        getPendingWelcomes(currentUser.value.username),
      ]);
      groups.value = groupList.filter(g => !g.is_channel);
      welcomes.value = welcomeList;
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to load data';
    } finally {
      loading.value = false;
    }
  };

  const startPolling = () => {
    const pollLoop = async () => {
      if (!currentUser.value) return;

      try {
        const response = await poll(currentUser.value.username);
        if (response.welcomes.length > 0 || response.messages.length > 0) {
          loadData();
        }
      } catch {
        // Ignore polling errors
      }

      // Continue polling
      if (currentUser.value) {
        setTimeout(pollLoop, 1000);
      }
    };

    pollLoop();
  };

  const handleAcceptInvite = async (welcome: PendingWelcome) => {
    if (!currentUser.value) return;

    try {
      // Process the welcome to get the group state
      await processWelcome(welcome.welcome_data);
      
      // Join the group on the server
      await joinGroup(welcome.group_id, currentUser.value.username);
      
      // Delete the welcome
      await deleteWelcome(welcome.id);
      
      // Reload data
      loadData();
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to join group';
    }
  };

  const handleLogout = () => {
    setCurrentUser(null);
    push('/login');
  };

  if (!currentUser.value) {
    return null;
  }

  return (
    <div class="min-h-screen bg-gray-100">
      <header class="bg-blue-600 text-white shadow">
        <div class="max-w-4xl mx-auto px-4 py-4 flex justify-between items-center">
          <h1 class="text-xl font-bold">MLS Chat</h1>
          <div class="flex items-center gap-4">
            <span>{currentUser.value.username}</span>
            <button
              onClick={handleLogout}
              class="px-3 py-1 bg-blue-700 hover:bg-blue-800 rounded text-sm"
            >
              Logout
            </button>
          </div>
        </div>
      </header>

      <nav class="bg-white shadow">
        <div class="max-w-4xl mx-auto px-4">
          <div class="flex gap-4">
            <a href="/groups" class="py-3 px-4 border-b-2 border-blue-600 text-blue-600 font-medium">
              Groups
            </a>
            <a href="/channels" class="py-3 px-4 border-b-2 border-transparent hover:border-gray-300 text-gray-600">
              Channels
            </a>
          </div>
        </div>
      </nav>

      <main class="max-w-4xl mx-auto px-4 py-6">
        {error.value && (
          <div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded mb-4">
            {error.value}
          </div>
        )}

        {welcomes.value.length > 0 && (
          <div class="mb-6">
            <h2 class="text-lg font-semibold mb-3">Pending Invitations</h2>
            <div class="space-y-2">
              {welcomes.value.map((welcome) => (
                <div
                  key={welcome.id}
                  class="bg-yellow-50 border border-yellow-200 rounded-lg p-4 flex justify-between items-center"
                >
                  <div>
                    <p class="font-medium">{welcome.group_name}</p>
                    <p class="text-sm text-gray-600">Invited by {welcome.inviter_name}</p>
                  </div>
                  <button
                    onClick={() => handleAcceptInvite(welcome)}
                    class="px-4 py-2 bg-green-600 text-white rounded hover:bg-green-700"
                  >
                    Accept
                  </button>
                </div>
              ))}
            </div>
          </div>
        )}

        <div class="flex justify-between items-center mb-4">
          <h2 class="text-lg font-semibold">Your Groups</h2>
          <a
            href="/groups/create"
            class="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
          >
            Create Group
          </a>
        </div>

        {loading.value ? (
          <div class="text-center py-8 text-gray-500">Loading...</div>
        ) : groups.value.length === 0 ? (
          <div class="text-center py-8 text-gray-500">
            <p>No groups yet</p>
            <p class="text-sm">Create a group or wait for an invitation</p>
          </div>
        ) : (
          <div class="space-y-2">
            {groups.value.map((group) => (
              <a
                key={group.group_id}
                href={`/groups/${group.group_id}`}
                class="block bg-white rounded-lg shadow p-4 hover:shadow-md transition-shadow"
              >
                <div class="flex justify-between items-center">
                  <div>
                    <h3 class="font-medium">{group.name}</h3>
                    <p class="text-sm text-gray-500">
                      {group.member_count} member{group.member_count !== 1 ? 's' : ''}
                      {group.is_admin && ' • Admin'}
                    </p>
                  </div>
                  <span class="text-gray-400">→</span>
                </div>
              </a>
            ))}
          </div>
        )}
      </main>
    </div>
  );
}
