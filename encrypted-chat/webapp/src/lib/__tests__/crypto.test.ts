/**
 * Unit tests for crypto operations
 */

import { describe, it, expect } from 'vitest';
import * as crypto from '../crypto';

describe('Crypto utilities', () => {
  describe('base64 encoding/decoding', () => {
    it('should encode and decode ArrayBuffer correctly', () => {
      const original = new Uint8Array([1, 2, 3, 4, 5]);
      const encoded = crypto.arrayBufferToBase64(original.buffer);
      const decoded = new Uint8Array(crypto.base64ToArrayBuffer(encoded));
      
      expect(decoded).toEqual(original);
    });
    
    it('should handle empty buffer', () => {
      const original = new Uint8Array([]);
      const encoded = crypto.arrayBufferToBase64(original.buffer);
      const decoded = new Uint8Array(crypto.base64ToArrayBuffer(encoded));
      
      expect(decoded.length).toBe(0);
    });
    
    it('should handle large buffer', () => {
      const original = new Uint8Array(1000);
      for (let idx = 0; idx < 1000; idx++) {
        original[idx] = idx % 256;
      }
      
      const encoded = crypto.arrayBufferToBase64(original.buffer);
      const decoded = new Uint8Array(crypto.base64ToArrayBuffer(encoded));
      
      expect(decoded).toEqual(original);
    });
  });
  
  describe('random byte generation', () => {
    it('should generate salt of correct length', () => {
      const salt = crypto.generateSalt();
      expect(salt.length).toBe(16);
    });
    
    it('should generate IV of correct length', () => {
      const iv = crypto.generateIV();
      expect(iv.length).toBe(12);
    });
    
    it('should generate random bytes of specified length', () => {
      const bytes = crypto.generateRandomBytes(32);
      expect(bytes.length).toBe(32);
    });
    
    it('should generate different values on each call', () => {
      const salt1 = crypto.generateSalt();
      const salt2 = crypto.generateSalt();
      
      // Very unlikely to be equal
      let areDifferent = false;
      for (let idx = 0; idx < salt1.length; idx++) {
        if (salt1[idx] !== salt2[idx]) {
          areDifferent = true;
          break;
        }
      }
      expect(areDifferent).toBe(true);
    });
  });
  
  describe('master key derivation', () => {
    it('should derive a key from password', async () => {
      const password = 'test-password-123';
      const salt = crypto.generateSalt();
      
      const key = await crypto.deriveMasterKey(password, salt);
      
      expect(key).toBeDefined();
      expect(key.type).toBe('secret');
      expect(key.algorithm.name).toBe('AES-GCM');
    });
    
    it('should derive same key from same password and salt', async () => {
      const password = 'test-password-123';
      const salt = crypto.generateSalt();
      
      const key1 = await crypto.deriveMasterKey(password, salt);
      const key2 = await crypto.deriveMasterKey(password, salt);
      
      // Export both keys and compare
      const exported1 = await globalThis.crypto.subtle.exportKey('raw', key1);
      const exported2 = await globalThis.crypto.subtle.exportKey('raw', key2);
      
      expect(new Uint8Array(exported1)).toEqual(new Uint8Array(exported2));
    });
    
    it('should derive different keys from different passwords', async () => {
      const salt = crypto.generateSalt();
      
      const key1 = await crypto.deriveMasterKey('password1', salt);
      const key2 = await crypto.deriveMasterKey('password2', salt);
      
      const exported1 = await globalThis.crypto.subtle.exportKey('raw', key1);
      const exported2 = await globalThis.crypto.subtle.exportKey('raw', key2);
      
      expect(new Uint8Array(exported1)).not.toEqual(new Uint8Array(exported2));
    });
    
    it('should derive different keys from different salts', async () => {
      const password = 'test-password';
      const salt1 = crypto.generateSalt();
      const salt2 = crypto.generateSalt();
      
      const key1 = await crypto.deriveMasterKey(password, salt1);
      const key2 = await crypto.deriveMasterKey(password, salt2);
      
      const exported1 = await globalThis.crypto.subtle.exportKey('raw', key1);
      const exported2 = await globalThis.crypto.subtle.exportKey('raw', key2);
      
      expect(new Uint8Array(exported1)).not.toEqual(new Uint8Array(exported2));
    });
  });
  
  describe('identity key generation', () => {
    it('should generate an identity key pair', async () => {
      const keyPair = await crypto.generateIdentityKeyPair();
      
      expect(keyPair.privateKey).toBeDefined();
      expect(keyPair.publicKey).toBeDefined();
      expect(keyPair.privateKey.type).toBe('private');
      expect(keyPair.publicKey.type).toBe('public');
    });
    
    it('should export public key', async () => {
      const keyPair = await crypto.generateIdentityKeyPair();
      const exported = await crypto.exportPublicKey(keyPair.publicKey);
      
      expect(exported.byteLength).toBeGreaterThan(0);
    });
    
    it('should export private key', async () => {
      const keyPair = await crypto.generateIdentityKeyPair();
      const exported = await crypto.exportPrivateKey(keyPair.privateKey);
      
      expect(exported.byteLength).toBeGreaterThan(0);
    });
  });
  
  describe('exchange key generation', () => {
    it('should generate an exchange key pair', async () => {
      const keyPair = await crypto.generateExchangeKeyPair();
      
      expect(keyPair.privateKey).toBeDefined();
      expect(keyPair.publicKey).toBeDefined();
      expect(keyPair.privateKey.type).toBe('private');
      expect(keyPair.publicKey.type).toBe('public');
    });
  });
  
  describe('signing and verification', () => {
    it('should sign and verify data', async () => {
      const keyPair = await crypto.generateIdentityKeyPair();
      const data = new TextEncoder().encode('Hello, World!');
      
      const signature = await crypto.sign(keyPair.privateKey, data.buffer);
      const isValid = await crypto.verify(keyPair.publicKey, signature, data.buffer);
      
      expect(isValid).toBe(true);
    });
    
    it('should fail verification with wrong data', async () => {
      const keyPair = await crypto.generateIdentityKeyPair();
      const data = new TextEncoder().encode('Hello, World!');
      const wrongData = new TextEncoder().encode('Hello, Wrong!');
      
      const signature = await crypto.sign(keyPair.privateKey, data.buffer);
      const isValid = await crypto.verify(keyPair.publicKey, signature, wrongData.buffer);
      
      expect(isValid).toBe(false);
    });
    
    it('should fail verification with wrong key', async () => {
      const keyPair1 = await crypto.generateIdentityKeyPair();
      const keyPair2 = await crypto.generateIdentityKeyPair();
      const data = new TextEncoder().encode('Hello, World!');
      
      const signature = await crypto.sign(keyPair1.privateKey, data.buffer);
      const isValid = await crypto.verify(keyPair2.publicKey, signature, data.buffer);
      
      expect(isValid).toBe(false);
    });
  });
  
  describe('key exchange', () => {
    it('should derive shared secret', async () => {
      const keyPair1 = await crypto.generateExchangeKeyPair();
      const keyPair2 = await crypto.generateExchangeKeyPair();
      
      const secret1 = await crypto.deriveSharedSecret(keyPair1.privateKey, keyPair2.publicKey);
      const secret2 = await crypto.deriveSharedSecret(keyPair2.privateKey, keyPair1.publicKey);
      
      const exported1 = await globalThis.crypto.subtle.exportKey('raw', secret1);
      const exported2 = await globalThis.crypto.subtle.exportKey('raw', secret2);
      
      expect(new Uint8Array(exported1)).toEqual(new Uint8Array(exported2));
    });
  });
  
  describe('encryption and decryption', () => {
    it('should encrypt and decrypt data', async () => {
      const keyPair = await crypto.generateExchangeKeyPair();
      const keyPair2 = await crypto.generateExchangeKeyPair();
      const sharedSecret = await crypto.deriveSharedSecret(keyPair.privateKey, keyPair2.publicKey);
      
      const plaintext = new TextEncoder().encode('Secret message!');
      const iv = crypto.generateIV();
      
      const ciphertext = await crypto.encrypt(sharedSecret, plaintext.buffer as ArrayBuffer, iv);
      const decrypted = await crypto.decrypt(sharedSecret, ciphertext, iv);
      
      const decryptedBytes = new Uint8Array(decrypted);
      const plaintextBytes = new Uint8Array(plaintext);
      expect(decryptedBytes.length).toBe(plaintextBytes.length);
      for (let idx = 0; idx < decryptedBytes.length; idx++) {
        expect(decryptedBytes[idx]).toBe(plaintextBytes[idx]);
      }
    });
    
    it('should fail decryption with wrong key', async () => {
      const keyPair1 = await crypto.generateExchangeKeyPair();
      const keyPair2 = await crypto.generateExchangeKeyPair();
      const keyPair3 = await crypto.generateExchangeKeyPair();
      
      const secret1 = await crypto.deriveSharedSecret(keyPair1.privateKey, keyPair2.publicKey);
      const secret2 = await crypto.deriveSharedSecret(keyPair2.privateKey, keyPair3.publicKey);
      
      const plaintext = new TextEncoder().encode('Secret message!');
      const iv = crypto.generateIV();
      
      const ciphertext = await crypto.encrypt(secret1, plaintext.buffer as ArrayBuffer, iv);
      
      await expect(crypto.decrypt(secret2, ciphertext, iv)).rejects.toThrow();
    });
    
    it('should fail decryption with wrong IV', async () => {
      const keyPair1 = await crypto.generateExchangeKeyPair();
      const keyPair2 = await crypto.generateExchangeKeyPair();
      const sharedSecret = await crypto.deriveSharedSecret(keyPair1.privateKey, keyPair2.publicKey);
      
      const plaintext = new TextEncoder().encode('Secret message!');
      const iv1 = crypto.generateIV();
      const iv2 = crypto.generateIV();
      
      const ciphertext = await crypto.encrypt(sharedSecret, plaintext.buffer as ArrayBuffer, iv1);
      
      await expect(crypto.decrypt(sharedSecret, ciphertext, iv2)).rejects.toThrow();
    });
  });
  
  describe('private key encryption', () => {
    it('should encrypt and decrypt private key', async () => {
      const password = 'secure-password';
      const salt = crypto.generateSalt();
      const masterKey = await crypto.deriveMasterKey(password, salt);
      
      const identityKeyPair = await crypto.generateIdentityKeyPair();
      const { encrypted, iv } = await crypto.encryptPrivateKey(masterKey, identityKeyPair.privateKey);
      
      const decrypted = await crypto.decryptIdentityPrivateKey(masterKey, encrypted, iv);
      
      expect(decrypted.type).toBe('private');
    });
    
    it('should fail decryption with wrong password', async () => {
      const salt = crypto.generateSalt();
      const masterKey1 = await crypto.deriveMasterKey('password1', salt);
      const masterKey2 = await crypto.deriveMasterKey('password2', salt);
      
      const identityKeyPair = await crypto.generateIdentityKeyPair();
      const { encrypted, iv } = await crypto.encryptPrivateKey(masterKey1, identityKeyPair.privateKey);
      
      await expect(crypto.decryptIdentityPrivateKey(masterKey2, encrypted, iv)).rejects.toThrow();
    });
  });
  
  describe('pre-key bundle', () => {
    it('should generate pre-key bundle with correct structure', async () => {
      const password = 'secure-password';
      const salt = crypto.generateSalt();
      const masterKey = await crypto.deriveMasterKey(password, salt);
      
      const identityKeyPair = await crypto.generateIdentityKeyPair();
      
      const preKeyData = await crypto.generatePreKeyBundle(identityKeyPair, masterKey, 5);
      
      // Check structure
      expect(preKeyData.bundle.signedPrekeyPublic).toBeDefined();
      expect(preKeyData.bundle.signedPrekeySignature).toBeDefined();
      expect(preKeyData.signedPrekeyPrivateEncrypted).toBeDefined();
      expect(preKeyData.signedPrekeyIv).toBeDefined();
    });
    
    it('should generate correct number of one-time pre-keys', async () => {
      const password = 'secure-password';
      const salt = crypto.generateSalt();
      const masterKey = await crypto.deriveMasterKey(password, salt);
      
      const identityKeyPair = await crypto.generateIdentityKeyPair();
      const preKeyData = await crypto.generatePreKeyBundle(identityKeyPair, masterKey, 5);
      
      expect(preKeyData.oneTimePrekeyPublics.length).toBe(5);
      expect(preKeyData.oneTimePrekeyPrivatesEncrypted.length).toBe(5);
      expect(preKeyData.oneTimePrekeyIvs.length).toBe(5);
    });
  });
});
