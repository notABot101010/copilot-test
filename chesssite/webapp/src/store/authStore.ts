import { signal, computed } from '@preact/signals';

export interface User {
  username: string;
  token: string;
}

export interface MatchInfo {
  id: string;
  white_player: string;
  black_player: string;
  status: string;
  createdAt: number;
  updatedAt: number;
}

export interface ChessMove {
  from: string;
  to: string;
  piece: { piece_type: string; color: string };
  captured: { piece_type: string; color: string } | null;
  promotion: string | null;
  timestamp: number;
}

export interface ChessState {
  board: (Piece | null)[][];
  current_turn: string;
  moves: ChessMove[];
  status: string;
  white_can_castle_kingside: boolean;
  white_can_castle_queenside: boolean;
  black_can_castle_kingside: boolean;
  black_can_castle_queenside: boolean;
  en_passant_target: [number, number] | null;
}

export interface Piece {
  piece_type: string;
  color: string;
}

// Auth state
export const currentUser = signal<User | null>(null);

// Matches state
export const matches = signal<MatchInfo[]>([]);
export const matchesLoading = signal(false);

// Current match state
export const currentMatch = signal<{
  id: string;
  white_player: string;
  black_player: string;
  status: string;
  chessState: ChessState | null;
} | null>(null);

// Replay state
export const replayIndex = signal<number | null>(null);
export const isReplaying = computed(() => replayIndex.value !== null);

// Selected square for moves
export const selectedSquare = signal<string | null>(null);

// WebSocket connection
export const wsConnection = signal<WebSocket | null>(null);

// Initialize from localStorage
export function initAuth() {
  const stored = localStorage.getItem('chesssite_user');
  if (stored) {
    try {
      currentUser.value = JSON.parse(stored);
    } catch {
      localStorage.removeItem('chesssite_user');
    }
  }
}

export async function login(username: string, password: string): Promise<boolean> {
  const response = await fetch('/api/login', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ username, password }),
  });

  if (response.ok) {
    const data = await response.json();
    const user: User = { username: data.username, token: data.token };
    currentUser.value = user;
    localStorage.setItem('chesssite_user', JSON.stringify(user));
    return true;
  }
  return false;
}

export async function register(username: string, password: string): Promise<boolean> {
  const response = await fetch('/api/register', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ username, password }),
  });

  if (response.ok) {
    return await login(username, password);
  }
  return false;
}

export async function logout() {
  const user = currentUser.value;
  if (user) {
    await fetch('/api/logout', {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${user.token}`,
      },
    });
  }
  currentUser.value = null;
  localStorage.removeItem('chesssite_user');
}

export async function fetchMatches() {
  const user = currentUser.value;
  if (!user) return;

  matchesLoading.value = true;
  try {
    const response = await fetch('/api/matches', {
      headers: {
        'Authorization': `Bearer ${user.token}`,
      },
    });

    if (response.ok) {
      const data = await response.json();
      matches.value = data.matches;
    }
  } finally {
    matchesLoading.value = false;
  }
}

export async function createMatch(opponentUsername: string): Promise<string | null> {
  const user = currentUser.value;
  if (!user) return null;

  const response = await fetch('/api/matches', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${user.token}`,
    },
    body: JSON.stringify({ opponent_username: opponentUsername }),
  });

  if (response.ok) {
    const data = await response.json();
    await fetchMatches();
    return data.id;
  }
  return null;
}

export async function fetchMatch(matchId: string) {
  const response = await fetch(`/api/matches/${matchId}`);

  if (response.ok) {
    const data = await response.json();
    
    // Decode automerge document
    const binaryStr = atob(data.document);
    const bytes = new Uint8Array(binaryStr.length);
    for (let idx = 0; idx < binaryStr.length; idx++) {
      bytes[idx] = binaryStr.charCodeAt(idx);
    }
    
    // Import automerge dynamically
    const Automerge = await import('@automerge/automerge');
    const doc = Automerge.load(bytes);
    
    const chessStateJson = (doc as Record<string, unknown>).chessState as string;
    const chessState = chessStateJson ? JSON.parse(chessStateJson) : null;
    
    currentMatch.value = {
      id: data.id,
      white_player: data.white_player,
      black_player: data.black_player,
      status: data.status,
      chessState,
    };
    
    // Reset replay state
    replayIndex.value = null;
    selectedSquare.value = null;
  }
}

export async function makeMove(matchId: string, from: string, to: string, promotion?: string): Promise<boolean> {
  const user = currentUser.value;
  if (!user) return false;

  const body: Record<string, string> = { from, to };
  if (promotion) {
    body.promotion = promotion;
  }

  const response = await fetch(`/api/matches/${matchId}/move`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${user.token}`,
    },
    body: JSON.stringify(body),
  });

  if (response.ok) {
    const data = await response.json();
    if (data.success) {
      // Decode updated document
      const binaryStr = atob(data.document);
      const bytes = new Uint8Array(binaryStr.length);
      for (let idx = 0; idx < binaryStr.length; idx++) {
        bytes[idx] = binaryStr.charCodeAt(idx);
      }
      
      const Automerge = await import('@automerge/automerge');
      const doc = Automerge.load(bytes);
      
      const chessStateJson = (doc as Record<string, unknown>).chessState as string;
      const chessState = chessStateJson ? JSON.parse(chessStateJson) : null;
      
      if (currentMatch.value) {
        currentMatch.value = {
          ...currentMatch.value,
          chessState,
          status: chessState?.status || currentMatch.value.status,
        };
      }
      
      selectedSquare.value = null;
      return true;
    }
  }
  return false;
}

export function connectToMatch(matchId: string) {
  // Close existing connection
  if (wsConnection.value) {
    wsConnection.value.close();
    wsConnection.value = null;
  }

  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  const wsUrl = `${protocol}//${window.location.host}/ws/matches/${matchId}`;
  const ws = new WebSocket(wsUrl);

  ws.onopen = () => {
    const user = currentUser.value;
    if (user) {
      ws.send(JSON.stringify({ type: 'identify', client_id: user.username }));
    }
  };

  ws.onmessage = async (event) => {
    try {
      const msg = JSON.parse(event.data);
      
      if (msg.type === 'connected' || msg.type === 'sync') {
        // Decode updated document
        const binaryStr = atob(msg.document);
        const bytes = new Uint8Array(binaryStr.length);
        for (let idx = 0; idx < binaryStr.length; idx++) {
          bytes[idx] = binaryStr.charCodeAt(idx);
        }
        
        const Automerge = await import('@automerge/automerge');
        const doc = Automerge.load(bytes);
        
        const chessStateJson = (doc as Record<string, unknown>).chessState as string;
        const chessState = chessStateJson ? JSON.parse(chessStateJson) : null;
        
        if (currentMatch.value && currentMatch.value.id === matchId) {
          currentMatch.value = {
            ...currentMatch.value,
            chessState,
            status: chessState?.status || currentMatch.value.status,
          };
        }
      }
    } catch (err) {
      console.error('WebSocket message error:', err);
    }
  };

  ws.onclose = () => {
    wsConnection.value = null;
  };

  wsConnection.value = ws;
}

export function disconnectFromMatch() {
  if (wsConnection.value) {
    wsConnection.value.close();
    wsConnection.value = null;
  }
}

export async function fetchUsers(): Promise<string[]> {
  const response = await fetch('/api/users');
  if (response.ok) {
    const data = await response.json();
    return data.users;
  }
  return [];
}
