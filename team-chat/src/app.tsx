import { useState, useEffect } from 'preact/hooks';
import { MantineProvider } from '@mantine/core';
import '@mantine/core/styles.css';
import { RouterProvider, RouterView } from '@copilot-test/preact-router';
import { router } from './router';
import { ChatContext } from './context';
import type { ChatContextValue } from './context';
import { chatService } from './services/mockChatService';
import type { Server, User, DirectMessage } from './types';

export function App() {
  const [currentUser, setCurrentUser] = useState<User | null>(null);
  const [servers, setServers] = useState<Server[]>([]);
  const [directMessages, setDirectMessages] = useState<DirectMessage[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadInitialData();
  }, []);

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
    } finally {
      setLoading(false);
    }
  }

  function addServer(server: Server) {
    setServers(prev => [...prev, server]);
  }

  if (loading || !currentUser) {
    return (
      <MantineProvider>
        <div className="flex items-center justify-center h-screen bg-[#313338]">
          <div className="text-[#dbdee1] text-xl">Loading...</div>
        </div>
      </MantineProvider>
    );
  }

  const chatContextValue: ChatContextValue = {
    currentUser,
    servers,
    directMessages,
    chatService,
    addServer,
  };

  return (
    <MantineProvider>
      <ChatContext.Provider value={chatContextValue}>
        <RouterProvider router={router}>
          <RouterView />
        </RouterProvider>
      </ChatContext.Provider>
    </MantineProvider>
  );
}
