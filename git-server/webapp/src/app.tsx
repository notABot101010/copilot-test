import { createRouter, RouterProvider, RouterView } from '@copilot-test/preact-router';
import { MantineProvider } from '@mantine/core';
import '@mantine/core/styles.css';
import { AppLayout } from './components/AppLayout';

// Pages
import { OrganizationsPage } from './pages/OrganizationsPage';
import { CreateOrgPage } from './pages/CreateOrgPage';
import { OrgPage } from './pages/OrgPage';
import { OrgSettingsPage } from './pages/OrgSettingsPage';
import { ProjectPage } from './pages/ProjectPage';
import { ProjectSettingsPage } from './pages/ProjectSettingsPage';
import { CreateProjectPage } from './pages/CreateProjectPage';
import { BranchesPage } from './pages/BranchesPage';
import { FilePage } from './pages/FilePage';
import { EditFilePage } from './pages/EditFilePage';
import { CreateFilePage } from './pages/CreateFilePage';
import { IssuesPage } from './pages/IssuesPage';
import { IssuePage } from './pages/IssuePage';
import { CreateIssuePage } from './pages/CreateIssuePage';
import { PullRequestsPage } from './pages/PullRequestsPage';
import { PullRequestPage } from './pages/PullRequestPage';
import { CreatePullRequestPage } from './pages/CreatePullRequestPage';
import { ForkRepoPage } from './pages/ForkRepoPage';

const router = createRouter({
  routes: [
    // Organizations
    { path: '/', name: 'home', component: OrganizationsPage },
    { path: '/organizations/new', name: 'new-org', component: CreateOrgPage },
    
    // Organization
    { path: '/:org', name: 'org', component: OrgPage },
    { path: '/:org/settings', name: 'org-settings', component: OrgSettingsPage },
    { path: '/:org/projects/new', name: 'new-project', component: CreateProjectPage },
    
    // Project (project = repository, project homepage shows code)
    { path: '/:org/:project', name: 'project', component: ProjectPage },
    { path: '/:org/:project/settings', name: 'project-settings', component: ProjectSettingsPage },
    { path: '/:org/:project/branches', name: 'branches', component: BranchesPage },
    { path: '/:org/:project/blob/:path+', name: 'file', component: FilePage },
    { path: '/:org/:project/edit/:path+', name: 'edit-file', component: EditFilePage },
    { path: '/:org/:project/files/new', name: 'new-file', component: CreateFilePage },
    { path: '/:org/:project/fork', name: 'fork-repo', component: ForkRepoPage },
    
    // Issues
    { path: '/:org/:project/issues', name: 'issues', component: IssuesPage },
    { path: '/:org/:project/issues/new', name: 'new-issue', component: CreateIssuePage },
    { path: '/:org/:project/issues/:number', name: 'issue', component: IssuePage },
    
    // Pull Requests
    { path: '/:org/:project/pulls', name: 'pulls', component: PullRequestsPage },
    { path: '/:org/:project/pulls/new', name: 'new-pull', component: CreatePullRequestPage },
    { path: '/:org/:project/pulls/:number', name: 'pull', component: PullRequestPage },
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
