import { signal, computed } from '@preact/signals';
import type { Workspace, Contact, Conversation, Message, Analytics } from './types';

export const workspaces = signal<Workspace[]>([]);
export const currentWorkspace = signal<Workspace | null>(null);

export const contacts = signal<Contact[]>([]);
export const selectedContact = signal<Contact | null>(null);

export const conversations = signal<Conversation[]>([]);
export const selectedConversation = signal<Conversation | null>(null);

export const messages = signal<Message[]>([]);

export const analytics = signal<Analytics | null>(null);

export const loading = signal<boolean>(false);
export const error = signal<string | null>(null);

// Computed values
export const openConversations = computed(() =>
  conversations.value.filter((c) => c.status === 'open')
);

export const closedConversations = computed(() =>
  conversations.value.filter((c) => c.status === 'closed')
);

// Actions
export function setWorkspace(workspace: Workspace | null) {
  currentWorkspace.value = workspace;
  if (!workspace) {
    contacts.value = [];
    conversations.value = [];
    messages.value = [];
    analytics.value = null;
    selectedContact.value = null;
    selectedConversation.value = null;
  }
}

export function setConversations(convs: Conversation[]) {
  conversations.value = convs;
}

export function setContacts(contactList: Contact[]) {
  contacts.value = contactList;
}

export function setMessages(msgs: Message[]) {
  messages.value = msgs;
}

export function addMessage(msg: Message) {
  messages.value = [...messages.value, msg];
}

export function updateConversationInList(conv: Conversation) {
  const index = conversations.value.findIndex((c) => c.id === conv.id);
  if (index >= 0) {
    const newConvs = [...conversations.value];
    newConvs[index] = conv;
    conversations.value = newConvs;
  } else {
    conversations.value = [conv, ...conversations.value];
  }
}

export function setAnalytics(data: Analytics | null) {
  analytics.value = data;
}
