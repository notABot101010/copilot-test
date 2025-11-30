import {
  generateX25519KeyPair,
  x25519SharedSecret,
  hkdfDerive,
  encryptAES256GCM,
  decryptAES256GCM,
  concatBytes,
  bytesToBase64,
  base64ToBytes,
  type X25519KeyPair,
  type EncryptedData,
} from './utils';

const INFO_MESSAGE_KEYS = new TextEncoder().encode('MessageKeys');
const INFO_CHAIN_KEY = new TextEncoder().encode('ChainKey');
const INFO_ROOT_KEY = new TextEncoder().encode('RootKey');

export interface RatchetState {
  rootKey: Uint8Array;
  chainKeySend: Uint8Array | null;
  chainKeyReceive: Uint8Array | null;
  sendingChainLength: number;
  receivingChainLength: number;
  previousSendingChainLength: number;
  dhKeyPairSend: X25519KeyPair | null;
  dhPublicKeyReceive: Uint8Array | null;
  skippedMessageKeys: Map<string, Uint8Array>;
}

export interface EncryptedMessage {
  ciphertext: string;
  iv: string;
  tag: string;
  dhPublicKey: string;
  messageNumber: number;
  previousChainLength: number;
}

// Initialize ratchet for sender (Alice)
export function initializeRatchetSender(
  sharedSecret: Uint8Array,
  senderDHKeyPair: X25519KeyPair,
  receiverDHPublicKey: Uint8Array
): RatchetState {
  // Derive initial root key from shared secret
  const rootKey = hkdfDerive(
    sharedSecret,
    new Uint8Array(32),
    INFO_ROOT_KEY,
    32
  );

  // Perform initial DH ratchet step
  const dhOutput = x25519SharedSecret(senderDHKeyPair.privateKey, receiverDHPublicKey);
  const derivedKeys = hkdfDerive(
    concatBytes(rootKey, dhOutput),
    new Uint8Array(32),
    INFO_ROOT_KEY,
    64
  );

  const newRootKey = derivedKeys.slice(0, 32);
  const chainKeySend = derivedKeys.slice(32, 64);

  return {
    rootKey: newRootKey,
    chainKeySend,
    chainKeyReceive: null,
    sendingChainLength: 0,
    receivingChainLength: 0,
    previousSendingChainLength: 0,
    dhKeyPairSend: senderDHKeyPair,
    dhPublicKeyReceive: receiverDHPublicKey,
    skippedMessageKeys: new Map(),
  };
}

// Initialize ratchet for receiver (Bob)
export function initializeRatchetReceiver(
  sharedSecret: Uint8Array,
  receiverDHKeyPair: X25519KeyPair
): RatchetState {
  const rootKey = hkdfDerive(
    sharedSecret,
    new Uint8Array(32),
    INFO_ROOT_KEY,
    32
  );

  return {
    rootKey,
    chainKeySend: null,
    chainKeyReceive: null,
    sendingChainLength: 0,
    receivingChainLength: 0,
    previousSendingChainLength: 0,
    dhKeyPairSend: receiverDHKeyPair,
    dhPublicKeyReceive: null,
    skippedMessageKeys: new Map(),
  };
}

// Derive message key from chain key
function deriveMessageKey(chainKey: Uint8Array): { messageKey: Uint8Array; nextChainKey: Uint8Array } {
  const messageKey = hkdfDerive(chainKey, new Uint8Array(32), INFO_MESSAGE_KEYS, 32);
  const nextChainKey = hkdfDerive(chainKey, new Uint8Array(32), INFO_CHAIN_KEY, 32);
  return { messageKey, nextChainKey };
}

// Perform DH ratchet step
function dhRatchet(
  state: RatchetState,
  receivedDHPublicKey: Uint8Array
): RatchetState {
  // Save current sending chain length
  state.previousSendingChainLength = state.sendingChainLength;

  // Update receiving chain
  if (state.dhKeyPairSend) {
    const dhOutput = x25519SharedSecret(state.dhKeyPairSend.privateKey, receivedDHPublicKey);
    const derivedKeys = hkdfDerive(
      concatBytes(state.rootKey, dhOutput),
      new Uint8Array(32),
      INFO_ROOT_KEY,
      64
    );
    state.rootKey = derivedKeys.slice(0, 32);
    state.chainKeyReceive = derivedKeys.slice(32, 64);
    state.dhPublicKeyReceive = receivedDHPublicKey;
    state.receivingChainLength = 0;
  }

  // Generate new sending key pair
  state.dhKeyPairSend = generateX25519KeyPair();
  if (state.dhPublicKeyReceive) {
    const dhOutput = x25519SharedSecret(state.dhKeyPairSend.privateKey, state.dhPublicKeyReceive);
    const derivedKeys = hkdfDerive(
      concatBytes(state.rootKey, dhOutput),
      new Uint8Array(32),
      INFO_ROOT_KEY,
      64
    );
    state.rootKey = derivedKeys.slice(0, 32);
    state.chainKeySend = derivedKeys.slice(32, 64);
    state.sendingChainLength = 0;
  }

  return state;
}

// Skip message keys (for out-of-order messages)
function skipMessageKeys(
  state: RatchetState,
  until: number
): RatchetState {
  if (!state.chainKeyReceive) {
    return state;
  }

  let chainKey = state.chainKeyReceive;
  while (state.receivingChainLength < until) {
    const { messageKey, nextChainKey } = deriveMessageKey(chainKey);
    const keyId = `${state.receivingChainLength}`;
    state.skippedMessageKeys.set(keyId, messageKey);
    chainKey = nextChainKey;
    state.receivingChainLength++;
  }
  state.chainKeyReceive = chainKey;

  return state;
}

// Encrypt a message
export async function ratchetEncrypt(
  state: RatchetState,
  plaintext: Uint8Array
): Promise<{ encrypted: EncryptedMessage; newState: RatchetState }> {
  if (!state.chainKeySend) {
    throw new Error('Sending chain not initialized');
  }

  const { messageKey, nextChainKey } = deriveMessageKey(state.chainKeySend);
  const encrypted = await encryptAES256GCM(plaintext, messageKey);

  const message: EncryptedMessage = {
    ciphertext: bytesToBase64(encrypted.ciphertext),
    iv: bytesToBase64(encrypted.iv),
    tag: bytesToBase64(encrypted.tag),
    dhPublicKey: state.dhKeyPairSend ? bytesToBase64(state.dhKeyPairSend.publicKey) : '',
    messageNumber: state.sendingChainLength,
    previousChainLength: state.previousSendingChainLength,
  };

  const newState = {
    ...state,
    chainKeySend: nextChainKey,
    sendingChainLength: state.sendingChainLength + 1,
  };

  return { encrypted: message, newState };
}

// Decrypt a message
export async function ratchetDecrypt(
  state: RatchetState,
  message: EncryptedMessage
): Promise<{ plaintext: Uint8Array; newState: RatchetState }> {
  const receivedDHPublicKey = base64ToBytes(message.dhPublicKey);

  // Check if we need to perform DH ratchet
  if (
    !state.dhPublicKeyReceive ||
    bytesToBase64(receivedDHPublicKey) !== bytesToBase64(state.dhPublicKeyReceive)
  ) {
    state = skipMessageKeys(state, state.receivingChainLength);
    state = dhRatchet(state, receivedDHPublicKey);
  }

  // Check for skipped message keys
  const keyId = `${message.messageNumber}`;
  if (state.skippedMessageKeys.has(keyId)) {
    const messageKey = state.skippedMessageKeys.get(keyId)!;
    state.skippedMessageKeys.delete(keyId);

    const encrypted: EncryptedData = {
      ciphertext: base64ToBytes(message.ciphertext),
      iv: base64ToBytes(message.iv),
      tag: base64ToBytes(message.tag),
    };

    const plaintext = await decryptAES256GCM(encrypted, messageKey);
    return { plaintext, newState: state };
  }

  // Skip message keys if needed
  state = skipMessageKeys(state, message.messageNumber);

  if (!state.chainKeyReceive) {
    throw new Error('Receiving chain not initialized');
  }

  const { messageKey, nextChainKey } = deriveMessageKey(state.chainKeyReceive);

  const encrypted: EncryptedData = {
    ciphertext: base64ToBytes(message.ciphertext),
    iv: base64ToBytes(message.iv),
    tag: base64ToBytes(message.tag),
  };

  const plaintext = await decryptAES256GCM(encrypted, messageKey);

  const newState = {
    ...state,
    chainKeyReceive: nextChainKey,
    receivingChainLength: state.receivingChainLength + 1,
  };

  return { plaintext, newState };
}

// Serialize ratchet state for storage
export function serializeRatchetState(state: RatchetState): string {
  return JSON.stringify({
    rootKey: bytesToBase64(state.rootKey),
    chainKeySend: state.chainKeySend ? bytesToBase64(state.chainKeySend) : null,
    chainKeyReceive: state.chainKeyReceive ? bytesToBase64(state.chainKeyReceive) : null,
    sendingChainLength: state.sendingChainLength,
    receivingChainLength: state.receivingChainLength,
    previousSendingChainLength: state.previousSendingChainLength,
    dhKeyPairSend: state.dhKeyPairSend
      ? {
          privateKey: bytesToBase64(state.dhKeyPairSend.privateKey),
          publicKey: bytesToBase64(state.dhKeyPairSend.publicKey),
        }
      : null,
    dhPublicKeyReceive: state.dhPublicKeyReceive
      ? bytesToBase64(state.dhPublicKeyReceive)
      : null,
  });
}

// Deserialize ratchet state from storage
export function deserializeRatchetState(serialized: string): RatchetState {
  const data = JSON.parse(serialized);
  return {
    rootKey: base64ToBytes(data.rootKey),
    chainKeySend: data.chainKeySend ? base64ToBytes(data.chainKeySend) : null,
    chainKeyReceive: data.chainKeyReceive ? base64ToBytes(data.chainKeyReceive) : null,
    sendingChainLength: data.sendingChainLength,
    receivingChainLength: data.receivingChainLength,
    previousSendingChainLength: data.previousSendingChainLength,
    dhKeyPairSend: data.dhKeyPairSend
      ? {
          privateKey: base64ToBytes(data.dhKeyPairSend.privateKey),
          publicKey: base64ToBytes(data.dhKeyPairSend.publicKey),
        }
      : null,
    dhPublicKeyReceive: data.dhPublicKeyReceive
      ? base64ToBytes(data.dhPublicKeyReceive)
      : null,
    skippedMessageKeys: new Map(),
  };
}
