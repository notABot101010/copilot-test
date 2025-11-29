import { useRouter, useRoute } from '@copilot-test/preact-router';
import { signal } from '@preact/signals';
import { Button, Card, Group, Text, Title, Stack, Breadcrumbs, Anchor, TextInput, Textarea, NumberInput, Select, Table, Badge, Code } from '@mantine/core';
import { queryNamespace } from '../api';
import type { QueryResponse, QueryParams, QueryResult } from '../api';
import type { JSX } from 'preact';

const queryType = signal<'vector' | 'text' | 'hybrid'>('vector');
const vectorInput = signal('');
const textInput = signal('');
const alphaInput = signal(0.5);
const topK = signal(10);
const includeAttributes = signal('');
const includeVector = signal(false);
const filtersInput = signal('');

const results = signal<QueryResponse | null>(null);
const loading = signal(false);
const error = signal<string | null>(null);

export function QueryPage() {
  const router = useRouter();
  const route = useRoute();
  const params = route.value.params;
  const namespaceName = decodeURIComponent((params.namespace as string) || '');

  const executeQuery = async () => {
    loading.value = true;
    error.value = null;
    results.value = null;

    try {
      // Build rank_by based on query type
      let rank_by: unknown;
      if (queryType.value === 'vector') {
        const vector = JSON.parse(vectorInput.value);
        rank_by = ['vector', 'ANN', vector];
      } else if (queryType.value === 'text') {
        rank_by = ['text', 'BM25', textInput.value];
      } else {
        const vector = JSON.parse(vectorInput.value);
        rank_by = ['hybrid', 'ANN+BM25', vector, textInput.value, alphaInput.value];
      }

      const queryParams: QueryParams = {
        rank_by,
        top_k: topK.value,
        include_vector: includeVector.value,
      };

      // Parse include_attributes
      if (includeAttributes.value.trim()) {
        queryParams.include_attributes = includeAttributes.value.split(',').map(s => s.trim()).filter(Boolean);
      }

      // Parse filters
      if (filtersInput.value.trim()) {
        queryParams.filters = JSON.parse(filtersInput.value);
      }

      const data = await queryNamespace(namespaceName, queryParams);
      results.value = data;
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Query failed';
    } finally {
      loading.value = false;
    }
  };

  const breadcrumbs = [
    { title: 'Namespaces', href: '/' },
    { title: namespaceName, href: `/namespaces/${encodeURIComponent(namespaceName)}` },
    { title: 'Query', href: '#' },
  ].map((item, index) => (
    <Anchor
      key={index}
      href={item.href}
      onClick={(e: Event) => {
        if (item.href !== '#') {
          e.preventDefault();
          router.push(item.href);
        }
      }}
    >
      {item.title}
    </Anchor>
  ));

  return (
    <div>
      <Breadcrumbs mb="md">{breadcrumbs}</Breadcrumbs>

      <Title order={2} mb="lg">Query: {namespaceName}</Title>

      <Stack gap="md">
        <Card withBorder>
          <Title order={4} mb="md">Query Parameters</Title>
          
          <Stack gap="md">
            <Select
              label="Query Type"
              data={[
                { value: 'vector', label: 'Vector Search (ANN)' },
                { value: 'text', label: 'Full-Text Search (BM25)' },
                { value: 'hybrid', label: 'Hybrid Search' },
              ]}
              value={queryType.value}
              onChange={(value: string | null) => queryType.value = (value as 'vector' | 'text' | 'hybrid') || 'vector'}
            />

            {(queryType.value === 'vector' || queryType.value === 'hybrid') && (
              <Textarea
                label="Query Vector (JSON array)"
                placeholder="[0.1, 0.2, 0.3, ...]"
                value={vectorInput.value}
                onChange={(e: JSX.TargetedEvent<HTMLTextAreaElement>) => vectorInput.value = e.currentTarget.value}
                rows={3}
              />
            )}

            {(queryType.value === 'text' || queryType.value === 'hybrid') && (
              <TextInput
                label="Text Query"
                placeholder="Enter search text..."
                value={textInput.value}
                onChange={(e: JSX.TargetedEvent<HTMLInputElement>) => textInput.value = e.currentTarget.value}
              />
            )}

            {queryType.value === 'hybrid' && (
              <NumberInput
                label="Alpha (0-1, higher = more vector weight)"
                min={0}
                max={1}
                step={0.1}
                value={alphaInput.value}
                onChange={(value: string | number) => alphaInput.value = typeof value === 'number' ? value : 0.5}
              />
            )}

            <NumberInput
              label="Top K Results"
              min={1}
              max={1000}
              value={topK.value}
              onChange={(value: string | number) => topK.value = typeof value === 'number' ? value : 10}
            />

            <TextInput
              label="Include Attributes (comma-separated)"
              placeholder="text, category, price"
              value={includeAttributes.value}
              onChange={(e: JSX.TargetedEvent<HTMLInputElement>) => includeAttributes.value = e.currentTarget.value}
            />

            <Textarea
              label="Filters (JSON, optional)"
              placeholder='["category", "Eq", "animal"]'
              value={filtersInput.value}
              onChange={(e: JSX.TargetedEvent<HTMLTextAreaElement>) => filtersInput.value = e.currentTarget.value}
              rows={3}
            />

            <Button onClick={executeQuery} loading={loading.value}>
              Execute Query
            </Button>
          </Stack>
        </Card>

        {error.value && (
          <Card withBorder className="bg-red-50">
            <Text c="red">{error.value}</Text>
          </Card>
        )}

        {results.value && (
          <Card withBorder>
            <Group justify="space-between" mb="md">
              <Title order={4}>Results</Title>
              <Badge size="lg">{results.value.total_count} results</Badge>
            </Group>

            {results.value.results.length === 0 ? (
              <Text c="dimmed">No results found</Text>
            ) : (
              <Table striped highlightOnHover>
                <Table.Thead>
                  <Table.Tr>
                    <Table.Th>ID</Table.Th>
                    <Table.Th>Score</Table.Th>
                    <Table.Th>Attributes</Table.Th>
                  </Table.Tr>
                </Table.Thead>
                <Table.Tbody>
                  {results.value.results.map((result: QueryResult) => (
                    <Table.Tr key={result.id}>
                      <Table.Td>
                        <Text
                          component="a"
                          href={`/namespaces/${encodeURIComponent(namespaceName)}/documents/${encodeURIComponent(result.id)}`}
                          onClick={(e: Event) => {
                            e.preventDefault();
                            router.push(`/namespaces/${encodeURIComponent(namespaceName)}/documents/${encodeURIComponent(result.id)}`);
                          }}
                          className="text-blue-600 hover:underline cursor-pointer font-mono"
                        >
                          {result.id}
                        </Text>
                      </Table.Td>
                      <Table.Td>
                        <Badge color="blue">{result.score.toFixed(4)}</Badge>
                      </Table.Td>
                      <Table.Td>
                        {result.attributes ? (
                          <Code block style={{ maxHeight: 100, overflow: 'auto' }}>
                            {JSON.stringify(result.attributes, null, 2)}
                          </Code>
                        ) : (
                          <Text c="dimmed" size="sm">-</Text>
                        )}
                      </Table.Td>
                    </Table.Tr>
                  ))}
                </Table.Tbody>
              </Table>
            )}
          </Card>
        )}
      </Stack>
    </div>
  );
}
