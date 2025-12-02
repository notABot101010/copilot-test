import { useEffect } from 'preact/hooks';
import { useRoute } from '@copilot-test/preact-router';
import { useSignal } from '@preact/signals';
import { filteredConversations, searchTerm, unreadTotal } from './state/chatStore';
import type { Conversation } from './data/mockData';
import { RouterView } from '@copilot-test/preact-router';

function ConversationItem({
  conversation,
  active,
  onSelect,
}: {
  conversation: Conversation;
  active: boolean;
  onSelect?: () => void;
}) {
  return (
    <a
      href={`/chats/${conversation.id}`}
      class={`block rounded-2xl border transition-colors ${active ? 'border-sky-400/40 bg-white/10' : 'border-transparent hover:border-white/10 hover:bg-white/5'}`}
      onClick={() => onSelect?.()}
    >
      <div class="flex items-start gap-3 px-3 py-3">
        <div
          class={`h-11 w-11 rounded-full flex items-center justify-center text-sm font-semibold text-white ${
            conversation.type === 'channel'
              ? 'bg-gradient-to-br from-sky-600 to-blue-500'
              : 'bg-gradient-to-br from-indigo-500 to-sky-500'
          }`}
        >
          {conversation.title.slice(0, 2).toUpperCase()}
        </div>
        <div class="min-w-0 flex-1 space-y-1">
          <div class="flex items-center justify-between gap-2">
            <p class="truncate text-[15px] font-semibold text-slate-50">{conversation.title}</p>
            <span class="text-xs text-slate-400 shrink-0">{conversation.lastActive}</span>
          </div>
          <p class="truncate text-sm text-slate-400">{conversation.preview}</p>
          <div class="flex items-center gap-2 text-[11px] text-slate-400">
            {conversation.username ? <span class="rounded-lg bg-white/5 px-2 py-0.5">{conversation.username}</span> : null}
            {conversation.muted ? <span class="rounded-lg bg-white/5 px-2 py-0.5">Muted</span> : null}
            {conversation.type === 'channel' ? <span class="rounded-lg bg-white/5 px-2 py-0.5">Channel</span> : null}
          </div>
        </div>
        {conversation.unread > 0 ? (
          <span class="rounded-full bg-sky-500 px-2.5 py-1 text-xs font-semibold text-white">{conversation.unread}</span>
        ) : null}
      </div>
    </a>
  );
}

function Sidebar({ isOpen, close }: { isOpen: boolean; close: () => void }) {
  const route = useRoute();
  const activeChatId = (route.value.params.chatId as string | undefined) ?? filteredConversations.value[0]?.id;

  const navLinks = [
    { href: '/explore', label: 'Explore' },
    { href: '/create-channel', label: 'Create channel' },
    { href: '/settings', label: 'Settings' },
  ];

  return (
    <aside
      class={`fixed inset-y-0 left-0 z-40 w-[320px] max-w-[80vw] border-r border-white/5 bg-slate-950/90 backdrop-blur-lg transition-transform duration-200 ease-out md:static md:max-w-[360px] md:translate-x-0 ${
        isOpen ? 'translate-x-0' : '-translate-x-full'
      }`}
    >
      <div class="flex items-center justify-between px-4 py-4">
        <div>
          <p class="text-lg font-semibold text-slate-50">Telegram</p>
          <p class="text-sm text-slate-400">Cloud-synced - {unreadTotal.value} unread</p>
        </div>
        <span class="rounded-full bg-sky-500/10 px-3 py-1 text-xs font-semibold text-sky-200 border border-sky-500/30">New</span>
      </div>
      <div class="px-4 pb-3">
        <div class="relative">
          <input
            type="search"
            class="w-full rounded-xl border border-white/10 bg-slate-900/80 px-4 py-2.5 text-sm text-slate-100 placeholder:text-slate-500 focus:border-sky-500 focus:outline-none"
            placeholder="Search chats and channels"
            value={searchTerm.value}
            onInput={(event) => (searchTerm.value = (event.target as HTMLInputElement).value)}
          />
          <div class="pointer-events-none absolute inset-y-0 right-3 flex items-center text-slate-500 text-sm">Ctrl+K</div>
        </div>
      </div>
      <div class="px-4 pb-3">
        <div class="flex items-center gap-2 text-xs uppercase tracking-wide text-slate-400">
          <span class="rounded-full bg-white/5 px-3 py-1 text-slate-200">All chats</span>
          <span class="rounded-full bg-white/5 px-3 py-1">Unread</span>
          <span class="rounded-full bg-white/5 px-3 py-1">Channels</span>
        </div>
      </div>
      <div class="px-3 space-y-2 overflow-y-auto pb-6 max-h-[calc(100vh-220px)]">
        {filteredConversations.value.map((conversation) => (
          <ConversationItem
            key={conversation.id}
            conversation={conversation}
            active={conversation.id === activeChatId}
            onSelect={() => {
              if (typeof window !== 'undefined' && window.innerWidth < 768) {
                close();
              }
            }}
          />
        ))}
      </div>
      <div class="border-t border-white/5 px-4 py-3 bg-slate-950/60 backdrop-blur">
        <div class="flex items-center justify-between text-sm text-slate-300">
          <span>Folders</span>
          <a href="/settings" class="text-sky-300 hover:text-sky-200 transition-colors">
            Edit
          </a>
        </div>
        <div class="mt-3 flex flex-wrap gap-2">
          {navLinks.map((link) => (
            <a
              key={link.href}
              href={link.href}
              class="rounded-xl border border-white/10 bg-white/5 px-3 py-2 text-sm text-slate-100 hover:border-sky-400/40 hover:text-sky-100 transition-colors"
            >
              {link.label}
            </a>
          ))}
        </div>
      </div>
    </aside>
  );
}

export function App() {
  const sidebarOpen = useSignal(typeof window !== 'undefined' ? window.innerWidth >= 768 : true);

  useEffect(() => {
    const handleResize = () => {
      if (window.innerWidth >= 768) {
        sidebarOpen.value = true;
      } else {
        sidebarOpen.value = false;
      }
    };
    handleResize();
    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, []);

  const closeSidebar = () => {
    if (window.innerWidth < 768) {
      sidebarOpen.value = false;
    }
  };

  return (
    <>
      <div class="flex min-h-screen bg-slate-950 text-slate-100">
        <Sidebar isOpen={sidebarOpen.value} close={closeSidebar} />
        {sidebarOpen.value && typeof window !== 'undefined' && window.innerWidth < 768 ? (
          <button
            type="button"
            class="fixed inset-0 z-30 bg-black/40 md:hidden"
            aria-label="Close sidebar"
            onClick={closeSidebar}
          />
        ) : null}
        <div class="flex min-h-screen flex-1 flex-col bg-slate-900/40">
          <div class="flex items-center gap-3 border-b border-white/5 bg-slate-900/60 px-4 py-3 md:hidden">
            <button
              type="button"
              class="rounded-lg border border-white/10 bg-white/5 px-3 py-2 text-sm text-slate-100"
              onClick={() => (sidebarOpen.value = true)}
            >
              Menu
            </button>
            <p class="text-sm text-slate-300">Chats</p>
          </div>
          <div class="flex flex-1 flex-col">
            <RouterView />
          </div>
        </div>
      </div>
    </>
  );
}
