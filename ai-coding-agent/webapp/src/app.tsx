import { useSignal } from '@preact/signals';
import { AppLayout } from './components/AppLayout';
import { SessionsPage } from './pages/SessionsPage';
import { SessionPage } from './pages/SessionPage';
import { TemplatesPage } from './pages/TemplatesPage';

function Router() {
  const path = useSignal(window.location.pathname);

  // Simple client-side routing
  if (typeof window !== 'undefined') {
    window.addEventListener('popstate', () => {
      path.value = window.location.pathname;
    });
  }

  // Parse route
  const sessionMatch = path.value.match(/^\/session\/([^/]+)/);
  
  if (sessionMatch) {
    return <SessionPage sessionId={sessionMatch[1]} />;
  }
  
  if (path.value === '/templates') {
    return <TemplatesPage />;
  }
  
  return <SessionsPage />;
}

export function App() {
  return (
    <AppLayout>
      <Router />
    </AppLayout>
  );
}
