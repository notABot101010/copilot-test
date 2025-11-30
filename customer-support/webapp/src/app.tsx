import { useEffect, useRef } from 'preact/hooks';
import { MantineProvider } from '@mantine/core';
import { createRouter, RouterProvider, RouterView, useRoute } from '@copilot-test/preact-router';
import {
  Sidebar,
  ChatView,
  AnalyticsView,
  ContactsView,
  WorkspaceSelector,
} from './components';
import { currentWorkspace, setWorkspace, addMessage, updateConversationInList, messages } from './state';
import * as api from './services/api';

function WorkspaceLayout() {
  const route = useRoute();
  const currentPath = route.value.path;
  const workspace = currentWorkspace.value;
  const abortControllerRef = useRef<AbortController | null>(null);

  // Extract workspace ID from params
  const workspaceId = route.value.params.workspaceId as string | undefined;

  useEffect(() => {
    if (workspaceId && (!workspace || workspace.id !== workspaceId)) {
      loadWorkspace(workspaceId);
    }
  }, [workspaceId]);

  // Start long polling when workspace is loaded
  useEffect(() => {
    if (!workspace) return;

    const pollLoop = async () => {
      while (true) {
        try {
          abortControllerRef.current = new AbortController();
          const result = await api.pollEvents(workspace.id, abortControllerRef.current.signal);

          // Process events
          for (const event of result.events) {
            if (event.type === 'new_message' && event.message && event.conversation_id) {
              // Only add the message if it doesn't already exist (prevent duplicates)
              const messageExists = messages.value.some(m => m.id === event.message.id);
              if (!messageExists) {
                addMessage(event.message);
              }
              // Refresh conversation to update last message
              try {
                const conv = await api.getConversation(workspace.id, event.conversation_id);
                updateConversationInList(conv);
              } catch (err) {
                console.error('Failed to refresh conversation:', err);
              }
            } else if (event.type === 'new_conversation' && event.conversation) {
              updateConversationInList(event.conversation);
            } else if (event.type === 'conversation_updated' && event.conversation) {
              updateConversationInList(event.conversation);
            }
          }
        } catch (err) {
          if (err instanceof Error && err.name === 'AbortError') {
            break;
          }
          console.error('Polling error:', err);
          await new Promise(resolve => setTimeout(resolve, 5000));
        }
      }
    };

    pollLoop();

    return () => {
      if (abortControllerRef.current) {
        abortControllerRef.current.abort();
      }
    };
  }, [workspace?.id]);

  async function loadWorkspace(id: string) {
    try {
      const ws = await api.getWorkspace(id);
      setWorkspace(ws);
    } catch (err) {
      console.error('Failed to load workspace:', err);
      window.location.href = '/';
    }
  }

  if (!workspace) {
    return (
      <div className="flex items-center justify-center h-screen bg-gray-100">
        <p className="text-gray-500">Loading workspace...</p>
      </div>
    );
  }

  // Determine which view to show
  let content;
  if (currentPath.includes('/analytics')) {
    content = <AnalyticsView />;
  } else if (currentPath.includes('/contacts')) {
    content = <ContactsView />;
  } else {
    content = <ChatView />;
  }

  return (
    <div className="flex h-screen">
      <Sidebar currentPath={currentPath} />
      {content}
    </div>
  );
}

function NotFound() {
  return (
    <div className="flex items-center justify-center h-screen bg-gray-100">
      <div className="text-center">
        <h1 className="text-2xl font-bold text-gray-900 mb-2">Page Not Found</h1>
        <a href="/" className="text-blue-600 hover:underline">
          Go to Home
        </a>
      </div>
    </div>
  );
}

const router = createRouter({
  routes: [
    { path: '/', component: WorkspaceSelector },
    { path: '/w/:workspaceId', component: WorkspaceLayout },
    { path: '/w/:workspaceId/chat', component: WorkspaceLayout },
    { path: '/w/:workspaceId/analytics', component: WorkspaceLayout },
    { path: '/w/:workspaceId/contacts', component: WorkspaceLayout },
    { path: '/:pathMatch(.*)*', component: NotFound },
  ],
});

export function App() {
  return (
    <MantineProvider>
      <RouterProvider router={router}>
        <RouterView />
      </RouterProvider>
    </MantineProvider>
  );
}
