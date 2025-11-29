import { useState, useEffect } from 'preact/hooks';
import { MantineProvider } from '@mantine/core';
import '@mantine/core/styles.css';
import { Sidebar, ChatView, EmptyState } from './components';
import { chatService } from './services/mockChatService';
import type { Conversation, User } from './types';

export function App() {
  const [currentUser, setCurrentUser] = useState<User | null>(null);
  const [conversations, setConversations] = useState<Conversation[]>([]);
  const [selectedConversationId, setSelectedConversationId] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [isMobile, setIsMobile] = useState(window.innerWidth < 768);

  useEffect(() => {
    loadData();
    
    const handleResize = () => {
      setIsMobile(window.innerWidth < 768);
    };
    
    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, []);

  async function loadData() {
    try {
      const [user, convs] = await Promise.all([
        chatService.getCurrentUser(),
        chatService.listConversations(),
      ]);
      setCurrentUser(user);
      setConversations(convs);
    } finally {
      setLoading(false);
    }
  }

  function handleSelectConversation(id: string) {
    setSelectedConversationId(id);
  }

  function handleBack() {
    setSelectedConversationId(null);
  }

  const selectedConversation = conversations.find(c => c.id === selectedConversationId);

  if (loading || !currentUser) {
    return (
      <MantineProvider>
        <div className="flex items-center justify-center h-screen bg-[#00a884]">
          <div className="text-white text-xl">Loading...</div>
        </div>
      </MantineProvider>
    );
  }

  // Mobile view
  if (isMobile) {
    return (
      <MantineProvider>
        <div className="h-screen w-screen overflow-hidden">
          {selectedConversation ? (
            <ChatView
              conversation={selectedConversation}
              currentUser={currentUser}
              chatService={chatService}
              onBack={handleBack}
              isMobileView={true}
            />
          ) : (
            <Sidebar
              conversations={conversations}
              currentUser={currentUser}
              selectedConversationId={selectedConversationId}
              onSelectConversation={handleSelectConversation}
              isMobileView={true}
            />
          )}
        </div>
      </MantineProvider>
    );
  }

  // Desktop view
  return (
    <MantineProvider>
      <div className="flex h-screen w-screen overflow-hidden bg-[#d1d7db]">
        {/* Green header bar */}
        <div className="absolute top-0 left-0 right-0 h-32 bg-[#00a884]" />
        
        {/* Main container - full width without max-w constraint */}
        <div className="relative flex w-full mx-4 my-5 shadow-xl rounded-sm overflow-hidden z-10" style={{ height: 'calc(100vh - 40px)' }}>
          <Sidebar
            conversations={conversations}
            currentUser={currentUser}
            selectedConversationId={selectedConversationId}
            onSelectConversation={handleSelectConversation}
          />
          
          <div className="flex-1 flex">
            {selectedConversation ? (
              <ChatView
                conversation={selectedConversation}
                currentUser={currentUser}
                chatService={chatService}
              />
            ) : (
              <EmptyState />
            )}
          </div>
        </div>
      </div>
    </MantineProvider>
  );
}
