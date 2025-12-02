import { settingsState } from '../state/chatStore';

const toggleItems = [
  { key: 'notifications', label: 'Notifications', description: 'Push alerts and badges across devices' },
  { key: 'readReceipts', label: 'Read receipts', description: 'Show ticks when messages are viewed' },
  { key: 'dataSaver', label: 'Data saver', description: 'Delay media downloads on mobile data' },
  { key: 'compactMode', label: 'Compact mode', description: 'Denser bubbles, smaller paddings' },
] as const;

export function SettingsPage() {
  return (
    <section class="flex h-full flex-col">
      <div class="border-b border-white/5 bg-slate-900/70 px-6 py-4 backdrop-blur">
        <p class="text-lg font-semibold text-slate-50">Settings</p>
        <p class="text-sm text-slate-400">Manage your Telegram presence, privacy, and devices.</p>
      </div>
      <div class="flex-1 space-y-6 overflow-y-auto px-6 py-6">
        <div class="rounded-2xl border border-white/10 bg-white/5 p-4">
          <div class="flex items-center gap-3">
            <div class="h-12 w-12 rounded-full bg-gradient-to-br from-sky-600 to-blue-500 flex items-center justify-center text-lg font-semibold text-white">
              TG
            </div>
            <div class="space-y-1">
              <p class="text-base font-semibold text-slate-50">Telegram Cloud</p>
              <p class="text-sm text-slate-400">Fast, secure, and synced on every device.</p>
            </div>
            <a
              href="/chats/ana"
              class="ml-auto rounded-xl border border-white/10 bg-white/10 px-3 py-2 text-sm text-slate-100 hover:border-sky-400/40"
            >
              Open chat
            </a>
          </div>
        </div>

        <div class="rounded-2xl border border-white/10 bg-white/5 p-4 space-y-3">
          <p class="text-sm font-semibold text-slate-200">Privacy & Security</p>
          {toggleItems.map((item) => (
            <div key={item.key} class="flex items-center gap-3 rounded-xl bg-slate-900/40 px-3 py-3 border border-white/5">
              <div class="flex-1">
                <p class="text-sm font-semibold text-slate-100">{item.label}</p>
                <p class="text-xs text-slate-400">{item.description}</p>
              </div>
              <button
                type="button"
                role="switch"
                aria-label={`Toggle ${item.label}`}
                aria-checked={settingsState.value[item.key]}
                aria-pressed={settingsState.value[item.key]}
                class={`h-9 w-16 rounded-full border border-white/10 bg-slate-800 transition-colors ${
                  settingsState.value[item.key] ? 'bg-sky-500/80 border-sky-400/50' : ''
                }`}
                onClick={() => {
                  settingsState.value = { ...settingsState.value, [item.key]: !settingsState.value[item.key] };
                }}
              >
                <span
                  class={`block h-7 w-7 rounded-full bg-white shadow-sm shadow-black/30 transition-all ${
                    settingsState.value[item.key] ? 'translate-x-7' : 'translate-x-1'
                  }`}
                />
              </button>
            </div>
          ))}
        </div>

        <div class="grid gap-4 md:grid-cols-2">
          <div class="rounded-2xl border border-white/10 bg-white/5 p-4 space-y-2">
            <p class="text-sm font-semibold text-slate-200">Active sessions</p>
            <div class="flex items-center justify-between rounded-xl bg-slate-900/40 px-3 py-3 border border-white/5">
              <div>
                <p class="text-sm text-slate-100">macOS - 2 hours ago</p>
                <p class="text-xs text-slate-500">San Francisco - Telegram Desktop</p>
              </div>
              <span class="rounded-lg bg-emerald-500/20 px-2 py-1 text-xs text-emerald-200">Current</span>
            </div>
            <div class="flex items-center justify-between rounded-xl bg-slate-900/40 px-3 py-3 border border-white/5">
              <div>
                <p class="text-sm text-slate-100">iOS - 1 day ago</p>
                <p class="text-xs text-slate-500">iPhone 15 - Telegram iOS</p>
              </div>
              <span class="text-xs text-slate-400">Last seen</span>
            </div>
          </div>
          <div class="rounded-2xl border border-white/10 bg-white/5 p-4 space-y-2">
            <p class="text-sm font-semibold text-slate-200">Storage</p>
            <div class="rounded-xl bg-slate-900/40 px-3 py-3 border border-white/5">
              <p class="text-xs text-slate-400 mb-2">Usage</p>
              <div class="h-2 rounded-full bg-slate-800 overflow-hidden">
                <div class="h-full w-2/3 rounded-full bg-gradient-to-r from-sky-500 to-blue-500" />
              </div>
              <p class="mt-2 text-sm text-slate-300">6.4 GB - auto-remove media after 30 days</p>
            </div>
            <a
              href="/explore"
              class="inline-flex items-center justify-center rounded-xl border border-white/10 bg-white/10 px-3 py-2 text-sm text-slate-100 hover:border-sky-400/40"
            >
              Manage media
            </a>
          </div>
        </div>
      </div>
    </section>
  );
}
