import { createRouter, RouterProvider, RouterView } from '@copilot-test/preact-router';
import { MantineProvider } from '@mantine/core';
import '@mantine/core/styles.css';
import { HomePage } from './pages/HomePage';
import { CreateRepoPage } from './pages/CreateRepoPage';
import { RepoPage } from './pages/RepoPage';
import { FilePage } from './pages/FilePage';
import { EditFilePage } from './pages/EditFilePage';
import { IssuesPage } from './pages/IssuesPage';
import { IssuePage } from './pages/IssuePage';
import { CreateIssuePage } from './pages/CreateIssuePage';
import { PullRequestsPage } from './pages/PullRequestsPage';
import { PullRequestPage } from './pages/PullRequestPage';
import { CreatePullRequestPage } from './pages/CreatePullRequestPage';
import { ForkRepoPage } from './pages/ForkRepoPage';
import { Header } from './components/Header';

const router = createRouter({
  routes: [
    { path: '/', name: 'home', component: HomePage },
    { path: '/new', name: 'new-repo', component: CreateRepoPage },
    { path: '/repos/:name', name: 'repo', component: RepoPage },
    { path: '/repos/:name/blob/:path+', name: 'file', component: FilePage },
    { path: '/repos/:name/edit/:path+', name: 'edit-file', component: EditFilePage },
    { path: '/repos/:name/fork', name: 'fork-repo', component: ForkRepoPage },
    { path: '/repos/:name/issues', name: 'issues', component: IssuesPage },
    { path: '/repos/:name/issues/new', name: 'new-issue', component: CreateIssuePage },
    { path: '/repos/:name/issues/:number', name: 'issue', component: IssuePage },
    { path: '/repos/:name/pulls', name: 'pulls', component: PullRequestsPage },
    { path: '/repos/:name/pulls/new', name: 'new-pull', component: CreatePullRequestPage },
    { path: '/repos/:name/pulls/:number', name: 'pull', component: PullRequestPage },
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
