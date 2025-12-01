import { currentMatch, replayIndex } from '../store/authStore';
import { IconPlayerPlay, IconPlayerPause, IconPlayerSkipBack, IconPlayerSkipForward, IconPlayerTrackNext, IconPlayerTrackPrev } from '@tabler/icons-preact';

export function MoveHistory() {
  const match = currentMatch.value;
  const replay = replayIndex.value;
  
  if (!match || !match.chessState) {
    return null;
  }
  
  const moves = match.chessState.moves;
  const isReplaying = replay !== null;
  
  const handleStartReplay = () => {
    replayIndex.value = -1;
  };
  
  const handleStopReplay = () => {
    replayIndex.value = null;
  };
  
  const handlePrevMove = () => {
    if (replay !== null && replay > -1) {
      replayIndex.value = replay - 1;
    }
  };
  
  const handleNextMove = () => {
    if (replay !== null && replay < moves.length - 1) {
      replayIndex.value = replay + 1;
    }
  };
  
  const handleFirstMove = () => {
    replayIndex.value = -1;
  };
  
  const handleLastMove = () => {
    if (moves.length > 0) {
      replayIndex.value = moves.length - 1;
    }
  };
  
  const handleMoveClick = (index: number) => {
    replayIndex.value = index;
  };
  
  // Format move for display
  const formatMove = (move: typeof moves[0]) => {
    const pieceSymbol = {
      'King': 'K',
      'Queen': 'Q',
      'Rook': 'R',
      'Bishop': 'B',
      'Knight': 'N',
      'Pawn': '',
    }[move.piece.piece_type] || '';
    
    const capture = move.captured ? 'x' : '';
    const promotion = move.promotion ? `=${move.promotion[0].toUpperCase()}` : '';
    
    return `${pieceSymbol}${capture}${move.to}${promotion}`;
  };
  
  // Group moves by pairs (white, black)
  const movePairs: { white: typeof moves[0]; black?: typeof moves[0]; number: number }[] = [];
  for (let idx = 0; idx < moves.length; idx += 2) {
    movePairs.push({
      white: moves[idx],
      black: moves[idx + 1],
      number: Math.floor(idx / 2) + 1,
    });
  }
  
  return (
    <div class="bg-gray-800 rounded-lg p-4 w-full max-w-md">
      <h3 class="text-lg font-bold mb-3">Move History</h3>
      
      {/* Replay controls */}
      <div class="flex items-center justify-center gap-2 mb-4">
        {isReplaying ? (
          <>
            <button
              class="p-2 bg-gray-700 rounded hover:bg-gray-600 disabled:opacity-50"
              onClick={handleFirstMove}
              disabled={replay === -1}
            >
              <IconPlayerTrackPrev size={20} />
            </button>
            <button
              class="p-2 bg-gray-700 rounded hover:bg-gray-600 disabled:opacity-50"
              onClick={handlePrevMove}
              disabled={replay === -1}
            >
              <IconPlayerSkipBack size={20} />
            </button>
            <button
              class="p-2 bg-red-600 rounded hover:bg-red-500"
              onClick={handleStopReplay}
            >
              <IconPlayerPause size={20} />
            </button>
            <button
              class="p-2 bg-gray-700 rounded hover:bg-gray-600 disabled:opacity-50"
              onClick={handleNextMove}
              disabled={replay === moves.length - 1}
            >
              <IconPlayerSkipForward size={20} />
            </button>
            <button
              class="p-2 bg-gray-700 rounded hover:bg-gray-600 disabled:opacity-50"
              onClick={handleLastMove}
              disabled={replay === moves.length - 1}
            >
              <IconPlayerTrackNext size={20} />
            </button>
          </>
        ) : (
          <button
            class="flex items-center gap-2 px-4 py-2 bg-blue-600 rounded hover:bg-blue-500 disabled:opacity-50"
            onClick={handleStartReplay}
            disabled={moves.length === 0}
          >
            <IconPlayerPlay size={20} />
            <span>Replay Game</span>
          </button>
        )}
      </div>
      
      {/* Move list */}
      <div class="max-h-64 overflow-y-auto">
        {moves.length === 0 ? (
          <p class="text-gray-400 text-center">No moves yet</p>
        ) : (
          <div class="space-y-1">
            {movePairs.map((pair, idx) => (
              <div key={idx} class="flex items-center gap-2 text-sm">
                <span class="w-8 text-gray-500">{pair.number}.</span>
                <button
                  class={`px-2 py-1 rounded flex-1 text-left ${
                    isReplaying && replay === idx * 2 
                      ? 'bg-blue-600' 
                      : 'hover:bg-gray-700'
                  }`}
                  onClick={() => handleMoveClick(idx * 2)}
                >
                  {formatMove(pair.white)}
                </button>
                {pair.black && (
                  <button
                    class={`px-2 py-1 rounded flex-1 text-left ${
                      isReplaying && replay === idx * 2 + 1 
                        ? 'bg-blue-600' 
                        : 'hover:bg-gray-700'
                    }`}
                    onClick={() => handleMoveClick(idx * 2 + 1)}
                  >
                    {formatMove(pair.black)}
                  </button>
                )}
              </div>
            ))}
          </div>
        )}
      </div>
      
      {isReplaying && (
        <p class="text-yellow-400 text-sm text-center mt-3">
          Replay mode - viewing move {replay === -1 ? 'start' : replay + 1} of {moves.length}
        </p>
      )}
    </div>
  );
}
