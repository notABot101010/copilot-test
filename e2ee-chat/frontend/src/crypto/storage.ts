import {
  generateSalt,
  deriveKeyFromPassword,
  generateEd25519KeyPair,
  generateX25519KeyPair,
  encryptIdentityKey,
  decryptIdentityKey,
  bytesToBase64,
  base64ToBytes,
  type Ed25519KeyPair,
  type X25519KeyPair,
} from './utils';
import { type RatchetState, serializeRatchetState, deserializeRatchetState } from './ratchet';

const STORAGE_PREFIX = 'e2ee_';

export interface UserKeyMaterial {
  masterKey: Uint8Array;
  identityKeyPair: Ed25519KeyPair;
  sealedSenderKeyPair: X25519KeyPair;
}

// Initialize new user keys from password
export async function initializeUserKeys(
  username: string,
  password: string
): Promise<{
  encryptedIdentityKey: string;
  identityPublicKey: string;
  prekeySignature: string;
  salt: string;
  sealedSenderPublicKey: string;
}> {
  // Generate salt and derive master key
  const salt = generateSalt();
  const masterKey = await deriveKeyFromPassword(password, salt);

  // Generate identity key pair
  const identityKeyPair = generateEd25519KeyPair();

  // Generate sealed sender key pair
  const sealedSenderKeyPair = generateX25519KeyPair();

  // Encrypt identity private key with master key
  const encryptedIdentityKey = await encryptIdentityKey(
    identityKeyPair.privateKey,
    masterKey
  );

  // Sign the sealed sender public key with identity key
  const { signEd25519 } = await import('./utils');
  const signature = signEd25519(sealedSenderKeyPair.publicKey, identityKeyPair.privateKey);

  // Store keys in local storage
  localStorage.setItem(
    `${STORAGE_PREFIX}${username}_salt`,
    bytesToBase64(salt)
  );
  localStorage.setItem(
    `${STORAGE_PREFIX}${username}_sealed_sender_private`,
    bytesToBase64(sealedSenderKeyPair.privateKey)
  );
  localStorage.setItem(
    `${STORAGE_PREFIX}${username}_sealed_sender_public`,
    bytesToBase64(sealedSenderKeyPair.publicKey)
  );

  return {
    encryptedIdentityKey,
    identityPublicKey: bytesToBase64(identityKeyPair.publicKey),
    prekeySignature: bytesToBase64(signature),
    salt: bytesToBase64(salt),
    sealedSenderPublicKey: bytesToBase64(sealedSenderKeyPair.publicKey),
  };
}

// Load user keys from storage
export async function loadUserKeys(
  username: string,
  password: string,
  encryptedIdentityKey: string
): Promise<UserKeyMaterial> {
  const saltBase64 = localStorage.getItem(`${STORAGE_PREFIX}${username}_salt`);
  if (!saltBase64) {
    throw new Error('User keys not found in storage');
  }

  const salt = base64ToBytes(saltBase64);
  const masterKey = await deriveKeyFromPassword(password, salt);

  // Decrypt identity key
  const identityPrivateKey = await decryptIdentityKey(encryptedIdentityKey, masterKey);
  const { ed25519 } = await import('@noble/curves/ed25519.js');
  const identityPublicKey = ed25519.getPublicKey(identityPrivateKey);

  const identityKeyPair: Ed25519KeyPair = {
    privateKey: identityPrivateKey,
    publicKey: identityPublicKey,
  };

  // Load sealed sender key pair
  const sealedSenderPrivateBase64 = localStorage.getItem(
    `${STORAGE_PREFIX}${username}_sealed_sender_private`
  );
  const sealedSenderPublicBase64 = localStorage.getItem(
    `${STORAGE_PREFIX}${username}_sealed_sender_public`
  );

  if (!sealedSenderPrivateBase64 || !sealedSenderPublicBase64) {
    throw new Error('Sealed sender keys not found');
  }

  const sealedSenderKeyPair: X25519KeyPair = {
    privateKey: base64ToBytes(sealedSenderPrivateBase64),
    publicKey: base64ToBytes(sealedSenderPublicBase64),
  };

  return {
    masterKey,
    identityKeyPair,
    sealedSenderKeyPair,
  };
}

// Save ratchet state to storage
export function saveRatchetState(username: string, peerUsername: string, state: RatchetState): void {
  const key = `${STORAGE_PREFIX}${username}_ratchet_${peerUsername}`;
  const serialized = serializeRatchetState(state);
  localStorage.setItem(key, serialized);
}

// Load ratchet state from storage
export function loadRatchetState(username: string, peerUsername: string): RatchetState | null {
  const key = `${STORAGE_PREFIX}${username}_ratchet_${peerUsername}`;
  const serialized = localStorage.getItem(key);

  if (!serialized) {
    return null;
  }

  return deserializeRatchetState(serialized);
}

// Clear user data from storage
export function clearUserStorage(username: string): void {
  const keysToRemove: string[] = [];

  for (let i = 0; i < localStorage.length; i++) {
    const key = localStorage.key(i);
    if (key && key.startsWith(`${STORAGE_PREFIX}${username}_`)) {
      keysToRemove.push(key);
    }
  }

  keysToRemove.forEach(key => localStorage.removeItem(key));
}

// Store current user session
export function setCurrentUser(username: string): void {
  sessionStorage.setItem(`${STORAGE_PREFIX}current_user`, username);
}

export function getCurrentUser(): string | null {
  return sessionStorage.getItem(`${STORAGE_PREFIX}current_user`);
}

export function clearCurrentUser(): void {
  sessionStorage.removeItem(`${STORAGE_PREFIX}current_user`);
}
