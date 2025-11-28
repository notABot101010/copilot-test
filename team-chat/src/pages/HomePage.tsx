import { useState, useContext } from 'preact/hooks';
import { ChatContext } from '../context';
import { DirectMessageList, ResizableSidebar, MobileHeader } from '../components';
import { ServerSidebar } from '../components';
import { Text } from '@mantine/core';
import { useMediaQuery } from '@mantine/hooks';
import { IconMessages } from '@tabler/icons-react';

export function HomePage() {
  const chatContext = useContext(ChatContext);
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);
  const isMobile = useMediaQuery('(max-width: 768px)');

  if (!chatContext || !chatContext.currentUser) {
    return null;
  }

  const { currentUser, servers, directMessages } = chatContext;

  return (
    <div className="flex h-screen w-screen overflow-hidden bg-[#313338]">
      {/* Server sidebar - hidden on mobile, shown via drawer */}
      {!isMobile && (
        <ServerSidebar
          servers={servers}
          selectedServerId={null}
          showDMs={true}
        />
      )}
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
              selectedServerId={null}
              showDMs={true}
            />
            <DirectMessageList
              directMessages={directMessages}
              currentUser={currentUser}
              selectedDMId={null}
              onSelectDM={() => setMobileMenuOpen(false)}
            />
          </div>
        )}
        {/* Show only DirectMessageList on desktop */}
        {!isMobile && (
          <DirectMessageList
            directMessages={directMessages}
            currentUser={currentUser}
            selectedDMId={null}
            onSelectDM={() => {}}
          />
        )}
      </ResizableSidebar>
      <div className="flex flex-col flex-1 min-w-0">
        {/* Mobile header */}
        {isMobile && (
          <MobileHeader
            server={null}
            channel={null}
            showDMs={true}
            onMenuClick={() => setMobileMenuOpen(true)}
          />
        )}
        <div className="flex flex-col items-center justify-center h-full bg-[#313338] flex-1">
          <div className="text-center max-w-md px-4">
            <div className="mb-6">
              <IconMessages size={80} stroke={1} className="mx-auto text-[#949ba4]" />
            </div>
            
            <Text size="xl" fw={600} className="text-[#dbdee1] mb-2">
              Direct Messages
            </Text>
            
            <Text size="sm" className="text-[#949ba4]">
              Select a conversation from the sidebar to start chatting.
            </Text>
          </div>
        </div>
      </div>
    </div>
  );
}
