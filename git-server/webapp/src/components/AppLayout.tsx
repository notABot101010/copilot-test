import { AppShell, Burger, Group, NavLink, Text, ScrollArea, Breadcrumbs, Anchor } from '@mantine/core';
import { useDisclosure } from '@mantine/hooks';
import { useRoute } from '@copilot-test/preact-router';

interface AppLayoutProps {
  children: preact.ComponentChildren;
}

export function AppLayout({ children }: AppLayoutProps) {
  const [opened, { toggle }] = useDisclosure();
  const route = useRoute();

  const params = route.value.params;
  const org = params.org as string | undefined;
  const project = params.project as string | undefined;

  // Determine active section from current path
  const path = route.value.path;

  // Build breadcrumb items for navigation
  const breadcrumbItems = [];
  if (org) {
    breadcrumbItems.push(
      <Anchor
        key="org"
        href={`/${org}`}
      >
        {org}
      </Anchor>
    );
  }
  if (project) {
    breadcrumbItems.push(
      <Anchor
        key="project"
        href={`/${org}/${project}`}
      >
        {project}
      </Anchor>
    );
  }

  return (
    <AppShell
      header={{ height: 60 }}
      navbar={{
        width: 280,
        breakpoint: 'sm',
        collapsed: { mobile: !opened },
      }}
      padding="md"
    >
      <AppShell.Header>
        <Group h="100%" px="md">
          <Group>
            <Burger opened={opened} onClick={toggle} hiddenFrom="sm" size="sm" />
            <a
              href="/"
              class="flex items-center gap-2 no-underline text-inherit"
            >
              <Text size="xl" fw={700} c="blue">
                📦 GitServer
              </Text>
            </a>
          </Group>
          <Group className="ml-5">
            {breadcrumbItems.length > 0 && (
              <Breadcrumbs separator="/">
                {breadcrumbItems}
              </Breadcrumbs>
            )}
          </Group>
        </Group>
      </AppShell.Header>

      <AppShell.Navbar p="md">
        {org && !project && (
          <AppShell.Section mt="md">
            <Text size="xs" c="dimmed" mb="xs" tt="uppercase">
              {org}
            </Text>
            <NavLink
              label="Projects"
              leftSection={<span>📁</span>}
              active={path === `/${org}` || path === `/${org}/`}
              href={`/${org}`}
            />
            <NavLink
              label="Settings"
              leftSection={<span>⚙️</span>}
              active={path === `/${org}/settings`}
              href={`/${org}/settings`}
            />
          </AppShell.Section>
        )}

        {org && project && (
          <>
            <AppShell.Section mt="md">
              <Text size="xs" c="dimmed" mb="xs" tt="uppercase">
                {project}
              </Text>
              <NavLink
                label="Code"
                leftSection={<span>📄</span>}
                active={path === `/${org}/${project}` || path.includes('/blob/') || path.includes('/edit/') || path.includes('/files/new')}
                href={`/${org}/${project}`}
              />
              <NavLink
                label="Issues"
                leftSection={<span>🐛</span>}
                active={path.includes('/issues')}
                href={`/${org}/${project}/issues`}
              />
              <NavLink
                label="Pull Requests"
                leftSection={<span>🔀</span>}
                active={path.includes('/pulls')}
                href={`/${org}/${project}/pulls`}
              />
              <NavLink
                label="Settings"
                leftSection={<span>⚙️</span>}
                active={path === `/${org}/${project}/settings`}
                href={`/${org}/${project}/settings`}
              />
            </AppShell.Section>
          </>
        )}

        <AppShell.Section grow component={ScrollArea} mt="md">
          {/* Additional navigation items can go here */}
        </AppShell.Section>
      </AppShell.Navbar>

      <AppShell.Main>
        {children}
      </AppShell.Main>
    </AppShell>
  );
}
