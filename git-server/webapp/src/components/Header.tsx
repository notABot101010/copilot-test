import { Button } from '@mantine/core';
import { useRouter } from '@copilot-test/preact-router';

export function Header() {
  const router = useRouter();

  return (
    <header class="bg-gray-900 text-white">
      <div class="max-w-6xl mx-auto px-4 py-4 flex items-center justify-between">
        <div class="flex items-center gap-6">
          <a href="/" class="text-xl font-semibold flex items-center gap-2 hover:text-gray-300">
            📦 Git Server
          </a>
          <nav class="flex gap-4">
            <a href="/" class="text-gray-300 hover:text-white">
              Repositories
            </a>
          </nav>
        </div>
        <Button
          variant="filled"
          color="green"
          size="sm"
          onClick={() => router.push('/new')}
        >
          + New Repository
        </Button>
      </div>
    </header>
  );
}
