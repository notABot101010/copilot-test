import { describe, it, expect, beforeAll } from 'vitest';
import {
  generateIdentityKeys,
  generateECDHKeys,
  exportPublicKey,
  sign,
  verify,
  deriveSharedSecret,
  deriveAESKey,
  encrypt,
  decrypt,
  arrayBufferToBase64Url,
  base64UrlToArrayBuffer,
} from '../crypto/keys';

// Polyfill crypto for Node.js test environment
beforeAll(async () => {
  if (typeof globalThis.crypto === 'undefined') {
    const nodeCrypto = await import('node:crypto');
    (globalThis as Record<string, unknown>).crypto = nodeCrypto.webcrypto;
  }
});

describe('Ed25519 Key Generation', () => {
  it('should generate a valid Ed25519 key pair', async () => {
    const keyPair = await generateIdentityKeys();
    
    expect(keyPair).toBeDefined();
    expect(keyPair.publicKey).toBeDefined();
    expect(keyPair.privateKey).toBeDefined();
    expect(keyPair.publicKey.algorithm.name).toBe('Ed25519');
    expect(keyPair.privateKey.algorithm.name).toBe('Ed25519');
  });

  it('should export public key to base64url format', async () => {
    const keyPair = await generateIdentityKeys();
    const exported = await exportPublicKey(keyPair.publicKey);
    
    expect(typeof exported).toBe('string');
    expect(exported.length).toBeGreaterThan(0);
    // Ed25519 public keys are 32 bytes, base64url encoded
    expect(exported.length).toBe(43); // 32 bytes -> 43 base64url chars
  });

  it('should generate unique keys each time', async () => {
    const keyPair1 = await generateIdentityKeys();
    const keyPair2 = await generateIdentityKeys();
    
    const exported1 = await exportPublicKey(keyPair1.publicKey);
    const exported2 = await exportPublicKey(keyPair2.publicKey);
    
    expect(exported1).not.toBe(exported2);
  });
});

describe('Ed25519 Signing and Verification', () => {
  it('should sign and verify data correctly', async () => {
    const keyPair = await generateIdentityKeys();
    const data = new TextEncoder().encode('Hello, World!');
    
    const signature = await sign(keyPair.privateKey, data.buffer as ArrayBuffer);
    const isValid = await verify(keyPair.publicKey, signature, data.buffer as ArrayBuffer);
    
    expect(isValid).toBe(true);
  });

  it('should fail verification with wrong data', async () => {
    const keyPair = await generateIdentityKeys();
    const data = new TextEncoder().encode('Hello, World!');
    const wrongData = new TextEncoder().encode('Goodbye, World!');
    
    const signature = await sign(keyPair.privateKey, data.buffer as ArrayBuffer);
    const isValid = await verify(keyPair.publicKey, signature, wrongData.buffer as ArrayBuffer);
    
    expect(isValid).toBe(false);
  });

  it('should fail verification with wrong key', async () => {
    const keyPair1 = await generateIdentityKeys();
    const keyPair2 = await generateIdentityKeys();
    const data = new TextEncoder().encode('Hello, World!');
    
    const signature = await sign(keyPair1.privateKey, data.buffer as ArrayBuffer);
    const isValid = await verify(keyPair2.publicKey, signature, data.buffer as ArrayBuffer);
    
    expect(isValid).toBe(false);
  });
});

describe('ECDH Key Exchange', () => {
  it('should generate a valid ECDH key pair', async () => {
    const keyPair = await generateECDHKeys();
    
    expect(keyPair).toBeDefined();
    expect(keyPair.publicKey).toBeDefined();
    expect(keyPair.privateKey).toBeDefined();
    expect(keyPair.publicKey.algorithm.name).toBe('ECDH');
    expect(keyPair.privateKey.algorithm.name).toBe('ECDH');
  });

  it('should derive the same shared secret on both sides', async () => {
    const aliceKeys = await generateECDHKeys();
    const bobKeys = await generateECDHKeys();
    
    const aliceShared = await deriveSharedSecret(aliceKeys.privateKey, bobKeys.publicKey);
    const bobShared = await deriveSharedSecret(bobKeys.privateKey, aliceKeys.publicKey);
    
    const aliceBytes = new Uint8Array(aliceShared);
    const bobBytes = new Uint8Array(bobShared);
    
    expect(aliceBytes.length).toBe(bobBytes.length);
    expect(Array.from(aliceBytes)).toEqual(Array.from(bobBytes));
  });
});

describe('AES-GCM Encryption', () => {
  it('should encrypt and decrypt data correctly', async () => {
    const aliceKeys = await generateECDHKeys();
    const bobKeys = await generateECDHKeys();
    
    const sharedSecret = await deriveSharedSecret(aliceKeys.privateKey, bobKeys.publicKey);
    const aesKey = await deriveAESKey(sharedSecret);
    
    const plaintext = new TextEncoder().encode('Secret message');
    const { ciphertext, iv } = await encrypt(aesKey, plaintext.buffer as ArrayBuffer);
    
    const decrypted = await decrypt(aesKey, ciphertext, iv);
    const decryptedText = new TextDecoder().decode(decrypted);
    
    expect(decryptedText).toBe('Secret message');
  });

  it('should produce different ciphertext for same plaintext', async () => {
    const aliceKeys = await generateECDHKeys();
    const bobKeys = await generateECDHKeys();
    
    const sharedSecret = await deriveSharedSecret(aliceKeys.privateKey, bobKeys.publicKey);
    const aesKey = await deriveAESKey(sharedSecret);
    
    const plaintext = new TextEncoder().encode('Secret message');
    const { ciphertext: ct1 } = await encrypt(aesKey, plaintext.buffer as ArrayBuffer);
    const { ciphertext: ct2 } = await encrypt(aesKey, plaintext.buffer as ArrayBuffer);
    
    // Due to random IV, ciphertexts should be different
    const bytes1 = new Uint8Array(ct1);
    const bytes2 = new Uint8Array(ct2);
    expect(Array.from(bytes1)).not.toEqual(Array.from(bytes2));
  });
});

describe('Base64URL Encoding', () => {
  it('should encode and decode correctly', () => {
    const original = new Uint8Array([0, 1, 2, 255, 254, 253]);
    const encoded = arrayBufferToBase64Url(original.buffer as ArrayBuffer);
    const decoded = base64UrlToArrayBuffer(encoded);
    
    expect(new Uint8Array(decoded)).toEqual(original);
  });

  it('should produce URL-safe output', () => {
    // Test with bytes that would produce + and / in standard base64
    const data = new Uint8Array([251, 255, 254]);
    const encoded = arrayBufferToBase64Url(data.buffer as ArrayBuffer);
    
    expect(encoded).not.toContain('+');
    expect(encoded).not.toContain('/');
    expect(encoded).not.toContain('=');
  });
});
