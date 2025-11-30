// Hardcoded Ed25519 identity key (for prototype only)
// In production, this should be derived from user authentication
const IDENTITY_SEED = new Uint8Array(32).fill(42);

// Generate a document encryption key
export async function generateDocumentKey(): Promise<CryptoKey> {
  return await crypto.subtle.generateKey(
    {
      name: 'AES-GCM',
      length: 256,
    },
    true,
    ['encrypt', 'decrypt']
  );
}

// Export key to raw format for storage
export async function exportKey(key: CryptoKey): Promise<string> {
  const exported = await crypto.subtle.exportKey('raw', key);
  return arrayBufferToBase64(exported);
}

// Import key from raw format
export async function importKey(keyData: string): Promise<CryptoKey> {
  const buffer = base64ToArrayBuffer(keyData);
  return await crypto.subtle.importKey(
    'raw',
    buffer,
    'AES-GCM',
    true,
    ['encrypt', 'decrypt']
  );
}

// Encrypt data with AES-GCM
export async function encrypt(
  data: Uint8Array,
  key: CryptoKey
): Promise<{ encrypted: string; iv: string }> {
  const iv = crypto.getRandomValues(new Uint8Array(12));

  const encrypted = await crypto.subtle.encrypt(
    {
      name: 'AES-GCM',
      iv: iv as BufferSource,
    },
    key,
    data as BufferSource
  );

  return {
    encrypted: arrayBufferToBase64(encrypted),
    iv: arrayBufferToBase64(iv.buffer),
  };
}

// Decrypt data with AES-GCM
export async function decrypt(
  encryptedData: string,
  ivData: string,
  key: CryptoKey
): Promise<Uint8Array> {
  const encrypted = base64ToArrayBuffer(encryptedData);
  const iv = base64ToArrayBuffer(ivData);

  const decrypted = await crypto.subtle.decrypt(
    {
      name: 'AES-GCM',
      iv,
    },
    key,
    encrypted
  );

  return new Uint8Array(decrypted);
}

// Utility functions for base64 encoding/decoding
function arrayBufferToBase64(buffer: ArrayBuffer): string {
  const bytes = new Uint8Array(buffer);
  let binary = '';
  for (let i = 0; i < bytes.byteLength; i++) {
    binary += String.fromCharCode(bytes[i]);
  }
  return btoa(binary);
}

function base64ToArrayBuffer(base64: string): ArrayBuffer {
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes.buffer;
}

// Generate identity keypair (Ed25519) - hardcoded for prototype
export async function getIdentityKeyPair() {
  // For a real implementation, use Ed25519
  // For this prototype, we'll just return the hardcoded seed
  return {
    publicKey: IDENTITY_SEED,
    privateKey: IDENTITY_SEED,
  };
}
