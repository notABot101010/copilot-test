import {
  generateX25519KeyPair,
  x25519SharedSecret,
  encryptAES256GCM,
  decryptAES256GCM,
  signEd25519,
  verifyEd25519,
  hkdfDerive,
  bytesToBase64,
  base64ToBytes,
  type Ed25519KeyPair,
} from './utils';

const INFO_SEALED_SENDER = new TextEncoder().encode('SealedSender');

export interface SealedMessage {
  encryptedContent: string;
  iv: string;
  tag: string;
  ephemeralPublicKey: string;
  senderIdentityKey: string | null; // null for fully sealed
  signature: string | null; // null for fully sealed
}

// Create a sealed sender message (fully anonymous)
export async function createSealedMessage(
  recipientPublicKey: Uint8Array,
  senderIdentityKeyPair: Ed25519KeyPair,
  messageContent: Uint8Array,
  fullySealed: boolean = true
): Promise<SealedMessage> {
  // Generate ephemeral key pair for this message
  const ephemeralKeyPair = generateX25519KeyPair();

  // Perform DH key exchange
  const sharedSecret = x25519SharedSecret(
    ephemeralKeyPair.privateKey,
    recipientPublicKey
  );

  // Derive encryption key
  const encryptionKey = hkdfDerive(
    sharedSecret,
    new Uint8Array(32),
    INFO_SEALED_SENDER,
    32
  );

  // Encrypt the message
  const encrypted = await encryptAES256GCM(messageContent, encryptionKey);

  let senderIdentityKey: string | null = null;
  let signature: string | null = null;

  if (!fullySealed) {
    // Include sender identity (but still hide from server)
    senderIdentityKey = bytesToBase64(senderIdentityKeyPair.publicKey);

    // Sign the ephemeral public key with identity key
    const signatureBytes = signEd25519(
      ephemeralKeyPair.publicKey,
      senderIdentityKeyPair.privateKey
    );
    signature = bytesToBase64(signatureBytes);
  }

  return {
    encryptedContent: bytesToBase64(encrypted.ciphertext),
    iv: bytesToBase64(encrypted.iv),
    tag: bytesToBase64(encrypted.tag),
    ephemeralPublicKey: bytesToBase64(ephemeralKeyPair.publicKey),
    senderIdentityKey,
    signature,
  };
}

// Open a sealed message
export async function openSealedMessage(
  message: SealedMessage,
  recipientPrivateKey: Uint8Array,
  senderIdentityPublicKey?: Uint8Array
): Promise<Uint8Array> {
  const ephemeralPublicKey = base64ToBytes(message.ephemeralPublicKey);

  // Verify signature if present
  if (message.signature && message.senderIdentityKey) {
    const signatureBytes = base64ToBytes(message.signature);
    const identityKey = base64ToBytes(message.senderIdentityKey);

    if (!verifyEd25519(signatureBytes, ephemeralPublicKey, identityKey)) {
      throw new Error('Invalid signature on sealed message');
    }

    // Verify it matches expected sender if provided
    if (senderIdentityPublicKey && bytesToBase64(identityKey) !== bytesToBase64(senderIdentityPublicKey)) {
      throw new Error('Sender identity mismatch');
    }
  }

  // Perform DH key exchange
  const sharedSecret = x25519SharedSecret(recipientPrivateKey, ephemeralPublicKey);

  // Derive encryption key
  const encryptionKey = hkdfDerive(
    sharedSecret,
    new Uint8Array(32),
    INFO_SEALED_SENDER,
    32
  );

  // Decrypt the message
  const encrypted = {
    ciphertext: base64ToBytes(message.encryptedContent),
    iv: base64ToBytes(message.iv),
    tag: base64ToBytes(message.tag),
  };

  return await decryptAES256GCM(encrypted, encryptionKey);
}
