export interface User {
  id: string;
  name: string;
  avatar: string;
  status: 'online' | 'offline' | 'typing';
  lastSeen?: Date;
}

export interface Message {
  id: string;
  conversationId: string;
  senderId: string;
  content: string;
  timestamp: Date;
  status: 'sent' | 'delivered' | 'read';
  type: 'text' | 'image' | 'audio' | 'document';
}

export interface Conversation {
  id: string;
  participants: User[];
  lastMessage?: Message;
  unreadCount: number;
  isGroup: boolean;
  groupName?: string;
  groupAvatar?: string;
  createdAt: Date;
  updatedAt: Date;
}

export interface ChatApiInterface {
  getCurrentUser(): Promise<User>;
  listConversations(): Promise<Conversation[]>;
  listMessagesForConversation(conversationId: string, limit?: number, before?: string): Promise<Message[]>;
  sendMessage(conversationId: string, content: string, type?: Message['type']): Promise<Message>;
  markAsRead(conversationId: string, messageId: string): Promise<void>;
  getConversation(conversationId: string): Promise<Conversation | null>;
  createConversation(participantIds: string[]): Promise<Conversation>;
  searchUsers(query: string): Promise<User[]>;
}
