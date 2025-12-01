import { useSignal } from '@preact/signals';
import { useEffect, useRef } from 'preact/hooks';
import { useRoute, useNavigation } from '@copilot-test/preact-router';
import { getGroup, getMessages, sendMessage, poll, type GroupInfo } from '../services/api';
import { currentUser } from '../app';
import { encryptMessage, decryptMessage, hasGroupState, loadAllGroupStates, processCommit } from '../services/mls';

interface DecodedMessage {
  id: number;
  text: string;
  sender: string;
  isMe: boolean;
  timestamp: string;
}

export default function Chat() {
  const route = useRoute();
  const { push } = useNavigation();
  const groupId = route.value.params.groupId as string;

  const group = useSignal<GroupInfo | null>(null);
  const messages = useSignal<DecodedMessage[]>([]);
  const newMessage = useSignal('');
  const loading = useSignal(true);
  const sending = useSignal(false);
  const error = useSignal('');
  const lastMessageId = useSignal(0);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!currentUser.value) {
      push('/login');
      return;
    }

    loadAllGroupStates();
    loadGroup();
    loadMessages();
    startPolling();
  }, [groupId]);

  useEffect(() => {
    // Scroll to bottom when messages change
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages.value]);

  const loadGroup = async () => {
    if (!currentUser.value) return;

    try {
      const groupInfo = await getGroup(groupId, currentUser.value.username);
      group.value = groupInfo;
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to load group';
    }
  };

  const loadMessages = async () => {
    if (!currentUser.value) return;

    try {
      const rawMessages = await getMessages(groupId, currentUser.value.username, lastMessageId.value);
      
      const decoded: DecodedMessage[] = [];
      for (const msg of rawMessages) {
        if (msg.message_type === 'application') {
          try {
            const text = await decryptMessage(groupId, msg.message_data);
            decoded.push({
              id: msg.id,
              text,
              sender: msg.sender_name || 'Unknown',
              isMe: msg.sender_name === currentUser.value.username,
              timestamp: msg.created_at,
            });
          } catch {
            decoded.push({
              id: msg.id,
              text: '[Unable to decrypt]',
              sender: msg.sender_name || 'Unknown',
              isMe: false,
              timestamp: msg.created_at,
            });
          }
        } else if (msg.message_type === 'commit') {
          // Process commit to update group state
          try {
            await processCommit(msg.message_data);
          } catch {
            // Ignore commit processing errors
          }
        }
        
        if (msg.id > lastMessageId.value) {
          lastMessageId.value = msg.id;
        }
      }

      if (decoded.length > 0) {
        messages.value = [...messages.value, ...decoded];
      }
    } catch (err) {
      console.error('Failed to load messages:', err);
    } finally {
      loading.value = false;
    }
  };

  const startPolling = () => {
    const pollLoop = async () => {
      if (!currentUser.value) return;

      try {
        const response = await poll(currentUser.value.username);
        
        // Process any commits first
        for (const msg of response.messages) {
          if (msg.message_type === 'commit' && msg.group_id === groupId) {
            try {
              await processCommit(msg.message_data);
            } catch {
              // Ignore
            }
          }
        }

        // Check for new messages in this group
        const hasNewMessages = response.messages.some(
          (m) => m.group_id === groupId && m.message_type === 'application'
        );
        
        if (hasNewMessages) {
          loadMessages();
        }
      } catch {
        // Ignore polling errors
      }

      if (currentUser.value) {
        setTimeout(pollLoop, 1000);
      }
    };

    pollLoop();
  };

  const handleSend = async (event: Event) => {
    event.preventDefault();
    if (!currentUser.value || !newMessage.value.trim() || sending.value) return;

    // Check if we have group state
    if (!hasGroupState(groupId)) {
      error.value = 'Group state not found. Please try refreshing the page.';
      return;
    }

    sending.value = true;
    const messageText = newMessage.value.trim();
    newMessage.value = '';

    try {
      const encrypted = await encryptMessage(groupId, messageText);
      await sendMessage(groupId, currentUser.value.username, encrypted, 'application');
      
      // Add message to local state immediately
      messages.value = [
        ...messages.value,
        {
          id: Date.now(),
          text: messageText,
          sender: currentUser.value.username,
          isMe: true,
          timestamp: new Date().toISOString(),
        },
      ];
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to send message';
      newMessage.value = messageText; // Restore message on error
    } finally {
      sending.value = false;
    }
  };

  if (!currentUser.value) {
    return null;
  }

  return (
    <div class="min-h-screen bg-gray-100 flex flex-col">
      <header class="bg-blue-600 text-white shadow flex-shrink-0">
        <div class="max-w-4xl mx-auto px-4 py-4 flex items-center justify-between">
          <div class="flex items-center gap-4">
            <a href="/groups" class="hover:bg-blue-700 p-2 rounded">
              ←
            </a>
            <div>
              <h1 class="text-xl font-bold">{group.value?.name || 'Loading...'}</h1>
              {group.value && (
                <p class="text-sm text-blue-200">
                  {group.value.member_count} member{group.value.member_count !== 1 ? 's' : ''}
                  {group.value.is_channel ? ' • Channel' : ''}
                </p>
              )}
            </div>
          </div>
          {group.value?.is_admin && !group.value?.is_channel && (
            <a
              href={`/groups/${groupId}/invite`}
              class="px-3 py-1 bg-blue-700 hover:bg-blue-800 rounded text-sm"
            >
              Invite
            </a>
          )}
        </div>
      </header>

      {error.value && (
        <div class="bg-red-50 border-b border-red-200 text-red-700 px-4 py-3 flex-shrink-0">
          {error.value}
          <button onClick={() => (error.value = '')} class="ml-2 underline">
            Dismiss
          </button>
        </div>
      )}

      <main class="flex-1 overflow-y-auto">
        <div class="max-w-4xl mx-auto px-4 py-4">
          {loading.value ? (
            <div class="text-center py-8 text-gray-500">Loading messages...</div>
          ) : messages.value.length === 0 ? (
            <div class="text-center py-8 text-gray-500">
              <p>No messages yet</p>
              <p class="text-sm">Start the conversation!</p>
            </div>
          ) : (
            <div class="space-y-2">
              {messages.value.map((msg) => (
                <div
                  key={msg.id}
                  class={`flex ${msg.isMe ? 'justify-end' : 'justify-start'}`}
                >
                  <div
                    class={`max-w-[70%] rounded-lg px-4 py-2 ${
                      msg.isMe
                        ? 'bg-blue-600 text-white'
                        : 'bg-white shadow text-gray-900'
                    }`}
                  >
                    {!msg.isMe && (
                      <p class="text-xs font-medium text-gray-500 mb-1">{msg.sender}</p>
                    )}
                    <p class="break-words">{msg.text}</p>
                  </div>
                </div>
              ))}
              <div ref={messagesEndRef} />
            </div>
          )}
        </div>
      </main>

      {(!group.value?.is_channel || group.value?.is_admin) && (
        <footer class="bg-white border-t shadow-lg flex-shrink-0">
          <form onSubmit={handleSend} class="max-w-4xl mx-auto px-4 py-3 flex gap-2">
            <input
              type="text"
              value={newMessage.value}
              onInput={(e) => (newMessage.value = (e.target as HTMLInputElement).value)}
              placeholder="Type a message..."
              class="flex-1 px-4 py-2 border border-gray-300 rounded-full focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
            <button
              type="submit"
              disabled={sending.value || !newMessage.value.trim()}
              class="px-6 py-2 bg-blue-600 text-white rounded-full hover:bg-blue-700 disabled:opacity-50"
            >
              Send
            </button>
          </form>
        </footer>
      )}
    </div>
  );
}
