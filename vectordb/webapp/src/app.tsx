import { createRouter, RouterProvider, RouterView } from '@copilot-test/preact-router';
import { MantineProvider } from '@mantine/core';
import '@mantine/core/styles.css';
import { AppLayout } from './components/AppLayout';

// Pages
import { NamespacesPage } from './pages/NamespacesPage';
import { NamespacePage } from './pages/NamespacePage';
import { DocumentsPage } from './pages/DocumentsPage';
import { DocumentPage } from './pages/DocumentPage';
import { QueryPage } from './pages/QueryPage';
import { ApiKeysPage } from './pages/ApiKeysPage';

const router = createRouter({
  routes: [
    { path: '/', name: 'home', component: NamespacesPage },
    { path: '/namespaces', name: 'namespaces', component: NamespacesPage },
    { path: '/namespaces/:namespace', name: 'namespace', component: NamespacePage },
    { path: '/namespaces/:namespace/documents', name: 'documents', component: DocumentsPage },
    { path: '/namespaces/:namespace/documents/:docId', name: 'document', component: DocumentPage },
    { path: '/namespaces/:namespace/query', name: 'query', component: QueryPage },
    { path: '/api-keys', name: 'api-keys', component: ApiKeysPage },
  ],
  mode: 'history',
});

export function App() {
  return (
    <MantineProvider>
      <RouterProvider router={router}>
        <AppLayout>
          <RouterView />
        </AppLayout>
      </RouterProvider>
    </MantineProvider>
  );
}
