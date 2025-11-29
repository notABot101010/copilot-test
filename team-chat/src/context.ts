import { createContext } from 'preact';
import type { Server, User, DirectMessage } from './types';
import type { ChatApiInterface } from './types';

export interface ChatContextValue {
  currentUser: User | null;
  servers: Server[];
  directMessages: DirectMessage[];
  chatService: ChatApiInterface;
  addServer: (server: Server) => void;
}

export const ChatContext = createContext<ChatContextValue | null>(null);
