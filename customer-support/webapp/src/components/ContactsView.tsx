import { useEffect, useState } from 'preact/hooks';
import { IconMail, IconUser, IconMessageCircle, IconClock } from '@tabler/icons-react';
import { contacts, setContacts, selectedContact, currentWorkspace } from '../state';
import * as api from '../services/api';
import type { Contact, Conversation } from '../types';

function formatDate(timestamp: number): string {
  return new Date(timestamp).toLocaleDateString([], {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}

function ContactCard({
  contact,
  isSelected,
  onClick,
}: {
  contact: Contact;
  isSelected: boolean;
  onClick: () => void;
}) {
  return (
    <div
      onClick={onClick}
      className={`p-4 border-b border-gray-200 cursor-pointer transition-colors ${
        isSelected ? 'bg-blue-50' : 'hover:bg-gray-50'
      }`}
    >
      <div className="flex items-center gap-3">
        <div className="w-10 h-10 rounded-full bg-gray-200 flex items-center justify-center">
          <IconUser size={20} className="text-gray-500" />
        </div>
        <div className="flex-1 min-w-0">
          <p className="font-medium text-gray-900 truncate">
            {contact.name || 'Visitor'}
          </p>
          {contact.email && (
            <p className="text-sm text-gray-500 truncate">{contact.email}</p>
          )}
        </div>
      </div>
      <div className="mt-2 text-xs text-gray-500">
        Last seen: {formatDate(contact.last_seen_at)}
      </div>
    </div>
  );
}

export function ContactsView() {
  const workspace = currentWorkspace.value;
  const contact = selectedContact.value;
  const [conversations, setConversations] = useState<Conversation[]>([]);
  const [loading, setLoading] = useState(true);
  const [editing, setEditing] = useState(false);
  const [editName, setEditName] = useState('');
  const [editEmail, setEditEmail] = useState('');

  useEffect(() => {
    if (workspace) {
      loadContacts();
    }
  }, [workspace?.id]);

  useEffect(() => {
    if (workspace && contact) {
      loadContactConversations();
      setEditName(contact.name || '');
      setEditEmail(contact.email || '');
    }
  }, [workspace?.id, contact?.id]);

  async function loadContacts() {
    if (!workspace) return;
    setLoading(true);
    try {
      const data = await api.listContacts(workspace.id);
      setContacts(data);
    } catch (err) {
      console.error('Failed to load contacts:', err);
    } finally {
      setLoading(false);
    }
  }

  async function loadContactConversations() {
    if (!workspace || !contact) return;
    try {
      const data = await api.getContactConversations(workspace.id, contact.id);
      setConversations(data);
    } catch (err) {
      console.error('Failed to load contact conversations:', err);
    }
  }

  async function handleSaveContact() {
    if (!workspace || !contact) return;
    try {
      const updated = await api.updateContact(workspace.id, contact.id, {
        name: editName || undefined,
        email: editEmail || undefined,
      });
      selectedContact.value = updated;
      // Update in list
      const idx = contacts.value.findIndex((c) => c.id === contact.id);
      if (idx >= 0) {
        const newContacts = [...contacts.value];
        newContacts[idx] = updated;
        setContacts(newContacts);
      }
      setEditing(false);
    } catch (err) {
      console.error('Failed to update contact:', err);
    }
  }

  function selectContact(c: Contact) {
    selectedContact.value = c;
    setEditing(false);
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
      {/* Contacts list */}
      <div className="w-80 border-r border-gray-200 bg-white flex flex-col">
        <div className="p-4 border-b border-gray-200">
          <h2 className="text-lg font-semibold text-gray-900">Contacts</h2>
          <p className="text-sm text-gray-500">{contacts.value.length} contacts</p>
        </div>
        <div className="flex-1 overflow-y-auto">
          {loading ? (
            <div className="p-4 text-center text-gray-500">Loading...</div>
          ) : contacts.value.length === 0 ? (
            <div className="p-4 text-center text-gray-500">No contacts yet</div>
          ) : (
            contacts.value.map((c) => (
              <ContactCard
                key={c.id}
                contact={c}
                isSelected={contact?.id === c.id}
                onClick={() => selectContact(c)}
              />
            ))
          )}
        </div>
      </div>

      {/* Contact details */}
      <div className="flex-1 bg-gray-50 overflow-y-auto">
        {contact ? (
          <div className="p-6">
            <div className="max-w-2xl">
              <div className="bg-white rounded-lg shadow-sm p-6 mb-6">
                <div className="flex items-center justify-between mb-4">
                  <h2 className="text-xl font-semibold text-gray-900">Contact Details</h2>
                  {!editing && (
                    <button
                      onClick={() => setEditing(true)}
                      className="px-3 py-1.5 text-sm bg-gray-100 hover:bg-gray-200 text-gray-700 rounded-md transition-colors"
                    >
                      Edit
                    </button>
                  )}
                </div>

                {editing ? (
                  <div className="space-y-4">
                    <div>
                      <label className="block text-sm font-medium text-gray-700 mb-1">
                        Name
                      </label>
                      <input
                        type="text"
                        value={editName}
                        onChange={(e) => setEditName((e.target as HTMLInputElement).value)}
                        className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:border-blue-500"
                        placeholder="Contact name"
                      />
                    </div>
                    <div>
                      <label className="block text-sm font-medium text-gray-700 mb-1">
                        Email
                      </label>
                      <input
                        type="email"
                        value={editEmail}
                        onChange={(e) => setEditEmail((e.target as HTMLInputElement).value)}
                        className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:border-blue-500"
                        placeholder="contact@example.com"
                      />
                    </div>
                    <div className="flex gap-2">
                      <button
                        onClick={handleSaveContact}
                        className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
                      >
                        Save
                      </button>
                      <button
                        onClick={() => setEditing(false)}
                        className="px-4 py-2 bg-gray-100 text-gray-700 rounded-lg hover:bg-gray-200 transition-colors"
                      >
                        Cancel
                      </button>
                    </div>
                  </div>
                ) : (
                  <div className="space-y-3">
                    <div className="flex items-center gap-3">
                      <IconUser size={18} className="text-gray-400" />
                      <span className="text-gray-900">
                        {contact.name || <span className="text-gray-400">No name set</span>}
                      </span>
                    </div>
                    <div className="flex items-center gap-3">
                      <IconMail size={18} className="text-gray-400" />
                      <span className="text-gray-900">
                        {contact.email || <span className="text-gray-400">No email set</span>}
                      </span>
                    </div>
                    <div className="flex items-center gap-3">
                      <IconClock size={18} className="text-gray-400" />
                      <span className="text-gray-600 text-sm">
                        First seen: {formatDate(contact.created_at)}
                      </span>
                    </div>
                    <div className="flex items-center gap-3">
                      <IconClock size={18} className="text-gray-400" />
                      <span className="text-gray-600 text-sm">
                        Last seen: {formatDate(contact.last_seen_at)}
                      </span>
                    </div>
                    <div className="pt-2">
                      <p className="text-xs text-gray-400">Visitor ID: {contact.visitor_id}</p>
                    </div>
                  </div>
                )}
              </div>

              {/* Conversation history */}
              <div className="bg-white rounded-lg shadow-sm">
                <div className="p-4 border-b border-gray-200 flex items-center gap-2">
                  <IconMessageCircle size={20} className="text-gray-500" />
                  <h3 className="font-semibold text-gray-900">Conversation History</h3>
                </div>
                <div className="divide-y divide-gray-100">
                  {conversations.length === 0 ? (
                    <div className="p-4 text-center text-gray-500 text-sm">
                      No conversations yet
                    </div>
                  ) : (
                    conversations.map((conv) => (
                      <a
                        key={conv.id}
                        href={`/w/${workspace.id}/chat`}
                        className="block p-4 hover:bg-gray-50 transition-colors"
                      >
                        <div className="flex items-center justify-between mb-1">
                          <span
                            className={`px-2 py-0.5 text-xs rounded-full ${
                              conv.status === 'open'
                                ? 'bg-green-100 text-green-800'
                                : 'bg-gray-100 text-gray-600'
                            }`}
                          >
                            {conv.status}
                          </span>
                          <span className="text-xs text-gray-500">
                            {formatDate(conv.updated_at)}
                          </span>
                        </div>
                        <p className="text-sm text-gray-600 truncate">
                          {conv.last_message || 'No messages'}
                        </p>
                      </a>
                    ))
                  )}
                </div>
              </div>
            </div>
          </div>
        ) : (
          <div className="flex-1 flex items-center justify-center h-full">
            <div className="text-center text-gray-500">
              <IconUser size={48} className="mx-auto mb-2 opacity-50" />
              <p>Select a contact to view details</p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
