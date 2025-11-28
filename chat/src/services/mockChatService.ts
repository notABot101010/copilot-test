import type { ChatApiInterface, Conversation, Message, User } from '../types';

// Mock data
const mockUsers: User[] = [
  { id: '1', name: 'You', avatar: 'https://api.dicebear.com/7.x/avataaars/svg?seed=You', status: 'online' },
  { id: '2', name: 'Alice Johnson', avatar: 'https://api.dicebear.com/7.x/avataaars/svg?seed=Alice', status: 'online' },
  { id: '3', name: 'Bob Smith', avatar: 'https://api.dicebear.com/7.x/avataaars/svg?seed=Bob', status: 'offline', lastSeen: new Date(Date.now() - 3600000) },
  { id: '4', name: 'Charlie Brown', avatar: 'https://api.dicebear.com/7.x/avataaars/svg?seed=Charlie', status: 'online' },
  { id: '5', name: 'Diana Prince', avatar: 'https://api.dicebear.com/7.x/avataaars/svg?seed=Diana', status: 'offline', lastSeen: new Date(Date.now() - 7200000) },
  { id: '6', name: 'Eve Wilson', avatar: 'https://api.dicebear.com/7.x/avataaars/svg?seed=Eve', status: 'online' },
];

const mockMessages: Message[] = [
  { id: 'm1', conversationId: 'c1', senderId: '2', content: 'Hey! How are you doing?', timestamp: new Date(Date.now() - 86400000), status: 'read', type: 'text' },
  { id: 'm2', conversationId: 'c1', senderId: '1', content: 'I\'m doing great, thanks! Just finished that project.', timestamp: new Date(Date.now() - 85000000), status: 'read', type: 'text' },
  { id: 'm3', conversationId: 'c1', senderId: '2', content: 'That\'s awesome! We should celebrate 🎉', timestamp: new Date(Date.now() - 84000000), status: 'read', type: 'text' },
  { id: 'm4', conversationId: 'c1', senderId: '1', content: 'Definitely! Coffee tomorrow?', timestamp: new Date(Date.now() - 83000000), status: 'read', type: 'text' },
  { id: 'm5', conversationId: 'c1', senderId: '2', content: 'Sounds perfect! See you at 10am ☕', timestamp: new Date(Date.now() - 3600000), status: 'read', type: 'text' },
  
  { id: 'm6', conversationId: 'c2', senderId: '3', content: 'Did you see the game last night?', timestamp: new Date(Date.now() - 172800000), status: 'read', type: 'text' },
  { id: 'm7', conversationId: 'c2', senderId: '1', content: 'Yes! It was incredible!', timestamp: new Date(Date.now() - 172000000), status: 'read', type: 'text' },
  { id: 'm8', conversationId: 'c2', senderId: '3', content: 'That last-minute goal was unreal', timestamp: new Date(Date.now() - 7200000), status: 'delivered', type: 'text' },
  
  { id: 'm9', conversationId: 'c3', senderId: '4', content: 'Meeting at 3pm today', timestamp: new Date(Date.now() - 14400000), status: 'read', type: 'text' },
  { id: 'm10', conversationId: 'c3', senderId: '1', content: 'Got it, I\'ll be there!', timestamp: new Date(Date.now() - 14000000), status: 'read', type: 'text' },
  
  { id: 'm11', conversationId: 'c4', senderId: '5', content: 'Happy birthday! 🎂🎁', timestamp: new Date(Date.now() - 259200000), status: 'read', type: 'text' },
  { id: 'm12', conversationId: 'c4', senderId: '1', content: 'Thank you so much! 😊', timestamp: new Date(Date.now() - 259000000), status: 'read', type: 'text' },
  
  { id: 'm13', conversationId: 'c5', senderId: '6', content: 'Can you send me the document?', timestamp: new Date(Date.now() - 1800000), status: 'read', type: 'text' },
  { id: 'm14', conversationId: 'c5', senderId: '1', content: 'Sure, sending it now', timestamp: new Date(Date.now() - 1700000), status: 'read', type: 'text' },
  { id: 'm15', conversationId: 'c5', senderId: '6', content: 'Thanks! Got it 👍', timestamp: new Date(Date.now() - 1600000), status: 'read', type: 'text' },
];

const mockConversations: Conversation[] = [
  {
    id: 'c1',
    participants: [mockUsers[0], mockUsers[1]],
    lastMessage: mockMessages.find(m => m.id === 'm5'),
    unreadCount: 0,
    isGroup: false,
    createdAt: new Date(Date.now() - 604800000),
    updatedAt: new Date(Date.now() - 3600000),
  },
  {
    id: 'c2',
    participants: [mockUsers[0], mockUsers[2]],
    lastMessage: mockMessages.find(m => m.id === 'm8'),
    unreadCount: 1,
    isGroup: false,
    createdAt: new Date(Date.now() - 604800000),
    updatedAt: new Date(Date.now() - 7200000),
  },
  {
    id: 'c3',
    participants: [mockUsers[0], mockUsers[3]],
    lastMessage: mockMessages.find(m => m.id === 'm10'),
    unreadCount: 0,
    isGroup: false,
    createdAt: new Date(Date.now() - 604800000),
    updatedAt: new Date(Date.now() - 14000000),
  },
  {
    id: 'c4',
    participants: [mockUsers[0], mockUsers[4]],
    lastMessage: mockMessages.find(m => m.id === 'm12'),
    unreadCount: 0,
    isGroup: false,
    createdAt: new Date(Date.now() - 604800000),
    updatedAt: new Date(Date.now() - 259000000),
  },
  {
    id: 'c5',
    participants: [mockUsers[0], mockUsers[5]],
    lastMessage: mockMessages.find(m => m.id === 'm15'),
    unreadCount: 0,
    isGroup: false,
    createdAt: new Date(Date.now() - 604800000),
    updatedAt: new Date(Date.now() - 1600000),
  },
];

// In-memory storage for dynamic data
let messages = [...mockMessages];
let conversations = [...mockConversations];
let messageIdCounter = 100;

// Simulate network delay
const delay = (ms: number) => new Promise(resolve => setTimeout(resolve, ms));

export class MockChatService implements ChatApiInterface {
  async getCurrentUser(): Promise<User> {
    await delay(100);
    return mockUsers[0];
  }

  async listConversations(): Promise<Conversation[]> {
    await delay(200);
    return conversations.sort((a, b) => b.updatedAt.getTime() - a.updatedAt.getTime());
  }

  async listMessagesForConversation(conversationId: string, limit = 50, _before?: string): Promise<Message[]> {
    await delay(150);
    return messages
      .filter(m => m.conversationId === conversationId)
      .sort((a, b) => a.timestamp.getTime() - b.timestamp.getTime())
      .slice(-limit);
  }

  async sendMessage(conversationId: string, content: string, type: Message['type'] = 'text'): Promise<Message> {
    await delay(100);
    const newMessage: Message = {
      id: `m${++messageIdCounter}`,
      conversationId,
      senderId: '1',
      content,
      timestamp: new Date(),
      status: 'sent',
      type,
    };
    
    messages.push(newMessage);
    
    // Update conversation
    const conversation = conversations.find(c => c.id === conversationId);
    if (conversation) {
      conversation.lastMessage = newMessage;
      conversation.updatedAt = new Date();
    }
    
    // Simulate status updates
    setTimeout(() => {
      newMessage.status = 'delivered';
    }, 500);
    
    setTimeout(() => {
      newMessage.status = 'read';
    }, 1500);
    
    return newMessage;
  }

  async markAsRead(conversationId: string, _messageId: string): Promise<void> {
    await delay(50);
    const conversation = conversations.find(c => c.id === conversationId);
    if (conversation) {
      conversation.unreadCount = 0;
    }
  }

  async getConversation(conversationId: string): Promise<Conversation | null> {
    await delay(100);
    return conversations.find(c => c.id === conversationId) || null;
  }

  async createConversation(participantIds: string[]): Promise<Conversation> {
    await delay(200);
    const participants = mockUsers.filter(u => participantIds.includes(u.id) || u.id === '1');
    const newConversation: Conversation = {
      id: `c${Date.now()}`,
      participants,
      unreadCount: 0,
      isGroup: participants.length > 2,
      createdAt: new Date(),
      updatedAt: new Date(),
    };
    conversations.push(newConversation);
    return newConversation;
  }

  async searchUsers(query: string): Promise<User[]> {
    await delay(100);
    const lowerQuery = query.toLowerCase();
    return mockUsers.filter(u => 
      u.id !== '1' && u.name.toLowerCase().includes(lowerQuery)
    );
  }
}

export const chatService = new MockChatService();
