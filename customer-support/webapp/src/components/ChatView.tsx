import { useEffect, useRef, useState } from 'preact/hooks';
import { IconSend, IconCheck, IconX } from '@tabler/icons-react';
import {
  conversations,
  selectedConversation,
  messages,
  setMessages,
  addMessage,
  updateConversationInList,
  currentWorkspace,
} from '../state';
import * as api from '../services/api';
import type { Conversation, Message } from '../types';

function formatTime(timestamp: number): string {
  const date = new Date(timestamp);
  const now = new Date();
  const diff = now.getTime() - date.getTime();
  const days = Math.floor(diff / (1000 * 60 * 60 * 24));

  if (days === 0) {
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  } else if (days === 1) {
    return 'Yesterday';
  } else if (days < 7) {
    return date.toLocaleDateString([], { weekday: 'short' });
  } else {
    return date.toLocaleDateString([], { month: 'short', day: 'numeric' });
  }
}

function ConversationItem({
  conversation,
  isSelected,
  onClick,
}: {
  conversation: Conversation;
  isSelected: boolean;
  onClick: () => void;
}) {
  return (
    <div
      onClick={onClick}
      className={`p-3 border-b border-gray-200 cursor-pointer transition-colors ${
        isSelected ? 'bg-blue-50' : 'hover:bg-gray-50'
      }`}
    >
      <div className="flex items-center justify-between mb-1">
        <span className="font-medium text-gray-900 truncate">
          {conversation.contact_name || 'Visitor'}
        </span>
        <span className="text-xs text-gray-500">{formatTime(conversation.updated_at)}</span>
      </div>
      <div className="flex items-center justify-between">
        <p className="text-sm text-gray-600 truncate flex-1">
          {conversation.last_message || 'No messages yet'}
        </p>
        <span
          className={`ml-2 px-2 py-0.5 text-xs rounded-full ${
            conversation.status === 'open'
              ? 'bg-green-100 text-green-800'
              : 'bg-gray-100 text-gray-600'
          }`}
        >
          {conversation.status}
        </span>
      </div>
    </div>
  );
}

function MessageBubble({ message }: { message: Message }) {
  const isAgent = message.sender_type === 'agent';

  return (
    <div className={`flex mb-3 ${isAgent ? 'justify-end' : 'justify-start'}`}>
      <div
        className={`max-w-[70%] px-4 py-2 rounded-2xl ${
          isAgent
            ? 'bg-blue-600 text-white rounded-br-md'
            : 'bg-gray-100 text-gray-900 rounded-bl-md'
        }`}
      >
        <p className="text-sm whitespace-pre-wrap">{message.content}</p>
        <p
          className={`text-xs mt-1 ${
            isAgent ? 'text-blue-200' : 'text-gray-500'
          }`}
        >
          {formatTime(message.created_at)}
        </p>
      </div>
    </div>
  );
}

export function ChatView() {
  const workspace = currentWorkspace.value;
  const conversation = selectedConversation.value;
  const [newMessage, setNewMessage] = useState('');
  const [sending, setSending] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (workspace) {
      loadConversations();
    }
  }, [workspace?.id]);

  useEffect(() => {
    if (workspace && conversation) {
      loadMessages();
    }
  }, [workspace?.id, conversation?.id]);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages.value]);

  async function loadConversations() {
    if (!workspace) return;
    try {
      const convs = await api.listConversations(workspace.id);
      conversations.value = convs;
    } catch (err) {
      console.error('Failed to load conversations:', err);
    }
  }

  async function loadMessages() {
    if (!workspace || !conversation) return;
    try {
      const msgs = await api.listMessages(workspace.id, conversation.id);
      setMessages(msgs);
    } catch (err) {
      console.error('Failed to load messages:', err);
    }
  }

  async function handleSend() {
    if (!workspace || !conversation || !newMessage.trim() || sending) return;

    setSending(true);
    try {
      const msg = await api.sendMessage(workspace.id, conversation.id, newMessage.trim());
      addMessage(msg);
      setNewMessage('');
      // Update conversation in list
      const updatedConv = await api.getConversation(workspace.id, conversation.id);
      updateConversationInList(updatedConv);
    } catch (err) {
      console.error('Failed to send message:', err);
    } finally {
      setSending(false);
    }
  }

  async function handleStatusChange(newStatus: 'open' | 'closed') {
    if (!workspace || !conversation) return;
    try {
      const updated = await api.updateConversation(workspace.id, conversation.id, newStatus);
      updateConversationInList(updated);
      selectedConversation.value = updated;
    } catch (err) {
      console.error('Failed to update conversation:', err);
    }
  }

  function selectConversation(conv: Conversation) {
    selectedConversation.value = conv;
  }

  if (!workspace) {
    return (
      <div className="flex-1 flex items-center justify-center bg-gray-50">
        <p className="text-gray-500">No workspace selected</p>
      </div>
    );
  }

  return (
    <div className="flex-1 flex h-screen overflow-hidden">
      {/* Conversations list */}
      <div className="w-80 border-r border-gray-200 bg-white flex flex-col">
        <div className="p-4 border-b border-gray-200">
          <h2 className="text-lg font-semibold text-gray-900">Conversations</h2>
        </div>
        <div className="flex-1 overflow-y-auto">
          {conversations.value.length === 0 ? (
            <div className="p-4 text-center text-gray-500">
              No conversations yet
            </div>
          ) : (
            conversations.value.map((conv) => (
              <ConversationItem
                key={conv.id}
                conversation={conv}
                isSelected={conversation?.id === conv.id}
                onClick={() => selectConversation(conv)}
              />
            ))
          )}
        </div>
      </div>

      {/* Chat area */}
      <div className="flex-1 flex flex-col bg-gray-50">
        {conversation ? (
          <>
            {/* Chat header */}
            <div className="p-4 bg-white border-b border-gray-200 flex items-center justify-between">
              <div>
                <h3 className="font-medium text-gray-900">
                  {conversation.contact_name || 'Visitor'}
                </h3>
                <p className="text-sm text-gray-500">Contact ID: {conversation.contact_id}</p>
              </div>
              <div className="flex gap-2">
                {conversation.status === 'open' ? (
                  <button
                    onClick={() => handleStatusChange('closed')}
                    className="flex items-center gap-1 px-3 py-1.5 bg-gray-100 hover:bg-gray-200 text-gray-700 rounded-md text-sm transition-colors"
                  >
                    <IconCheck size={16} />
                    Close
                  </button>
                ) : (
                  <button
                    onClick={() => handleStatusChange('open')}
                    className="flex items-center gap-1 px-3 py-1.5 bg-green-100 hover:bg-green-200 text-green-700 rounded-md text-sm transition-colors"
                  >
                    <IconX size={16} />
                    Reopen
                  </button>
                )}
              </div>
            </div>

            {/* Messages */}
            <div className="flex-1 overflow-y-auto p-4">
              {messages.value.map((msg) => (
                <MessageBubble key={msg.id} message={msg} />
              ))}
              <div ref={messagesEndRef} />
            </div>

            {/* Message input */}
            <div className="p-4 bg-white border-t border-gray-200">
              <div className="flex gap-2">
                <input
                  type="text"
                  value={newMessage}
                  onChange={(e) => setNewMessage((e.target as HTMLInputElement).value)}
                  onKeyDown={(e) => e.key === 'Enter' && handleSend()}
                  placeholder="Type a message..."
                  className="flex-1 px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:border-blue-500"
                  disabled={conversation.status === 'closed'}
                />
                <button
                  onClick={handleSend}
                  disabled={!newMessage.trim() || sending || conversation.status === 'closed'}
                  className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                >
                  <IconSend size={20} />
                </button>
              </div>
            </div>
          </>
        ) : (
          <div className="flex-1 flex items-center justify-center">
            <div className="text-center text-gray-500">
              <IconMessageCircle size={48} className="mx-auto mb-2 opacity-50" />
              <p>Select a conversation to start chatting</p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

import { IconMessageCircle } from '@tabler/icons-react';
