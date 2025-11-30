import { MantineProvider } from '@mantine/core';
import { DocumentList } from './components/DocumentList';
import { DocumentEditor } from './components/DocumentEditor';
import { currentDocument } from './store';
import '@mantine/core/styles.css';

export function App() {
  const doc = currentDocument.value;

  return (
    <MantineProvider>
      {doc ? <DocumentEditor /> : <DocumentList />}
    </MantineProvider>
  );
}
