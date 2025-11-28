import { Avatar, Tooltip, UnstyledButton } from '@mantine/core';
import { IconHome, IconPlus, IconCompass } from '@tabler/icons-react';
import type { Server } from '../types';

interface ServerSidebarProps {
  servers: Server[];
  selectedServerId: string | null;
  onSelectServer: (id: string | null) => void;
  showDMs: boolean;
  onToggleDMs: () => void;
}

export function ServerSidebar({
  servers,
  selectedServerId,
  onSelectServer,
  showDMs,
  onToggleDMs,
}: ServerSidebarProps) {
  return (
    <div className="flex flex-col items-center w-18 bg-[#1e1f22] py-3 gap-2 overflow-y-auto overflow-x-hidden shrink-0">
      {/* Home / DMs button */}
      <Tooltip label="Direct Messages" position="right" withArrow>
        <UnstyledButton
          onClick={onToggleDMs}
          className={`flex items-center justify-center w-12 h-12 rounded-[24px] transition-all duration-200 ${
            showDMs
              ? 'bg-[#5865f2] rounded-[16px]'
              : 'bg-[#313338] hover:bg-[#5865f2] hover:rounded-[16px]'
          }`}
        >
          <IconHome size={24} className="text-white" />
        </UnstyledButton>
      </Tooltip>
      
      {/* Separator */}
      <div className="w-8 h-0.5 bg-[#35363c] rounded-full my-1" />
      
      {/* Server list */}
      {servers.map((server) => (
        <Tooltip key={server.id} label={server.name} position="right" withArrow>
          <UnstyledButton
            onClick={() => onSelectServer(server.id)}
            className={`relative flex items-center justify-center w-12 h-12 rounded-[24px] transition-all duration-200 ${
              selectedServerId === server.id
                ? 'rounded-[16px]'
                : 'hover:rounded-[16px]'
            }`}
          >
            {/* Selection indicator */}
            <div
              className={`absolute left-0 w-1 bg-white rounded-r-full transition-all duration-200 ${
                selectedServerId === server.id ? 'h-10' : 'h-0 group-hover:h-5'
              }`}
              style={{ transform: 'translateX(-16px)' }}
            />
            <Avatar
              src={server.icon}
              alt={server.name}
              size={48}
              radius={selectedServerId === server.id ? 16 : 24}
              className="transition-all duration-200 hover:rounded-[16px]"
            />
          </UnstyledButton>
        </Tooltip>
      ))}
      
      {/* Separator */}
      <div className="w-8 h-0.5 bg-[#35363c] rounded-full my-1" />
      
      {/* Add server button */}
      <Tooltip label="Add a Server" position="right" withArrow>
        <UnstyledButton className="flex items-center justify-center w-12 h-12 rounded-[24px] bg-[#313338] text-[#23a559] hover:bg-[#23a559] hover:text-white hover:rounded-[16px] transition-all duration-200">
          <IconPlus size={24} />
        </UnstyledButton>
      </Tooltip>
      
      {/* Explore button */}
      <Tooltip label="Explore Discoverable Servers" position="right" withArrow>
        <UnstyledButton className="flex items-center justify-center w-12 h-12 rounded-[24px] bg-[#313338] text-[#23a559] hover:bg-[#23a559] hover:text-white hover:rounded-[16px] transition-all duration-200">
          <IconCompass size={24} />
        </UnstyledButton>
      </Tooltip>
    </div>
  );
}
