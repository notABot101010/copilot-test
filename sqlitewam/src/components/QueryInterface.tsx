import { useSignal } from '@preact/signals';
import { Paper, Title, Stack, Textarea, Button, Group, Alert, Text, Table, ScrollArea, Code } from '@mantine/core';
import { executeQuery } from '../utils/database';

export function QueryInterface() {
  const query = useSignal('SELECT * FROM users');
  const result = useSignal<{ columns: string[], rows: any[][] } | null>(null);
  const error = useSignal<string | null>(null);
  const executing = useSignal(false);

  const handleExecute = async () => {
    if (!query.value.trim()) {
      error.value = 'Please enter a SQL query';
      return;
    }

    executing.value = true;
    error.value = null;
    result.value = null;

    try {
      const queryResult = executeQuery(query.value);
      result.value = queryResult;
    } catch (err: any) {
      console.error('Query error:', err);
      error.value = err.message || 'Failed to execute query';
    } finally {
      executing.value = false;
    }
  };

  const sampleQueries = [
    'SELECT * FROM users',
    'SELECT * FROM products WHERE price > 50',
    'SELECT u.name, o.total, o.status FROM orders o JOIN users u ON o.user_id = u.id',
    'SELECT name, COUNT(*) as total FROM users GROUP BY name',
  ];

  const loadSampleQuery = (sampleQuery: string) => {
    query.value = sampleQuery;
  };

  return (
    <Paper shadow="sm" p="lg" withBorder>
      <Stack gap="md">
        <div>
          <Title order={3} className="mb-2">SQL Query Interface</Title>
          <Text size="sm" c="dimmed">
            Write and execute SQL queries against the database
          </Text>
        </div>

        <div>
          <Text size="sm" fw={500} className="mb-2">Sample Queries:</Text>
          <Group gap="xs">
            {sampleQueries.map((sq, idx) => (
              <Button
                key={idx}
                size="xs"
                variant="light"
                onClick={() => loadSampleQuery(sq)}
              >
                Query {idx + 1}
              </Button>
            ))}
          </Group>
        </div>

        <Textarea
          label="SQL Query"
          placeholder="Enter your SQL query here..."
          value={query.value}
          onChange={(e) => (query.value = e.currentTarget.value)}
          minRows={4}
          autosize
          maxRows={10}
        />

        <Button
          onClick={handleExecute}
          loading={executing.value}
          disabled={!query.value.trim()}
        >
          Execute Query
        </Button>

        {error.value && (
          <Alert color="red" title="Error">
            {error.value}
          </Alert>
        )}

        {result.value && (
          <Paper p="md" withBorder className="bg-gray-50 dark:bg-gray-900">
            <Stack gap="sm">
              <Group justify="space-between">
                <Text fw={600}>Results</Text>
                <Text size="sm" c="dimmed">
                  {result.value.rows.length} row{result.value.rows.length !== 1 ? 's' : ''} returned
                </Text>
              </Group>

              {result.value.rows.length > 0 ? (
                <ScrollArea>
                  <Table striped highlightOnHover withTableBorder withColumnBorders>
                    <Table.Thead>
                      <Table.Tr>
                        {result.value.columns.map((col, idx) => (
                          <Table.Th key={idx}>
                            <Text size="sm" fw={600}>{col}</Text>
                          </Table.Th>
                        ))}
                      </Table.Tr>
                    </Table.Thead>
                    <Table.Tbody>
                      {result.value.rows.map((row, rowIdx) => (
                        <Table.Tr key={rowIdx}>
                          {row.map((cell, cellIdx) => (
                            <Table.Td key={cellIdx}>
                              <Text size="sm" className="font-mono">
                                {cell === null ? (
                                  <span className="text-gray-400 italic">NULL</span>
                                ) : (
                                  String(cell)
                                )}
                              </Text>
                            </Table.Td>
                          ))}
                        </Table.Tr>
                      ))}
                    </Table.Tbody>
                  </Table>
                </ScrollArea>
              ) : (
                <Text c="dimmed" size="sm">Query executed successfully. No rows returned.</Text>
              )}
            </Stack>
          </Paper>
        )}
      </Stack>
    </Paper>
  );
}
