import { Text } from '@mantine/core';
import { IconMessages } from '@tabler/icons-react';

export function EmptyState() {
  return (
    <div className="flex flex-col items-center justify-center h-full bg-[#313338] flex-1">
      <div className="text-center max-w-md px-4">
        <div className="mb-6">
          <IconMessages size={80} stroke={1} className="mx-auto text-[#949ba4]" />
        </div>
        
        <Text size="xl" fw={600} className="text-white mb-2">
          No Text Channels
        </Text>
        
        <Text size="sm" className="text-[#949ba4]">
          Select a server from the sidebar, then choose a text channel to start chatting.
        </Text>
      </div>
    </div>
  );
}
