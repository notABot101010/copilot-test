import { useEffect } from 'preact/hooks';
import { useRoute, useRouter } from '@copilot-test/preact-router';
import { conversations, drafts, sendMessage, updateDraft } from '../state/chatStore';
import type { Conversation, Message } from '../data/mockData';

function ChatHeader({ conversation }: { conversation: Conversation }) {
  return (
    <div class="flex items-center justify-between border-b border-white/5 px-6 py-4 bg-slate-900/60 backdrop-blur">
      <div class="flex items-center gap-3">
        <div class="h-11 w-11 rounded-full bg-gradient-to-br from-sky-500 to-blue-500 flex items-center justify-center text-lg font-semibold text-white">
          {conversation.title.slice(0, 2).toUpperCase()}
        </div>
        <div>
          <div class="flex items-center gap-2">
            <p class="text-lg font-semibold text-slate-50">{conversation.title}</p>
            {conversation.pinned ? (
              <span class="text-[11px] px-2 py-0.5 rounded-full bg-white/10 text-sky-200 border border-white/10">Pinned</span>
            ) : null}
          </div>
          <p class="text-sm text-slate-400">
            {conversation.type === 'channel'
              ? `${conversation.members ?? 1} members - ${conversation.lastActive}`
              : conversation.online
                ? 'online'
                : `last seen ${conversation.lastActive}`}
          </p>
        </div>
      </div>
      <div class="hidden md:flex items-center gap-2 text-sm text-slate-300">
        <button class="px-3 py-2 rounded-xl border border-white/5 bg-white/5 hover:border-sky-400/50 transition-colors">
          Search
        </button>
        <button class="px-3 py-2 rounded-xl border border-white/5 bg-white/5 hover:border-sky-400/50 transition-colors">
          Voice
        </button>
        <button class="px-3 py-2 rounded-xl border border-white/5 bg-white/5 hover:border-sky-400/50 transition-colors">
          More
        </button>
      </div>
    </div>
  );
}

function MessageBubble({ message }: { message: Message }) {
  const isMine = message.sender === 'me';
  return (
    <div class={`flex ${isMine ? 'justify-end' : 'justify-start'}`}>
      <div
        class={`max-w-[70%] rounded-2xl px-4 py-3 shadow-lg shadow-black/20 ${isMine ? 'bg-sky-600 text-white' : 'bg-slate-800/80 text-slate-100'}`}
      >
        <p class="text-sm leading-relaxed">{message.text}</p>
        <div class={`mt-2 flex items-center gap-2 text-[11px] ${isMine ? 'text-slate-100/80 justify-end' : 'text-slate-400'}`}>
          <span>{message.time}</span>
          {isMine ? <span class="text-[10px] uppercase tracking-wide">sent</span> : null}
        </div>
      </div>
    </div>
  );
}

function MessageComposer({ conversation }: { conversation: Conversation }) {
  const draft = drafts.value[conversation.id] ?? '';

  const handleSend = () => {
    sendMessage(conversation.id, draft);
  };

  const handleKeyDown = (event: KeyboardEvent) => {
    if (event.key === 'Enter' && !event.shiftKey) {
      event.preventDefault();
      handleSend();
    }
  };

  return (
    <div class="border-t border-white/5 bg-slate-900/70 backdrop-blur px-4 py-3">
      <div class="rounded-2xl border border-white/10 bg-slate-800/70 px-3 py-2 shadow-inner shadow-black/30">
        <textarea
          class="w-full bg-transparent text-slate-100 placeholder:text-slate-500 focus:outline-none resize-none text-sm"
          rows={2}
          placeholder="Message"
          value={draft}
          onInput={(event) => updateDraft(conversation.id, (event.target as HTMLTextAreaElement).value)}
          onKeyDown={handleKeyDown}
        />
        <div class="flex items-center justify-between pt-2">
          <div class="flex items-center gap-2 text-slate-400 text-sm">
            <span class="px-2 py-1 rounded-lg bg-white/5 border border-white/5">Attach</span>
            <span class="px-2 py-1 rounded-lg bg-white/5 border border-white/5">Voice</span>
          </div>
          <button
            type="button"
            class="px-4 py-2 rounded-xl bg-sky-500 text-white font-semibold hover:bg-sky-400 transition-colors"
            onClick={handleSend}
          >
            Send
          </button>
        </div>
      </div>
    </div>
  );
}

export function ChatPage() {
  const route = useRoute();
  const router = useRouter();

  const chatId = (route.value.params.chatId as string | undefined) ?? conversations.value[0]?.id;
  const conversation = conversations.value.find((item) => item.id === chatId);

  useEffect(() => {
    if (!route.value.params.chatId && conversations.value[0]) {
      router.replace(`/chats/${conversations.value[0].id}`);
    }
  }, [route.value.params.chatId, router]);

  if (!conversation) {
    return (
      <section class="flex h-full flex-1 items-center justify-center bg-slate-900/40">
        <div class="text-center space-y-3">
          <p class="text-xl font-semibold text-slate-100">No conversation selected</p>
          <p class="text-slate-400">Pick a chat on the left or create a new channel.</p>
          <div class="flex justify-center gap-2 text-sm">
            <a href="/create-channel" class="px-4 py-2 rounded-xl bg-sky-500 text-white font-semibold hover:bg-sky-400 transition-colors">
              Create channel
            </a>
            <a href="/explore" class="px-4 py-2 rounded-xl bg-white/10 text-slate-100 font-semibold hover:bg-white/20 transition-colors">
              Explore
            </a>
          </div>
        </div>
      </section>
    );
  }

  return (
    <section class="flex h-full flex-1 flex-col">
      <ChatHeader conversation={conversation} />
      <div class="flex-1 overflow-y-auto px-4 sm:px-6 py-6 space-y-4 bg-gradient-to-b from-slate-950 to-slate-900">
        {conversation.messages.map((message) => (
          <MessageBubble key={message.id} message={message} />
        ))}
      </div>
      <MessageComposer conversation={conversation} />
    </section>
  );
}
