import { useEffect } from 'preact/hooks';
import { MantineProvider, createTheme } from '@mantine/core';
import '@mantine/core/styles.css';
import { Sidebar, PageEditor } from './components';
import { initializeStore } from './store';

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

export function App() {
  useEffect(() => {
    initializeStore();
  }, []);

  return (
    <MantineProvider theme={theme} defaultColorScheme="dark">
      <div className="flex h-screen bg-zinc-900 text-white">
        <Sidebar />
        <PageEditor />
      </div>
    </MantineProvider>
  );
}
