import type { ComponentChildren } from 'preact';
import { useRouter, useRoute } from '@copilot-test/preact-router';
import { AppShell, Burger, Group, NavLink, Title } from '@mantine/core';
import { signal } from '@preact/signals';

const opened = signal(true);

interface AppLayoutProps {
  children: ComponentChildren;
}

export function AppLayout({ children }: AppLayoutProps) {
  const router = useRouter();
  const route = useRoute();

  const isActive = (path: string) => route.value.path === path || route.value.path.startsWith(path + '/');

  return (
    <AppShell
      header={{ height: 60 }}
      navbar={{ width: 250, breakpoint: 'sm', collapsed: { mobile: !opened.value } }}
      padding="md"
    >
      <AppShell.Header>
        <Group h="100%" px="md">
          <Burger opened={opened.value} onClick={() => opened.value = !opened.value} hiddenFrom="sm" size="sm" />
          <Title order={3} className="text-blue-600">VectorDB Dashboard</Title>
        </Group>
      </AppShell.Header>

      <AppShell.Navbar p="md">
        <NavLink
          label="Namespaces"
          active={route.value.path === '/' || isActive('/namespaces')}
          onClick={() => router.push('/')}
          className="cursor-pointer"
        />
        <NavLink
          label="API Keys"
          active={isActive('/api-keys')}
          onClick={() => router.push('/api-keys')}
          className="cursor-pointer"
        />
      </AppShell.Navbar>

      <AppShell.Main>
        {children}
      </AppShell.Main>
    </AppShell>
  );
}
