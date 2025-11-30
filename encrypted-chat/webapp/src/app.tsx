import { MantineProvider } from '@mantine/core';
import '@mantine/core/styles.css';
import { RouterProvider, RouterView } from '@copilot-test/preact-router';
import { router } from './router';

export function App() {
  return (
    <MantineProvider defaultColorScheme="dark">
      <RouterProvider router={router}>
        <RouterView />
      </RouterProvider>
    </MantineProvider>
  );
}
