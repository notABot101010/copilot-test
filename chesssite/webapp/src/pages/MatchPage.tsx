import { useEffect } from 'preact/hooks';
import { useRoute, useRouter } from '@copilot-test/preact-router';
import { Button } from '@mantine/core';
import { IconArrowLeft } from '@tabler/icons-preact';
import { 
  currentMatch, 
  fetchMatch, 
  connectToMatch, 
  disconnectFromMatch,
  currentUser 
} from '../store/authStore';
import { ChessBoard } from '../components/ChessBoard';
import { MoveHistory } from '../components/MoveHistory';

export function MatchPage() {
  const route = useRoute();
  const router = useRouter();
  const matchId = route.value.params.id;
  const match = currentMatch.value;
  const user = currentUser.value;
  
  useEffect(() => {
    if (!user) {
      router.push('/login');
      return;
    }
    
    if (matchId) {
      fetchMatch(matchId);
      connectToMatch(matchId);
      
      return () => {
        disconnectFromMatch();
        currentMatch.value = null;
      };
    }
  }, [matchId, user]);
  
  if (!user) {
    return null;
  }
  
  if (!match) {
    return (
      <div class="min-h-screen bg-gray-900 flex items-center justify-center">
        <p class="text-gray-400">Loading match...</p>
      </div>
    );
  }
  
  return (
    <div class="min-h-screen bg-gray-900 p-4">
      <div class="max-w-6xl mx-auto">
        {/* Header */}
        <div class="flex items-center gap-4 mb-6">
          <a href="/">
            <Button variant="subtle" leftSection={<IconArrowLeft size={18} />}>
              Back
            </Button>
          </a>
          <h1 class="text-xl font-bold">
            {match.white_player} vs {match.black_player}
          </h1>
        </div>
        
        {/* Main content */}
        <div class="flex flex-col lg:flex-row gap-8 items-start justify-center">
          {/* Chess board */}
          <div class="flex-shrink-0">
            <ChessBoard matchId={matchId || ''} />
          </div>
          
          {/* Move history */}
          <div class="w-full lg:w-80">
            <MoveHistory />
          </div>
        </div>
      </div>
    </div>
  );
}
