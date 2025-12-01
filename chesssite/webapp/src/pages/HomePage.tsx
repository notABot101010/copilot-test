import { useEffect, useState } from 'preact/hooks';
import { Button, Card, Select } from '@mantine/core';
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
  
  // Use useState for modal to ensure proper re-render with Mantine
  const [modalOpened, setModalOpened] = useState(false);
  const [usersList, setUsersList] = useState<string[]>([]);
  const [opponent, setOpponent] = useState('');
  const [creating, setCreating] = useState(false);
  
  useEffect(() => {
    if (!user) {
      router.push('/login');
      return;
    }
    fetchMatches();
  }, [user]);
  
  const handleCreateMatch = async () => {
    if (!opponent) return;
    
    setCreating(true);
    try {
      const matchId = await createMatch(opponent);
      if (matchId) {
        setModalOpened(false);
        setOpponent('');
        router.push(`/match/${matchId}`);
      }
    } finally {
      setCreating(false);
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
          <button
            type="button"
            class="flex items-center gap-2 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors cursor-pointer"
            onClick={() => {
              setModalOpened(true);
              fetchUsers().then(users => {
                setUsersList(users.filter(u => u !== user?.username));
              });
            }}
          >
            <IconPlus size={18} />
            <span>New Match</span>
          </button>
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
      {modalOpened && (
        <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div class="bg-gray-800 rounded-lg p-6 w-full max-w-md shadow-xl">
            <h2 class="text-xl font-bold mb-4">Create New Match</h2>
            <div class="space-y-4">
              <Select
                label="Select opponent"
                placeholder="Choose a player"
                data={usersList.map(u => ({ value: u, label: u }))}
                value={opponent}
                onChange={(value: string | null) => setOpponent(value || '')}
                searchable
              />
              
              {usersList.length === 0 && (
                <p class="text-gray-400 text-sm">No other users available. Ask a friend to register!</p>
              )}
              
              <div class="flex justify-end gap-2">
                <Button 
                  variant="subtle" 
                  onClick={() => setModalOpened(false)}
                >
                  Cancel
                </Button>
                <Button
                  onClick={handleCreateMatch}
                  loading={creating}
                  disabled={!opponent}
                >
                  Create Match
                </Button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
