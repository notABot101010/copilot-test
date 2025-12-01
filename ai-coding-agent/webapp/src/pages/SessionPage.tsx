import { useSignal } from '@preact/signals';
import { useEffect, useRef } from 'preact/hooks';
import type { Message, StreamEvent } from '../types';
import { getMessages, sendMessage, createSessionStream } from '../api';
import { MessageBubble } from '../components/MessageBubble';
import { SteeringControls } from '../components/SteeringControls';

interface Props {
  sessionId: string;
}

export function SessionPage({ sessionId }: Props) {
  const messages = useSignal<Message[]>([]);
  const loading = useSignal(true);
  const sending = useSignal(false);
  const input = useSignal('');
  const isRunning = useSignal(false);
  const isPaused = useSignal(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    loadMessages();
    const ws = connectWebSocket();
    return () => ws?.close();
  }, [sessionId]);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages.value]);

  const loadMessages = async () => {
    loading.value = true;
    try {
      messages.value = await getMessages(sessionId);
    } catch (err) {
      console.error('Failed to load messages:', err);
    } finally {
      loading.value = false;
    }
  };

  const connectWebSocket = () => {
    try {
      const ws = createSessionStream(sessionId);
      
      ws.onmessage = (event) => {
        try {
          const streamEvent: StreamEvent = JSON.parse(event.data);
          handleStreamEvent(streamEvent);
        } catch (err) {
          console.error('Failed to parse stream event:', err);
        }
      };
      
      ws.onerror = (err) => {
        console.error('WebSocket error:', err);
      };
      
      return ws;
    } catch (err) {
      console.error('Failed to connect WebSocket:', err);
      return null;
    }
  };

  const handleStreamEvent = (event: StreamEvent) => {
    switch (event.event_type) {
      case 'task_started':
        isRunning.value = true;
        isPaused.value = false;
        break;
      case 'task_completed':
      case 'task_failed':
        loadMessages();
        break;
      case 'agent_response':
        isRunning.value = false;
        isPaused.value = false;
        loadMessages();
        break;
      case 'agent_thinking':
        if (event.data.steering === 'Pause') {
          isPaused.value = true;
        } else if (event.data.steering === 'Resume') {
          isPaused.value = false;
        } else if (event.data.steering === 'Cancel') {
          isRunning.value = false;
          isPaused.value = false;
        }
        break;
    }
  };

  const handleSend = async () => {
    if (!input.value.trim() || sending.value) return;
    
    sending.value = true;
    try {
      const message = await sendMessage(sessionId, input.value);
      messages.value = [...messages.value, message];
      input.value = '';
      isRunning.value = true;
    } catch (err) {
      console.error('Failed to send message:', err);
    } finally {
      sending.value = false;
    }
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  return (
    <div className="flex flex-col h-[calc(100vh-120px)]">
      <div className="flex items-center justify-between mb-4">
        <a href="/" className="text-blue-600 hover:text-blue-800">← Back to Sessions</a>
        <span className="text-gray-500 text-sm">Session: {sessionId.slice(0, 8)}...</span>
      </div>

      <SteeringControls 
        sessionId={sessionId} 
        isRunning={isRunning.value} 
        isPaused={isPaused.value} 
      />

      <div className="flex-1 overflow-y-auto bg-gray-100 rounded-lg p-4 my-4">
        {loading.value ? (
          <div className="text-center py-8 text-gray-500">Loading messages...</div>
        ) : messages.value.length === 0 ? (
          <div className="text-center py-8 text-gray-500">
            No messages yet. Start a conversation!
          </div>
        ) : (
          <div className="space-y-4">
            {messages.value.map((msg) => (
              <MessageBubble key={msg.id} message={msg} />
            ))}
            <div ref={messagesEndRef} />
          </div>
        )}
      </div>

      <div className="flex gap-3">
        <textarea
          value={input.value}
          onInput={(e) => input.value = (e.target as HTMLTextAreaElement).value}
          onKeyDown={handleKeyDown}
          placeholder="Type your message... (Enter to send, Shift+Enter for new line)"
          className="flex-1 px-4 py-3 border border-gray-300 rounded-lg resize-none focus:outline-none focus:ring-2 focus:ring-blue-500"
          rows={2}
          disabled={sending.value}
        />
        <button
          onClick={handleSend}
          disabled={sending.value || !input.value.trim()}
          className="px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {sending.value ? 'Sending...' : 'Send'}
        </button>
      </div>
    </div>
  );
}
