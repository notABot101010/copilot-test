import { useState, useContext } from 'preact/hooks';
import { useRouter } from '@copilot-test/preact-router';
import { ChatContext } from '../context';
import { ServerSidebar } from '../components';
import { TextInput, Button, Text } from '@mantine/core';
import { IconServer, IconArrowLeft } from '@tabler/icons-react';
import type { Server } from '../types';

export function CreateServerPage() {
  const router = useRouter();
  const chatContext = useContext(ChatContext);
  const [serverName, setServerName] = useState('');
  const [creating, setCreating] = useState(false);

  if (!chatContext || !chatContext.currentUser) {
    return null;
  }

  const { servers, currentUser, addServer } = chatContext;

  async function handleCreateServer(e: Event) {
    e.preventDefault();
    if (!serverName.trim() || creating) return;

    setCreating(true);
    try {
      // Create a new server with a unique ID
      const newServer: Server = {
        id: `s${Date.now()}`,
        name: serverName.trim(),
        icon: `https://api.dicebear.com/7.x/identicon/svg?seed=${encodeURIComponent(serverName.trim())}`,
        ownerId: currentUser.id,
        channels: [],
        members: [currentUser],
        createdAt: new Date(),
      };

      addServer(newServer);
      router.push(`/server/${newServer.id}`);
    } finally {
      setCreating(false);
    }
  }

  return (
    <div className="flex h-screen w-screen overflow-hidden bg-[#313338]">
      <ServerSidebar
        servers={servers}
        selectedServerId={null}
        showDMs={false}
      />
      <div className="flex-1 flex items-center justify-center">
        <div className="w-full max-w-md p-8 bg-[#2b2d31] rounded-lg shadow-lg">
          <div className="flex items-center gap-3 mb-6">
            <button
              onClick={() => router.back()}
              className="p-2 rounded-full hover:bg-[#35373c] text-[#b5bac1] hover:text-[#dbdee1] transition-colors"
            >
              <IconArrowLeft size={20} />
            </button>
            <div className="w-12 h-12 rounded-full bg-[#5865f2] flex items-center justify-center">
              <IconServer size={24} className="text-white" />
            </div>
            <div>
              <Text size="xl" fw={700} className="text-[#dbdee1]">
                Create a Server
              </Text>
              <Text size="sm" className="text-[#949ba4]">
                Give your server a personality with a name
              </Text>
            </div>
          </div>

          <form onSubmit={handleCreateServer}>
            <div className="mb-6">
              <Text size="xs" fw={700} className="text-[#b5bac1] uppercase tracking-wide mb-2">
                Server Name
              </Text>
              <TextInput
                placeholder="My Awesome Server"
                value={serverName}
                onChange={(e) => setServerName((e.target as HTMLInputElement).value)}
                size="md"
                styles={{
                  input: {
                    backgroundColor: '#1e1f22',
                    border: 'none',
                    color: '#dbdee1',
                    '&::placeholder': {
                      color: '#6d6f78',
                    },
                  },
                }}
              />
            </div>

            <div className="flex gap-3">
              <Button
                variant="subtle"
                color="gray"
                onClick={() => router.back()}
                className="flex-1"
              >
                Cancel
              </Button>
              <Button
                type="submit"
                color="indigo"
                loading={creating}
                disabled={!serverName.trim()}
                className="flex-1"
              >
                Create Server
              </Button>
            </div>
          </form>
        </div>
      </div>
    </div>
  );
}
