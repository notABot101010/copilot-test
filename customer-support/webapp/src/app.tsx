import { useEffect } from 'preact/hooks';
import { MantineProvider } from '@mantine/core';
import { createRouter, RouterProvider, RouterView, useRoute } from '@copilot-test/preact-router';
import {
  Sidebar,
  ChatView,
  AnalyticsView,
  ContactsView,
  WorkspaceSelector,
} from './components';
import { currentWorkspace, setWorkspace } from './state';
import * as api from './services/api';

function WorkspaceLayout() {
  const route = useRoute();
  const currentPath = route.value.path;
  const workspace = currentWorkspace.value;

  // Extract workspace ID from params
  const workspaceId = route.value.params.workspaceId as string | undefined;

  useEffect(() => {
    if (workspaceId && (!workspace || workspace.id !== workspaceId)) {
      loadWorkspace(workspaceId);
    }
  }, [workspaceId]);

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
