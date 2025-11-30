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
  ratchetState?: crypto.RatchetState;
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
  
  // Fetch and decrypt signed prekey private for receiving messages
  // We need to regenerate prekeys since we don't have the private key stored locally
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
  
  // Get or create conversation
  let conversation = conversations.value.get(recipientUsername);
  
  if (!conversation) {
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
    
    // Import keys
    const recipientSignedPrekey = await crypto.importExchangePublicKey(
      crypto.base64ToArrayBuffer(preKeyBundle.signed_prekey_public)
    );
    
    const recipientOneTimePrekey = preKeyBundle.one_time_prekey
      ? await crypto.importExchangePublicKey(crypto.base64ToArrayBuffer(preKeyBundle.one_time_prekey))
      : undefined;
    
    // Initialize ratchet
    const { state, initialMessage } = await crypto.initRatchetAsSender(
      user.identityKeyPair.publicKey,
      recipientSignedPrekey,
      recipientOneTimePrekey
    );
    
    conversation = {
      peerUsername: recipientUsername,
      messages: [],
      ratchetState: state,
      peerIdentityPublicKey: preKeyBundle.identity_public_key,
      unread: 0
    };
    
    // Store initial message as first "message" (key exchange info)
    // This will be included in the first real message
    (conversation as { initialMessage?: Uint8Array }).initialMessage = initialMessage;
  }
  
  if (!conversation.ratchetState) {
    throw new Error('Ratchet state not initialized');
  }
  
  // Encrypt message with ratchet
  const { ciphertext, header, newState } = await crypto.ratchetEncrypt(
    conversation.ratchetState,
    content
  );
  
  // Fetch recipient's pre-key for sealed sender
  const preKeyBundle = await api.getPreKeyBundle(recipientUsername);
  const recipientSignedPrekey = await crypto.importExchangePublicKey(
    crypto.base64ToArrayBuffer(preKeyBundle.signed_prekey_public)
  );
  
  // Create sealed sender envelope
  // If this is the first message, prepend initial message to header
  let finalHeader = header;
  const initialMessage = (conversation as { initialMessage?: Uint8Array }).initialMessage;
  if (initialMessage) {
    finalHeader = new Uint8Array(1 + initialMessage.length + header.length);
    finalHeader[0] = 1; // Flag: has initial message
    finalHeader.set(initialMessage, 1);
    finalHeader.set(header, 1 + initialMessage.length);
    delete (conversation as { initialMessage?: Uint8Array }).initialMessage;
  } else {
    const headerWithFlag = new Uint8Array(1 + header.length);
    headerWithFlag[0] = 0; // Flag: no initial message
    headerWithFlag.set(header, 1);
    finalHeader = headerWithFlag;
  }
  
  const envelope = await crypto.createSealedSenderEnvelope(
    user.username,
    user.identityKeyPair.publicKey,
    recipientSignedPrekey,
    finalHeader,
    ciphertext
  );
  
  // Send message
  const response = await api.sendMessage(user.username, {
    recipient_username: recipientUsername,
    sealed_sender_envelope: JSON.stringify(envelope)
  });
  
  // Track sent message to prevent duplicate display
  sentMessageIds.add(response.id);
  
  // Update conversation
  conversation.ratchetState = newState;
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
    
    // Get or create conversation
    let conversation = conversations.value.get(senderUsername);
    
    // Parse header
    const headerWithFlag = new Uint8Array(crypto.base64ToArrayBuffer(inner.messageHeader));
    const hasInitialMessage = headerWithFlag[0] === 1;
    
    let header: Uint8Array;
    
    if (hasInitialMessage) {
      // Extract initial message (162 bytes) and actual header
      const initialMessage = headerWithFlag.slice(1, 163);
      header = headerWithFlag.slice(163);
      
      // Initialize ratchet as receiver
      const ratchetState = await crypto.initRatchetAsReceiver(signedPrekeyPrivate, initialMessage);
      
      conversation = {
        peerUsername: senderUsername,
        messages: [],
        ratchetState,
        peerIdentityPublicKey: inner.senderIdentityPublicKey,
        unread: 0
      };
    } else {
      header = headerWithFlag.slice(1);
      
      if (!conversation || !conversation.ratchetState) {
        console.error('Received message but no ratchet state exists');
        return;
      }
    }
    
    if (!conversation || !conversation.ratchetState) {
      console.error('No conversation found for decryption');
      return;
    }
    
    // Decrypt message
    const ciphertext = crypto.base64ToArrayBuffer(inner.ciphertext);
    const { plaintext, newState } = await crypto.ratchetDecrypt(
      conversation.ratchetState,
      header,
      ciphertext
    );
    
    // Update conversation
    conversation.ratchetState = newState;
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
