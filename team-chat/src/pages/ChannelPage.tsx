import { useState, useEffect, useContext } from 'preact/hooks';
import { useRoute, useRouter } from '@copilot-test/preact-router';
import { effect } from '@preact/signals';
import { ChatContext } from '../context';
import { ServerSidebar, ChannelList, ChatArea, MemberList, ResizableSidebar } from '../components';
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
            selectedChannelId={currentChannelId}
            onSelectChannel={handleSelectChannel}
          />
        </ResizableSidebar>
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
      {selectedServer && showMembers && (
        <MemberList
          members={members}
          currentUserId={currentUser.id}
        />
      )}
    </div>
  );
}
