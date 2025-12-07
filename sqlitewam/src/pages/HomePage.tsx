import { useSignal } from '@preact/signals';
import { useEffect } from 'preact/hooks';
import { Container, Title, Stack, Loader, Text, Alert } from '@mantine/core';
import { TablesList } from '../components/TablesList';
import { QueryInterface } from '../components/QueryInterface';
import { initDatabase } from '../utils/database';

export function HomePage() {
  const loading = useSignal(true);
  const error = useSignal<string | null>(null);
  const initialized = useSignal(false);

  useEffect(() => {
    async function init() {
      try {
        loading.value = true;
        error.value = null;
        await initDatabase();
        initialized.value = true;
        loading.value = false;
        console.log('Database ready!');
      } catch (err: any) {
        console.error('Failed to initialize database:', err);
        error.value = err.message || 'Failed to initialize database';
        loading.value = false;
      }
    }

    init();
  }, []);

  if (loading.value) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <Stack align="center" gap="md">
          <Loader size="xl" />
          <Text size="lg">Initializing SQLite WASM...</Text>
        </Stack>
      </div>
    );
  }

  if (error.value) {
    return (
      <Container size="md" className="py-8">
        <Alert color="red" title="Error">
          {error.value}
        </Alert>
      </Container>
    );
  }

  return (
    <Container size="xl" className="py-8">
      <Stack gap="xl">
        <div>
          <Title order={1} className="mb-2">SQLite WASM Demo</Title>
          <Text c="dimmed">
            Explore tables and execute SQL queries in an in-memory SQLite database
          </Text>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          <div className="lg:col-span-1">
            <TablesList />
          </div>
          
          <div className="lg:col-span-2">
            <QueryInterface />
          </div>
        </div>
      </Stack>
    </Container>
  );
}
