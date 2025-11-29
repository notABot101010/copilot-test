import { useState } from 'preact/hooks';
import { Avatar, TextInput, ActionIcon, Text } from '@mantine/core';
import { IconDotsVertical, IconSearch } from '@tabler/icons-react';
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
    <div className={`flex flex-col h-full bg-white ${isMobileView ? 'w-full' : 'w-full md:w-96 border-r border-gray-200'}`}>
      {/* Header */}
      <div className="flex items-center justify-between p-4 bg-gray-50 border-b border-gray-200">
        <div className="flex items-center gap-3">
          <Avatar src={currentUser.avatar} alt={currentUser.name} size="md" radius="xl" />
          <Text fw={500}>{currentUser.name}</Text>
        </div>
        <div className="flex gap-2">
          <ActionIcon variant="subtle" color="gray" size="lg">
            <IconDotsVertical size={20} />
          </ActionIcon>
        </div>
      </div>
      
      {/* Search */}
      <div className="p-2 bg-white">
        <TextInput
          placeholder="Search or start new chat"
          value={searchQuery}
          onChange={(e) => setSearchQuery((e.target as HTMLInputElement).value)}
          leftSection={<IconSearch size={16} />}
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
      <div className="flex-1 overflow-y-auto">
        {filteredConversations.length === 0 ? (
          <div className="flex items-center justify-center h-32">
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
