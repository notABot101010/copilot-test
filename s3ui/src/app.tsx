import { createRouter, RouterProvider, RouterView } from '@copilot-test/preact-router';
import { MantineProvider } from '@mantine/core';
import '@mantine/core/styles.css';
import { AppLayout } from './components/AppLayout';

// Pages
import { BucketsPage } from './pages/BucketsPage';
import { ObjectBrowserPage } from './pages/ObjectBrowserPage';

const router = createRouter({
  routes: [
    { path: '/', name: 'home', component: BucketsPage },
    { path: '/bucket/:bucket', name: 'bucket', component: ObjectBrowserPage },
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
