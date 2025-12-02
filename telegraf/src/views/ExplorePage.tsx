import { useSignal } from '@preact/signals';
import { discoveryChannels, joinDiscoveryChannel } from '../state/chatStore';

export function ExplorePage() {
  const filter = useSignal('');

  const filtered = discoveryChannels.value.filter((channel) => {
    const term = filter.value.trim().toLowerCase();
    if (!term) return true;
    return (
      channel.title.toLowerCase().includes(term) ||
      channel.category.toLowerCase().includes(term) ||
      channel.description.toLowerCase().includes(term)
    );
  });

  return (
    <section class="flex h-full flex-col">
      <div class="border-b border-white/5 bg-slate-900/70 px-6 py-4 backdrop-blur">
        <p class="text-lg font-semibold text-slate-50">Explore</p>
        <p class="text-sm text-slate-400">Discover channels that feel like Telegram - minimal and fast.</p>
      </div>
      <div class="flex items-center gap-3 border-b border-white/5 bg-slate-900/50 px-6 py-3">
        <input
          type="search"
          placeholder="Search categories"
          class="w-full rounded-xl border border-white/10 bg-slate-900/70 px-3 py-2 text-sm text-slate-100 placeholder:text-slate-500 focus:border-sky-500 focus:outline-none"
          value={filter.value}
          onInput={(event) => (filter.value = (event.target as HTMLInputElement).value)}
        />
        <a
          href="/create-channel"
          class="rounded-xl border border-white/10 bg-white/5 px-3 py-2 text-sm text-slate-100 hover:border-sky-400/40"
        >
          Create
        </a>
      </div>
      <div class="grid flex-1 grid-cols-1 gap-4 overflow-y-auto px-6 py-6 md:grid-cols-2 xl:grid-cols-3">
        {filtered.map((channel) => (
          <div
            key={channel.id}
            class="rounded-2xl border border-white/10 bg-white/5 p-4 space-y-2 shadow-sm shadow-black/20"
          >
            <div class="flex items-start justify-between gap-2">
              <div>
                <p class="text-base font-semibold text-slate-50">{channel.title}</p>
                <p class="text-xs text-slate-400 uppercase tracking-wide">{channel.category}</p>
              </div>
              <span class="rounded-full bg-slate-900/80 px-2 py-1 text-xs text-slate-300 border border-white/10">
                {channel.members}
              </span>
            </div>
            <p class="text-sm text-slate-300 leading-relaxed">{channel.description}</p>
            <div class="flex items-center justify-between pt-2">
              <span class="text-xs text-slate-400">Verified Telegram-style UI</span>
              <button
                type="button"
                aria-label={`${channel.joined ? 'Joined' : 'Join'} ${channel.title}`}
                class={`rounded-xl px-3 py-2 text-sm font-semibold transition-colors ${
                  channel.joined ? 'bg-emerald-600 text-white' : 'bg-sky-500 text-white hover:bg-sky-400'
                }`}
                onClick={() => joinDiscoveryChannel(channel.id)}
              >
                {channel.joined ? 'Joined' : 'Join'}
              </button>
            </div>
          </div>
        ))}
      </div>
    </section>
  );
}
