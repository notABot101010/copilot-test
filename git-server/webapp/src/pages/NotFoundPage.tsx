import { Card, Text, Button, Group } from '@mantine/core';
import { useRouter } from '@copilot-test/preact-router';

export function NotFoundPage() {
  const router = useRouter();

  return (
    <div class="flex items-center justify-center min-h-[60vh]">
      <Card shadow="sm" padding="xl" radius="md" withBorder class="text-center max-w-md">
        <Text size="6xl" fw={700} c="dimmed" mb="md">
          404
        </Text>
        <Text size="xl" fw={600} mb="sm">
          Page Not Found
        </Text>
        <Text c="dimmed" mb="lg">
          The page you're looking for doesn't exist or has been moved.
        </Text>
        <Group justify="center">
          <Button onClick={() => router.back()} variant="outline">
            Go Back
          </Button>
          <Button onClick={() => router.push('/')}>
            Go Home
          </Button>
        </Group>
      </Card>
    </div>
  );
}
