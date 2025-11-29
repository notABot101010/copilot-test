import { useSignal, useSignalEffect } from '@preact/signals';
import { Card, Text, TextInput, Textarea, Button, Alert, Group, Loader } from '@mantine/core';
import { useRoute, useRouter } from '@copilot-test/preact-router';
import { getOrganization, updateOrganization, type Organization } from '../api';

export function OrgSettingsPage() {
  const route = useRoute();
  const router = useRouter();
  const org = useSignal<Organization | null>(null);
  const displayName = useSignal('');
  const description = useSignal('');
  const loading = useSignal(true);
  const saving = useSignal(false);
  const error = useSignal<string | null>(null);
  const success = useSignal(false);

  const params = route.value.params;
  const orgName = params.org as string;

  useSignalEffect(() => {
    loadOrg();
  });

  async function loadOrg() {
    try {
      loading.value = true;
      error.value = null;
      const orgData = await getOrganization(orgName);
      org.value = orgData;
      displayName.value = orgData.display_name;
      description.value = orgData.description;
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load organization';
    } finally {
      loading.value = false;
    }
  }

  async function handleSubmit(e: Event) {
    e.preventDefault();
    
    try {
      saving.value = true;
      error.value = null;
      success.value = false;
      await updateOrganization(orgName, {
        display_name: displayName.value.trim(),
        description: description.value.trim(),
      });
      success.value = true;
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to update organization';
    } finally {
      saving.value = false;
    }
  }

  if (loading.value) {
    return (
      <div class="flex justify-center py-12">
        <Loader size="lg" />
      </div>
    );
  }

  if (!org.value) {
    return (
      <Alert color="red" title="Error">
        Organization not found
      </Alert>
    );
  }

  return (
    <Card shadow="sm" padding="lg" radius="md" withBorder>
      <Text size="xl" fw={600} mb="lg">
        ⚙️ Organization Settings
      </Text>

      {error.value && (
        <Alert color="red" title="Error" mb="md">
          {error.value}
        </Alert>
      )}

      {success.value && (
        <Alert color="green" title="Success" mb="md">
          Organization settings updated successfully
        </Alert>
      )}

      <form onSubmit={handleSubmit}>
        <TextInput
          label="Organization slug"
          description="This cannot be changed"
          value={orgName}
          disabled
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
          <Button type="submit" loading={saving.value} color="blue">
            Save Changes
          </Button>
          <Button
            variant="outline"
            onClick={() => router.push(`/${orgName}`)}
            disabled={saving.value}
          >
            Back to Organization
          </Button>
        </Group>
      </form>
    </Card>
  );
}
