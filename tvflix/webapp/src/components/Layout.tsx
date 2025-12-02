import type { ComponentChildren } from 'preact';
import { useRouter } from '@copilot-test/preact-router';
import { user, logout } from '../hooks/state';

interface LayoutProps {
  children: ComponentChildren;
}

export function Layout({ children }: LayoutProps) {
  const router = useRouter();
  const currentPath = router.currentRoute.value?.path || '/';

  const handleLogout = async () => {
    await logout();
    router.push('/login');
  };

  const navItems = [
    { path: '/videos', label: 'Videos', icon: '🎬' },
    { path: '/music', label: 'Music', icon: '🎵' },
    { path: '/photos', label: 'Photos', icon: '📷' },
  ];

  return (
    <div class="flex h-screen bg-neutral-900 text-white">
      {/* Sidebar */}
      <aside class="w-16 md:w-64 bg-black flex flex-col shrink-0">
        {/* Logo */}
        <div class="p-4 border-b border-neutral-800">
          <a href="/" class="flex items-center gap-2">
            <span class="text-2xl">📺</span>
            <span class="hidden md:block text-xl font-bold text-red-600">TVflix</span>
          </a>
        </div>

        {/* Navigation */}
        <nav class="flex-1 py-4">
          {navItems.map(item => {
            const isActive = currentPath.startsWith(item.path);
            return (
              <a
                key={item.path}
                href={item.path}
                class={`flex items-center gap-3 px-4 py-3 transition-colors ${
                  isActive
                    ? 'bg-neutral-800 text-white border-l-4 border-red-600'
                    : 'text-neutral-400 hover:bg-neutral-800 hover:text-white border-l-4 border-transparent'
                }`}
              >
                <span class="text-xl">{item.icon}</span>
                <span class="hidden md:block">{item.label}</span>
              </a>
            );
          })}
        </nav>

        {/* User section */}
        <div class="p-4 border-t border-neutral-800">
          <div class="flex items-center gap-2 mb-2">
            <div class="w-8 h-8 rounded-full bg-red-600 flex items-center justify-center text-sm font-bold">
              {user.value?.username.charAt(0).toUpperCase()}
            </div>
            <span class="hidden md:block text-sm truncate">{user.value?.username}</span>
          </div>
          <button
            onClick={handleLogout}
            class="w-full text-left text-sm text-neutral-400 hover:text-white transition-colors py-1"
          >
            <span class="hidden md:inline">Logout</span>
            <span class="md:hidden">🚪</span>
          </button>
        </div>
      </aside>

      {/* Main content */}
      <main class="flex-1 overflow-y-auto">
        {children}
      </main>
    </div>
  );
}
