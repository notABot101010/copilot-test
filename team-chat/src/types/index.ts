export interface User {
  id: string;
  username: string;
  displayName: string;
  avatar: string;
  status: 'online' | 'idle' | 'dnd' | 'offline';
  isBot?: boolean;
}

export interface Server {
  id: string;
  name: string;
  icon: string;
  ownerId: string;
  channels: Channel[];
  members: User[];
  createdAt: Date;
}

export interface Channel {
  id: string;
  serverId: string;
  name: string;
  type: 'text' | 'voice' | 'announcement';
  category?: string;
  topic?: string;
  position: number;
}

export interface Message {
  id: string;
  channelId: string;
  author: User;
  content: string;
  timestamp: Date;
  editedTimestamp?: Date;
  attachments?: Attachment[];
  reactions?: Reaction[];
  replyTo?: Message;
  isPinned?: boolean;
}

export interface Attachment {
  id: string;
  filename: string;
  url: string;
  contentType: string;
  size: number;
}

export interface Reaction {
  emoji: string;
  count: number;
  users: string[];
}

export interface DirectMessage {
  id: string;
  participants: User[];
  lastMessage?: Message;
  unreadCount: number;
}

export interface ChatApiInterface {
  // User
  getCurrentUser(): Promise<User>;
  
  // Servers
  listServers(): Promise<Server[]>;
  getServer(serverId: string): Promise<Server | null>;
  
  // Channels
  listChannelsForServer(serverId: string): Promise<Channel[]>;
  getChannel(channelId: string): Promise<Channel | null>;
  
  // Messages
  listMessagesForChannel(channelId: string, limit?: number, before?: string): Promise<Message[]>;
  sendMessage(channelId: string, content: string): Promise<Message>;
  editMessage(messageId: string, content: string): Promise<Message>;
  deleteMessage(messageId: string): Promise<void>;
  
  // Members
  listMembersForServer(serverId: string): Promise<User[]>;
  
  // Direct Messages
  listDirectMessages(): Promise<DirectMessage[]>;
  
  // Reactions
  addReaction(messageId: string, emoji: string): Promise<void>;
  removeReaction(messageId: string, emoji: string): Promise<void>;
}
