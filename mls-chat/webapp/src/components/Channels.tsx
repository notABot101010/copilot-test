import { useSignal } from '@preact/signals';
import { useEffect } from 'preact/hooks';
import { useNavigation } from '@copilot-test/preact-router';
import { listChannels, listGroups, subscribeChannel, type GroupInfo } from '../services/api';
import { currentUser, setCurrentUser } from '../app';
import { createMlsGroup, loadAllGroupStates } from '../services/mls';

export default function Channels() {
  const allChannels = useSignal<GroupInfo[]>([]);
  const myChannels = useSignal<GroupInfo[]>([]);
  const loading = useSignal(true);
  const error = useSignal('');
  const subscribing = useSignal<string | null>(null);
  const { push } = useNavigation();

  useEffect(() => {
    if (!currentUser.value) {
      push('/login');
      return;
    }

    loadAllGroupStates();
    loadData();
  }, []);

  const loadData = async () => {
    if (!currentUser.value) return;

    try {
      const [channelList, myGroupList] = await Promise.all([
        listChannels(),
        listGroups(currentUser.value.username),
      ]);
      
      allChannels.value = channelList;
      myChannels.value = myGroupList.filter(g => g.is_channel);
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to load channels';
    } finally {
      loading.value = false;
    }
  };

  const handleSubscribe = async (channel: GroupInfo) => {
    if (!currentUser.value || subscribing.value !== null) return;

    subscribing.value = channel.group_id;
    error.value = '';

    try {
      await subscribeChannel(channel.group_id, currentUser.value.username);
      
      // Create local MLS state for the channel
      await createMlsGroup(channel.group_id);
      
      // Reload data
      loadData();
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to subscribe';
    } finally {
      subscribing.value = null;
    }
  };

  const handleLogout = () => {
    setCurrentUser(null);
    push('/login');
  };

  const isSubscribed = (channelId: string) => {
    return myChannels.value.some(c => c.group_id === channelId);
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
            <a href="/groups" class="py-3 px-4 border-b-2 border-transparent hover:border-gray-300 text-gray-600">
              Groups
            </a>
            <a href="/channels" class="py-3 px-4 border-b-2 border-blue-600 text-blue-600 font-medium">
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

        <div class="flex justify-between items-center mb-4">
          <h2 class="text-lg font-semibold">Your Channels</h2>
          <a
            href="/channels/create"
            class="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
          >
            Create Channel
          </a>
        </div>

        {loading.value ? (
          <div class="text-center py-8 text-gray-500">Loading...</div>
        ) : (
          <>
            {myChannels.value.length > 0 && (
              <div class="mb-8">
                <div class="space-y-2">
                  {myChannels.value.map((channel) => (
                    <a
                      key={channel.group_id}
                      href={`/groups/${channel.group_id}`}
                      class="block bg-white rounded-lg shadow p-4 hover:shadow-md transition-shadow"
                    >
                      <div class="flex justify-between items-center">
                        <div>
                          <h3 class="font-medium">{channel.name}</h3>
                          <p class="text-sm text-gray-500">
                            {channel.member_count} subscriber{channel.member_count !== 1 ? 's' : ''}
                            {channel.is_admin && ' • Admin'}
                          </p>
                        </div>
                        <span class="text-gray-400">→</span>
                      </div>
                    </a>
                  ))}
                </div>
              </div>
            )}

            <h2 class="text-lg font-semibold mb-4">Browse Channels</h2>
            {allChannels.value.length === 0 ? (
              <div class="text-center py-8 text-gray-500">
                <p>No channels available</p>
                <p class="text-sm">Create a channel to get started</p>
              </div>
            ) : (
              <div class="space-y-2">
                {allChannels.value.map((channel) => (
                  <div
                    key={channel.group_id}
                    class="bg-white rounded-lg shadow p-4 flex justify-between items-center"
                  >
                    <div>
                      <h3 class="font-medium">{channel.name}</h3>
                      <p class="text-sm text-gray-500">
                        {channel.member_count} subscriber{channel.member_count !== 1 ? 's' : ''}
                      </p>
                    </div>
                    {isSubscribed(channel.group_id) ? (
                      <a
                        href={`/groups/${channel.group_id}`}
                        class="px-4 py-2 bg-gray-200 text-gray-700 rounded hover:bg-gray-300"
                      >
                        Open
                      </a>
                    ) : (
                      <button
                        onClick={() => handleSubscribe(channel)}
                        disabled={subscribing.value !== null}
                        data-subscribe-channel={channel.name}
                        class="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50"
                      >
                        {subscribing.value === channel.group_id ? 'Subscribing...' : 'Subscribe'}
                      </button>
                    )}
                  </div>
                ))}
              </div>
            )}
          </>
        )}
      </main>
    </div>
  );
}
