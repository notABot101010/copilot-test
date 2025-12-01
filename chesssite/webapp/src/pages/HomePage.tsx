import { useSignal } from '@preact/signals';
import { useEffect } from 'preact/hooks';
import { Button, Card, Modal, Select } from '@mantine/core';
import { IconPlus, IconLogout, IconChess } from '@tabler/icons-preact';
import { 
  currentUser, 
  matches, 
  matchesLoading, 
  fetchMatches, 
  createMatch, 
  logout,
  fetchUsers 
} from '../store/authStore';
import { useRouter } from '@copilot-test/preact-router';

export function HomePage() {
  const router = useRouter();
  const user = currentUser.value;
  const matchList = matches.value;
  const loading = matchesLoading.value;
  
  const showNewMatchModal = useSignal(false);
  const availableUsers = useSignal<string[]>([]);
  const selectedOpponent = useSignal('');
  const creatingMatch = useSignal(false);
  
  useEffect(() => {
    if (!user) {
      router.push('/login');
      return;
    }
    fetchMatches();
  }, [user]);
  
  const handleNewMatch = async () => {
    const users = await fetchUsers();
    availableUsers.value = users.filter(u => u !== user?.username);
    showNewMatchModal.value = true;
  };
  
  const handleCreateMatch = async () => {
    if (!selectedOpponent.value) return;
    
    creatingMatch.value = true;
    try {
      const matchId = await createMatch(selectedOpponent.value);
      if (matchId) {
        showNewMatchModal.value = false;
        selectedOpponent.value = '';
        router.push(`/match/${matchId}`);
      }
    } finally {
      creatingMatch.value = false;
    }
  };
  
  const handleLogout = async () => {
    await logout();
    router.push('/login');
  };
  
  const formatDate = (timestamp: number) => {
    return new Date(timestamp).toLocaleString();
  };
  
  const getStatusDisplay = (status: string) => {
    if (status.startsWith('checkmate:')) {
      const winner = status.split(':')[1];
      return `${winner === 'white' ? 'White' : 'Black'} wins`;
    }
    if (status === 'stalemate') return 'Draw';
    if (status === 'check') return 'In Check';
    return 'Active';
  };
  
  const getStatusColor = (status: string) => {
    if (status.startsWith('checkmate:')) return 'text-yellow-400';
    if (status === 'stalemate') return 'text-gray-400';
    if (status === 'check') return 'text-orange-400';
    return 'text-green-400';
  };
  
  if (!user) {
    return null;
  }
  
  return (
    <div class="min-h-screen bg-gray-900 p-4">
      {/* Header */}
      <div class="max-w-4xl mx-auto">
        <div class="flex items-center justify-between mb-8">
          <div class="flex items-center gap-3">
            <IconChess size={32} />
            <h1 class="text-2xl font-bold">Chess Site</h1>
          </div>
          <div class="flex items-center gap-4">
            <span class="text-gray-400">Welcome, {user.username}</span>
            <Button
              variant="subtle"
              color="red"
              leftSection={<IconLogout size={18} />}
              onClick={handleLogout}
            >
              Logout
            </Button>
          </div>
        </div>
        
        {/* New Match Button */}
        <div class="mb-6">
          <Button
            leftSection={<IconPlus size={18} />}
            onClick={handleNewMatch}
          >
            New Match
          </Button>
        </div>
        
        {/* Match List */}
        <div>
          <h2 class="text-xl font-semibold mb-4">Your Matches</h2>
          
          {loading ? (
            <p class="text-gray-400">Loading matches...</p>
          ) : matchList.length === 0 ? (
            <p class="text-gray-400">No matches yet. Create one to get started!</p>
          ) : (
            <div class="grid gap-4 md:grid-cols-2">
              {matchList.map(match => {
                const opponent = match.white_player === user.username 
                  ? match.black_player 
                  : match.white_player;
                const myColor = match.white_player === user.username ? 'White' : 'Black';
                
                return (
                  <a href={`/match/${match.id}`} key={match.id}>
                    <Card 
                      shadow="sm" 
                      padding="md" 
                      radius="md" 
                      class="bg-gray-800 hover:bg-gray-700 transition-colors cursor-pointer"
                    >
                      <div class="flex items-center justify-between mb-2">
                        <span class="font-semibold">vs {opponent}</span>
                        <span class={`text-sm ${getStatusColor(match.status)}`}>
                          {getStatusDisplay(match.status)}
                        </span>
                      </div>
                      <div class="text-sm text-gray-400">
                        <p>Playing as {myColor}</p>
                        <p>Last updated: {formatDate(match.updatedAt)}</p>
                      </div>
                    </Card>
                  </a>
                );
              })}
            </div>
          )}
        </div>
      </div>
      
      {/* New Match Modal */}
      <Modal
        opened={showNewMatchModal.value}
        onClose={() => showNewMatchModal.value = false}
        title="Create New Match"
        centered
      >
        <div class="space-y-4">
          <Select
            label="Select opponent"
            placeholder="Choose a player"
            data={availableUsers.value.map(u => ({ value: u, label: u }))}
            value={selectedOpponent.value}
            onChange={(value: string | null) => selectedOpponent.value = value || ''}
            searchable
          />
          
          {availableUsers.value.length === 0 && (
            <p class="text-gray-400 text-sm">No other users available. Ask a friend to register!</p>
          )}
          
          <div class="flex justify-end gap-2">
            <Button 
              variant="subtle" 
              onClick={() => showNewMatchModal.value = false}
            >
              Cancel
            </Button>
            <Button
              onClick={handleCreateMatch}
              loading={creatingMatch.value}
              disabled={!selectedOpponent.value}
            >
              Create Match
            </Button>
          </div>
        </div>
      </Modal>
    </div>
  );
}
