/**
 * End-to-end encrypted chat crypto library using WebCrypto API
 * 
 * Features:
 * - PBKDF2 for master key derivation from password
 * - Ed25519 for identity keys (signed using identity key)
 * - X25519 for key exchange per message
 * - AES-256-GCM for message encryption
 * - Double ratchet for forward secrecy
 * - Sealed sender for metadata protection
 */

// WebCrypto doesn't natively support Ed25519/X25519, so we need to use SubtleCrypto
// with ECDSA/ECDH P-256 as a fallback or implement Ed25519/X25519 manually.
// For MVP, we'll use ECDSA (P-256) for signatures and ECDH (P-256) for key exchange.

const PBKDF2_ITERATIONS = 100000;
const SALT_LENGTH = 16;
const IV_LENGTH = 12;
const KEY_LENGTH = 256;

// Helper functions for base64 encoding/decoding
export function arrayBufferToBase64(buffer: ArrayBuffer): string {
  const bytes = new Uint8Array(buffer);
  let binary = '';
  for (let idx = 0; idx < bytes.byteLength; idx++) {
    binary += String.fromCharCode(bytes[idx]);
  }
  return btoa(binary);
}

export function base64ToArrayBuffer(base64: string): ArrayBuffer {
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let idx = 0; idx < binary.length; idx++) {
    bytes[idx] = binary.charCodeAt(idx);
  }
  return bytes.buffer;
}

// Generate random bytes
export function generateRandomBytes(length: number): Uint8Array {
  return crypto.getRandomValues(new Uint8Array(length));
}

// Generate salt for PBKDF2
export function generateSalt(): Uint8Array {
  return generateRandomBytes(SALT_LENGTH);
}

// Generate IV for AES-GCM
export function generateIV(): Uint8Array {
  return generateRandomBytes(IV_LENGTH);
}

// Derive master key from password using PBKDF2
export async function deriveMasterKey(password: string, salt: Uint8Array): Promise<CryptoKey> {
  const encoder = new TextEncoder();
  const passwordBytes = encoder.encode(password);
  
  const passwordKey = await crypto.subtle.importKey(
    'raw',
    passwordBytes,
    'PBKDF2',
    false,
    ['deriveKey']
  );
  
  return crypto.subtle.deriveKey(
    {
      name: 'PBKDF2',
      salt: new Uint8Array(salt) as BufferSource,
      iterations: PBKDF2_ITERATIONS,
      hash: 'SHA-256'
    },
    passwordKey,
    { name: 'AES-GCM', length: KEY_LENGTH },
    true,
    ['encrypt', 'decrypt']
  );
}

// Generate identity key pair (ECDSA P-256 for signatures)
// Note: WebCrypto doesn't support Ed25519 directly, using ECDSA as alternative
export async function generateIdentityKeyPair(): Promise<CryptoKeyPair> {
  return crypto.subtle.generateKey(
    {
      name: 'ECDSA',
      namedCurve: 'P-256'
    },
    true, // extractable
    ['sign', 'verify']
  );
}

// Generate exchange key pair (ECDH P-256 for key exchange)
// Note: WebCrypto doesn't support X25519 directly, using ECDH as alternative
export async function generateExchangeKeyPair(): Promise<CryptoKeyPair> {
  return crypto.subtle.generateKey(
    {
      name: 'ECDH',
      namedCurve: 'P-256'
    },
    true, // extractable
    ['deriveKey', 'deriveBits']
  );
}

// Export public key to raw bytes
export async function exportPublicKey(publicKey: CryptoKey): Promise<ArrayBuffer> {
  return crypto.subtle.exportKey('raw', publicKey);
}

// Export private key to PKCS8 format
export async function exportPrivateKey(privateKey: CryptoKey): Promise<ArrayBuffer> {
  return crypto.subtle.exportKey('pkcs8', privateKey);
}

// Import ECDSA public key from raw bytes
export async function importIdentityPublicKey(keyData: ArrayBuffer): Promise<CryptoKey> {
  return crypto.subtle.importKey(
    'raw',
    keyData,
    { name: 'ECDSA', namedCurve: 'P-256' },
    true,
    ['verify']
  );
}

// Import ECDSA private key from PKCS8
export async function importIdentityPrivateKey(keyData: ArrayBuffer): Promise<CryptoKey> {
  return crypto.subtle.importKey(
    'pkcs8',
    keyData,
    { name: 'ECDSA', namedCurve: 'P-256' },
    true,
    ['sign']
  );
}

// Import ECDH public key from raw bytes
export async function importExchangePublicKey(keyData: ArrayBuffer): Promise<CryptoKey> {
  return crypto.subtle.importKey(
    'raw',
    keyData,
    { name: 'ECDH', namedCurve: 'P-256' },
    true,
    []
  );
}

// Import ECDH private key from PKCS8
export async function importExchangePrivateKey(keyData: ArrayBuffer): Promise<CryptoKey> {
  return crypto.subtle.importKey(
    'pkcs8',
    keyData,
    { name: 'ECDH', namedCurve: 'P-256' },
    true,
    ['deriveKey', 'deriveBits']
  );
}

// Sign data with identity key
export async function sign(privateKey: CryptoKey, data: ArrayBuffer): Promise<ArrayBuffer> {
  return crypto.subtle.sign(
    { name: 'ECDSA', hash: 'SHA-256' },
    privateKey,
    data
  );
}

// Verify signature with identity public key
export async function verify(publicKey: CryptoKey, signature: ArrayBuffer, data: ArrayBuffer): Promise<boolean> {
  return crypto.subtle.verify(
    { name: 'ECDSA', hash: 'SHA-256' },
    publicKey,
    signature,
    data
  );
}

// Derive shared secret from ECDH key exchange
export async function deriveSharedSecret(
  privateKey: CryptoKey,
  publicKey: CryptoKey
): Promise<CryptoKey> {
  return crypto.subtle.deriveKey(
    {
      name: 'ECDH',
      public: publicKey
    },
    privateKey,
    { name: 'AES-GCM', length: KEY_LENGTH },
    true,
    ['encrypt', 'decrypt']
  );
}

// Encrypt data with AES-GCM
export async function encrypt(key: CryptoKey, data: ArrayBuffer, iv: Uint8Array): Promise<ArrayBuffer> {
  return crypto.subtle.encrypt(
    { name: 'AES-GCM', iv: new Uint8Array(iv) as BufferSource },
    key,
    data
  );
}

// Decrypt data with AES-GCM
export async function decrypt(key: CryptoKey, data: ArrayBuffer, iv: Uint8Array): Promise<ArrayBuffer> {
  return crypto.subtle.decrypt(
    { name: 'AES-GCM', iv: new Uint8Array(iv) as BufferSource },
    key,
    data
  );
}

// Encrypt private key with master key
export async function encryptPrivateKey(
  masterKey: CryptoKey,
  privateKey: CryptoKey
): Promise<{ encrypted: ArrayBuffer; iv: Uint8Array }> {
  const exportedKey = await exportPrivateKey(privateKey);
  const iv = generateIV();
  const encrypted = await encrypt(masterKey, exportedKey, iv);
  return { encrypted, iv };
}

// Decrypt private key with master key
export async function decryptIdentityPrivateKey(
  masterKey: CryptoKey,
  encryptedKey: ArrayBuffer,
  iv: Uint8Array
): Promise<CryptoKey> {
  const decrypted = await decrypt(masterKey, encryptedKey, iv);
  return importIdentityPrivateKey(decrypted);
}

export async function decryptExchangePrivateKey(
  masterKey: CryptoKey,
  encryptedKey: ArrayBuffer,
  iv: Uint8Array
): Promise<CryptoKey> {
  const decrypted = await decrypt(masterKey, encryptedKey, iv);
  return importExchangePrivateKey(decrypted);
}

// Key Derivation Function for ratcheting (HKDF-like using PBKDF2)
export async function deriveRatchetKey(
  inputKey: CryptoKey,
  salt: Uint8Array
): Promise<{ chainKey: CryptoKey; messageKey: CryptoKey }> {
  // Export the input key to use as base material
  const keyMaterial = await crypto.subtle.exportKey('raw', inputKey);
  
  // Import as HKDF key
  const hkdfKey = await crypto.subtle.importKey(
    'raw',
    keyMaterial,
    'HKDF',
    false,
    ['deriveKey']
  );
  
  // Derive chain key
  const chainKey = await crypto.subtle.deriveKey(
    {
      name: 'HKDF',
      salt: new Uint8Array(salt) as BufferSource,
      info: new TextEncoder().encode('chain'),
      hash: 'SHA-256'
    },
    hkdfKey,
    { name: 'AES-GCM', length: KEY_LENGTH },
    true,
    ['encrypt', 'decrypt']
  );
  
  // Derive message key
  const messageKey = await crypto.subtle.deriveKey(
    {
      name: 'HKDF',
      salt: new Uint8Array(salt) as BufferSource,
      info: new TextEncoder().encode('message'),
      hash: 'SHA-256'
    },
    hkdfKey,
    { name: 'AES-GCM', length: KEY_LENGTH },
    true,
    ['encrypt', 'decrypt']
  );
  
  return { chainKey, messageKey };
}

// Double Ratchet State
export interface RatchetState {
  // DH ratchet
  dhKeyPair: CryptoKeyPair;
  remoteDhPublicKey?: CryptoKey;
  
  // Chain keys
  sendingChainKey?: CryptoKey;
  receivingChainKey?: CryptoKey;
  
  // Message counters
  sendingCounter: number;
  receivingCounter: number;
  previousChainLength: number;
  
  // Root key
  rootKey: CryptoKey;
}

// Initialize ratchet for sender (initiator)
export async function initRatchetAsSender(
  _recipientIdentityPublicKey: CryptoKey,
  recipientSignedPrekey: CryptoKey,
  _recipientOneTimePrekey?: CryptoKey
): Promise<{ state: RatchetState; initialMessage: Uint8Array }> {
  // Generate ephemeral key pair
  const ephemeralKeyPair = await generateExchangeKeyPair();
  
  // Compute initial shared secret (X3DH-like)
  // In a full implementation, this would be:
  // DH1 = DH(IK_A, SPK_B)
  // DH2 = DH(EK_A, IK_B)
  // DH3 = DH(EK_A, SPK_B)
  // DH4 = DH(EK_A, OPK_B) if OPK_B exists
  // For simplicity, we do DH(EK_A, SPK_B)
  const sharedSecret = await deriveSharedSecret(ephemeralKeyPair.privateKey, recipientSignedPrekey);
  
  // Generate DH ratchet key pair
  const dhKeyPair = await generateExchangeKeyPair();
  
  // Derive initial chain keys
  const salt = generateRandomBytes(32);
  const { chainKey: sendingChainKey } = await deriveRatchetKey(sharedSecret, salt);
  
  const state: RatchetState = {
    dhKeyPair,
    remoteDhPublicKey: recipientSignedPrekey,
    sendingChainKey,
    sendingCounter: 0,
    receivingCounter: 0,
    previousChainLength: 0,
    rootKey: sharedSecret
  };
  
  // Create initial message containing ephemeral public key and DH public key
  const ephemeralPublic = await exportPublicKey(ephemeralKeyPair.publicKey);
  const dhPublic = await exportPublicKey(dhKeyPair.publicKey);
  
  // Pack keys: [ephemeralPublic (65 bytes), dhPublic (65 bytes), salt (32 bytes)]
  const initialMessage = new Uint8Array(65 + 65 + 32);
  initialMessage.set(new Uint8Array(ephemeralPublic), 0);
  initialMessage.set(new Uint8Array(dhPublic), 65);
  initialMessage.set(salt, 130);
  
  return { state, initialMessage };
}

// Initialize ratchet for receiver
export async function initRatchetAsReceiver(
  signedPrekeyPrivate: CryptoKey,
  initialMessage: Uint8Array
): Promise<RatchetState> {
  // Parse initial message
  const ephemeralPublicRaw = initialMessage.slice(0, 65);
  const dhPublicRaw = initialMessage.slice(65, 130);
  const salt = initialMessage.slice(130, 162);
  
  const ephemeralPublic = await importExchangePublicKey(ephemeralPublicRaw.buffer);
  const remoteDhPublicKey = await importExchangePublicKey(dhPublicRaw.buffer);
  
  // Compute shared secret
  const sharedSecret = await deriveSharedSecret(signedPrekeyPrivate, ephemeralPublic);
  
  // Generate DH ratchet key pair
  const dhKeyPair = await generateExchangeKeyPair();
  
  // Derive initial chain keys (receiving is what sender used for sending)
  const { chainKey: receivingChainKey } = await deriveRatchetKey(sharedSecret, salt);
  
  return {
    dhKeyPair,
    remoteDhPublicKey,
    receivingChainKey,
    sendingCounter: 0,
    receivingCounter: 0,
    previousChainLength: 0,
    rootKey: sharedSecret
  };
}

// Encrypt message using double ratchet
export async function ratchetEncrypt(
  state: RatchetState,
  plaintext: string
): Promise<{ ciphertext: ArrayBuffer; header: Uint8Array; newState: RatchetState }> {
  if (!state.sendingChainKey) {
    throw new Error('Sending chain key not initialized');
  }
  
  // Derive message key and advance chain
  const salt = generateRandomBytes(32);
  const { chainKey: newChainKey, messageKey } = await deriveRatchetKey(state.sendingChainKey, salt);
  
  // Encrypt message
  const iv = generateIV();
  const encoder = new TextEncoder();
  const plaintextBytes = encoder.encode(plaintext);
  const ciphertext = await encrypt(messageKey, plaintextBytes.buffer, iv);
  
  // Create header: [counter (4 bytes), DH public key (65 bytes), iv (12 bytes), salt (32 bytes)]
  const dhPublic = await exportPublicKey(state.dhKeyPair.publicKey);
  const header = new Uint8Array(4 + 65 + 12 + 32);
  const counterView = new DataView(header.buffer);
  counterView.setUint32(0, state.sendingCounter, false);
  header.set(new Uint8Array(dhPublic), 4);
  header.set(iv, 69);
  header.set(salt, 81);
  
  const newState: RatchetState = {
    ...state,
    sendingChainKey: newChainKey,
    sendingCounter: state.sendingCounter + 1
  };
  
  return { ciphertext, header, newState };
}

// Decrypt message using double ratchet
export async function ratchetDecrypt(
  state: RatchetState,
  header: Uint8Array,
  ciphertext: ArrayBuffer
): Promise<{ plaintext: string; newState: RatchetState }> {
  // Parse header
  // Skip the counter (first 4 bytes) - used for out-of-order message handling in full implementation
  const dhPublicRaw = header.slice(4, 69);
  const iv = header.slice(69, 81);
  const salt = header.slice(81, 113);
  
  const remoteDhPublic = await importExchangePublicKey(dhPublicRaw.buffer);
  
  let newState = { ...state };
  
  // Check if we need to do a DH ratchet step
  const currentRemotePublicRaw = state.remoteDhPublicKey ? 
    await exportPublicKey(state.remoteDhPublicKey) : null;
  const newRemotePublicRaw = dhPublicRaw.buffer;
  
  const keysAreDifferent = !currentRemotePublicRaw || 
    !arrayBuffersEqual(currentRemotePublicRaw, newRemotePublicRaw);
  
  if (keysAreDifferent) {
    // Perform DH ratchet step
    const sharedSecret = await deriveSharedSecret(state.dhKeyPair.privateKey, remoteDhPublic);
    const { chainKey: receivingChainKey } = await deriveRatchetKey(sharedSecret, generateRandomBytes(32));
    
    // Generate new DH key pair for sending
    const newDhKeyPair = await generateExchangeKeyPair();
    const sendingSharedSecret = await deriveSharedSecret(newDhKeyPair.privateKey, remoteDhPublic);
    const { chainKey: sendingChainKey } = await deriveRatchetKey(sendingSharedSecret, generateRandomBytes(32));
    
    newState = {
      ...state,
      dhKeyPair: newDhKeyPair,
      remoteDhPublicKey: remoteDhPublic,
      receivingChainKey,
      sendingChainKey,
      previousChainLength: state.sendingCounter,
      sendingCounter: 0,
      receivingCounter: 0
    };
  }
  
  if (!newState.receivingChainKey) {
    throw new Error('Receiving chain key not initialized');
  }
  
  // Derive message key
  const { chainKey: newChainKey, messageKey } = await deriveRatchetKey(newState.receivingChainKey, salt);
  
  // Decrypt message
  const plaintextBytes = await decrypt(messageKey, ciphertext, iv);
  const decoder = new TextDecoder();
  const plaintext = decoder.decode(plaintextBytes);
  
  newState.receivingChainKey = newChainKey;
  newState.receivingCounter++;
  
  return { plaintext, newState };
}

function arrayBuffersEqual(buf1: ArrayBuffer, buf2: ArrayBuffer): boolean {
  if (buf1.byteLength !== buf2.byteLength) return false;
  const view1 = new Uint8Array(buf1);
  const view2 = new Uint8Array(buf2);
  for (let idx = 0; idx < view1.length; idx++) {
    if (view1[idx] !== view2[idx]) return false;
  }
  return true;
}

// Sealed Sender envelope
// Structure:
// - Ephemeral public key (65 bytes) - for decrypting the inner envelope
// - Encrypted inner envelope (variable) - contains sender identity and message
export interface SealedSenderEnvelope {
  ephemeralPublicKey: string; // base64
  encryptedInner: string; // base64
  iv: string; // base64
}

export interface InnerEnvelope {
  senderUsername: string;
  senderIdentityPublicKey: string; // base64
  messageHeader: string; // base64 - ratchet header
  ciphertext: string; // base64 - encrypted message
}

// Create sealed sender envelope
export async function createSealedSenderEnvelope(
  senderUsername: string,
  senderIdentityPublicKey: CryptoKey,
  recipientSignedPrekey: CryptoKey,
  messageHeader: Uint8Array,
  ciphertext: ArrayBuffer
): Promise<SealedSenderEnvelope> {
  // Generate ephemeral key for sealed sender
  const ephemeralKeyPair = await generateExchangeKeyPair();
  
  // Derive key for encrypting inner envelope
  const encryptionKey = await deriveSharedSecret(ephemeralKeyPair.privateKey, recipientSignedPrekey);
  
  // Create inner envelope
  const senderPubKeyRaw = await exportPublicKey(senderIdentityPublicKey);
  const inner: InnerEnvelope = {
    senderUsername,
    senderIdentityPublicKey: arrayBufferToBase64(senderPubKeyRaw),
    messageHeader: arrayBufferToBase64(messageHeader.buffer as ArrayBuffer),
    ciphertext: arrayBufferToBase64(ciphertext)
  };
  
  // Encrypt inner envelope
  const iv = generateIV();
  const innerJson = JSON.stringify(inner);
  const encoder = new TextEncoder();
  const encryptedInner = await encrypt(encryptionKey, encoder.encode(innerJson).buffer as ArrayBuffer, iv);
  
  // Export ephemeral public key
  const ephemeralPubKeyRaw = await exportPublicKey(ephemeralKeyPair.publicKey);
  
  return {
    ephemeralPublicKey: arrayBufferToBase64(ephemeralPubKeyRaw),
    encryptedInner: arrayBufferToBase64(encryptedInner),
    iv: arrayBufferToBase64(iv.buffer as ArrayBuffer)
  };
}

// Open sealed sender envelope
export async function openSealedSenderEnvelope(
  envelope: SealedSenderEnvelope,
  recipientSignedPrekeyPrivate: CryptoKey
): Promise<InnerEnvelope> {
  // Import ephemeral public key
  const ephemeralPubKeyRaw = base64ToArrayBuffer(envelope.ephemeralPublicKey);
  const ephemeralPublicKey = await importExchangePublicKey(ephemeralPubKeyRaw);
  
  // Derive decryption key
  const decryptionKey = await deriveSharedSecret(recipientSignedPrekeyPrivate, ephemeralPublicKey);
  
  // Decrypt inner envelope
  const encryptedInner = base64ToArrayBuffer(envelope.encryptedInner);
  const iv = new Uint8Array(base64ToArrayBuffer(envelope.iv));
  const decryptedInner = await decrypt(decryptionKey, encryptedInner, iv);
  
  // Parse inner envelope
  const decoder = new TextDecoder();
  const innerJson = decoder.decode(decryptedInner);
  return JSON.parse(innerJson);
}

// Pre-key bundle for X3DH-like key exchange
export interface PreKeyBundle {
  identityPublicKey: string; // base64
  signedPrekeyPublic: string; // base64
  signedPrekeySignature: string; // base64
  oneTimePrekey?: string; // base64, optional
}

// Generate and sign a pre-key bundle
export async function generatePreKeyBundle(
  identityKeyPair: CryptoKeyPair,
  masterKey: CryptoKey,
  numOneTimePrekeys: number = 10
): Promise<{
  bundle: Omit<PreKeyBundle, 'identityPublicKey'>;
  signedPrekeyPrivateEncrypted: string;
  signedPrekeyIv: string;
  oneTimePrekeyPublics: string[];
  oneTimePrekeyPrivatesEncrypted: string[];
  oneTimePrekeyIvs: string[];
}> {
  // Generate signed pre-key
  const signedPrekeyPair = await generateExchangeKeyPair();
  const signedPrekeyPublicRaw = await exportPublicKey(signedPrekeyPair.publicKey);
  
  // Sign the pre-key with identity key
  const signature = await sign(identityKeyPair.privateKey, signedPrekeyPublicRaw);
  
  // Encrypt signed pre-key private
  const { encrypted: signedPrekeyPrivateEncrypted, iv: signedPrekeyIv } = 
    await encryptPrivateKey(masterKey, signedPrekeyPair.privateKey);
  
  // Generate one-time pre-keys
  const oneTimePrekeyPublics: string[] = [];
  const oneTimePrekeyPrivatesEncrypted: string[] = [];
  const oneTimePrekeyIvs: string[] = [];
  
  for (let idx = 0; idx < numOneTimePrekeys; idx++) {
    const otpk = await generateExchangeKeyPair();
    const publicRaw = await exportPublicKey(otpk.publicKey);
    const { encrypted, iv } = await encryptPrivateKey(masterKey, otpk.privateKey);
    
    oneTimePrekeyPublics.push(arrayBufferToBase64(publicRaw));
    oneTimePrekeyPrivatesEncrypted.push(arrayBufferToBase64(encrypted));
    oneTimePrekeyIvs.push(arrayBufferToBase64(iv.buffer as ArrayBuffer));
  }
  
  return {
    bundle: {
      signedPrekeyPublic: arrayBufferToBase64(signedPrekeyPublicRaw),
      signedPrekeySignature: arrayBufferToBase64(signature)
    },
    signedPrekeyPrivateEncrypted: arrayBufferToBase64(signedPrekeyPrivateEncrypted),
    signedPrekeyIv: arrayBufferToBase64(signedPrekeyIv.buffer as ArrayBuffer),
    oneTimePrekeyPublics,
    oneTimePrekeyPrivatesEncrypted,
    oneTimePrekeyIvs
  };
}

// Verify pre-key bundle signature
export async function verifyPreKeyBundle(bundle: PreKeyBundle): Promise<boolean> {
  const identityPublicKey = await importIdentityPublicKey(
    base64ToArrayBuffer(bundle.identityPublicKey)
  );
  const signedPrekeyPublic = base64ToArrayBuffer(bundle.signedPrekeyPublic);
  const signature = base64ToArrayBuffer(bundle.signedPrekeySignature);
  
  return verify(identityPublicKey, signature, signedPrekeyPublic);
}
