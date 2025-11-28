import { useState, useEffect } from 'preact/hooks';
import { MantineProvider, Drawer } from '@mantine/core';
import '@mantine/core/styles.css';
import { ServerSidebar, ChannelList, ChatArea, MemberList, EmptyState, DirectMessageList, MobileHeader } from './components';
import { chatService } from './services/mockChatService';
import type { Server, Channel, User, DirectMessage } from './types';

export function App() {
  const [currentUser, setCurrentUser] = useState<User | null>(null);
  const [servers, setServers] = useState<Server[]>([]);
  const [selectedServerId, setSelectedServerId] = useState<string | null>(null);
  const [selectedChannelId, setSelectedChannelId] = useState<string | null>(null);
  const [channels, setChannels] = useState<Channel[]>([]);
  const [members, setMembers] = useState<User[]>([]);
  const [directMessages, setDirectMessages] = useState<DirectMessage[]>([]);
  const [showDMs, setShowDMs] = useState(false);
  const [selectedDMId, setSelectedDMId] = useState<string | null>(null);
  const [showMembers, setShowMembers] = useState(true);
  const [loading, setLoading] = useState(true);
  const [isMobile, setIsMobile] = useState(window.innerWidth < 768);
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);

  useEffect(() => {
    loadInitialData();
    
    const handleResize = () => {
      setIsMobile(window.innerWidth < 768);
    };
    
    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, []);

  useEffect(() => {
    if (selectedServerId) {
      loadServerData(selectedServerId);
    }
  }, [selectedServerId]);

  async function loadInitialData() {
    try {
      const [user, serverList, dms] = await Promise.all([
        chatService.getCurrentUser(),
        chatService.listServers(),
        chatService.listDirectMessages(),
      ]);
      setCurrentUser(user);
      setServers(serverList);
      setDirectMessages(dms);
      
      // Auto-select first server
      if (serverList.length > 0) {
        setSelectedServerId(serverList[0].id);
      }
    } finally {
      setLoading(false);
    }
  }

  async function loadServerData(serverId: string) {
    const [channelList, memberList] = await Promise.all([
      chatService.listChannelsForServer(serverId),
      chatService.listMembersForServer(serverId),
    ]);
    setChannels(channelList);
    setMembers(memberList);
    
    // Auto-select first text channel
    const firstTextChannel = channelList.find(c => c.type === 'text');
    if (firstTextChannel) {
      setSelectedChannelId(firstTextChannel.id);
    }
  }

  function handleSelectServer(id: string | null) {
    setShowDMs(false);
    setSelectedServerId(id);
    setSelectedDMId(null);
    setMobileMenuOpen(false);
  }

  function handleToggleDMs() {
    setShowDMs(true);
    setSelectedServerId(null);
    setSelectedChannelId(null);
    setMobileMenuOpen(false);
  }

  function handleSelectChannel(id: string) {
    setSelectedChannelId(id);
    setMobileMenuOpen(false);
  }

  function handleSelectDM(id: string) {
    setSelectedDMId(id);
    setMobileMenuOpen(false);
  }

  const selectedServer = servers.find(s => s.id === selectedServerId);
  const selectedChannel = channels.find(c => c.id === selectedChannelId);

  if (loading || !currentUser) {
    return (
      <MantineProvider>
        <div className="flex items-center justify-center h-screen bg-[#313338]">
          <div className="text-[#dbdee1] text-xl">Loading...</div>
        </div>
      </MantineProvider>
    );
  }

  // Mobile view
  if (isMobile) {
    return (
      <MantineProvider>
        <div className="h-screen w-screen overflow-hidden flex flex-col bg-[#313338]">
          <MobileHeader
            server={selectedServer || null}
            channel={selectedChannel || null}
            showDMs={showDMs}
            onMenuClick={() => setMobileMenuOpen(true)}
            onBack={selectedChannel ? () => setSelectedChannelId(null) : undefined}
          />
          
          {/* Mobile menu drawer */}
          <Drawer
            opened={mobileMenuOpen}
            onClose={() => setMobileMenuOpen(false)}
            position="left"
            size="100%"
            withCloseButton={false}
            styles={{
              body: { padding: 0, height: '100%' },
              content: { backgroundColor: '#1e1f22' },
            }}
          >
            <div className="flex h-full">
              <ServerSidebar
                servers={servers}
                selectedServerId={selectedServerId}
                onSelectServer={handleSelectServer}
                showDMs={showDMs}
                onToggleDMs={handleToggleDMs}
              />
              {showDMs ? (
                <DirectMessageList
                  directMessages={directMessages}
                  currentUser={currentUser}
                  selectedDMId={selectedDMId}
                  onSelectDM={handleSelectDM}
                />
              ) : selectedServer ? (
                <ChannelList
                  server={selectedServer}
                  channels={channels}
                  selectedChannelId={selectedChannelId}
                  onSelectChannel={handleSelectChannel}
                />
              ) : null}
            </div>
          </Drawer>
          
          {/* Main content */}
          <div className="flex-1 overflow-hidden">
            {selectedChannel && currentUser ? (
              <ChatArea
                channel={selectedChannel}
                currentUser={currentUser}
                chatService={chatService}
                onToggleMembers={() => setShowMembers(!showMembers)}
                showMembers={showMembers}
              />
            ) : (
              <EmptyState />
            )}
          </div>
        </div>
      </MantineProvider>
    );
  }

  // Desktop view
  return (
    <MantineProvider>
      <div className="flex h-screen w-screen overflow-hidden bg-[#313338]">
        {/* Server sidebar */}
        <ServerSidebar
          servers={servers}
          selectedServerId={selectedServerId}
          onSelectServer={handleSelectServer}
          showDMs={showDMs}
          onToggleDMs={handleToggleDMs}
        />
        
        {/* Channel list / DM list */}
        {showDMs ? (
          <DirectMessageList
            directMessages={directMessages}
            currentUser={currentUser}
            selectedDMId={selectedDMId}
            onSelectDM={handleSelectDM}
          />
        ) : selectedServer ? (
          <ChannelList
            server={selectedServer}
            channels={channels}
            selectedChannelId={selectedChannelId}
            onSelectChannel={handleSelectChannel}
          />
        ) : null}
        
        {/* Chat area */}
        {selectedChannel ? (
          <ChatArea
            channel={selectedChannel}
            currentUser={currentUser}
            chatService={chatService}
            onToggleMembers={() => setShowMembers(!showMembers)}
            showMembers={showMembers}
          />
        ) : (
          <EmptyState />
        )}
        
        {/* Member list (desktop only, when showing server) */}
        {!showDMs && selectedServer && showMembers && !isMobile && (
          <MemberList
            members={members}
            currentUserId={currentUser.id}
          />
        )}
      </div>
    </MantineProvider>
  );
}
