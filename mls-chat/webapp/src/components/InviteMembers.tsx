import { useSignal } from '@preact/signals';
import { useEffect } from 'preact/hooks';
import { useRoute, useNavigation } from '@copilot-test/preact-router';
import { listUsers, getKeyPackage, inviteMember, getGroup, type UserInfo } from '../services/api';
import { currentUser } from '../app';
import { createInvite } from '../services/mls';

export default function InviteMembers() {
  const route = useRoute();
  const { push } = useNavigation();
  const groupId = route.value.params.groupId as string;

  const users = useSignal<UserInfo[]>([]);
  const groupName = useSignal('');
  const loading = useSignal(true);
  const inviting = useSignal<number | null>(null);
  const error = useSignal('');
  const success = useSignal('');

  useEffect(() => {
    if (!currentUser.value) {
      push('/login');
      return;
    }

    loadData();
  }, [groupId]);

  const loadData = async () => {
    if (!currentUser.value) return;

    try {
      const [userList, groupInfo] = await Promise.all([
        listUsers(currentUser.value.username),
        getGroup(groupId, currentUser.value.username),
      ]);
      users.value = userList;
      groupName.value = groupInfo.name;
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to load data';
    } finally {
      loading.value = false;
    }
  };

  const handleInvite = async (user: UserInfo) => {
    if (!currentUser.value || inviting.value !== null) return;

    inviting.value = user.id;
    error.value = '';
    success.value = '';

    try {
      // Get the user's key package
      const keyPackageResponse = await getKeyPackage(user.username);
      
      // Create welcome and commit messages
      const { welcome, commit } = await createInvite(groupId, keyPackageResponse.key_package);
      
      // Send the invite
      await inviteMember(
        groupId,
        currentUser.value.username,
        user.username,
        welcome,
        commit
      );

      success.value = `Invited ${user.username} to the group`;
      
      // Remove the invited user from the list
      users.value = users.value.filter((u) => u.id !== user.id);
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to invite user';
    } finally {
      inviting.value = null;
    }
  };

  if (!currentUser.value) {
    return null;
  }

  return (
    <div class="min-h-screen bg-gray-100">
      <header class="bg-blue-600 text-white shadow">
        <div class="max-w-4xl mx-auto px-4 py-4 flex items-center gap-4">
          <a href={`/groups/${groupId}`} class="hover:bg-blue-700 p-2 rounded">
            ← Back
          </a>
          <div>
            <h1 class="text-xl font-bold">Invite Members</h1>
            <p class="text-sm text-blue-200">{groupName.value}</p>
          </div>
        </div>
      </header>

      <main class="max-w-md mx-auto px-4 py-6">
        {error.value && (
          <div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded mb-4">
            {error.value}
          </div>
        )}

        {success.value && (
          <div class="bg-green-50 border border-green-200 text-green-700 px-4 py-3 rounded mb-4">
            {success.value}
          </div>
        )}

        {loading.value ? (
          <div class="text-center py-8 text-gray-500">Loading...</div>
        ) : users.value.length === 0 ? (
          <div class="text-center py-8 text-gray-500">
            <p>No users available to invite</p>
          </div>
        ) : (
          <div class="bg-white rounded-lg shadow divide-y">
            {users.value.map((user) => (
              <div key={user.id} class="p-4 flex justify-between items-center" data-user={user.username}>
                <span class="font-medium">{user.username}</span>
                <button
                  onClick={() => handleInvite(user)}
                  disabled={inviting.value !== null}
                  data-invite-user={user.username}
                  class="px-4 py-2 bg-blue-600 text-white text-sm rounded hover:bg-blue-700 disabled:opacity-50"
                >
                  {inviting.value === user.id ? 'Inviting...' : 'Invite'}
                </button>
              </div>
            ))}
          </div>
        )}
      </main>
    </div>
  );
}
