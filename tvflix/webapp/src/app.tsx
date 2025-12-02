import { useEffect } from 'preact/hooks';
import { createRouter, RouterProvider, RouterView, useRouter } from '@copilot-test/preact-router';
import { MantineProvider } from '@mantine/core';
import '@mantine/core/styles.css';

import { Layout } from './components/Layout';
import { LoginPage } from './pages/LoginPage';
import { VideosPage } from './pages/VideosPage';
import { MusicPage } from './pages/MusicPage';
import { PhotosPage } from './pages/PhotosPage';
import { initAuth, isAuthenticated, authLoading } from './hooks/state';

// Home page redirects to videos
function HomePage() {
  const router = useRouter();
  useEffect(() => {
    if (!authLoading.value && isAuthenticated.value) {
      router.push('/videos');
    } else if (!authLoading.value && !isAuthenticated.value) {
      router.push('/login');
    }
  }, [authLoading.value, isAuthenticated.value]);
  return (
    <div class="min-h-screen bg-neutral-900 flex items-center justify-center">
      <div class="text-white">Loading...</div>
    </div>
  );
}

// Protected route wrapper
function ProtectedRoute({ children }: { children: preact.ComponentChildren }) {
  const router = useRouter();

  useEffect(() => {
    if (!authLoading.value && !isAuthenticated.value) {
      router.push('/login');
    }
  }, [authLoading.value, isAuthenticated.value]);

  if (authLoading.value) {
    return (
      <div class="min-h-screen bg-neutral-900 flex items-center justify-center">
        <div class="text-white">Loading...</div>
      </div>
    );
  }

  if (!isAuthenticated.value) {
    return null;
  }

  return <Layout>{children}</Layout>;
}

const router = createRouter({
  routes: [
    { path: '/', name: 'home', component: HomePage },
    { path: '/login', name: 'login', component: LoginPage },
    {
      path: '/videos',
      name: 'videos',
      component: () => (
        <ProtectedRoute>
          <VideosPage />
        </ProtectedRoute>
      ),
    },
    {
      path: '/music',
      name: 'music',
      component: () => (
        <ProtectedRoute>
          <MusicPage />
        </ProtectedRoute>
      ),
    },
    {
      path: '/photos',
      name: 'photos',
      component: () => (
        <ProtectedRoute>
          <PhotosPage />
        </ProtectedRoute>
      ),
    },
  ],
  mode: 'history',
});

export function App() {
  useEffect(() => {
    initAuth();
  }, []);

  return (
    <MantineProvider>
      <RouterProvider router={router}>
        <RouterView />
      </RouterProvider>
    </MantineProvider>
  );
}
