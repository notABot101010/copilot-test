import { UserKeyMaterial, loadRatchetState, saveRatchetState } from '../crypto/storage';
import {
  initializeRatchetSender,
  initializeRatchetReceiver,
  ratchetEncrypt,
  ratchetDecrypt,
  RatchetState,
  EncryptedMessage,
} from '../crypto/ratchet';
import { generateX25519KeyPair, x25519SharedSecret, bytesToBase64, base64ToBytes } from '../crypto/utils';
import { getUserKeys } from '../api/client';

export class ChatService {
  private username: string;
  private keys: UserKeyMaterial;
  private ratchetStates: Map<string, RatchetState>;

  constructor(username: string, keys: UserKeyMaterial) {
    this.username = username;
    this.keys = keys;
    this.ratchetStates = new Map();
  }

  private async getRatchetState(peerUsername: string): Promise<RatchetState> {
    if (this.ratchetStates.has(peerUsername)) {
      return this.ratchetStates.get(peerUsername)!;
    }

    let state = loadRatchetState(this.username, peerUsername);

    if (!state) {
      // Initialize new ratchet state
      const peerKeys = await getUserKeys(peerUsername);
      const peerPublicKey = base64ToBytes(peerKeys.identity_public_key);

      // For simplicity, using identity keys for initial exchange
      // In production, should use prekeys
      const sharedSecret = new Uint8Array(32); // Simplified for MVP
      const ourKeyPair = generateX25519KeyPair();

      // Convert Ed25519 to X25519 (simplified - should use proper conversion)
      const peerX25519 = peerPublicKey.slice(0, 32);

      state = initializeRatchetSender(sharedSecret, ourKeyPair, peerX25519);
    }

    this.ratchetStates.set(peerUsername, state);
    return state;
  }

  async encryptMessage(
    recipientUsername: string,
    plaintext: Uint8Array
  ): Promise<EncryptedMessage> {
    const state = await this.getRatchetState(recipientUsername);
    const { encrypted, newState } = await ratchetEncrypt(state, plaintext);

    this.ratchetStates.set(recipientUsername, newState);
    saveRatchetState(this.username, recipientUsername, newState);

    return encrypted;
  }

  async decryptMessage(
    senderUsername: string,
    message: EncryptedMessage
  ): Promise<Uint8Array> {
    const state = await this.getRatchetState(senderUsername);
    const { plaintext, newState } = await ratchetDecrypt(state, message);

    this.ratchetStates.set(senderUsername, newState);
    saveRatchetState(this.username, senderUsername, newState);

    return plaintext;
  }
}
