import { useState, useEffect, useRef } from 'preact/hooks';
import { Avatar, TextInput, ActionIcon, Text, Loader } from '@mantine/core';
import { IconArrowLeft, IconPhone, IconDotsVertical, IconMoodSmile, IconPaperclip, IconSend } from '@tabler/icons-react';
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
    <div className="flex flex-col h-full bg-[#efeae2]">
      {/* Header */}
      <div className="flex items-center gap-3 p-3 bg-gray-50 border-b border-gray-200">
        {isMobileView && onBack && (
          <ActionIcon variant="subtle" color="gray" size="lg" onClick={onBack}>
            <IconArrowLeft size={20} />
          </ActionIcon>
        )}
        
        <Avatar src={avatar} alt={displayName} size="md" radius="xl" />
        
        <div className="flex-1">
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
        
        <div className="flex gap-1">
          <ActionIcon variant="subtle" color="gray" size="lg">
            <IconPhone size={20} />
          </ActionIcon>
          <ActionIcon variant="subtle" color="gray" size="lg">
            <IconDotsVertical size={20} />
          </ActionIcon>
        </div>
      </div>
      
      {/* Messages */}
      <div className="flex-1 overflow-y-auto p-4">
        {loading ? (
          <div className="flex items-center justify-center h-full">
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
      <div className="flex items-center gap-2 p-3 bg-gray-50 border-t border-gray-200">
        <ActionIcon variant="subtle" color="gray" size="lg">
          <IconMoodSmile size={20} />
        </ActionIcon>
        
        <ActionIcon variant="subtle" color="gray" size="lg">
          <IconPaperclip size={20} />
        </ActionIcon>
        
        <TextInput
          placeholder="Type a message"
          value={newMessage}
          onChange={(e) => setNewMessage((e.target as HTMLInputElement).value)}
          onKeyPress={handleKeyPress}
          className="flex-1"
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
          <IconSend size={18} />
        </ActionIcon>
      </div>
    </div>
  );
}
