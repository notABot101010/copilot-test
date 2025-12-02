// End-to-end encryption for WebRTC media streams using Insertable Streams API
// Uses ephemeral ECDH keys and AES-GCM-256 for encryption

import {
  generateECDHKeys,
  exportPublicKey,
  importECDHPublicKey,
  deriveSharedSecret,
  deriveAESKey,
  KeyPair,
} from './keys';

export interface E2EEContext {
  keyPair: KeyPair;
  publicKeyBase64: string;
  sharedKey: CryptoKey | null;
  frameCounter: number;
}

// Create a new E2EE context with ephemeral ECDH keys
export async function createE2EEContext(): Promise<E2EEContext> {
  const keyPair = await generateECDHKeys();
  const publicKeyBase64 = await exportPublicKey(keyPair.publicKey);
  
  return {
    keyPair,
    publicKeyBase64,
    sharedKey: null,
    frameCounter: 0,
  };
}

// Establish shared key from peer's public key
export async function establishSharedKey(
  context: E2EEContext,
  peerPublicKeyBase64: string
): Promise<void> {
  const peerPublicKey = await importECDHPublicKey(peerPublicKeyBase64);
  const sharedSecret = await deriveSharedSecret(context.keyPair.privateKey, peerPublicKey);
  context.sharedKey = await deriveAESKey(sharedSecret);
}

// Frame header format:
// - 4 bytes: frame counter (for IV generation)
// - 12 bytes: IV (nonce for AES-GCM)
// - N bytes: encrypted data
// - 16 bytes: GCM auth tag (included in ciphertext by Web Crypto)

const HEADER_SIZE = 4; // Frame counter
const IV_SIZE = 12;
const OVERHEAD = HEADER_SIZE + IV_SIZE;

// Generate IV from frame counter to ensure uniqueness
function generateIV(frameCounter: number): Uint8Array {
  const iv = new Uint8Array(IV_SIZE);
  // Use frame counter as part of IV to ensure uniqueness
  const view = new DataView(iv.buffer);
  view.setUint32(0, frameCounter, true);
  // Add random bytes for additional entropy
  crypto.getRandomValues(iv.subarray(4));
  return iv;
}

// Encrypt a single frame
export async function encryptFrame(
  context: E2EEContext,
  frame: ArrayBuffer
): Promise<ArrayBuffer> {
  if (!context.sharedKey) {
    // No shared key yet, return frame unencrypted but with marker
    return frame;
  }

  const frameCounter = context.frameCounter++;
  const iv = generateIV(frameCounter);

  // Use Uint8Array for better compatibility with Node.js WebCrypto
  const frameData = new Uint8Array(frame);
  const ciphertext = await crypto.subtle.encrypt(
    { name: 'AES-GCM', iv: iv as unknown as BufferSource },
    context.sharedKey,
    frameData
  );

  // Combine header + IV + ciphertext
  const result = new Uint8Array(OVERHEAD + ciphertext.byteLength);
  const view = new DataView(result.buffer);
  view.setUint32(0, frameCounter, true);
  result.set(iv, HEADER_SIZE);
  result.set(new Uint8Array(ciphertext), OVERHEAD);

  return result.buffer;
}

// Decrypt a single frame
export async function decryptFrame(
  context: E2EEContext,
  encryptedFrame: ArrayBuffer
): Promise<ArrayBuffer> {
  if (!context.sharedKey) {
    // No shared key yet, return frame as-is
    return encryptedFrame;
  }

  if (encryptedFrame.byteLength < OVERHEAD) {
    // Frame too small to be encrypted, return as-is
    return encryptedFrame;
  }

  const data = new Uint8Array(encryptedFrame);
  const iv = data.slice(HEADER_SIZE, OVERHEAD);
  const ciphertext = data.slice(OVERHEAD);

  try {
    const plaintext = await crypto.subtle.decrypt(
      { name: 'AES-GCM', iv: iv as unknown as BufferSource },
      context.sharedKey,
      ciphertext
    );
    return plaintext;
  } catch {
    // Decryption failed, frame might be corrupted or not encrypted
    return encryptedFrame;
  }
}

// Check if Insertable Streams API is supported
export function isInsertableStreamsSupported(): boolean {
  return (
    typeof RTCRtpSender !== 'undefined' &&
    'createEncodedStreams' in RTCRtpSender.prototype
  ) || (
    typeof RTCRtpSender !== 'undefined' &&
    typeof (RTCRtpSender.prototype as { transform?: unknown }).transform !== 'undefined'
  );
}

// Apply encryption transform to an RTCRtpSender
export function applyEncryptionTransform(
  sender: RTCRtpSender,
  context: E2EEContext
): void {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const senderAny = sender as any;
  
  if (typeof senderAny.transform !== 'undefined') {
    // Use TransformStream-based approach (Chrome 94+)
    const transformer = new TransformStream({
      async transform(frame, controller) {
        const data = frame.data as ArrayBuffer;
        const encryptedData = await encryptFrame(context, data);
        frame.data = encryptedData;
        controller.enqueue(frame);
      },
    });
    senderAny.transform = transformer;
  } else if (typeof senderAny.createEncodedStreams === 'function') {
    // Legacy Insertable Streams API (Chrome 86-93)
    const { readable, writable } = senderAny.createEncodedStreams();
    const transformer = new TransformStream({
      async transform(frame, controller) {
        const data = frame.data as ArrayBuffer;
        const encryptedData = await encryptFrame(context, data);
        frame.data = encryptedData;
        controller.enqueue(frame);
      },
    });
    readable.pipeThrough(transformer).pipeTo(writable);
  }
}

// Apply decryption transform to an RTCRtpReceiver
export function applyDecryptionTransform(
  receiver: RTCRtpReceiver,
  context: E2EEContext
): void {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const receiverAny = receiver as any;
  
  if (typeof receiverAny.transform !== 'undefined') {
    // Use TransformStream-based approach (Chrome 94+)
    const transformer = new TransformStream({
      async transform(frame, controller) {
        const data = frame.data as ArrayBuffer;
        const decryptedData = await decryptFrame(context, data);
        frame.data = decryptedData;
        controller.enqueue(frame);
      },
    });
    receiverAny.transform = transformer;
  } else if (typeof receiverAny.createEncodedStreams === 'function') {
    // Legacy Insertable Streams API (Chrome 86-93)
    const { readable, writable } = receiverAny.createEncodedStreams();
    const transformer = new TransformStream({
      async transform(frame, controller) {
        const data = frame.data as ArrayBuffer;
        const decryptedData = await decryptFrame(context, data);
        frame.data = decryptedData;
        controller.enqueue(frame);
      },
    });
    readable.pipeThrough(transformer).pipeTo(writable);
  }
}
