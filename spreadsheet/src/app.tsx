import { createRouter, RouterProvider, RouterView } from '@copilot-test/preact-router';
import { MantineProvider, createTheme } from '@mantine/core';
import '@mantine/core/styles.css';
import '@mantine/charts/styles.css';
import { HomePage } from './pages/HomePage';
import { SpreadsheetPage } from './pages/SpreadsheetPage';

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
    path: '/',
    name: 'home',
    component: HomePage,
  },
  {
    path: '/spreadsheets/:id',
    name: 'spreadsheet',
    component: SpreadsheetPage,
  },
];

const router = createRouter({
  routes,
  mode: 'history',
});

export function App() {
  return (
    <MantineProvider theme={theme} defaultColorScheme="dark">
      <RouterProvider router={router}>
        <RouterView />
      </RouterProvider>
    </MantineProvider>
  );
}
