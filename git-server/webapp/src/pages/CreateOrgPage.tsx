import { useSignal } from '@preact/signals';
import { Card, Text, TextInput, Textarea, Button, Alert, Group } from '@mantine/core';
import { useRouter } from '@copilot-test/preact-router';
import { createOrganization } from '../api';

export function CreateOrgPage() {
  const router = useRouter();
  const name = useSignal('');
  const displayName = useSignal('');
  const description = useSignal('');
  const loading = useSignal(false);
  const error = useSignal<string | null>(null);

  async function handleSubmit(e: Event) {
    e.preventDefault();
    
    if (!name.value.trim()) {
      error.value = 'Organization name is required';
      return;
    }

    try {
      loading.value = true;
      error.value = null;
      await createOrganization(
        name.value.trim(),
        displayName.value.trim() || name.value.trim(),
        description.value.trim()
      );
      router.push(`/${name.value.trim()}`);
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to create organization';
      loading.value = false;
    }
  }

  return (
    <Card shadow="sm" padding="lg" radius="md" withBorder>
      <Text size="xl" fw={600} mb="lg">
        Create a new organization
      </Text>

      {error.value && (
        <Alert color="red" title="Error" mb="md">
          {error.value}
        </Alert>
      )}

      <form onSubmit={handleSubmit}>
        <TextInput
          label="Organization name"
          description="Unique identifier for your organization (lowercase, no spaces)"
          placeholder="my-org"
          value={name.value}
          onChange={(e: Event) => (name.value = (e.target as HTMLInputElement).value.toLowerCase().replace(/[^a-z0-9-_]/g, '-'))}
          required
          mb="md"
        />

        <TextInput
          label="Display name"
          description="Human-readable name for your organization"
          placeholder="My Organization"
          value={displayName.value}
          onChange={(e: Event) => (displayName.value = (e.target as HTMLInputElement).value)}
          mb="md"
        />

        <Textarea
          label="Description"
          description="Optional description of your organization"
          placeholder="What does your organization do?"
          value={description.value}
          onChange={(e: Event) => (description.value = (e.target as HTMLTextAreaElement).value)}
          mb="lg"
        />

        <Group>
          <Button type="submit" loading={loading.value} color="green">
            Create Organization
          </Button>
          <Button
            variant="outline"
            onClick={() => router.push('/')}
            disabled={loading.value}
          >
            Cancel
          </Button>
        </Group>
      </form>
    </Card>
  );
}
