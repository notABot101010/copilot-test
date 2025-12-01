import {
  IconChessKing,
  IconChessQueen,
  IconChessRook,
  IconChessBishop,
  IconChessKnight,
  IconChess,
} from '@tabler/icons-preact';
import { currentMatch, currentUser, selectedSquare, makeMove, replayIndex, ChessState, Piece } from '../store/authStore';
import { useSignal } from '@preact/signals';

interface ChessBoardProps {
  matchId: string;
}

function getPieceIcon(piece: Piece, size: number = 40) {
  const color = piece.color === 'White' ? '#ffffff' : '#1a1a1a';
  const stroke = piece.color === 'White' ? '#1a1a1a' : '#ffffff';
  
  const props = { size, color, stroke, strokeWidth: 1.5 };
  
  switch (piece.piece_type) {
    case 'King':
      return <IconChessKing {...props} />;
    case 'Queen':
      return <IconChessQueen {...props} />;
    case 'Rook':
      return <IconChessRook {...props} />;
    case 'Bishop':
      return <IconChessBishop {...props} />;
    case 'Knight':
      return <IconChessKnight {...props} />;
    case 'Pawn':
      return <IconChess {...props} />;
    default:
      return null;
  }
}

function squareToNotation(row: number, col: number): string {
  return `${String.fromCharCode(97 + col)}${row + 1}`;
}

function notationToSquare(notation: string): [number, number] | null {
  if (notation.length !== 2) return null;
  const col = notation.charCodeAt(0) - 97;
  const row = parseInt(notation[1]) - 1;
  if (col < 0 || col > 7 || row < 0 || row > 7) return null;
  return [row, col];
}

function getBoardStateAtMove(chessState: ChessState, moveIndex: number): (Piece | null)[][] {
  // Start with initial board and replay moves up to moveIndex
  const board: (Piece | null)[][] = Array(8).fill(null).map(() => Array(8).fill(null));
  
  // Initial setup
  // White pieces
  board[0][0] = { piece_type: 'Rook', color: 'White' };
  board[0][1] = { piece_type: 'Knight', color: 'White' };
  board[0][2] = { piece_type: 'Bishop', color: 'White' };
  board[0][3] = { piece_type: 'Queen', color: 'White' };
  board[0][4] = { piece_type: 'King', color: 'White' };
  board[0][5] = { piece_type: 'Bishop', color: 'White' };
  board[0][6] = { piece_type: 'Knight', color: 'White' };
  board[0][7] = { piece_type: 'Rook', color: 'White' };
  for (let col = 0; col < 8; col++) {
    board[1][col] = { piece_type: 'Pawn', color: 'White' };
  }
  
  // Black pieces
  board[7][0] = { piece_type: 'Rook', color: 'Black' };
  board[7][1] = { piece_type: 'Knight', color: 'Black' };
  board[7][2] = { piece_type: 'Bishop', color: 'Black' };
  board[7][3] = { piece_type: 'Queen', color: 'Black' };
  board[7][4] = { piece_type: 'King', color: 'Black' };
  board[7][5] = { piece_type: 'Bishop', color: 'Black' };
  board[7][6] = { piece_type: 'Knight', color: 'Black' };
  board[7][7] = { piece_type: 'Rook', color: 'Black' };
  for (let col = 0; col < 8; col++) {
    board[6][col] = { piece_type: 'Pawn', color: 'Black' };
  }
  
  // Apply moves up to moveIndex
  for (let idx = 0; idx <= moveIndex && idx < chessState.moves.length; idx++) {
    const move = chessState.moves[idx];
    const from = notationToSquare(move.from);
    const to = notationToSquare(move.to);
    
    if (from && to) {
      const piece = board[from[0]][from[1]];
      board[from[0]][from[1]] = null;
      
      // Handle castling
      if (piece?.piece_type === 'King') {
        const colDiff = to[1] - from[1];
        if (colDiff === 2) {
          // Kingside castling
          board[from[0]][5] = board[from[0]][7];
          board[from[0]][7] = null;
        } else if (colDiff === -2) {
          // Queenside castling
          board[from[0]][3] = board[from[0]][0];
          board[from[0]][0] = null;
        }
      }
      
      // Handle en passant
      if (piece?.piece_type === 'Pawn' && from[1] !== to[1] && board[to[0]][to[1]] === null) {
        // En passant capture
        const capturedRow = piece.color === 'White' ? to[0] - 1 : to[0] + 1;
        board[capturedRow][to[1]] = null;
      }
      
      // Handle promotion
      if (move.promotion && piece) {
        const promotionType = move.promotion.charAt(0).toUpperCase() + move.promotion.slice(1).toLowerCase();
        board[to[0]][to[1]] = { piece_type: promotionType, color: piece.color };
      } else {
        board[to[0]][to[1]] = piece;
      }
    }
  }
  
  return board;
}

export function ChessBoard({ matchId }: ChessBoardProps) {
  const match = currentMatch.value;
  const user = currentUser.value;
  const selected = selectedSquare.value;
  const replay = replayIndex.value;
  const promotionDialog = useSignal<{ from: string; to: string } | null>(null);
  
  if (!match || !match.chessState) {
    return <div class="flex items-center justify-center h-64">Loading...</div>;
  }
  
  const chessState = match.chessState;
  const isReplayMode = replay !== null;
  
  // Get board state based on replay mode
  const displayBoard = isReplayMode
    ? getBoardStateAtMove(chessState, replay)
    : chessState.board;
  
  // Determine player's color
  const playerColor = user?.username === match.white_player ? 'White' : 
                      user?.username === match.black_player ? 'Black' : null;
  
  // Flip board for black player
  const flipBoard = playerColor === 'Black';
  
  // Check if it's player's turn
  const isMyTurn = playerColor && chessState.current_turn === playerColor && !isReplayMode;
  
  // Game status
  const isGameOver = chessState.status.startsWith('checkmate:') || chessState.status === 'stalemate';
  
  const handleSquareClick = async (row: number, col: number) => {
    if (isReplayMode || isGameOver || !isMyTurn) return;
    
    const notation = squareToNotation(row, col);
    const piece = chessState.board[row][col];
    
    if (selected) {
      if (selected === notation) {
        // Deselect
        selectedSquare.value = null;
      } else {
        // Try to move
        const fromSquare = notationToSquare(selected);
        if (fromSquare) {
          const movingPiece = chessState.board[fromSquare[0]][fromSquare[1]];
          
          // Check for pawn promotion
          const isPawnPromotion = movingPiece?.piece_type === 'Pawn' && 
            ((movingPiece.color === 'White' && row === 7) || 
             (movingPiece.color === 'Black' && row === 0));
          
          if (isPawnPromotion) {
            promotionDialog.value = { from: selected, to: notation };
          } else {
            await makeMove(matchId, selected, notation);
          }
        }
      }
    } else if (piece && piece.color === playerColor) {
      // Select piece
      selectedSquare.value = notation;
    }
  };
  
  const handlePromotion = async (pieceType: string) => {
    if (promotionDialog.value) {
      await makeMove(matchId, promotionDialog.value.from, promotionDialog.value.to, pieceType);
      promotionDialog.value = null;
    }
  };
  
  // Render board
  const rows = flipBoard ? [0, 1, 2, 3, 4, 5, 6, 7] : [7, 6, 5, 4, 3, 2, 1, 0];
  const cols = flipBoard ? [7, 6, 5, 4, 3, 2, 1, 0] : [0, 1, 2, 3, 4, 5, 6, 7];
  
  return (
    <div class="flex flex-col items-center">
      {/* Status display */}
      <div class="mb-4 text-center">
        {isGameOver ? (
          <p class="text-xl font-bold">
            {chessState.status.startsWith('checkmate:') 
              ? `Checkmate! ${chessState.status.split(':')[1] === 'white' ? 'White' : 'Black'} wins!`
              : 'Stalemate! Draw.'}
          </p>
        ) : (
          <p class="text-lg">
            {chessState.status === 'check' && <span class="text-red-500 font-bold mr-2">Check!</span>}
            {isMyTurn ? "Your turn" : `${chessState.current_turn}'s turn`}
          </p>
        )}
      </div>
      
      {/* Chess board */}
      <div class="border-4 border-gray-700 rounded-lg overflow-hidden shadow-lg">
        {rows.map((row) => (
          <div class="flex" key={row}>
            {cols.map((col) => {
              const isLight = (row + col) % 2 === 0;
              const piece = displayBoard[row][col];
              const notation = squareToNotation(row, col);
              const isSelected = selected === notation;
              
              // Highlight last move
              const lastMove = chessState.moves.length > 0 ? chessState.moves[chessState.moves.length - 1] : null;
              const isLastMoveSquare = lastMove && (lastMove.from === notation || lastMove.to === notation);
              
              return (
                <div
                  key={col}
                  class={`w-12 h-12 md:w-16 md:h-16 flex items-center justify-center cursor-pointer transition-colors
                    ${isLight ? 'bg-amber-100' : 'bg-amber-700'}
                    ${isSelected ? 'ring-4 ring-blue-500 ring-inset' : ''}
                    ${isLastMoveSquare && !isReplayMode ? 'bg-opacity-75 ring-2 ring-green-400' : ''}
                    ${isMyTurn && !isGameOver ? 'hover:brightness-110' : ''}
                  `}
                  onClick={() => handleSquareClick(row, col)}
                >
                  {piece && getPieceIcon(piece)}
                </div>
              );
            })}
          </div>
        ))}
      </div>
      
      {/* Column labels */}
      <div class="flex mt-1">
        {(flipBoard ? ['h', 'g', 'f', 'e', 'd', 'c', 'b', 'a'] : ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h']).map(letter => (
          <div key={letter} class="w-12 md:w-16 text-center text-gray-400 text-sm">{letter}</div>
        ))}
      </div>
      
      {/* Players info */}
      <div class="mt-4 flex justify-between w-full max-w-md">
        <div class={`flex items-center gap-2 ${chessState.current_turn === 'White' && !isGameOver ? 'font-bold' : ''}`}>
          <div class="w-4 h-4 bg-white border border-gray-400 rounded"></div>
          <span>{match.white_player}</span>
        </div>
        <div class={`flex items-center gap-2 ${chessState.current_turn === 'Black' && !isGameOver ? 'font-bold' : ''}`}>
          <div class="w-4 h-4 bg-gray-900 border border-gray-400 rounded"></div>
          <span>{match.black_player}</span>
        </div>
      </div>
      
      {/* Promotion dialog */}
      {promotionDialog.value && (
        <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div class="bg-gray-800 p-6 rounded-lg shadow-xl">
            <h3 class="text-lg font-bold mb-4 text-center">Choose promotion piece</h3>
            <div class="flex gap-4">
              {['queen', 'rook', 'bishop', 'knight'].map(pieceType => {
                const piece = { piece_type: pieceType.charAt(0).toUpperCase() + pieceType.slice(1), color: playerColor || 'White' };
                return (
                  <button
                    key={pieceType}
                    class="p-2 bg-amber-200 rounded hover:bg-amber-300 transition-colors"
                    onClick={() => handlePromotion(pieceType)}
                  >
                    {getPieceIcon(piece, 48)}
                  </button>
                );
              })}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
