import { Avatar, Badge, Text } from '@mantine/core';
import type { Conversation, User } from '../types';

interface ConversationItemProps {
  conversation: Conversation;
  currentUser: User;
  isSelected: boolean;
  onClick: () => void;
}

function getOtherParticipant(conversation: Conversation, currentUser: User): User {
  return conversation.participants.find(p => p.id !== currentUser.id) || conversation.participants[0];
}

function formatTime(date: Date): string {
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));
  
  if (diffDays === 0) {
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  } else if (diffDays === 1) {
    return 'Yesterday';
  } else if (diffDays < 7) {
    return date.toLocaleDateString([], { weekday: 'short' });
  } else {
    return date.toLocaleDateString([], { month: 'short', day: 'numeric' });
  }
}

export function ConversationItem({ conversation, currentUser, isSelected, onClick }: ConversationItemProps) {
  const otherUser = getOtherParticipant(conversation, currentUser);
  const displayName = conversation.isGroup ? conversation.groupName : otherUser.name;
  const avatar = conversation.isGroup ? conversation.groupAvatar : otherUser.avatar;
  
  return (
    <div
      onClick={onClick}
      class={`flex items-center gap-3 p-3 cursor-pointer hover:bg-gray-100 transition-colors ${
        isSelected ? 'bg-gray-100' : ''
      }`}
    >
      <div class="relative">
        <Avatar src={avatar} alt={displayName} size="lg" radius="xl" />
        {!conversation.isGroup && otherUser.status === 'online' && (
          <div class="absolute bottom-0 right-0 w-3 h-3 bg-green-500 rounded-full border-2 border-white" />
        )}
      </div>
      
      <div class="flex-1 min-w-0">
        <div class="flex justify-between items-center">
          <Text fw={conversation.unreadCount > 0 ? 700 : 400} size="sm" truncate>
            {displayName}
          </Text>
          <Text size="xs" c={conversation.unreadCount > 0 ? 'teal' : 'dimmed'}>
            {conversation.lastMessage ? formatTime(conversation.lastMessage.timestamp) : ''}
          </Text>
        </div>
        
        <div class="flex justify-between items-center mt-0.5">
          <Text size="xs" c="dimmed" truncate class="flex-1">
            {conversation.lastMessage?.senderId === currentUser.id && (
              <span class="text-blue-500 mr-1">
                {conversation.lastMessage.status === 'read' ? '✓✓' : conversation.lastMessage.status === 'delivered' ? '✓✓' : '✓'}
              </span>
            )}
            {conversation.lastMessage?.content || 'No messages yet'}
          </Text>
          {conversation.unreadCount > 0 && (
            <Badge size="sm" circle color="teal" class="ml-2">
              {conversation.unreadCount}
            </Badge>
          )}
        </div>
      </div>
    </div>
  );
}
