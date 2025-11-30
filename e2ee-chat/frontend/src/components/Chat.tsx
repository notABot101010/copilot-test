import { signal } from '@preact/signals';
import { useEffect, useState } from 'preact/hooks';
import { Button, TextInput, Paper, Title, ScrollArea } from '@mantine/core';
import { useRouter } from '@copilot-test/preact-router';
import { currentUser } from '../app';
import { clearCurrentUser, loadUserKeys, getCurrentUser } from '../crypto/storage';
import { getMessages, sendMessage as apiSendMessage, pollMessages, type Message as APIMessage } from '../api/client';
import { ChatService } from '../services/ChatService';

const messages = signal<{ from: string; to: string; text: string; timestamp: Date }[]>([]);
const selectedUser = signal<string>('');
const newRecipient = signal<string>('');

let chatService: ChatService | null = null;
let pollingActive = false;

export default function Chat() {
  const router = useRouter();
  const [messageText, setMessageText] = useState('');
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    const username = getCurrentUser();
    if (!username) {
      router.push('/login');
      return;
    }

    initializeChatService(username);
    startPolling(username);

    return () => {
      pollingActive = false;
    };
  }, []);

  const initializeChatService = async (username: string) => {
    try {
      const password = prompt('Enter your password to decrypt keys:');
      if (!password) {
        handleLogout();
        return;
      }

      const response = await fetch(`/api/auth/login`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username, password }),
      });

      if (!response.ok) {
        throw new Error('Failed to authenticate');
      }

      const data = await response.json();
      const keys = await loadUserKeys(username, password, data.encrypted_identity_key);
      chatService = new ChatService(username, keys);
    } catch (err) {
      console.error('Failed to initialize chat service:', err);
      handleLogout();
    }
  };

  const startPolling = async (username: string) => {
    pollingActive = true;
    while (pollingActive) {
      try {
        const newMessages = await pollMessages(username);
        for (const msg of newMessages) {
          await handleIncomingMessage(msg);
        }
      } catch (err) {
        console.error('Polling error:', err);
      }
      await new Promise(resolve => setTimeout(resolve, 1000));
    }
  };

  const handleIncomingMessage = async (msg: APIMessage) => {
    if (!chatService) return;

    try {
      const decrypted = await chatService.decryptMessage(msg.from_user, {
        ciphertext: msg.encrypted_content,
        iv: '',
        tag: '',
        dhPublicKey: msg.ephemeral_public_key,
        messageNumber: msg.message_number,
        previousChainLength: msg.previous_chain_length,
      });

      const text = new TextDecoder().decode(decrypted);
      messages.value = [...messages.value, {
        from: msg.from_user,
        to: msg.to_user,
        text,
        timestamp: new Date(msg.created_at),
      }];
    } catch (err) {
      console.error('Failed to decrypt message:', err);
    }
  };

  const loadConversation = async (username: string) => {
    if (!chatService || !currentUser.value) return;

    selectedUser.value = username;
    setLoading(true);

    try {
      const msgs = await getMessages(username, currentUser.value);
      const decryptedMsgs = [];

      for (const msg of msgs) {
        try {
          const decrypted = await chatService.decryptMessage(msg.from_user === currentUser.value ? msg.to_user : msg.from_user, {
            ciphertext: msg.encrypted_content,
            iv: '',
            tag: '',
            dhPublicKey: msg.ephemeral_public_key,
            messageNumber: msg.message_number,
            previousChainLength: msg.previous_chain_length,
          });

          const text = new TextDecoder().decode(decrypted);
          decryptedMsgs.push({
            from: msg.from_user,
            to: msg.to_user,
            text,
            timestamp: new Date(msg.created_at),
          });
        } catch (err) {
          console.error('Failed to decrypt message:', err);
        }
      }

      messages.value = decryptedMsgs;
    } catch (err) {
      console.error('Failed to load messages:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleSendMessage = async (e: Event) => {
    e.preventDefault();
    if (!messageText.trim() || !selectedUser.value || !chatService || !currentUser.value) return;

    try {
      const encrypted = await chatService.encryptMessage(
        selectedUser.value,
        new TextEncoder().encode(messageText)
      );

      await apiSendMessage(
        currentUser.value,
        selectedUser.value,
        encrypted.ciphertext,
        encrypted.dhPublicKey,
        null,
        null,
        encrypted.messageNumber,
        encrypted.previousChainLength
      );

      messages.value = [...messages.value, {
        from: currentUser.value,
        to: selectedUser.value,
        text: messageText,
        timestamp: new Date(),
      }];

      setMessageText('');
    } catch (err) {
      console.error('Failed to send message:', err);
    }
  };

  const handleNewConversation = () => {
    if (newRecipient.value.trim()) {
      loadConversation(newRecipient.value);
      newRecipient.value = '';
    }
  };

  const handleLogout = () => {
    clearCurrentUser();
    currentUser.value = null;
    pollingActive = false;
    router.push('/login');
  };

  const conversationMessages = messages.value.filter(
    m => (m.from === selectedUser.value || m.to === selectedUser.value) &&
         (m.from === currentUser.value || m.to === currentUser.value)
  );

  const conversations = Array.from(
    new Set(
      messages.value
        .map(m => m.from === currentUser.value ? m.to : m.from)
        .filter(u => u !== currentUser.value)
    )
  );

  return (
    <div class="flex h-screen">
      <div class="w-64 bg-white border-r border-gray-200 p-4">
        <div class="mb-4">
          <Title order={4}>E2EE Chat</Title>
          <p class="text-sm text-gray-600">{currentUser.value}</p>
        </div>

        <div class="mb-4">
          <TextInput
            placeholder="New conversation..."
            value={newRecipient.value}
            onChange={(e) => newRecipient.value = (e.target as HTMLInputElement).value}
            onKeyPress={(e) => e.key === 'Enter' && handleNewConversation()}
          />
        </div>

        <ScrollArea class="h-[calc(100vh-200px)]">
          {conversations.map(user => (
            <div
              key={user}
              onClick={() => loadConversation(user)}
              class={`p-3 cursor-pointer rounded mb-2 ${selectedUser.value === user ? 'bg-blue-100' : 'hover:bg-gray-100'}`}
            >
              <p class="font-medium">{user}</p>
            </div>
          ))}
        </ScrollArea>

        <Button onClick={handleLogout} fullWidth class="mt-4" variant="outline">
          Logout
        </Button>
      </div>

      <div class="flex-1 flex flex-col">
        {selectedUser.value ? (
          <>
            <div class="bg-white border-b border-gray-200 p-4">
              <Title order={4}>{selectedUser.value}</Title>
            </div>

            <ScrollArea class="flex-1 p-4">
              {loading ? (
                <p class="text-center text-gray-500">Loading messages...</p>
              ) : conversationMessages.length === 0 ? (
                <p class="text-center text-gray-500">No messages yet. Start the conversation!</p>
              ) : (
                <div class="space-y-4">
                  {conversationMessages.map((msg, i) => (
                    <div
                      key={i}
                      class={`flex ${msg.from === currentUser.value ? 'justify-end' : 'justify-start'}`}
                    >
                      <Paper
                        class={`max-w-md p-3 ${msg.from === currentUser.value ? 'bg-blue-500 text-white' : 'bg-white'}`}
                      >
                        <p>{msg.text}</p>
                        <p class={`text-xs mt-1 ${msg.from === currentUser.value ? 'text-blue-100' : 'text-gray-500'}`}>
                          {msg.timestamp.toLocaleTimeString()}
                        </p>
                      </Paper>
                    </div>
                  ))}
                </div>
              )}
            </ScrollArea>

            <form onSubmit={handleSendMessage} class="bg-white border-t border-gray-200 p-4 flex gap-2">
              <TextInput
                class="flex-1"
                placeholder="Type a message..."
                value={messageText}
                onChange={(e) => setMessageText((e.target as HTMLInputElement).value)}
              />
              <Button type="submit">Send</Button>
            </form>
          </>
        ) : (
          <div class="flex-1 flex items-center justify-center">
            <p class="text-gray-500">Select a conversation or start a new one</p>
          </div>
        )}
      </div>
    </div>
  );
}
