import { Avatar, Text } from '@mantine/core';
import type { DirectMessage, User } from '../types';

interface DirectMessageListProps {
  directMessages: DirectMessage[];
  currentUser: User;
  selectedDMId: string | null;
  onSelectDM: (id: string) => void;
}

export function DirectMessageList({ directMessages, currentUser, selectedDMId, onSelectDM }: DirectMessageListProps) {
  return (
    <div className="flex flex-col h-full bg-[#2b2d31] w-60 shrink-0">
      {/* Header */}
      <div className="flex items-center justify-between px-4 h-12 border-b border-[#1f2023] shadow-sm">
        <Text fw={600} size="sm" className="text-white">
          Direct Messages
        </Text>
      </div>
      
      {/* Search */}
      <div className="p-2">
        <div className="flex items-center px-2 py-1.5 bg-[#1e1f22] rounded text-[#949ba4] text-sm cursor-pointer">
          Find or start a conversation
        </div>
      </div>
      
      {/* Friends link */}
      <div className="px-2 mb-2">
        <div className="flex items-center gap-3 px-2 py-2 rounded hover:bg-[#35373c] cursor-pointer text-[#949ba4] hover:text-[#dbdee1]">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
            <path d="M13 10a4 4 0 1 0 0-8 4 4 0 0 0 0 8z" />
            <path d="M3 18a6 6 0 0 1 6-6h8a6 6 0 0 1 6 6v1H3v-1z" />
          </svg>
          <Text size="sm" fw={500}>Friends</Text>
        </div>
      </div>
      
      {/* DM section header */}
      <div className="flex items-center justify-between px-4 py-2">
        <Text size="xs" fw={600} className="text-[#949ba4] uppercase tracking-wide">
          Direct Messages
        </Text>
        <Text className="text-[#b5bac1] cursor-pointer hover:text-[#dbdee1]">+</Text>
      </div>
      
      {/* DM list */}
      <div className="flex-1 overflow-y-auto px-2">
        {directMessages.map(dm => {
          const otherUser = dm.participants.find(p => p.id !== currentUser.id) || dm.participants[0];
          const isSelected = selectedDMId === dm.id;
          
          return (
            <div
              key={dm.id}
              onClick={() => onSelectDM(dm.id)}
              className={`flex items-center gap-3 px-2 py-1.5 rounded cursor-pointer mb-0.5 group ${
                isSelected ? 'bg-[#404249]' : 'hover:bg-[#35373c]'
              }`}
            >
              <div className="relative">
                <Avatar
                  src={otherUser.avatar}
                  alt={otherUser.displayName}
                  size={32}
                  radius="xl"
                />
                <div
                  className={`absolute bottom-0 right-0 w-3 h-3 rounded-full border-2 border-[#2b2d31] ${
                    otherUser.status === 'online' ? 'bg-[#23a559]' :
                    otherUser.status === 'idle' ? 'bg-[#f0b232]' :
                    otherUser.status === 'dnd' ? 'bg-[#f23f43]' : 'bg-[#80848e]'
                  }`}
                />
              </div>
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2">
                  <Text
                    size="sm"
                    className={isSelected ? 'text-white' : 'text-[#949ba4] group-hover:text-[#dbdee1]'}
                    truncate
                  >
                    {otherUser.displayName}
                  </Text>
                  {dm.unreadCount > 0 && (
                    <div className="w-2 h-2 rounded-full bg-[#f23f43]" />
                  )}
                </div>
              </div>
            </div>
          );
        })}
      </div>
      
      {/* User area */}
      <div className="h-[52px] bg-[#232428] px-2 flex items-center gap-2">
        <div className="relative">
          <Avatar
            src={currentUser.avatar}
            alt={currentUser.displayName}
            size={32}
            radius="xl"
          />
          <div className="absolute bottom-0 right-0 w-3 h-3 rounded-full border-2 border-[#232428] bg-[#23a559]" />
        </div>
        <div className="flex-1 min-w-0">
          <Text size="xs" fw={600} className="text-white truncate">{currentUser.displayName}</Text>
          <Text size="xs" className="text-[#949ba4] truncate">Online</Text>
        </div>
      </div>
    </div>
  );
}
