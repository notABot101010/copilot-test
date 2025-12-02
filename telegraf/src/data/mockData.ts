export type MessageSender = 'me' | 'contact';

export interface Message {
  id: string;
  sender: MessageSender;
  text: string;
  time: string;
  delivered?: boolean;
}

export type ConversationType = 'direct' | 'group' | 'channel';

export interface Conversation {
  id: string;
  title: string;
  username?: string;
  type: ConversationType;
  preview: string;
  unread: number;
  pinned?: boolean;
  muted?: boolean;
  online?: boolean;
  lastActive: string;
  members?: number;
  messages: Message[];
}

export interface ExploreChannel {
  id: string;
  title: string;
  category: string;
  members: string;
  description: string;
  verified?: boolean;
  joined?: boolean;
}

export const seededConversations: Conversation[] = [
  {
    id: 'ana',
    title: 'Ana',
    username: '@ana_now',
    type: 'direct',
    preview: "Let's sync about the product launch details.",
    unread: 2,
    online: true,
    lastActive: 'online',
    messages: [
      { id: 'm1', sender: 'contact', text: "Let's sync about the product launch details.", time: '10:02' },
      { id: 'm2', sender: 'me', text: 'Sure, want to do a quick call?', time: '10:03', delivered: true },
      { id: 'm3', sender: 'contact', text: 'Send me the final assets here.', time: '10:04' },
    ],
  },
  {
    id: 'product-updates',
    title: 'Product Updates',
    type: 'channel',
    preview: 'Sprint 24 summary and release candidate build.',
    unread: 0,
    pinned: true,
    lastActive: '08:12',
    members: 6421,
    messages: [
      { id: 'm1', sender: 'contact', text: 'Sprint 24 summary and release candidate build.', time: '08:12' },
      { id: 'm2', sender: 'contact', text: 'New beta is live for QA.', time: '08:15' },
    ],
  },
  {
    id: 'design-guild',
    title: 'Design Guild',
    type: 'group',
    preview: 'Uploading new icon set right now.',
    unread: 5,
    muted: true,
    lastActive: '09:27',
    members: 18,
    messages: [
      { id: 'm1', sender: 'contact', text: 'Uploading new icon set right now.', time: '09:27' },
      { id: 'm2', sender: 'contact', text: 'Take a look at the gradients in the Figma.', time: '09:32' },
      { id: 'm3', sender: 'me', text: 'Looks sharp, I love the new strokes.', time: '09:35', delivered: true },
    ],
  },
  {
    id: 'lena',
    title: 'Lena',
    username: '@lena_dsgn',
    type: 'direct',
    preview: 'Boarding in 20, ping me if you need anything.',
    unread: 0,
    lastActive: 'yesterday',
    messages: [
      { id: 'm1', sender: 'contact', text: 'Boarding in 20, ping me if you need anything.', time: 'Yesterday' },
      { id: 'm2', sender: 'me', text: 'Safe travels!', time: 'Yesterday', delivered: true },
    ],
  },
  {
    id: 'dev-rel',
    title: 'Dev Rel',
    type: 'group',
    preview: 'Livestream deck is ready for review.',
    unread: 1,
    lastActive: 'Mon',
    members: 42,
    messages: [
      { id: 'm1', sender: 'contact', text: 'Livestream deck is ready for review.', time: 'Mon' },
      { id: 'm2', sender: 'me', text: "Drop it here, I'll review in 30.", time: 'Mon', delivered: true },
    ],
  },
];

export const exploreSuggestions: ExploreChannel[] = [
  {
    id: 'tech-weekly',
    title: 'Tech Weekly',
    category: 'News',
    members: '128k',
    description: 'Signals-first dispatch on shipping software and product craft.',
    verified: true,
  },
  {
    id: 'motion-club',
    title: 'Motion Club',
    category: 'Design',
    members: '32k',
    description: 'Beautiful UI motion ideas and After Effects snippets.',
  },
  {
    id: 'founders-lounge',
    title: 'Founders Lounge',
    category: 'Startups',
    members: '51k',
    description: 'Candid chats on fundraising, hiring, and building in public.',
  },
];
