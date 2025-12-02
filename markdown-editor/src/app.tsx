import { useSignal } from '@preact/signals';
import { MantineProvider, createTheme } from '@mantine/core';
import '@mantine/core/styles.css';
import { Sidebar } from './components/Sidebar';
import { Editor } from './components/Editor';

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
  const sidebarOpen = useSignal(false);

  const toggleSidebar = () => {
    sidebarOpen.value = !sidebarOpen.value;
  };

  const closeSidebar = () => {
    sidebarOpen.value = false;
  };

  return (
    <MantineProvider theme={theme} defaultColorScheme="dark">
      <div class="flex h-screen overflow-hidden">
        <Sidebar isOpen={sidebarOpen.value} onClose={closeSidebar} />
        <Editor onToggleSidebar={toggleSidebar} />
      </div>
    </MantineProvider>
  );
}
