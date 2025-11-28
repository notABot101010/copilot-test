import { Text } from '@mantine/core';
import { IconMessages, IconLock } from '@tabler/icons-react';

export function EmptyState() {
  return (
    <div className="flex flex-col items-center justify-center h-full w-full bg-[#f0f2f5]">
      <div className="text-center max-w-md px-4">
        <div className="mb-6">
          <IconMessages size={80} stroke={1} className="mx-auto text-gray-400" />
        </div>
        
        <Text size="xl" fw={300} c="dimmed" className="mb-2">
          WhatsApp Clone
        </Text>
        
        <Text size="sm" c="dimmed">
          Send and receive messages without keeping your phone online.
          <br />
          Use WhatsApp on up to 4 linked devices and 1 phone at the same time.
        </Text>
        
        <div className="mt-8 flex items-center justify-center gap-2 text-gray-400">
          <IconLock size={16} />
          <Text size="xs" c="dimmed">
            End-to-end encrypted
          </Text>
        </div>
      </div>
    </div>
  );
}
