import { useSignal, useComputed } from '@preact/signals';
import { useEffect } from 'preact/hooks';
import { Paper, Title, Stack, Button, Group, Badge, Text, Collapse, Code, ScrollArea } from '@mantine/core';
import { listTables, getTableSchema, executeQuery } from '../utils/database';

export function TablesList() {
  const tables = useSignal<string[]>([]);
  const expandedTable = useSignal<string | null>(null);
  const tableSchema = useSignal<{ columns: string[], rows: any[][] } | null>(null);
  const tableData = useSignal<{ columns: string[], rows: any[][] } | null>(null);

  useEffect(() => {
    loadTables();
  }, []);

  const loadTables = () => {
    try {
      const tableList = listTables();
      tables.value = tableList;
    } catch (err) {
      console.error('Error loading tables:', err);
    }
  };

  const toggleTable = async (tableName: string) => {
    if (expandedTable.value === tableName) {
      expandedTable.value = null;
      tableSchema.value = null;
      tableData.value = null;
    } else {
      expandedTable.value = tableName;
      try {
        const schema = getTableSchema(tableName);
        tableSchema.value = schema;
        
        const data = executeQuery(`SELECT * FROM ${tableName} LIMIT 5`);
        tableData.value = data;
      } catch (err) {
        console.error('Error loading table details:', err);
      }
    }
  };

  return (
    <Paper shadow="sm" p="lg" withBorder>
      <Stack gap="md">
        <div>
          <Title order={3} className="mb-2">Database Tables</Title>
          <Text size="sm" c="dimmed">
            Click on a table to view its schema and sample data
          </Text>
        </div>

        {tables.value.length === 0 ? (
          <Text c="dimmed" size="sm">No tables found</Text>
        ) : (
          <Stack gap="xs">
            {tables.value.map((table) => (
              <div key={table}>
                <Button
                  variant={expandedTable.value === table ? 'light' : 'subtle'}
                  fullWidth
                  onClick={() => toggleTable(table)}
                  className="justify-start"
                >
                  <Group justify="space-between" className="w-full">
                    <Text fw={500}>{table}</Text>
                    <Badge size="sm" variant="light">
                      {expandedTable.value === table ? '▼' : '▶'}
                    </Badge>
                  </Group>
                </Button>

                <Collapse in={expandedTable.value === table}>
                  <Paper p="sm" mt="xs" withBorder className="bg-gray-50 dark:bg-gray-900">
                    <Stack gap="sm">
                      {tableSchema.value && (
                        <div>
                          <Text size="xs" fw={600} className="mb-1">Schema:</Text>
                          <ScrollArea>
                            <div className="text-xs space-y-1">
                              {tableSchema.value.rows.map((row: any, idx: number) => (
                                <div key={idx} className="font-mono">
                                  <span className="text-blue-600 dark:text-blue-400">{row[1]}</span>
                                  <span className="text-gray-600 dark:text-gray-400"> {row[2]}</span>
                                  {row[3] === 1 && <span className="text-red-600 dark:text-red-400"> NOT NULL</span>}
                                  {row[5] === 1 && <span className="text-yellow-600 dark:text-yellow-400"> PK</span>}
                                </div>
                              ))}
                            </div>
                          </ScrollArea>
                        </div>
                      )}

                      {tableData.value && tableData.value.rows.length > 0 && (
                        <div>
                          <Text size="xs" fw={600} className="mb-1">Sample Data (first 5 rows):</Text>
                          <Text size="xs" c="dimmed" className="font-mono">
                            {tableData.value.rows.length} row{tableData.value.rows.length !== 1 ? 's' : ''} returned
                          </Text>
                        </div>
                      )}
                    </Stack>
                  </Paper>
                </Collapse>
              </div>
            ))}
          </Stack>
        )}
      </Stack>
    </Paper>
  );
}
