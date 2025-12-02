import { computed, signal } from '@preact/signals';
import { exploreSuggestions, seededConversations } from '../data/mockData';
import type { Conversation, ExploreChannel, Message } from '../data/mockData';

export const conversations = signal<Conversation[]>(seededConversations);
export const drafts = signal<Record<string, string>>({});
export const searchTerm = signal('');
export const settingsState = signal({
  notifications: true,
  readReceipts: true,
  dataSaver: false,
  compactMode: false,
});

export const discoveryChannels = signal<ExploreChannel[]>(exploreSuggestions);

export const filteredConversations = computed(() => {
  const term = searchTerm.value.trim().toLowerCase();
  const items = conversations.value.slice().sort((left, right) => Number(Boolean(right.pinned)) - Number(Boolean(left.pinned)));
  if (!term) {
    return items;
  }
  return items.filter((conversation) => {
    return (
      conversation.title.toLowerCase().includes(term) ||
      conversation.preview.toLowerCase().includes(term) ||
      conversation.username?.toLowerCase().includes(term)
    );
  });
});

export const unreadTotal = computed(() => conversations.value.reduce((sum, conversation) => sum + conversation.unread, 0));

export function updateDraft(chatId: string, value: string) {
  drafts.value = { ...drafts.value, [chatId]: value };
}

export function sendMessage(chatId: string, text: string) {
  const trimmed = text.trim();
  if (!trimmed) {
    return;
  }

  const now = new Date();
  const nextMessage: Message = {
    id: `${chatId}-${now.getTime()}`,
    sender: 'me',
    text: trimmed,
    time: `${now.getHours().toString().padStart(2, '0')}:${now.getMinutes().toString().padStart(2, '0')}`,
    delivered: true,
  };

  conversations.value = conversations.value.map((conversation) => {
    if (conversation.id !== chatId) {
      return conversation;
    }
    return {
      ...conversation,
      preview: trimmed,
      unread: 0,
      lastActive: 'just now',
      messages: [...conversation.messages, nextMessage],
    };
  });

  drafts.value = { ...drafts.value, [chatId]: '' };
}

export function createChannel(input: { name: string; description: string; visibility: 'public' | 'private' }) {
  const now = new Date();
  const id = `channel-${now.getTime()}`;
  const starterMessage: Message = {
    id: `${id}-welcome`,
    sender: 'contact',
    text: input.description || 'Welcome to your new channel.',
    time: `${now.getHours().toString().padStart(2, '0')}:${now.getMinutes().toString().padStart(2, '0')}`,
  };

  const newConversation: Conversation = {
    id,
    title: input.name,
    type: 'channel',
    preview: starterMessage.text,
    unread: 0,
    pinned: true,
    lastActive: 'just now',
    members: 1,
    username: input.visibility === 'public' ? `@${input.name.toLowerCase().replace(/\s+/g, '_')}` : undefined,
    messages: [starterMessage],
  };

  conversations.value = [newConversation, ...conversations.value];
  drafts.value = { ...drafts.value, [id]: '' };
  return newConversation;
}

export function joinDiscoveryChannel(id: string) {
  discoveryChannels.value = discoveryChannels.value.map((channel) =>
    channel.id === id ? { ...channel, joined: true } : channel,
  );
}
