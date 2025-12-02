import { useSignal } from '@preact/signals';
import { useRouter } from '@copilot-test/preact-router';
import { createChannel } from '../state/chatStore';

export function CreateChannelPage() {
  const router = useRouter();
  const name = useSignal('');
  const description = useSignal('');
  const visibility = useSignal<'public' | 'private'>('public');
  const error = useSignal('');

  const handleSubmit = (event: Event) => {
    event.preventDefault();
    if (!name.value.trim()) {
      error.value = 'Channel name is required';
      return;
    }
    error.value = '';
    const channel = createChannel({
      name: name.value.trim(),
      description: description.value.trim(),
      visibility: visibility.value,
    });
    router.push(`/chats/${channel.id}`);
  };

  return (
    <section class="flex h-full flex-col">
      <div class="border-b border-white/5 bg-slate-900/70 px-6 py-4 backdrop-blur">
        <p class="text-lg font-semibold text-slate-50">Create channel</p>
        <p class="text-sm text-slate-400">Broadcast updates with Telegram's familiar channel interface.</p>
      </div>
      <div class="flex-1 overflow-y-auto px-6 py-6">
        <form onSubmit={handleSubmit} class="max-w-2xl space-y-4">
          <div class="rounded-2xl border border-white/10 bg-white/5 p-4 space-y-2">
            <label class="text-sm font-semibold text-slate-200" htmlFor="channel-name">
              Channel name
            </label>
            <input
              id="channel-name"
              class="w-full rounded-xl border border-white/10 bg-slate-900/70 px-3 py-2 text-sm text-slate-100 placeholder:text-slate-500 focus:border-sky-500 focus:outline-none"
              placeholder="Product Launches"
              value={name.value}
              onInput={(event) => (name.value = (event.target as HTMLInputElement).value)}
            />
            {error.value ? <p class="text-xs text-rose-300">{error.value}</p> : null}
          </div>

          <div class="rounded-2xl border border-white/10 bg-white/5 p-4 space-y-2">
            <label class="text-sm font-semibold text-slate-200" htmlFor="channel-description">
              Description
            </label>
            <textarea
              id="channel-description"
              rows={3}
              class="w-full rounded-xl border border-white/10 bg-slate-900/70 px-3 py-2 text-sm text-slate-100 placeholder:text-slate-500 focus:border-sky-500 focus:outline-none"
              placeholder="Tell people what to expect inside your channel."
              value={description.value}
              onInput={(event) => (description.value = (event.target as HTMLTextAreaElement).value)}
            />
            <p class="text-xs text-slate-500">You can edit this later.</p>
          </div>

          <div class="rounded-2xl border border-white/10 bg-white/5 p-4 space-y-3">
            <p class="text-sm font-semibold text-slate-200">Visibility</p>
            <div class="grid gap-3 md:grid-cols-2">
              {(['public', 'private'] as const).map((mode) => (
                <button
                  key={mode}
                  type="button"
                  class={`rounded-xl border px-3 py-3 text-left transition-colors ${
                    visibility.value === mode
                      ? 'border-sky-400/60 bg-sky-500/10 text-sky-100'
                      : 'border-white/10 bg-slate-900/60 text-slate-200 hover:border-white/20'
                  }`}
                  onClick={() => (visibility.value = mode)}
                >
                  <p class="text-sm font-semibold capitalize">{mode}</p>
                  <p class="text-xs text-slate-400">
                    {mode === 'public' ? 'Discoverable via links and search.' : 'Join only with invite links.'}
                  </p>
                </button>
              ))}
            </div>
          </div>

          <div class="flex items-center gap-3">
            <button
              type="submit"
              class="rounded-xl bg-sky-500 px-4 py-2 text-sm font-semibold text-white hover:bg-sky-400 transition-colors"
            >
              Create channel
            </button>
            <a
              href="/explore"
              class="rounded-xl border border-white/10 bg-white/5 px-4 py-2 text-sm text-slate-100 hover:border-sky-400/40"
            >
              Explore channels
            </a>
          </div>
        </form>
      </div>
    </section>
  );
}
