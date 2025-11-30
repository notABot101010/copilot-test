import { useState, useEffect, useRef } from 'preact/hooks';
import { Button, TextInput, Paper, Container, Title } from '@mantine/core';
import { useRoute } from '@copilot-test/preact-router';
import { currentUser, sendMessage, getConversation, markConversationAsRead, conversations } from '../services/chatService';
import { router } from '../router';

export function ChatPage() {
  const route = useRoute();
  const peerUsername = route.value.params.username as string;
  
  const [message, setMessage] = useState('');
  const [sending, setSending] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  
  const user = currentUser.value;
  
  // Subscribe to conversations signal to get updates
  conversations.value; // Access to subscribe
  const conversation = getConversation(peerUsername);

  // Redirect if not logged in
  if (!user) {
    router.push('/');
    return null;
  }

  // Mark conversation as read when viewing
  useEffect(() => {
    if (conversation) {
      markConversationAsRead(peerUsername);
    }
  }, [peerUsername, conversation?.messages.length]);

  // Scroll to bottom when new messages arrive
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [conversation?.messages.length]);

  async function handleSend(event?: Event) {
    event?.preventDefault();
    
    if (!message.trim() || sending) {
      return;
    }

    setSending(true);
    try {
      await sendMessage(peerUsername, message.trim());
      setMessage('');
    } catch (err) {
      console.error('Failed to send message:', err);
      alert('Failed to send message');
    } finally {
      setSending(false);
    }
  }

  function formatTime(timestamp: number): string {
    const date = new Date(timestamp);
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (event.key === 'Enter' && !event.shiftKey) {
      event.preventDefault();
      handleSend();
    }
  }

  const messages = conversation?.messages || [];

  return (
    <Container size="md" className="h-screen flex flex-col py-4">
      {/* Header */}
      <Paper shadow="md" p="md" radius="md" className="mb-4 flex-shrink-0">
        <div className="flex items-center gap-4">
          <a href="/conversations" className="text-gray-400 hover:text-white">
            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M19 12H5M12 19l-7-7 7-7"/>
            </svg>
          </a>
          <div>
            <Title order={3} className="m-0">{peerUsername}</Title>
            <p className="text-gray-400 text-xs m-0">End-to-end encrypted</p>
          </div>
        </div>
      </Paper>

      {/* Messages */}
      <Paper shadow="md" radius="md" className="flex-1 overflow-hidden flex flex-col min-h-0">
        <div className="flex-1 overflow-y-auto p-4 space-y-3">
          {messages.length === 0 ? (
            <div className="text-center text-gray-400 py-8">
              <p>No messages yet</p>
              <p className="text-sm">Send a message to start the conversation</p>
            </div>
          ) : (
            messages.map((msg) => (
              <div
                key={msg.id}
                className={`flex ${msg.isOutgoing ? 'justify-end' : 'justify-start'}`}
              >
                <div
                  className={`max-w-[70%] rounded-lg px-4 py-2 ${
                    msg.isOutgoing
                      ? 'bg-blue-600 text-white'
                      : 'bg-gray-700 text-white'
                  }`}
                >
                  <p className="m-0 break-words">{msg.content}</p>
                  <p className={`text-xs mt-1 m-0 ${msg.isOutgoing ? 'text-blue-200' : 'text-gray-400'}`}>
                    {formatTime(msg.timestamp)}
                  </p>
                </div>
              </div>
            ))
          )}
          <div ref={messagesEndRef} />
        </div>

        {/* Input */}
        <div className="p-4 border-t border-gray-700 flex-shrink-0">
          <form onSubmit={handleSend} className="flex gap-2">
            <TextInput
              placeholder="Type a message..."
              value={message}
              onChange={(event: Event) => setMessage((event.target as HTMLInputElement).value)}
              onKeyDown={handleKeyDown}
              className="flex-1"
              size="md"
              disabled={sending}
            />
            <Button 
              type="submit" 
              loading={sending}
              disabled={!message.trim()}
              size="md"
            >
              Send
            </Button>
          </form>
        </div>
      </Paper>
    </Container>
  );
}
