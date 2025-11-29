import type { ChatApiInterface, Server, Channel, Message, User, DirectMessage } from '../types';

// Mock Users
const mockUsers: User[] = [
  { id: '1', username: 'you', displayName: 'You', avatar: 'https://api.dicebear.com/7.x/avataaars/svg?seed=You', status: 'online' },
  { id: '2', username: 'alice', displayName: 'Alice Johnson', avatar: 'https://api.dicebear.com/7.x/avataaars/svg?seed=Alice', status: 'online' },
  { id: '3', username: 'bob', displayName: 'Bob Smith', avatar: 'https://api.dicebear.com/7.x/avataaars/svg?seed=Bob', status: 'idle' },
  { id: '4', username: 'charlie', displayName: 'Charlie Brown', avatar: 'https://api.dicebear.com/7.x/avataaars/svg?seed=Charlie', status: 'dnd' },
  { id: '5', username: 'diana', displayName: 'Diana Prince', avatar: 'https://api.dicebear.com/7.x/avataaars/svg?seed=Diana', status: 'offline' },
  { id: '6', username: 'eve', displayName: 'Eve Wilson', avatar: 'https://api.dicebear.com/7.x/avataaars/svg?seed=Eve', status: 'online' },
  { id: '7', username: 'frank', displayName: 'Frank Castle', avatar: 'https://api.dicebear.com/7.x/avataaars/svg?seed=Frank', status: 'online' },
  { id: '8', username: 'discordbot', displayName: 'Discord Bot', avatar: 'https://api.dicebear.com/7.x/bottts/svg?seed=Bot', status: 'online', isBot: true },
];

// Mock Channels
const mockChannels: Channel[] = [
  // Server 1 - Gaming Hub
  { id: 'ch1', serverId: 's1', name: 'welcome', type: 'text', category: 'Information', position: 0 },
  { id: 'ch2', serverId: 's1', name: 'rules', type: 'text', category: 'Information', position: 1 },
  { id: 'ch3', serverId: 's1', name: 'announcements', type: 'announcement', category: 'Information', position: 2 },
  { id: 'ch4', serverId: 's1', name: 'general', type: 'text', category: 'Text Channels', position: 3, topic: 'General discussion about anything' },
  { id: 'ch5', serverId: 's1', name: 'gaming', type: 'text', category: 'Text Channels', position: 4, topic: 'Share your gaming moments!' },
  { id: 'ch6', serverId: 's1', name: 'memes', type: 'text', category: 'Text Channels', position: 5 },
  { id: 'ch7', serverId: 's1', name: 'General', type: 'voice', category: 'Voice Channels', position: 6 },
  { id: 'ch8', serverId: 's1', name: 'Gaming', type: 'voice', category: 'Voice Channels', position: 7 },
  
  // Server 2 - Dev Team
  { id: 'ch9', serverId: 's2', name: 'general', type: 'text', category: 'General', position: 0 },
  { id: 'ch10', serverId: 's2', name: 'backend', type: 'text', category: 'Development', position: 1, topic: 'Backend development discussions' },
  { id: 'ch11', serverId: 's2', name: 'frontend', type: 'text', category: 'Development', position: 2, topic: 'Frontend development discussions' },
  { id: 'ch12', serverId: 's2', name: 'code-review', type: 'text', category: 'Development', position: 3 },
  { id: 'ch13', serverId: 's2', name: 'standup', type: 'voice', category: 'Voice', position: 4 },
  
  // Server 3 - Music Lovers
  { id: 'ch14', serverId: 's3', name: 'chat', type: 'text', category: 'General', position: 0 },
  { id: 'ch15', serverId: 's3', name: 'share-music', type: 'text', category: 'General', position: 1 },
  { id: 'ch16', serverId: 's3', name: 'Music Room', type: 'voice', category: 'Voice', position: 2 },
];

// Mock Servers
const mockServers: Server[] = [
  {
    id: 's1',
    name: 'Gaming Hub',
    icon: 'https://api.dicebear.com/7.x/identicon/svg?seed=Gaming',
    ownerId: '2',
    channels: mockChannels.filter(c => c.serverId === 's1'),
    members: [mockUsers[0], mockUsers[1], mockUsers[2], mockUsers[3], mockUsers[4], mockUsers[7]],
    createdAt: new Date(Date.now() - 30 * 24 * 60 * 60 * 1000),
  },
  {
    id: 's2',
    name: 'Dev Team',
    icon: 'https://api.dicebear.com/7.x/identicon/svg?seed=DevTeam',
    ownerId: '1',
    channels: mockChannels.filter(c => c.serverId === 's2'),
    members: [mockUsers[0], mockUsers[1], mockUsers[5], mockUsers[6]],
    createdAt: new Date(Date.now() - 60 * 24 * 60 * 60 * 1000),
  },
  {
    id: 's3',
    name: 'Music Lovers',
    icon: 'https://api.dicebear.com/7.x/identicon/svg?seed=Music',
    ownerId: '4',
    channels: mockChannels.filter(c => c.serverId === 's3'),
    members: [mockUsers[0], mockUsers[2], mockUsers[3], mockUsers[4], mockUsers[5]],
    createdAt: new Date(Date.now() - 15 * 24 * 60 * 60 * 1000),
  },
];

// Mock Messages
const mockMessages: Message[] = [
  // General channel (ch4)
  { id: 'm1', channelId: 'ch4', author: mockUsers[1], content: 'Hey everyone! Welcome to the server! 🎮', timestamp: new Date(Date.now() - 86400000), reactions: [{ emoji: '👋', count: 3, users: ['2', '3', '4'] }] },
  { id: 'm2', channelId: 'ch4', author: mockUsers[2], content: 'Thanks for having me here!', timestamp: new Date(Date.now() - 85000000) },
  { id: 'm3', channelId: 'ch4', author: mockUsers[0], content: 'Glad to be part of this community', timestamp: new Date(Date.now() - 84000000) },
  { id: 'm4', channelId: 'ch4', author: mockUsers[3], content: 'Anyone up for some games tonight?', timestamp: new Date(Date.now() - 3600000) },
  { id: 'm5', channelId: 'ch4', author: mockUsers[1], content: 'Count me in! What are we playing?', timestamp: new Date(Date.now() - 3500000) },
  { id: 'm6', channelId: 'ch4', author: mockUsers[7], content: '🤖 Welcome to Gaming Hub! Remember to check out the #rules channel.', timestamp: new Date(Date.now() - 3400000) },
  
  // Gaming channel (ch5)
  { id: 'm7', channelId: 'ch5', author: mockUsers[2], content: 'Just got a victory royale! 🏆', timestamp: new Date(Date.now() - 7200000), reactions: [{ emoji: '🎉', count: 5, users: ['1', '2', '3', '4', '5'] }] },
  { id: 'm8', channelId: 'ch5', author: mockUsers[3], content: 'Nice one! Share the clip!', timestamp: new Date(Date.now() - 7100000) },
  { id: 'm9', channelId: 'ch5', author: mockUsers[0], content: 'GG! That was intense', timestamp: new Date(Date.now() - 7000000) },
  
  // Dev Team general (ch9)
  { id: 'm10', channelId: 'ch9', author: mockUsers[5], content: 'Good morning team! ☕', timestamp: new Date(Date.now() - 14400000) },
  { id: 'm11', channelId: 'ch9', author: mockUsers[0], content: 'Morning! Ready for the sprint review?', timestamp: new Date(Date.now() - 14300000) },
  { id: 'm12', channelId: 'ch9', author: mockUsers[6], content: 'Just pushed my PR, can someone review?', timestamp: new Date(Date.now() - 14200000), reactions: [{ emoji: '👍', count: 2, users: ['1', '6'] }] },
  
  // Frontend channel (ch11)
  { id: 'm13', channelId: 'ch11', author: mockUsers[0], content: 'Working on the new dashboard design', timestamp: new Date(Date.now() - 1800000) },
  { id: 'm14', channelId: 'ch11', author: mockUsers[5], content: 'Looking forward to seeing it! Need any help?', timestamp: new Date(Date.now() - 1700000) },
  { id: 'm15', channelId: 'ch11', author: mockUsers[0], content: 'I might need some help with the animations later', timestamp: new Date(Date.now() - 1600000) },
  
  // Music chat (ch14)
  { id: 'm16', channelId: 'ch14', author: mockUsers[3], content: 'Anyone listened to the new album?', timestamp: new Date(Date.now() - 259200000) },
  { id: 'm17', channelId: 'ch14', author: mockUsers[4], content: 'Yes! It\'s incredible, especially track 5', timestamp: new Date(Date.now() - 259100000) },
  { id: 'm18', channelId: 'ch14', author: mockUsers[2], content: 'Going to check it out now', timestamp: new Date(Date.now() - 259000000) },
];

// Mock Direct Messages
const mockDirectMessages: DirectMessage[] = [
  { id: 'dm1', participants: [mockUsers[0], mockUsers[1]], unreadCount: 2 },
  { id: 'dm2', participants: [mockUsers[0], mockUsers[2]], unreadCount: 0 },
  { id: 'dm3', participants: [mockUsers[0], mockUsers[5]], unreadCount: 1 },
];

// In-memory storage for dynamic data
let messages = [...mockMessages];
let messageIdCounter = 100;

// Simulate network delay
const delay = (ms: number) => new Promise(resolve => setTimeout(resolve, ms));

export class MockChatService implements ChatApiInterface {
  async getCurrentUser(): Promise<User> {
    await delay(100);
    return mockUsers[0];
  }

  async listServers(): Promise<Server[]> {
    await delay(200);
    return mockServers;
  }

  async getServer(serverId: string): Promise<Server | null> {
    await delay(100);
    return mockServers.find(s => s.id === serverId) || null;
  }

  async listChannelsForServer(serverId: string): Promise<Channel[]> {
    await delay(150);
    return mockChannels.filter(c => c.serverId === serverId).sort((a, b) => a.position - b.position);
  }

  async getChannel(channelId: string): Promise<Channel | null> {
    await delay(100);
    return mockChannels.find(c => c.id === channelId) || null;
  }

  async listMessagesForChannel(channelId: string, limit = 50, _before?: string): Promise<Message[]> {
    await delay(150);
    return messages
      .filter(m => m.channelId === channelId)
      .sort((a, b) => a.timestamp.getTime() - b.timestamp.getTime())
      .slice(-limit);
  }

  async sendMessage(channelId: string, content: string): Promise<Message> {
    await delay(100);
    const newMessage: Message = {
      id: `m${++messageIdCounter}`,
      channelId,
      author: mockUsers[0],
      content,
      timestamp: new Date(),
    };
    
    messages.push(newMessage);
    return newMessage;
  }

  async editMessage(messageId: string, content: string): Promise<Message> {
    await delay(100);
    const message = messages.find(m => m.id === messageId);
    if (!message) throw new Error('Message not found');
    
    message.content = content;
    message.editedTimestamp = new Date();
    return message;
  }

  async deleteMessage(messageId: string): Promise<void> {
    await delay(100);
    messages = messages.filter(m => m.id !== messageId);
  }

  async listMembersForServer(serverId: string): Promise<User[]> {
    await delay(150);
    const server = mockServers.find(s => s.id === serverId);
    return server?.members || [];
  }

  async listDirectMessages(): Promise<DirectMessage[]> {
    await delay(150);
    return mockDirectMessages;
  }

  async addReaction(messageId: string, emoji: string): Promise<void> {
    await delay(50);
    const message = messages.find(m => m.id === messageId);
    if (message) {
      if (!message.reactions) message.reactions = [];
      const existingReaction = message.reactions.find(r => r.emoji === emoji);
      if (existingReaction) {
        if (!existingReaction.users.includes('1')) {
          existingReaction.count++;
          existingReaction.users.push('1');
        }
      } else {
        message.reactions.push({ emoji, count: 1, users: ['1'] });
      }
    }
  }

  async removeReaction(messageId: string, emoji: string): Promise<void> {
    await delay(50);
    const message = messages.find(m => m.id === messageId);
    if (message && message.reactions) {
      const reaction = message.reactions.find(r => r.emoji === emoji);
      if (reaction && reaction.users.includes('1')) {
        reaction.count--;
        reaction.users = reaction.users.filter(u => u !== '1');
        if (reaction.count === 0) {
          message.reactions = message.reactions.filter(r => r.emoji !== emoji);
        }
      }
    }
  }
}

export const chatService = new MockChatService();
