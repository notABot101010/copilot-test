/**
 * End-to-end encrypted chat crypto library using WebCrypto API
 * 
 * Features:
 * - PBKDF2 for master key derivation from password
 * - ECDSA (P-256) for identity keys (signing)
 * - ECDH (P-256) for ephemeral key exchange per message
 * - AES-256-GCM for message encryption
 * - Sealed sender for metadata protection
 * 
 * Encryption model: Each message uses a fresh ephemeral ECDH key pair.
 * The shared secret is derived from the ephemeral private key and the
 * recipient's prekey. This provides forward secrecy without complex ratcheting.
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
  // Create a new ArrayBuffer and copy the data to ensure proper type
  const buffer = new ArrayBuffer(bytes.length);
  new Uint8Array(buffer).set(bytes);
  return buffer;
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
  // Ensure we have a proper ArrayBuffer (not SharedArrayBuffer)
  const buffer = new Uint8Array(keyData).buffer as ArrayBuffer;
  return crypto.subtle.importKey(
    'raw',
    buffer,
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
  // Ensure we have a proper buffer by creating a new Uint8Array and using it directly
  const bytes = new Uint8Array(keyData);
  return crypto.subtle.importKey(
    'raw',
    bytes,
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
  // Ensure we have proper Uint8Arrays for jsdom compatibility
  const dataBytes = new Uint8Array(data);
  const ivBytes = new Uint8Array(iv);
  return crypto.subtle.decrypt(
    { name: 'AES-GCM', iv: ivBytes as BufferSource },
    key,
    dataBytes
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

// Simple per-message encryption result
export interface EncryptedMessageData {
  ephemeralPublicKey: string; // base64
  iv: string; // base64
  ciphertext: string; // base64
}

/**
 * Encrypt a message using ephemeral ECDH with the recipient's prekey.
 * Each message generates a new ephemeral key pair for forward secrecy.
 * 
 * @param recipientPrekey - The recipient's signed prekey (public key)
 * @param plaintext - The message content to encrypt
 * @returns Encrypted message data including ephemeral public key, IV, and ciphertext
 */
export async function encryptMessage(
  recipientPrekey: CryptoKey,
  plaintext: string
): Promise<EncryptedMessageData> {
  // Generate ephemeral key pair for this message
  const ephemeralKeyPair = await generateExchangeKeyPair();
  
  // Derive shared secret: DH(ephemeral_private, recipient_prekey)
  const sharedSecret = await deriveSharedSecret(ephemeralKeyPair.privateKey, recipientPrekey);
  
  // Generate random IV
  const iv = generateIV();
  
  // Encrypt the message
  const encoder = new TextEncoder();
  const plaintextBytes = encoder.encode(plaintext);
  const ciphertext = await encrypt(sharedSecret, plaintextBytes.buffer as ArrayBuffer, iv);
  
  // Export ephemeral public key
  const ephemeralPublicRaw = await exportPublicKey(ephemeralKeyPair.publicKey);
  
  return {
    ephemeralPublicKey: arrayBufferToBase64(ephemeralPublicRaw),
    iv: arrayBufferToBase64(iv.buffer as ArrayBuffer),
    ciphertext: arrayBufferToBase64(ciphertext)
  };
}

/**
 * Decrypt a message using the recipient's prekey private key.
 * 
 * @param recipientPrekeyPrivate - The recipient's signed prekey (private key)
 * @param encryptedData - The encrypted message data
 * @returns The decrypted message content
 */
export async function decryptMessage(
  recipientPrekeyPrivate: CryptoKey,
  encryptedData: EncryptedMessageData
): Promise<string> {
  // Import the sender's ephemeral public key
  const ephemeralPublicRaw = base64ToArrayBuffer(encryptedData.ephemeralPublicKey);
  const ephemeralPublicKey = await importExchangePublicKey(ephemeralPublicRaw);
  
  // Derive shared secret: DH(recipient_prekey_private, ephemeral_public)
  const sharedSecret = await deriveSharedSecret(recipientPrekeyPrivate, ephemeralPublicKey);
  
  // Get IV and ciphertext
  const iv = new Uint8Array(base64ToArrayBuffer(encryptedData.iv));
  const ciphertext = base64ToArrayBuffer(encryptedData.ciphertext);
  
  // Decrypt the message
  const plaintextBytes = await decrypt(sharedSecret, ciphertext, iv);
  const decoder = new TextDecoder();
  return decoder.decode(plaintextBytes);
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
  // Encrypted message data (ephemeral ECDH per message)
  encryptedMessage: EncryptedMessageData;
}

// Create sealed sender envelope
export async function createSealedSenderEnvelope(
  senderUsername: string,
  senderIdentityPublicKey: CryptoKey,
  recipientSignedPrekey: CryptoKey,
  encryptedMessage: EncryptedMessageData
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
    encryptedMessage
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
