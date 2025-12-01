import { useEffect } from 'preact/hooks';
import { createRouter, RouterProvider, RouterView } from '@copilot-test/preact-router';
import { MantineProvider, createTheme } from '@mantine/core';
import '@mantine/core/styles.css';
import { LoginPage } from './pages/LoginPage';
import { HomePage } from './pages/HomePage';
import { MatchPage } from './pages/MatchPage';
import { initAuth } from './store/authStore';

const theme = createTheme({
  primaryColor: 'blue',
  fontFamily: 'system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
  colors: {
    dark: [
      '#C1C2C5',
      '#A6A7AB',
      '#909296',
      '#5c5f66',
      '#373A40',
      '#2C2E33',
      '#25262b',
      '#1A1B1E',
      '#141517',
      '#101113',
    ],
  },
});

const routes = [
  {
    path: '/login',
    name: 'login',
    component: LoginPage,
  },
  {
    path: '/',
    name: 'home',
    component: HomePage,
  },
  {
    path: '/match/:id',
    name: 'match',
    component: MatchPage,
  },
];

const router = createRouter({
  routes,
  mode: 'history',
});

function AppContent() {
  useEffect(() => {
    initAuth();
  }, []);
  
  return <RouterView />;
}

export function App() {
  return (
    <MantineProvider theme={theme} defaultColorScheme="dark">
      <RouterProvider router={router}>
        <AppContent />
      </RouterProvider>
    </MantineProvider>
  );
}
