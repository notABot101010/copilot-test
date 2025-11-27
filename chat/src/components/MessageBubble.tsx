import { Avatar, Text } from '@mantine/core';
import type { Message, User } from '../types';

interface MessageBubbleProps {
  message: Message;
  isOwnMessage: boolean;
  showAvatar: boolean;
  sender?: User;
}

function formatTime(date: Date): string {
  return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
}

function MessageStatus({ status }: { status: Message['status'] }) {
  const color = status === 'read' ? 'text-blue-500' : 'text-gray-400';
  return (
    <span class={`ml-1 ${color}`}>
      {status === 'sent' && '✓'}
      {status === 'delivered' && '✓✓'}
      {status === 'read' && '✓✓'}
    </span>
  );
}

export function MessageBubble({ message, isOwnMessage, showAvatar, sender }: MessageBubbleProps) {
  return (
    <div class={`flex ${isOwnMessage ? 'justify-end' : 'justify-start'} mb-1`}>
      <div class={`flex items-end gap-2 max-w-[80%] ${isOwnMessage ? 'flex-row-reverse' : ''}`}>
        {showAvatar && !isOwnMessage && sender && (
          <Avatar src={sender.avatar} alt={sender.name} size="sm" radius="xl" class="mb-1" />
        )}
        {!showAvatar && !isOwnMessage && <div class="w-8" />}
        
        <div
          class={`px-3 py-2 rounded-lg ${
            isOwnMessage
              ? 'bg-green-100 rounded-tr-none'
              : 'bg-white rounded-tl-none shadow-sm'
          }`}
        >
          <Text size="sm" class="whitespace-pre-wrap break-words">
            {message.content}
          </Text>
          <div class="flex items-center justify-end gap-1 mt-1">
            <Text size="xs" c="dimmed">
              {formatTime(message.timestamp)}
            </Text>
            {isOwnMessage && <MessageStatus status={message.status} />}
          </div>
        </div>
      </div>
    </div>
  );
}
