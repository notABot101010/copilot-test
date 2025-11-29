import { useSignal, useSignalEffect } from '@preact/signals';
import {
  Card,
  Text,
  TextInput,
  Textarea,
  Button,
  Alert,
  Select,
  Group,
  Loader,
} from '@mantine/core';
import { useRoute, useRouter } from '@copilot-test/preact-router';
import { createPullRequest, getRepoBranches, listRepos, type RepoInfo } from '../api';

export function CreatePullRequestPage() {
  const route = useRoute();
  const router = useRouter();
  const title = useSignal('');
  const body = useSignal('');
  const sourceRepo = useSignal('');
  const sourceBranch = useSignal('');
  const targetBranch = useSignal('');
  const loading = useSignal(false);
  const loadingData = useSignal(true);
  const error = useSignal<string | null>(null);
  const repos = useSignal<RepoInfo[]>([]);
  const branches = useSignal<string[]>([]);
  const sourceBranches = useSignal<string[]>([]);

  const params = route.value.params;
  const orgName = params.org as string;
  const repoName = params.name as string;

  useSignalEffect(() => {
    loadData();
  });

  async function loadData() {
    try {
      loadingData.value = true;
      const [reposData, branchesData] = await Promise.all([
        listRepos(orgName),
        getRepoBranches(orgName, repoName),
      ]);
      repos.value = reposData;
      branches.value = branchesData;
      sourceBranches.value = branchesData;
      sourceRepo.value = `${orgName}/${repoName}`;
      if (branchesData.length > 0) {
        targetBranch.value = branchesData.includes('main') ? 'main' : branchesData[0];
        sourceBranch.value = branchesData.length > 1 ? branchesData[1] : branchesData[0];
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load data';
    } finally {
      loadingData.value = false;
    }
  }

  async function handleSourceRepoChange(value: string | null) {
    if (!value) return;
    sourceRepo.value = value;
    try {
      // Parse org/name from the value
      const parts = value.split('/');
      if (parts.length === 2) {
        const branchesData = await getRepoBranches(parts[0], parts[1]);
        sourceBranches.value = branchesData;
        if (branchesData.length > 0) {
          sourceBranch.value = branchesData[0];
        }
      }
    } catch (e) {
      // Ignore errors loading branches
    }
  }

  async function handleSubmit(e: Event) {
    e.preventDefault();

    if (!title.value.trim()) {
      error.value = 'Pull request title is required';
      return;
    }

    if (!sourceBranch.value || !targetBranch.value) {
      error.value = 'Source and target branches are required';
      return;
    }

    try {
      loading.value = true;
      error.value = null;
      const pr = await createPullRequest(
        orgName,
        repoName,
        title.value.trim(),
        body.value.trim(),
        sourceRepo.value,
        sourceBranch.value,
        targetBranch.value
      );
      router.push(`/${orgName}/${repoName}/pulls/${pr.number}`);
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to create pull request';
      loading.value = false;
    }
  }

  if (loadingData.value) {
    return (
      <div class="flex justify-center py-12">
        <Loader size="lg" />
      </div>
    );
  }

  return (
    <Card shadow="sm" padding="lg" radius="md" withBorder>
      <Text size="xl" fw={600} mb="lg">
        🔀 Create a pull request
      </Text>

      {error.value && (
        <Alert color="red" title="Error" mb="md">
          {error.value}
        </Alert>
      )}

      <form onSubmit={handleSubmit}>
        <div class="grid grid-cols-2 gap-4 mb-4">
          <Select
            label="Source repository"
            placeholder="Select repository"
            data={repos.value.map((r) => ({ value: `${r.org_name}/${r.name}`, label: `${r.org_name}/${r.name}` }))}
            value={sourceRepo.value}
            onChange={handleSourceRepoChange}
          />
          <Select
            label="Source branch"
            placeholder="Select branch"
            data={sourceBranches.value.map((b) => ({ value: b, label: b }))}
            value={sourceBranch.value}
            onChange={(v: string | null) => (sourceBranch.value = v || '')}
          />
        </div>

        <div class="mb-4">
          <Select
            label="Target branch"
            placeholder="Select branch"
            data={branches.value.map((b) => ({ value: b, label: b }))}
            value={targetBranch.value}
            onChange={(v: string | null) => (targetBranch.value = v || '')}
          />
        </div>

        <TextInput
          label="Title"
          placeholder="Pull request title"
          value={title.value}
          onChange={(e: Event) => (title.value = (e.target as HTMLInputElement).value)}
          required
          mb="lg"
        />

        <Textarea
          label="Description"
          placeholder="Describe your changes..."
          value={body.value}
          onChange={(e: Event) => (body.value = (e.target as HTMLTextAreaElement).value)}
          minRows={6}
          mb="lg"
        />

        <Group>
          <Button type="submit" loading={loading.value} color="green">
            Create pull request
          </Button>
          <Button
            variant="outline"
            onClick={() => router.push(`/${orgName}/${repoName}/pulls`)}
            disabled={loading.value}
          >
            Cancel
          </Button>
        </Group>
      </form>
    </Card>
  );
}
