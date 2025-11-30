declare module '@noble/curves/ed25519.js' {
  export const ed25519: {
    utils: {
      randomPrivateKey(): Uint8Array;
    };
    getPublicKey(privateKey: Uint8Array): Uint8Array;
    sign(message: Uint8Array, privateKey: Uint8Array): Uint8Array;
    verify(signature: Uint8Array, message: Uint8Array, publicKey: Uint8Array): boolean;
  };

  export const x25519: {
    utils: {
      randomPrivateKey(): Uint8Array;
    };
    getPublicKey(privateKey: Uint8Array): Uint8Array;
    getSharedSecret(privateKey: Uint8Array, publicKey: Uint8Array): Uint8Array;
  };
}

declare module '@noble/hashes/hkdf.js' {
  export function hkdf(
    hash: any,
    ikm: Uint8Array,
    salt: Uint8Array | undefined,
    info: Uint8Array,
    length: number
  ): Uint8Array;
}

declare module '@noble/hashes/sha2.js' {
  export function sha256(data: Uint8Array): Uint8Array;
}

declare module '@noble/hashes/sha512.js' {
  export function sha512(data: Uint8Array): Uint8Array;
}
