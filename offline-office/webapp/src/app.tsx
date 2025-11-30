import Router from 'preact-router';
import { MantineProvider } from '@mantine/core';
import { DocumentList } from './components/DocumentList';
import { DocumentEditor } from './components/DocumentEditor';
import { PresentationEditor } from './components/PresentationEditor';
import '@mantine/core/styles.css';

interface RouteProps {
  id?: string;
  path?: string;
}

function DocumentPage({ id }: RouteProps) {
  return <DocumentEditor documentId={id!} title="Document" />;
}

function PresentationPage({ id }: RouteProps) {
  return <PresentationEditor documentId={id!} title="Presentation" />;
}

export function App() {
  return (
    <MantineProvider>
      <Router>
        <DocumentList path="/" />
        <DocumentPage path="/document/:id" />
        <PresentationPage path="/presentation/:id" />
      </Router>
    </MantineProvider>
  );
}
