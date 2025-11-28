import { useState, useEffect, useContext } from 'preact/hooks';
import { useRoute, useRouter } from '@copilot-test/preact-router';
import { effect } from '@preact/signals';
import { useMediaQuery } from '@mantine/hooks';
import { ChatContext } from '../context';
import { ServerSidebar, ChannelList, EmptyState, ResizableSidebar, MobileHeader } from '../components';
import type { Channel } from '../types';

export function ServerHomePage() {
  const route = useRoute();
  const router = useRouter();
  const chatContext = useContext(ChatContext);
  const [channels, setChannels] = useState<Channel[]>([]);
  const [currentServerId, setCurrentServerId] = useState<string | null>(null);
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);
  const isMobile = useMediaQuery('(max-width: 768px)');

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
    setMobileMenuOpen(false);
  }

  return (
    <div className="flex h-screen w-screen overflow-hidden bg-[#313338]">
      {/* Server sidebar - hidden on mobile, shown via drawer */}
      {!isMobile && (
        <ServerSidebar
          servers={servers}
          selectedServerId={currentServerId}
          showDMs={false}
        />
      )}
      {selectedServer && (
        <ResizableSidebar 
          minWidth={200} 
          maxWidth={400} 
          defaultWidth={240}
          mobileOpen={mobileMenuOpen}
          onMobileClose={() => setMobileMenuOpen(false)}
        >
          {/* Show ServerSidebar inside drawer on mobile */}
          {isMobile && (
            <div className="flex h-full">
              <ServerSidebar
                servers={servers}
                selectedServerId={currentServerId}
                showDMs={false}
              />
              <ChannelList
                server={selectedServer}
                channels={channels}
                selectedChannelId={null}
                onSelectChannel={handleSelectChannel}
              />
            </div>
          )}
          {/* Show only ChannelList on desktop */}
          {!isMobile && (
            <ChannelList
              server={selectedServer}
              channels={channels}
              selectedChannelId={null}
              onSelectChannel={handleSelectChannel}
            />
          )}
        </ResizableSidebar>
      )}
      <div className="flex flex-col flex-1 min-w-0">
        {/* Mobile header */}
        {isMobile && (
          <MobileHeader
            server={selectedServer || null}
            channel={null}
            showDMs={false}
            onMenuClick={() => setMobileMenuOpen(true)}
          />
        )}
        <EmptyState />
      </div>
    </div>
  );
}
