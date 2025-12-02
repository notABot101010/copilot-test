import { describe, it, expect, beforeAll } from 'vitest';
import {
  createE2EEContext,
  establishSharedKey,
  encryptFrame,
  decryptFrame,
} from '../crypto/e2ee';

// Polyfill crypto for Node.js test environment
beforeAll(async () => {
  if (typeof globalThis.crypto === 'undefined') {
    const nodeCrypto = await import('node:crypto');
    (globalThis as Record<string, unknown>).crypto = nodeCrypto.webcrypto;
  }
});

describe('E2EE Context Creation', () => {
  it('should create a valid E2EE context with ECDH keys', async () => {
    const context = await createE2EEContext();
    
    expect(context).toBeDefined();
    expect(context.keyPair).toBeDefined();
    expect(context.keyPair.publicKey).toBeDefined();
    expect(context.keyPair.privateKey).toBeDefined();
    expect(context.publicKeyBase64).toBeDefined();
    expect(typeof context.publicKeyBase64).toBe('string');
    expect(context.publicKeyBase64.length).toBeGreaterThan(0);
    expect(context.sharedKey).toBeNull();
    expect(context.frameCounter).toBe(0);
  });

  it('should generate unique keys each time', async () => {
    const context1 = await createE2EEContext();
    const context2 = await createE2EEContext();
    
    expect(context1.publicKeyBase64).not.toBe(context2.publicKeyBase64);
  });
});

describe('E2EE Key Exchange', () => {
  it('should establish shared key from peer public key', async () => {
    const aliceContext = await createE2EEContext();
    const bobContext = await createE2EEContext();
    
    // Both establish shared keys using each other's public keys
    await establishSharedKey(aliceContext, bobContext.publicKeyBase64);
    await establishSharedKey(bobContext, aliceContext.publicKeyBase64);
    
    expect(aliceContext.sharedKey).not.toBeNull();
    expect(bobContext.sharedKey).not.toBeNull();
  });
});

describe('E2EE Frame Encryption/Decryption', () => {
  it('should encrypt and decrypt frames correctly', async () => {
    const aliceContext = await createE2EEContext();
    const bobContext = await createE2EEContext();
    
    // Establish shared keys
    await establishSharedKey(aliceContext, bobContext.publicKeyBase64);
    await establishSharedKey(bobContext, aliceContext.publicKeyBase64);
    
    // Create test frame data
    const originalData = new TextEncoder().encode('Test video frame data');
    const originalBuffer = originalData.buffer as ArrayBuffer;
    
    // Alice encrypts
    const encrypted = await encryptFrame(aliceContext, originalBuffer);
    
    // Encrypted data should be different and larger
    expect(encrypted.byteLength).toBeGreaterThan(originalBuffer.byteLength);
    expect(new Uint8Array(encrypted)).not.toEqual(new Uint8Array(originalBuffer));
    
    // Bob decrypts
    const decrypted = await decryptFrame(bobContext, encrypted);
    
    // Decrypted data should match original
    const decryptedText = new TextDecoder().decode(decrypted);
    expect(decryptedText).toBe('Test video frame data');
  });

  it('should pass through unencrypted frames when no shared key', async () => {
    const context = await createE2EEContext();
    // Don't establish shared key
    
    const originalData = new TextEncoder().encode('Unencrypted frame');
    const originalBuffer = originalData.buffer as ArrayBuffer;
    
    // Without shared key, frame should pass through unchanged
    const result = await encryptFrame(context, originalBuffer);
    
    // Should be the same (no encryption applied)
    expect(new Uint8Array(result)).toEqual(new Uint8Array(originalBuffer));
  });

  it('should produce different ciphertext for same plaintext (due to frame counter)', async () => {
    const aliceContext = await createE2EEContext();
    const bobContext = await createE2EEContext();
    
    await establishSharedKey(aliceContext, bobContext.publicKeyBase64);
    
    const frameData = new TextEncoder().encode('Same frame data');
    const frameBuffer = frameData.buffer as ArrayBuffer;
    
    // Encrypt the same data twice
    const encrypted1 = await encryptFrame(aliceContext, frameBuffer);
    const encrypted2 = await encryptFrame(aliceContext, frameBuffer);
    
    // Ciphertexts should be different due to frame counter
    expect(new Uint8Array(encrypted1)).not.toEqual(new Uint8Array(encrypted2));
  });

  it('should handle large frames', async () => {
    const aliceContext = await createE2EEContext();
    const bobContext = await createE2EEContext();
    
    await establishSharedKey(aliceContext, bobContext.publicKeyBase64);
    await establishSharedKey(bobContext, aliceContext.publicKeyBase64);
    
    // Create a larger frame (simulating video data)
    const largeFrame = new Uint8Array(64 * 1024); // 64KB
    crypto.getRandomValues(largeFrame);
    
    const encrypted = await encryptFrame(aliceContext, largeFrame.buffer as ArrayBuffer);
    const decrypted = await decryptFrame(bobContext, encrypted);
    
    expect(new Uint8Array(decrypted)).toEqual(largeFrame);
  });

  it('should increment frame counter after each encryption', async () => {
    const context = await createE2EEContext();
    const peerContext = await createE2EEContext();
    
    await establishSharedKey(context, peerContext.publicKeyBase64);
    
    expect(context.frameCounter).toBe(0);
    
    const frameData = new TextEncoder().encode('Frame');
    await encryptFrame(context, frameData.buffer as ArrayBuffer);
    expect(context.frameCounter).toBe(1);
    
    await encryptFrame(context, frameData.buffer as ArrayBuffer);
    expect(context.frameCounter).toBe(2);
    
    await encryptFrame(context, frameData.buffer as ArrayBuffer);
    expect(context.frameCounter).toBe(3);
  });
});

describe('E2EE Bidirectional Communication', () => {
  it('should allow bidirectional encrypted communication', async () => {
    const aliceContext = await createE2EEContext();
    const bobContext = await createE2EEContext();
    
    // Both establish shared keys
    await establishSharedKey(aliceContext, bobContext.publicKeyBase64);
    await establishSharedKey(bobContext, aliceContext.publicKeyBase64);
    
    // Alice sends to Bob
    const aliceMessage = new TextEncoder().encode('Hello from Alice');
    const encryptedFromAlice = await encryptFrame(aliceContext, aliceMessage.buffer as ArrayBuffer);
    const decryptedByBob = await decryptFrame(bobContext, encryptedFromAlice);
    expect(new TextDecoder().decode(decryptedByBob)).toBe('Hello from Alice');
    
    // Bob sends to Alice
    const bobMessage = new TextEncoder().encode('Hello from Bob');
    const encryptedFromBob = await encryptFrame(bobContext, bobMessage.buffer as ArrayBuffer);
    const decryptedByAlice = await decryptFrame(aliceContext, encryptedFromBob);
    expect(new TextDecoder().decode(decryptedByAlice)).toBe('Hello from Bob');
  });
});
