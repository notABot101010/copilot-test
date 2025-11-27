import { useState } from 'preact/hooks';
import { Avatar, TextInput, ActionIcon, Text } from '@mantine/core';
import { ConversationItem } from './ConversationItem';
import type { Conversation, User } from '../types';

interface SidebarProps {
  conversations: Conversation[];
  currentUser: User;
  selectedConversationId: string | null;
  onSelectConversation: (id: string) => void;
  isMobileView?: boolean;
}

export function Sidebar({
  conversations,
  currentUser,
  selectedConversationId,
  onSelectConversation,
  isMobileView = false,
}: SidebarProps) {
  const [searchQuery, setSearchQuery] = useState('');
  
  const filteredConversations = conversations.filter(conv => {
    const otherUser = conv.participants.find(p => p.id !== currentUser.id);
    const name = conv.isGroup ? conv.groupName : otherUser?.name;
    return name?.toLowerCase().includes(searchQuery.toLowerCase());
  });

  return (
    <div class={`flex flex-col h-full bg-white ${isMobileView ? 'w-full' : 'w-full md:w-96 border-r border-gray-200'}`}>
      {/* Header */}
      <div class="flex items-center justify-between p-4 bg-gray-50 border-b border-gray-200">
        <div class="flex items-center gap-3">
          <Avatar src={currentUser.avatar} alt={currentUser.name} size="md" radius="xl" />
          <Text fw={500}>{currentUser.name}</Text>
        </div>
        <div class="flex gap-2">
          <ActionIcon variant="subtle" color="gray" size="lg">
            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="12" cy="12" r="1" />
              <circle cx="19" cy="12" r="1" />
              <circle cx="5" cy="12" r="1" />
            </svg>
          </ActionIcon>
        </div>
      </div>
      
      {/* Search */}
      <div class="p-2 bg-white">
        <TextInput
          placeholder="Search or start new chat"
          value={searchQuery}
          onChange={(e) => setSearchQuery((e.target as HTMLInputElement).value)}
          leftSection={
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="11" cy="11" r="8" />
              <path d="m21 21-4.3-4.3" />
            </svg>
          }
          size="sm"
          radius="lg"
          styles={{
            input: {
              backgroundColor: '#f0f2f5',
              border: 'none',
            },
          }}
        />
      </div>
      
      {/* Conversations List */}
      <div class="flex-1 overflow-y-auto">
        {filteredConversations.length === 0 ? (
          <div class="flex items-center justify-center h-32">
            <Text c="dimmed" size="sm">No conversations found</Text>
          </div>
        ) : (
          filteredConversations.map(conversation => (
            <ConversationItem
              key={conversation.id}
              conversation={conversation}
              currentUser={currentUser}
              isSelected={conversation.id === selectedConversationId}
              onClick={() => onSelectConversation(conversation.id)}
            />
          ))
        )}
      </div>
    </div>
  );
}
