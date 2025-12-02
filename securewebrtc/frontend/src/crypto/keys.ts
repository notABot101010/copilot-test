// Ed25519 key generation and management using WebCrypto

export interface KeyPair {
  publicKey: CryptoKey;
  privateKey: CryptoKey;
}

export interface ExportedKeyPair {
  publicKey: string;
  privateKey: string;
}

// Generate a new Ed25519 key pair for identity
export async function generateIdentityKeys(): Promise<KeyPair> {
  const keyPair = await crypto.subtle.generateKey(
    {
      name: 'Ed25519',
    },
    true,
    ['sign', 'verify']
  );
  return keyPair as KeyPair;
}

// Generate a new ECDH key pair for key exchange (used for deriving shared secrets)
export async function generateECDHKeys(): Promise<KeyPair> {
  const keyPair = await crypto.subtle.generateKey(
    {
      name: 'ECDH',
      namedCurve: 'P-256',
    },
    true,
    ['deriveBits']
  );
  return keyPair as KeyPair;
}

// Export public key to base64url format
export async function exportPublicKey(key: CryptoKey): Promise<string> {
  const exported = await crypto.subtle.exportKey('raw', key);
  return arrayBufferToBase64Url(exported);
}

// Export private key to base64url format (for storage)
export async function exportPrivateKey(key: CryptoKey): Promise<string> {
  const exported = await crypto.subtle.exportKey('pkcs8', key);
  return arrayBufferToBase64Url(exported);
}

// Import public key from base64url format (Ed25519)
export async function importEd25519PublicKey(base64: string): Promise<CryptoKey> {
  const data = base64UrlToArrayBuffer(base64);
  return crypto.subtle.importKey(
    'raw',
    data,
    { name: 'Ed25519' },
    true,
    ['verify']
  );
}

// Import ECDH public key from base64url format
export async function importECDHPublicKey(base64: string): Promise<CryptoKey> {
  const data = base64UrlToArrayBuffer(base64);
  return crypto.subtle.importKey(
    'raw',
    data,
    { name: 'ECDH', namedCurve: 'P-256' },
    true,
    []
  );
}

// Import private key from base64url format (ECDH)
export async function importECDHPrivateKey(base64: string): Promise<CryptoKey> {
  const data = base64UrlToArrayBuffer(base64);
  return crypto.subtle.importKey(
    'pkcs8',
    data,
    { name: 'ECDH', namedCurve: 'P-256' },
    true,
    ['deriveBits']
  );
}

// Sign data with Ed25519 private key
export async function sign(privateKey: CryptoKey, data: ArrayBuffer): Promise<ArrayBuffer> {
  return crypto.subtle.sign(
    { name: 'Ed25519' },
    privateKey,
    data
  );
}

// Verify signature with Ed25519 public key
export async function verify(publicKey: CryptoKey, signature: ArrayBuffer, data: ArrayBuffer): Promise<boolean> {
  return crypto.subtle.verify(
    { name: 'Ed25519' },
    publicKey,
    signature,
    data
  );
}

// Derive a shared secret using ECDH
export async function deriveSharedSecret(
  privateKey: CryptoKey,
  publicKey: CryptoKey
): Promise<ArrayBuffer> {
  const sharedBits = await crypto.subtle.deriveBits(
    {
      name: 'ECDH',
      public: publicKey,
    },
    privateKey,
    256
  );
  return sharedBits;
}

// Derive an AES-GCM key from shared secret
export async function deriveAESKey(sharedSecret: ArrayBuffer): Promise<CryptoKey> {
  const keyMaterial = await crypto.subtle.importKey(
    'raw',
    sharedSecret,
    'HKDF',
    false,
    ['deriveKey']
  );
  
  return crypto.subtle.deriveKey(
    {
      name: 'HKDF',
      hash: 'SHA-256',
      salt: new Uint8Array(32),
      info: new TextEncoder().encode('webrtc-e2ee'),
    },
    keyMaterial,
    { name: 'AES-GCM', length: 256 },
    true,
    ['encrypt', 'decrypt']
  );
}

// Encrypt data with AES-GCM
export async function encrypt(key: CryptoKey, data: ArrayBuffer): Promise<{ ciphertext: ArrayBuffer; iv: Uint8Array }> {
  const iv = crypto.getRandomValues(new Uint8Array(12));
  const ciphertext = await crypto.subtle.encrypt(
    { name: 'AES-GCM', iv: iv as Uint8Array<ArrayBuffer> },
    key,
    data
  );
  return { ciphertext, iv };
}

// Decrypt data with AES-GCM
export async function decrypt(key: CryptoKey, ciphertext: ArrayBuffer, iv: Uint8Array<ArrayBuffer>): Promise<ArrayBuffer> {
  return crypto.subtle.decrypt(
    { name: 'AES-GCM', iv },
    key,
    ciphertext
  );
}

// Convert ArrayBuffer to base64url string
export function arrayBufferToBase64Url(buffer: ArrayBuffer): string {
  const bytes = new Uint8Array(buffer);
  let binary = '';
  for (let idx = 0; idx < bytes.length; idx++) {
    binary += String.fromCharCode(bytes[idx]);
  }
  return btoa(binary)
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=/g, '');
}

// Convert base64url string to ArrayBuffer
export function base64UrlToArrayBuffer(base64url: string): ArrayBuffer {
  const base64 = base64url
    .replace(/-/g, '+')
    .replace(/_/g, '/');
  const padding = '='.repeat((4 - (base64.length % 4)) % 4);
  const binary = atob(base64 + padding);
  const bytes = new Uint8Array(binary.length);
  for (let idx = 0; idx < binary.length; idx++) {
    bytes[idx] = binary.charCodeAt(idx);
  }
  return bytes.buffer;
}
