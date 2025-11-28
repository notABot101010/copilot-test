import { useState, useEffect, useContext } from 'preact/hooks';
import { useRoute, useRouter } from '@copilot-test/preact-router';
import { effect } from '@preact/signals';
import { useMediaQuery } from '@mantine/hooks';
import { ChatContext } from '../context';
import { ServerSidebar, ChannelList, ChatArea, MemberList, ResizableSidebar, MobileHeader } from '../components';
import type { Channel, User } from '../types';

export function ChannelPage() {
  const route = useRoute();
  const router = useRouter();
  const chatContext = useContext(ChatContext);
  const [channels, setChannels] = useState<Channel[]>([]);
  const [members, setMembers] = useState<User[]>([]);
  const [showMembers, setShowMembers] = useState(true);
  const [currentServerId, setCurrentServerId] = useState<string | null>(null);
  const [currentChannelId, setCurrentChannelId] = useState<string | null>(null);
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);
  const isMobile = useMediaQuery('(max-width: 768px)');

  // Subscribe to route changes
  useEffect(() => {
    const dispose = effect(() => {
      const serverId = route.value.params.serverId;
      const channelId = route.value.params.channelId;
      setCurrentServerId(serverId || null);
      setCurrentChannelId(channelId || null);
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
    const [channelList, memberList] = await Promise.all([
      chatContext.chatService.listChannelsForServer(serverId),
      chatContext.chatService.listMembersForServer(serverId),
    ]);
    setChannels(channelList);
    setMembers(memberList);
  }

  if (!chatContext || !chatContext.currentUser) {
    return null;
  }

  const { currentUser, servers, chatService } = chatContext;
  const selectedServer = servers.find(s => s.id === currentServerId);
  const selectedChannel = channels.find(c => c.id === currentChannelId);

  function handleSelectChannel(id: string) {
    router.push(`/server/${currentServerId}/channels/${id}`);
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
                selectedChannelId={currentChannelId}
                onSelectChannel={handleSelectChannel}
              />
            </div>
          )}
          {/* Show only ChannelList on desktop */}
          {!isMobile && (
            <ChannelList
              server={selectedServer}
              channels={channels}
              selectedChannelId={currentChannelId}
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
            channel={selectedChannel || null}
            showDMs={false}
            onMenuClick={() => setMobileMenuOpen(true)}
          />
        )}
        {selectedChannel ? (
          <ChatArea
            key={currentChannelId}
            channel={selectedChannel}
            currentUser={currentUser}
            chatService={chatService}
            onToggleMembers={() => setShowMembers(!showMembers)}
            showMembers={showMembers}
          />
        ) : null}
      </div>
      {selectedServer && showMembers && !isMobile && (
        <MemberList
          members={members}
          currentUserId={currentUser.id}
        />
      )}
    </div>
  );
}
