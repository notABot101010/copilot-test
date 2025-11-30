/**
 * Chat service that handles encryption and communication
 */

import { signal } from '@preact/signals';
import { api, type EncryptedMessage as ApiEncryptedMessage } from './api';
import * as crypto from '../lib/crypto';

export interface UserSession {
  username: string;
  identityKeyPair: CryptoKeyPair;
  masterKey: CryptoKey;
  salt: string;
}

export interface DecryptedMessage {
  id: string;
  senderUsername: string;
  content: string;
  timestamp: number;
  isOutgoing: boolean;
}

export interface Conversation {
  peerUsername: string;
  messages: DecryptedMessage[];
  peerIdentityPublicKey?: string;
  unread: number;
}

// Global state
export const currentUser = signal<UserSession | null>(null);
export const conversations = signal<Map<string, Conversation>>(new Map());
export const isPolling = signal(false);

// Track sent message IDs to prevent duplicates
const sentMessageIds = new Set<string>();

// Store signed prekey private key for decryption
let signedPrekeyPrivate: CryptoKey | null = null;

// Polling abort controller
let pollAbortController: AbortController | null = null;

export async function register(username: string, password: string): Promise<void> {
  // Generate salt
  const salt = crypto.generateSalt();
  const saltBase64 = crypto.arrayBufferToBase64(salt.buffer as ArrayBuffer);
  
  // Derive master key from password
  const masterKey = await crypto.deriveMasterKey(password, salt);
  
  // Generate identity key pair
  const identityKeyPair = await crypto.generateIdentityKeyPair();
  
  // Export and encrypt identity private key
  const identityPublicRaw = await crypto.exportPublicKey(identityKeyPair.publicKey);
  const { encrypted, iv } = await crypto.encryptPrivateKey(masterKey, identityKeyPair.privateKey);
  
  // Register with server
  await api.register({
    username,
    identity_public_key: crypto.arrayBufferToBase64(identityPublicRaw),
    salt: saltBase64,
    encrypted_identity_private_key: crypto.arrayBufferToBase64(encrypted),
    identity_key_iv: crypto.arrayBufferToBase64(iv.buffer as ArrayBuffer)
  });
  
  // Generate and upload pre-key bundle
  const preKeyData = await crypto.generatePreKeyBundle(identityKeyPair, masterKey, 10);
  
  await api.uploadPreKeyBundle(username, {
    signed_prekey_public: preKeyData.bundle.signedPrekeyPublic,
    signed_prekey_signature: preKeyData.bundle.signedPrekeySignature,
    encrypted_signed_prekey_private: preKeyData.signedPrekeyPrivateEncrypted,
    signed_prekey_iv: preKeyData.signedPrekeyIv,
    one_time_prekeys: preKeyData.oneTimePrekeyPublics,
    encrypted_one_time_prekey_privates: preKeyData.oneTimePrekeyPrivatesEncrypted,
    one_time_prekey_ivs: preKeyData.oneTimePrekeyIvs
  });
  
  // Store session
  currentUser.value = {
    username,
    identityKeyPair,
    masterKey,
    salt: saltBase64
  };
  
  // Store signed prekey private for receiving messages
  const signedPrekeyIvBuffer = new Uint8Array(crypto.base64ToArrayBuffer(preKeyData.signedPrekeyIv));
  const encryptedSignedPrekeyPrivate = crypto.base64ToArrayBuffer(preKeyData.signedPrekeyPrivateEncrypted);
  signedPrekeyPrivate = await crypto.decryptExchangePrivateKey(
    masterKey,
    encryptedSignedPrekeyPrivate,
    signedPrekeyIvBuffer
  );
  
  // Start polling for messages
  startPolling();
}

export async function login(username: string, password: string): Promise<void> {
  // Fetch user data from server
  const userData = await api.login({ username });
  
  // Derive master key from password
  const salt = new Uint8Array(crypto.base64ToArrayBuffer(userData.salt));
  const masterKey = await crypto.deriveMasterKey(password, salt);
  
  // Decrypt identity private key
  const encryptedPrivateKey = crypto.base64ToArrayBuffer(userData.encrypted_identity_private_key);
  const iv = new Uint8Array(crypto.base64ToArrayBuffer(userData.identity_key_iv));
  
  let identityPrivateKey: CryptoKey;
  try {
    identityPrivateKey = await crypto.decryptIdentityPrivateKey(masterKey, encryptedPrivateKey, iv);
  } catch (err) {
    throw new Error('Invalid password');
  }
  
  // Import identity public key
  const identityPublicKey = await crypto.importIdentityPublicKey(
    crypto.base64ToArrayBuffer(userData.identity_public_key)
  );
  
  // Store session
  currentUser.value = {
    username,
    identityKeyPair: { privateKey: identityPrivateKey, publicKey: identityPublicKey },
    masterKey,
    salt: userData.salt
  };
  
  // Fetch existing prekey private key from server
  // This allows us to decrypt messages sent before this login
  try {
    const myPreKeys = await api.getMyPreKeys(username);
    const signedPrekeyIvBuffer = new Uint8Array(crypto.base64ToArrayBuffer(myPreKeys.signed_prekey_iv));
    const encryptedSignedPrekeyPrivate = crypto.base64ToArrayBuffer(myPreKeys.encrypted_signed_prekey_private);
    signedPrekeyPrivate = await crypto.decryptExchangePrivateKey(
      masterKey,
      encryptedSignedPrekeyPrivate,
      signedPrekeyIvBuffer
    );
  } catch (err) {
    // If no prekeys exist yet (e.g., new account), generate new ones
    console.log('No existing prekeys found, generating new ones');
    const preKeyData = await crypto.generatePreKeyBundle(
      { privateKey: identityPrivateKey, publicKey: identityPublicKey },
      masterKey,
      10
    );
    
    await api.uploadPreKeyBundle(username, {
      signed_prekey_public: preKeyData.bundle.signedPrekeyPublic,
      signed_prekey_signature: preKeyData.bundle.signedPrekeySignature,
      encrypted_signed_prekey_private: preKeyData.signedPrekeyPrivateEncrypted,
      signed_prekey_iv: preKeyData.signedPrekeyIv,
      one_time_prekeys: preKeyData.oneTimePrekeyPublics,
      encrypted_one_time_prekey_privates: preKeyData.oneTimePrekeyPrivatesEncrypted,
      one_time_prekey_ivs: preKeyData.oneTimePrekeyIvs
    });
    
    const signedPrekeyIvBuffer = new Uint8Array(crypto.base64ToArrayBuffer(preKeyData.signedPrekeyIv));
    const encryptedSignedPrekeyPrivate = crypto.base64ToArrayBuffer(preKeyData.signedPrekeyPrivateEncrypted);
    signedPrekeyPrivate = await crypto.decryptExchangePrivateKey(
      masterKey,
      encryptedSignedPrekeyPrivate,
      signedPrekeyIvBuffer
    );
  }
  
  // Start polling for messages
  startPolling();
}

export function logout(): void {
  stopPolling();
  currentUser.value = null;
  conversations.value = new Map();
  signedPrekeyPrivate = null;
  sentMessageIds.clear();
}

export async function listUsers(): Promise<string[]> {
  return api.listUsers();
}

export async function sendMessage(recipientUsername: string, content: string): Promise<string> {
  const user = currentUser.value;
  if (!user) {
    throw new Error('Not logged in');
  }
  
  // Fetch recipient's pre-key bundle
  const preKeyBundle = await api.getPreKeyBundle(recipientUsername);
  
  // Verify signature
  const bundle: crypto.PreKeyBundle = {
    identityPublicKey: preKeyBundle.identity_public_key,
    signedPrekeyPublic: preKeyBundle.signed_prekey_public,
    signedPrekeySignature: preKeyBundle.signed_prekey_signature,
    oneTimePrekey: preKeyBundle.one_time_prekey
  };
  
  const isValid = await crypto.verifyPreKeyBundle(bundle);
  if (!isValid) {
    throw new Error('Invalid pre-key bundle signature');
  }
  
  // Import recipient's signed prekey
  const recipientSignedPrekey = await crypto.importExchangePublicKey(
    crypto.base64ToArrayBuffer(preKeyBundle.signed_prekey_public)
  );
  
  // Encrypt message using ephemeral ECDH (no ratcheting)
  const encryptedMessage = await crypto.encryptMessage(recipientSignedPrekey, content);
  
  // Create sealed sender envelope
  const envelope = await crypto.createSealedSenderEnvelope(
    user.username,
    user.identityKeyPair.publicKey,
    recipientSignedPrekey,
    encryptedMessage
  );
  
  // Send message
  const response = await api.sendMessage(user.username, {
    recipient_username: recipientUsername,
    sealed_sender_envelope: JSON.stringify(envelope)
  });
  
  // Track sent message to prevent duplicate display
  sentMessageIds.add(response.id);
  
  // Get or create conversation
  let conversation = conversations.value.get(recipientUsername);
  
  if (!conversation) {
    conversation = {
      peerUsername: recipientUsername,
      messages: [],
      peerIdentityPublicKey: preKeyBundle.identity_public_key,
      unread: 0
    };
  }
  
  // Update conversation
  conversation.messages.push({
    id: response.id,
    senderUsername: user.username,
    content,
    timestamp: response.created_at,
    isOutgoing: true
  });
  
  // Update conversations signal
  const newConversations = new Map(conversations.value);
  newConversations.set(recipientUsername, conversation);
  conversations.value = newConversations;
  
  return response.id;
}

async function processIncomingMessage(encryptedMsg: ApiEncryptedMessage): Promise<void> {
  const user = currentUser.value;
  if (!user || !signedPrekeyPrivate) {
    return;
  }
  
  // Skip if we already processed this message (sent by us)
  if (sentMessageIds.has(encryptedMsg.id)) {
    return;
  }
  
  try {
    // Parse sealed sender envelope
    const envelope: crypto.SealedSenderEnvelope = JSON.parse(encryptedMsg.sealed_sender_envelope);
    
    // Open sealed sender envelope
    const inner = await crypto.openSealedSenderEnvelope(envelope, signedPrekeyPrivate);
    
    const senderUsername = inner.senderUsername;
    
    // Decrypt message using ephemeral ECDH (no ratcheting)
    const plaintext = await crypto.decryptMessage(signedPrekeyPrivate, inner.encryptedMessage);
    
    // Get or create conversation
    let conversation = conversations.value.get(senderUsername);
    
    if (!conversation) {
      conversation = {
        peerUsername: senderUsername,
        messages: [],
        peerIdentityPublicKey: inner.senderIdentityPublicKey,
        unread: 0
      };
    }
    
    // Add message to conversation
    conversation.messages.push({
      id: encryptedMsg.id,
      senderUsername,
      content: plaintext,
      timestamp: encryptedMsg.created_at,
      isOutgoing: false
    });
    conversation.unread++;
    
    // Update conversations signal
    const newConversations = new Map(conversations.value);
    newConversations.set(senderUsername, conversation);
    conversations.value = newConversations;
    
    // Acknowledge message
    await api.ackMessages(user.username, { message_ids: [encryptedMsg.id] });
  } catch (err) {
    console.error('Failed to process incoming message:', err);
  }
}

async function poll(): Promise<void> {
  const user = currentUser.value;
  if (!user) {
    return;
  }
  
  try {
    const response = await api.pollMessages(user.username, 25);
    
    for (const msg of response.messages) {
      await processIncomingMessage(msg);
    }
  } catch (err) {
    console.error('Polling error:', err);
  }
}

function startPolling(): void {
  if (isPolling.value) {
    return;
  }
  
  isPolling.value = true;
  pollAbortController = new AbortController();
  
  const pollLoop = async () => {
    while (isPolling.value && currentUser.value) {
      await poll();
      // Small delay between polls
      await new Promise(resolve => setTimeout(resolve, 100));
    }
  };
  
  pollLoop();
}

function stopPolling(): void {
  isPolling.value = false;
  if (pollAbortController) {
    pollAbortController.abort();
    pollAbortController = null;
  }
}

export function markConversationAsRead(peerUsername: string): void {
  const conversation = conversations.value.get(peerUsername);
  if (conversation) {
    conversation.unread = 0;
    const newConversations = new Map(conversations.value);
    newConversations.set(peerUsername, conversation);
    conversations.value = newConversations;
  }
}

export function getConversation(peerUsername: string): Conversation | undefined {
  return conversations.value.get(peerUsername);
}

export function getConversationList(): Conversation[] {
  return Array.from(conversations.value.values()).sort((firstConv, secondConv) => {
    const firstLastMsg = firstConv.messages[firstConv.messages.length - 1];
    const secondLastMsg = secondConv.messages[secondConv.messages.length - 1];
    const firstTime = firstLastMsg?.timestamp || 0;
    const secondTime = secondLastMsg?.timestamp || 0;
    return secondTime - firstTime;
  });
}
