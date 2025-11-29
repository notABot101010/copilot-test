import { createRouter, RouterProvider, RouterView } from '@copilot-test/preact-router';
import { MantineProvider } from '@mantine/core';
import '@mantine/core/styles.css';
import { HomePage } from './pages/HomePage';
import { CreateRepoPage } from './pages/CreateRepoPage';
import { RepoPage } from './pages/RepoPage';
import { FilePage } from './pages/FilePage';
import { EditFilePage } from './pages/EditFilePage';
import { Header } from './components/Header';

const router = createRouter({
  routes: [
    { path: '/', name: 'home', component: HomePage },
    { path: '/new', name: 'new-repo', component: CreateRepoPage },
    { path: '/repos/:name', name: 'repo', component: RepoPage },
    { path: '/repos/:name/blob/:path+', name: 'file', component: FilePage },
    { path: '/repos/:name/edit/:path+', name: 'edit-file', component: EditFilePage },
  ],
  mode: 'history',
});

export function App() {
  return (
    <MantineProvider>
      <RouterProvider router={router}>
        <div class="min-h-screen bg-gray-50">
          <Header />
          <main class="max-w-6xl mx-auto px-4 py-6">
            <RouterView />
          </main>
        </div>
      </RouterProvider>
    </MantineProvider>
  );
}
