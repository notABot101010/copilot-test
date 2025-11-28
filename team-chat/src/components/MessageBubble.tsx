import { Avatar, Text } from '@mantine/core';
import type { Message } from '../types';

interface MessageBubbleProps {
  message: Message;
  isOwnMessage: boolean;
  showHeader: boolean;
}

function formatTime(date: Date): string {
  const now = new Date();
  const isToday = date.toDateString() === now.toDateString();
  const yesterday = new Date(now);
  yesterday.setDate(yesterday.getDate() - 1);
  const isYesterday = date.toDateString() === yesterday.toDateString();
  
  const timeStr = date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  
  if (isToday) {
    return `Today at ${timeStr}`;
  } else if (isYesterday) {
    return `Yesterday at ${timeStr}`;
  } else {
    return `${date.toLocaleDateString([], { month: '2-digit', day: '2-digit', year: 'numeric' })} ${timeStr}`;
  }
}

function formatShortTime(date: Date): string {
  return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
}

export function MessageBubble({ message, showHeader }: MessageBubbleProps) {
  return (
    <div className={`group flex gap-4 hover:bg-[#2e3035] px-1 py-0.5 rounded ${showHeader ? 'mt-4' : ''}`}>
      {showHeader ? (
        <Avatar
          src={message.author.avatar}
          alt={message.author.displayName}
          size={40}
          radius="xl"
          className="mt-0.5 shrink-0"
        />
      ) : (
        <div className="w-10 shrink-0 flex items-center justify-center">
          <Text size="xs" className="text-[#949ba4] opacity-0 group-hover:opacity-100">
            {formatShortTime(message.timestamp)}
          </Text>
        </div>
      )}
      
      <div className="flex-1 min-w-0">
        {showHeader && (
          <div className="flex items-center gap-2 mb-0.5">
            <Text fw={500} size="sm" className={message.author.isBot ? 'text-[#5865f2]' : 'text-white hover:underline cursor-pointer'}>
              {message.author.displayName}
            </Text>
            {message.author.isBot && (
              <span className="px-1.5 py-0.5 bg-[#5865f2] rounded text-[10px] text-white font-medium">
                BOT
              </span>
            )}
            <Text size="xs" className="text-[#949ba4]">
              {formatTime(message.timestamp)}
            </Text>
            {message.editedTimestamp && (
              <Text size="xs" className="text-[#949ba4]">
                (edited)
              </Text>
            )}
          </div>
        )}
        
        <Text size="sm" className="text-[#dbdee1] whitespace-pre-wrap break-words">
          {message.content}
        </Text>
        
        {/* Reactions */}
        {message.reactions && message.reactions.length > 0 && (
          <div className="flex flex-wrap gap-1 mt-1">
            {message.reactions.map((reaction, index) => (
              <div
                key={index}
                className="flex items-center gap-1 px-2 py-0.5 bg-[#2b2d31] border border-[#3f4147] rounded cursor-pointer hover:border-[#5865f2]"
              >
                <span>{reaction.emoji}</span>
                <Text size="xs" className="text-[#dbdee1]">{reaction.count}</Text>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
