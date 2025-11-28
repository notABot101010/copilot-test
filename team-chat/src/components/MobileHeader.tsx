import { Text, ActionIcon } from '@mantine/core';
import { IconMenu2, IconArrowLeft, IconHash } from '@tabler/icons-react';
import type { Server, Channel } from '../types';

interface MobileHeaderProps {
  server: Server | null;
  channel: Channel | null;
  showDMs: boolean;
  onMenuClick: () => void;
  onBack?: () => void;
}

export function MobileHeader({ server, channel, showDMs, onMenuClick, onBack }: MobileHeaderProps) {
  return (
    <div className="flex items-center h-12 px-2 bg-[#313338] border-b border-[#1f2023] shrink-0">
      {onBack ? (
        <ActionIcon variant="transparent" onClick={onBack} className="text-[#b5bac1]">
          <IconArrowLeft size={24} />
        </ActionIcon>
      ) : (
        <ActionIcon variant="transparent" onClick={onMenuClick} className="text-[#b5bac1]">
          <IconMenu2 size={24} />
        </ActionIcon>
      )}
      
      <div className="flex items-center ml-2">
        {channel ? (
          <>
            <IconHash size={20} className="text-[#80848e] mr-1" />
            <Text fw={600} size="sm" className="text-white">
              {channel.name}
            </Text>
          </>
        ) : showDMs ? (
          <Text fw={600} size="sm" className="text-white">
            Direct Messages
          </Text>
        ) : server ? (
          <Text fw={600} size="sm" className="text-white">
            {server.name}
          </Text>
        ) : (
          <Text fw={600} size="sm" className="text-white">
            Discord Clone
          </Text>
        )}
      </div>
    </div>
  );
}
