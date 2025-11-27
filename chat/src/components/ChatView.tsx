import { useState, useEffect, useRef } from 'preact/hooks';
import { Avatar, TextInput, ActionIcon, Text, Loader } from '@mantine/core';
import { MessageBubble } from './MessageBubble';
import type { Conversation, Message, User } from '../types';
import type { ChatApiInterface } from '../types';

interface ChatViewProps {
  conversation: Conversation;
  currentUser: User;
  chatService: ChatApiInterface;
  onBack?: () => void;
  isMobileView?: boolean;
}

function getOtherParticipant(conversation: Conversation, currentUser: User): User {
  return conversation.participants.find(p => p.id !== currentUser.id) || conversation.participants[0];
}

function formatLastSeen(date?: Date): string {
  if (!date) return '';
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMins = Math.floor(diffMs / (1000 * 60));
  
  if (diffMins < 1) return 'just now';
  if (diffMins < 60) return `${diffMins} min ago`;
  if (diffMins < 1440) return `${Math.floor(diffMins / 60)} hours ago`;
  return date.toLocaleDateString();
}

export function ChatView({ conversation, currentUser, chatService, onBack, isMobileView = false }: ChatViewProps) {
  const [messages, setMessages] = useState<Message[]>([]);
  const [newMessage, setNewMessage] = useState('');
  const [loading, setLoading] = useState(true);
  const [sending, setSending] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  
  const otherUser = getOtherParticipant(conversation, currentUser);
  const displayName = conversation.isGroup ? conversation.groupName : otherUser.name;
  const avatar = conversation.isGroup ? conversation.groupAvatar : otherUser.avatar;
  
  useEffect(() => {
    loadMessages();
  }, [conversation.id]);
  
  useEffect(() => {
    scrollToBottom();
  }, [messages]);
  
  async function loadMessages() {
    setLoading(true);
    try {
      const msgs = await chatService.listMessagesForConversation(conversation.id);
      setMessages(msgs);
    } finally {
      setLoading(false);
    }
  }
  
  function scrollToBottom() {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }
  
  async function handleSendMessage() {
    if (!newMessage.trim() || sending) return;
    
    setSending(true);
    try {
      const msg = await chatService.sendMessage(conversation.id, newMessage.trim());
      setMessages(prev => [...prev, msg]);
      setNewMessage('');
    } finally {
      setSending(false);
    }
  }
  
  function handleKeyPress(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSendMessage();
    }
  }
  
  function getMessageSender(senderId: string): User | undefined {
    return conversation.participants.find(p => p.id === senderId);
  }
  
  function shouldShowAvatar(index: number): boolean {
    if (index === 0) return true;
    const currentMsg = messages[index];
    const prevMsg = messages[index - 1];
    return currentMsg.senderId !== prevMsg.senderId;
  }
  
  return (
    <div class="flex flex-col h-full bg-[#efeae2]">
      {/* Header */}
      <div class="flex items-center gap-3 p-3 bg-gray-50 border-b border-gray-200">
        {isMobileView && onBack && (
          <ActionIcon variant="subtle" color="gray" size="lg" onClick={onBack}>
            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="m15 18-6-6 6-6" />
            </svg>
          </ActionIcon>
        )}
        
        <Avatar src={avatar} alt={displayName} size="md" radius="xl" />
        
        <div class="flex-1">
          <Text fw={500} size="sm">{displayName}</Text>
          <Text size="xs" c="dimmed">
            {!conversation.isGroup && (
              otherUser.status === 'online' 
                ? 'online' 
                : otherUser.status === 'typing'
                  ? 'typing...'
                  : `last seen ${formatLastSeen(otherUser.lastSeen)}`
            )}
            {conversation.isGroup && `${conversation.participants.length} participants`}
          </Text>
        </div>
        
        <div class="flex gap-1">
          <ActionIcon variant="subtle" color="gray" size="lg">
            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M22 16.92v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.5 19.5 0 0 1-6-6 19.79 19.79 0 0 1-3.07-8.67A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72 12.84 12.84 0 0 0 .7 2.81 2 2 0 0 1-.45 2.11L8.09 9.91a16 16 0 0 0 6 6l1.27-1.27a2 2 0 0 1 2.11-.45 12.84 12.84 0 0 0 2.81.7A2 2 0 0 1 22 16.92z" />
            </svg>
          </ActionIcon>
          <ActionIcon variant="subtle" color="gray" size="lg">
            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="12" cy="12" r="1" />
              <circle cx="19" cy="12" r="1" />
              <circle cx="5" cy="12" r="1" />
            </svg>
          </ActionIcon>
        </div>
      </div>
      
      {/* Messages */}
      <div class="flex-1 overflow-y-auto p-4">
        {loading ? (
          <div class="flex items-center justify-center h-full">
            <Loader color="teal" />
          </div>
        ) : (
          <>
            {messages.map((message, index) => (
              <MessageBubble
                key={message.id}
                message={message}
                isOwnMessage={message.senderId === currentUser.id}
                showAvatar={shouldShowAvatar(index)}
                sender={getMessageSender(message.senderId)}
              />
            ))}
            <div ref={messagesEndRef} />
          </>
        )}
      </div>
      
      {/* Input */}
      <div class="flex items-center gap-2 p-3 bg-gray-50 border-t border-gray-200">
        <ActionIcon variant="subtle" color="gray" size="lg">
          <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="10" />
            <path d="M8 14s1.5 2 4 2 4-2 4-2" />
            <line x1="9" x2="9.01" y1="9" y2="9" />
            <line x1="15" x2="15.01" y1="9" y2="9" />
          </svg>
        </ActionIcon>
        
        <ActionIcon variant="subtle" color="gray" size="lg">
          <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="m21.44 11.05-9.19 9.19a6 6 0 0 1-8.49-8.49l8.57-8.57A4 4 0 1 1 18 8.84l-8.59 8.57a2 2 0 0 1-2.83-2.83l8.49-8.48" />
          </svg>
        </ActionIcon>
        
        <TextInput
          placeholder="Type a message"
          value={newMessage}
          onChange={(e) => setNewMessage((e.target as HTMLInputElement).value)}
          onKeyPress={handleKeyPress}
          class="flex-1"
          size="md"
          radius="lg"
          styles={{
            input: {
              backgroundColor: 'white',
            },
          }}
        />
        
        <ActionIcon 
          variant="filled" 
          color="teal" 
          size="lg" 
          radius="xl"
          onClick={handleSendMessage}
          loading={sending}
          disabled={!newMessage.trim()}
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="m22 2-7 20-4-9-9-4Z" />
            <path d="M22 2 11 13" />
          </svg>
        </ActionIcon>
      </div>
    </div>
  );
}
