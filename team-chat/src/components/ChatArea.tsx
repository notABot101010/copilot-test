import { useState, useEffect, useRef } from 'preact/hooks';
import { TextInput, ActionIcon, Text, Loader, Tooltip } from '@mantine/core';
import { IconHash, IconBell, IconPin, IconUsers, IconSearch, IconInbox, IconHelp, IconGift, IconMoodSmile, IconPlus, IconSend } from '@tabler/icons-react';
import { MessageBubble } from './MessageBubble';
import type { Channel, Message, User } from '../types';
import type { ChatApiInterface } from '../types';

interface ChatAreaProps {
  channel: Channel;
  currentUser: User;
  chatService: ChatApiInterface;
  onToggleMembers: () => void;
  showMembers: boolean;
}

// Maximum number of lines for the textarea
const MAX_LINES = 5;
const LINE_HEIGHT = 24; // approximate line height in pixels

export function ChatArea({ channel, currentUser, chatService, onToggleMembers, showMembers }: ChatAreaProps) {
  const [messages, setMessages] = useState<Message[]>([]);
  const [newMessage, setNewMessage] = useState('');
  const [loading, setLoading] = useState(true);
  const [sending, setSending] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    loadMessages();
  }, [channel.id]);

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  // Auto-resize textarea based on content
  useEffect(() => {
    if (textareaRef.current) {
      // Reset height to auto to get the correct scrollHeight
      textareaRef.current.style.height = 'auto';
      // Calculate new height, capped at max lines
      const maxHeight = MAX_LINES * LINE_HEIGHT;
      const newHeight = Math.min(textareaRef.current.scrollHeight, maxHeight);
      textareaRef.current.style.height = `${newHeight}px`;
    }
  }, [newMessage]);

  async function loadMessages() {
    setLoading(true);
    try {
      const msgs = await chatService.listMessagesForChannel(channel.id);
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
      const msg = await chatService.sendMessage(channel.id, newMessage.trim());
      setMessages(prev => [...prev, msg]);
      setNewMessage('');
    } finally {
      setSending(false);
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    // Enter without Shift sends the message
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSendMessage();
    }
    // Shift+Enter adds a new line (default behavior, no need to handle)
  }

  function shouldShowHeader(index: number): boolean {
    if (index === 0) return true;
    const currentMsg = messages[index];
    const prevMsg = messages[index - 1];

    // Show header if different author or more than 5 minutes apart
    if (currentMsg.author.id !== prevMsg.author.id) return true;
    const timeDiff = currentMsg.timestamp.getTime() - prevMsg.timestamp.getTime();
    return timeDiff > 5 * 60 * 1000;
  }

  return (
    <div className="flex flex-col h-full bg-[#313338] flex-1 min-w-0">
      {/* Channel header */}
      <div className="flex items-center h-12 px-4 border-b border-[#1f2023] shadow-sm shrink-0">
        <IconHash size={20} className="text-[#80848e] mr-2" />
        <Text fw={600} size="sm" className="text-white">
          {channel.name}
        </Text>
        {channel.topic && (
          <>
            <div className="w-px h-6 bg-[#3f4147] mx-3" />
            <Text size="sm" className="text-[#949ba4] truncate flex-1">
              {channel.topic}
            </Text>
          </>
        )}

        <div className="flex items-center gap-4 ml-auto">
          <Tooltip label="Threads">
            <IconHash size={20} className="text-[#b5bac1] cursor-pointer hover:text-[#dbdee1]" />
          </Tooltip>
          <Tooltip label="Notification Settings">
            <IconBell size={20} className="text-[#b5bac1] cursor-pointer hover:text-[#dbdee1]" />
          </Tooltip>
          <Tooltip label="Pinned Messages">
            <IconPin size={20} className="text-[#b5bac1] cursor-pointer hover:text-[#dbdee1]" />
          </Tooltip>
          <Tooltip label={showMembers ? 'Hide Member List' : 'Show Member List'}>
            <ActionIcon variant="transparent" onClick={onToggleMembers}>
              <IconUsers size={20} className={`cursor-pointer ${showMembers ? 'text-white' : 'text-[#b5bac1] hover:text-[#dbdee1]'}`} />
            </ActionIcon>
          </Tooltip>

          <div className="relative hidden md:block">
            <TextInput
              placeholder="Search"
              size="xs"
              leftSection={<IconSearch size={14} />}
              className="w-36"
              styles={{
                input: {
                  backgroundColor: '#1e1f22',
                  border: 'none',
                  color: '#dbdee1',
                  '&::placeholder': {
                    color: '#949ba4',
                  },
                },
              }}
            />
          </div>

          <Tooltip label="Inbox">
            <IconInbox size={20} className="text-[#b5bac1] cursor-pointer hover:text-[#dbdee1] hidden md:block" />
          </Tooltip>
          <Tooltip label="Help">
            <IconHelp size={20} className="text-[#b5bac1] cursor-pointer hover:text-[#dbdee1] hidden md:block" />
          </Tooltip>
        </div>
      </div>

      {/* Messages area */}
      <div className="flex-1 overflow-y-auto px-4">
        {loading ? (
          <div className="flex items-center justify-center h-full">
            <Loader color="gray" />
          </div>
        ) : messages.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full">
            <div className="w-16 h-16 rounded-full bg-[#5865f2] flex items-center justify-center mb-4">
              <IconHash size={32} className="text-white" />
            </div>
            <Text fw={700} size="xl" className="text-white mb-2">
              Welcome to #{channel.name}!
            </Text>
            <Text size="sm" className="text-[#949ba4]">
              This is the start of the #{channel.name} channel.
            </Text>
          </div>
        ) : (
          <>
            <div className="pt-4" />
            {messages.map((message, index) => (
              <MessageBubble
                key={message.id}
                message={message}
                isOwnMessage={message.author.id === currentUser.id}
                showHeader={shouldShowHeader(index)}
              />
            ))}
            <div ref={messagesEndRef} className="h-6" />
          </>
        )}
      </div>

      {/* Message input */}
      <div className="px-3 pb-3 shrink-0">
        <div className="flex items-start bg-[#383a40] rounded-lg px-4 py-2">
          <ActionIcon variant="transparent" className="text-[#b5bac1] hover:text-[#dbdee1] mt-1">
            <IconPlus size={20} />
          </ActionIcon>

          <textarea
            ref={textareaRef}
            placeholder={`Message #${channel.name}`}
            value={newMessage}
            onInput={(e) => setNewMessage((e.target as HTMLTextAreaElement).value)}
            onKeyDown={handleKeyDown}
            className="flex-1 bg-transparent border-none outline-none resize-none text-[#dbdee1] placeholder-[#6d6f78] py-1 px-2 text-sm leading-6"
            style={{ minHeight: '24px', maxHeight: `${MAX_LINES * LINE_HEIGHT}px` }}
            rows={1}
          />

          <div className="flex items-center gap-2 mt-1">
            <ActionIcon variant="transparent" className="text-[#b5bac1] hover:text-[#dbdee1] hidden md:flex">
              <IconGift size={20} />
            </ActionIcon>
            <ActionIcon variant="transparent" className="text-[#b5bac1] hover:text-[#dbdee1]">
              <IconMoodSmile size={20} />
            </ActionIcon>
            {newMessage.trim() && (
              <ActionIcon
                variant="filled"
                color="indigo"
                onClick={handleSendMessage}
                loading={sending}
              >
                <IconSend size={16} />
              </ActionIcon>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
