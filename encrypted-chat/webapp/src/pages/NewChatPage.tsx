import { useEffect } from 'preact/hooks';
import { useSignal } from '@preact/signals';
import { Button, Paper, Container, Title, TextInput, Loader } from '@mantine/core';
import { currentUser, listUsers } from '../services/chatService';
import { router } from '../router';

export function NewChatPage() {
  const users = useSignal<string[]>([]);
  const loading = useSignal(true);
  const searchQuery = useSignal('');
  
  const user = currentUser.value;

  // Redirect if not logged in
  if (!user) {
    router.push('/');
    return null;
  }

  useEffect(() => {
    async function fetchUsers() {
      try {
        const userList = await listUsers();
        // Filter out current user
        users.value = userList.filter((u) => u !== user?.username);
      } catch (err) {
        console.error('Failed to fetch users:', err);
      } finally {
        loading.value = false;
      }
    }
    fetchUsers();
  }, [user?.username]);

  const filteredUsers = users.value.filter((u) =>
    u.toLowerCase().includes(searchQuery.value.toLowerCase())
  );

  return (
    <Container size="md" className="min-h-screen py-4">
      {/* Header */}
      <Paper shadow="md" p="md" radius="md" className="mb-4">
        <div className="flex items-center gap-4">
          <a href="/conversations" className="text-gray-400 hover:text-white">
            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M19 12H5M12 19l-7-7 7-7"/>
            </svg>
          </a>
          <Title order={2} className="m-0">New Chat</Title>
        </div>
      </Paper>

      {/* Search */}
      <Paper shadow="md" p="md" radius="md" className="mb-4">
        <TextInput
          placeholder="Search users..."
          value={searchQuery.value}
          onChange={(event: Event) => { searchQuery.value = (event.target as HTMLInputElement).value; }}
          size="md"
        />
      </Paper>

      {/* User list */}
      <Paper shadow="md" p="md" radius="md">
        {loading.value ? (
          <div className="flex justify-center py-8">
            <Loader />
          </div>
        ) : filteredUsers.length === 0 ? (
          <div className="text-center py-8 text-gray-400">
            {searchQuery.value ? 'No users found' : 'No other users yet'}
          </div>
        ) : (
          <div className="space-y-2">
            {filteredUsers.map((username) => (
              <a 
                key={username}
                href={`/chat/${username}`}
                className="block no-underline"
              >
                <div className="flex items-center justify-between p-3 rounded-lg hover:bg-gray-800 transition-colors cursor-pointer">
                  <div className="flex items-center gap-3">
                    <div className="w-10 h-10 rounded-full bg-gray-600 flex items-center justify-center text-white font-semibold">
                      {username.charAt(0).toUpperCase()}
                    </div>
                    <span className="text-white">{username}</span>
                  </div>
                  <Button variant="subtle" size="sm">
                    Message
                  </Button>
                </div>
              </a>
            ))}
          </div>
        )}
      </Paper>
    </Container>
  );
}
