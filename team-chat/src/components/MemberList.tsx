import { Avatar, Text } from '@mantine/core';
import type { User } from '../types';

interface MemberListProps {
  members: User[];
  currentUserId: string;
}

function getStatusColor(status: User['status']): string {
  switch (status) {
    case 'online':
      return 'bg-[#23a559]';
    case 'idle':
      return 'bg-[#f0b232]';
    case 'dnd':
      return 'bg-[#f23f43]';
    default:
      return 'bg-[#80848e]';
  }
}

function getStatusLabel(status: User['status']): string {
  switch (status) {
    case 'online':
      return 'Online';
    case 'idle':
      return 'Idle';
    case 'dnd':
      return 'Do Not Disturb';
    default:
      return 'Offline';
  }
}

export function MemberList({ members, currentUserId }: MemberListProps) {
  const onlineMembers = members.filter(m => m.status === 'online' || m.status === 'idle' || m.status === 'dnd');
  const offlineMembers = members.filter(m => m.status === 'offline');
  
  return (
    <div className="flex flex-col h-full bg-[#2b2d31] w-60 shrink-0 overflow-hidden">
      <div className="flex-1 overflow-y-auto px-2 py-4">
        {/* Online members */}
        {onlineMembers.length > 0 && (
          <div className="mb-4">
            <Text size="xs" fw={600} className="text-[#949ba4] uppercase tracking-wide px-2 mb-2">
              Online — {onlineMembers.length}
            </Text>
            {onlineMembers.map(member => (
              <div
                key={member.id}
                className="flex items-center gap-3 px-2 py-1.5 rounded hover:bg-[#35373c] cursor-pointer group"
              >
                <div className="relative">
                  <Avatar
                    src={member.avatar}
                    alt={member.displayName}
                    size={32}
                    radius="xl"
                  />
                  <div
                    className={`absolute bottom-0 right-0 w-3.5 h-3.5 rounded-full border-2 border-[#2b2d31] ${getStatusColor(member.status)}`}
                    title={getStatusLabel(member.status)}
                  />
                </div>
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-1">
                    <Text
                      size="sm"
                      className={`truncate ${
                        member.id === currentUserId ? 'text-white' : 'text-[#949ba4] group-hover:text-[#dbdee1]'
                      }`}
                    >
                      {member.displayName}
                    </Text>
                    {member.isBot && (
                      <span className="px-1 py-0.5 bg-[#5865f2] rounded text-[9px] text-white font-medium shrink-0">
                        BOT
                      </span>
                    )}
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
        
        {/* Offline members */}
        {offlineMembers.length > 0 && (
          <div>
            <Text size="xs" fw={600} className="text-[#949ba4] uppercase tracking-wide px-2 mb-2">
              Offline — {offlineMembers.length}
            </Text>
            {offlineMembers.map(member => (
              <div
                key={member.id}
                className="flex items-center gap-3 px-2 py-1.5 rounded hover:bg-[#35373c] cursor-pointer opacity-50 hover:opacity-100 group"
              >
                <div className="relative">
                  <Avatar
                    src={member.avatar}
                    alt={member.displayName}
                    size={32}
                    radius="xl"
                  />
                  <div
                    className={`absolute bottom-0 right-0 w-3.5 h-3.5 rounded-full border-2 border-[#2b2d31] ${getStatusColor(member.status)}`}
                    title={getStatusLabel(member.status)}
                  />
                </div>
                <div className="flex-1 min-w-0">
                  <Text
                    size="sm"
                    className="text-[#949ba4] group-hover:text-[#dbdee1] truncate"
                  >
                    {member.displayName}
                  </Text>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
