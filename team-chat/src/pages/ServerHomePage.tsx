import { useState, useEffect, useContext } from 'preact/hooks';
import { useRoute, useRouter } from '@copilot-test/preact-router';
import { effect } from '@preact/signals';
import { ChatContext } from '../context';
import { ServerSidebar, ChannelList, EmptyState, ResizableSidebar } from '../components';
import type { Channel } from '../types';

export function ServerHomePage() {
  const route = useRoute();
  const router = useRouter();
  const chatContext = useContext(ChatContext);
  const [channels, setChannels] = useState<Channel[]>([]);
  const [currentServerId, setCurrentServerId] = useState<string | null>(null);

  // Subscribe to route changes
  useEffect(() => {
    const dispose = effect(() => {
      const serverId = route.value.params.serverId;
      setCurrentServerId(serverId || null);
    });
    return dispose;
  }, [route]);

  useEffect(() => {
    if (currentServerId && chatContext) {
      loadServerData(currentServerId);
    }
  }, [currentServerId, chatContext]);

  async function loadServerData(serverId: string) {
    if (!chatContext) return;
    const channelList = await chatContext.chatService.listChannelsForServer(serverId);
    setChannels(channelList);
    
    // Auto-select first text channel and navigate
    const firstTextChannel = channelList.find(c => c.type === 'text');
    if (firstTextChannel) {
      router.replace(`/server/${serverId}/channels/${firstTextChannel.id}`);
    }
  }

  if (!chatContext || !chatContext.currentUser) {
    return null;
  }

  const { servers } = chatContext;
  const selectedServer = servers.find(s => s.id === currentServerId);

  function handleSelectChannel(channelId: string) {
    router.push(`/server/${currentServerId}/channels/${channelId}`);
  }

  return (
    <div className="flex h-screen w-screen overflow-hidden bg-[#313338]">
      <ServerSidebar
        servers={servers}
        selectedServerId={currentServerId}
        showDMs={false}
      />
      {selectedServer && (
        <ResizableSidebar minWidth={200} maxWidth={400} defaultWidth={240}>
          <ChannelList
            server={selectedServer}
            channels={channels}
            selectedChannelId={null}
            onSelectChannel={handleSelectChannel}
          />
        </ResizableSidebar>
      )}
      <EmptyState />
    </div>
  );
}
