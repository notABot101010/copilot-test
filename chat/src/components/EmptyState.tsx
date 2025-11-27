import { Text } from '@mantine/core';

export function EmptyState() {
  return (
    <div class="flex flex-col items-center justify-center h-full bg-[#f0f2f5]">
      <div class="text-center max-w-md px-4">
        <div class="mb-6">
          <svg 
            xmlns="http://www.w3.org/2000/svg" 
            width="80" 
            height="80" 
            viewBox="0 0 24 24" 
            fill="none" 
            stroke="currentColor" 
            stroke-width="1" 
            stroke-linecap="round" 
            stroke-linejoin="round"
            class="mx-auto text-gray-400"
          >
            <path d="M14 9a2 2 0 0 1-2 2H6l-4 4V4c0-1.1.9-2 2-2h8a2 2 0 0 1 2 2v5Z" />
            <path d="M18 9h2a2 2 0 0 1 2 2v11l-4-4h-6a2 2 0 0 1-2-2v-1" />
          </svg>
        </div>
        
        <Text size="xl" fw={300} c="dimmed" class="mb-2">
          WhatsApp Clone
        </Text>
        
        <Text size="sm" c="dimmed">
          Send and receive messages without keeping your phone online.
          <br />
          Use WhatsApp on up to 4 linked devices and 1 phone at the same time.
        </Text>
        
        <div class="mt-8 flex items-center justify-center gap-2 text-gray-400">
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <rect width="18" height="11" x="3" y="11" rx="2" ry="2" />
            <path d="M7 11V7a5 5 0 0 1 10 0v4" />
          </svg>
          <Text size="xs" c="dimmed">
            End-to-end encrypted
          </Text>
        </div>
      </div>
    </div>
  );
}
