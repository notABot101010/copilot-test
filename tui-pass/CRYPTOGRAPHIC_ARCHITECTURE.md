# Cryptographic Architecture - TUI Password Manager

## Overview

This document describes the cryptographic architecture used in the TUI Password Manager to secure password vaults. The design follows modern cryptographic best practices and uses well-established, audited algorithms.

**Major Update (v2)**: The vault now encrypts each credential entry individually, ensuring that not all entries are decrypted in memory at the same time. Data serialization uses Protocol Buffers (protobuf) instead of JSON for better efficiency and structure.

**Security Enhancement**: The application no longer stores the master password in memory. Instead, it derives a master key from the password once and immediately zeroizes the password. Only the master key remains in memory for the duration of the session, reducing the attack surface.

## Security Goals

1. **Confidentiality**: Vault contents must remain encrypted and unreadable without the master password
2. **Integrity**: Any tampering with the vault must be detected
3. **Authentication**: Only users with the correct master password can decrypt the vault
4. **Key Derivation**: Master password must be transformed into cryptographic keys using a memory-hard function
5. **Memory Safety**: Individual entries are decrypted on-demand to minimize plaintext exposure in memory
6. **Password Protection**: Master password is never stored in memory - only the derived key is kept

## Cryptographic Components

### 1. Password-Based Key Derivation Function (PBKDF)

**Algorithm**: Argon2id

**Purpose**: Derive cryptographic keys from the user's master password

**Configuration**:
- Memory cost: 64 MB (65536 KiB)
- Time cost: 3 iterations
- Parallelism: 4 threads
- Output length: 32 bytes (256 bits)

**Rationale**: Argon2id is the winner of the Password Hashing Competition and provides the best balance between side-channel resistance (from Argon2i) and GPU/ASIC resistance (from Argon2d). The parameters are chosen to provide strong security while maintaining reasonable performance on modern hardware.

**Salt**: A unique 16-byte random salt is generated for each vault and stored in the vault header. This prevents rainbow table attacks and ensures that the same password produces different keys for different vaults.

### 2. Authenticated Encryption

**Algorithm**: ChaCha20-Poly1305

**Purpose**: Encrypt individual credential entries with authenticated encryption

**Key derivation**: 
- Encryption key: 32-byte key derived from Argon2id output
- Nonce: 12-byte random nonce generated for each credential entry

**Rationale**: ChaCha20-Poly1305 is a modern authenticated encryption with associated data (AEAD) cipher that provides:
- High performance on software implementations
- Constant-time operation (resistant to timing attacks)
- Built-in authentication via Poly1305 MAC
- No known practical attacks

**Individual Entry Encryption**: Each credential is encrypted separately with its own unique nonce. This ensures:
- Only selected credentials are decrypted into memory when needed
- Reduced attack surface by minimizing plaintext exposure
- Per-entry authentication tags detect tampering at granular level

The 12-byte nonce is sufficient for the birthday bound given the expected number of encryptions per credential.

### 3. Key Derivation and Message Authentication

**Algorithm**: BLAKE3

**Purpose**: Additional key derivation and integrity verification

**Usage**:
- Deterministic salt generation from master password
- Domain separation in key derivation contexts
- File integrity checksums

**Rationale**: BLAKE3 is significantly faster than traditional hash functions while providing the same security level. It's designed for parallelism and is resistant to length extension attacks.

### 4. Data Serialization

**Format**: Protocol Buffers (protobuf)

**Purpose**: Efficient, structured serialization of vault data

**Rationale**: Protocol Buffers provide:
- Compact binary representation
- Schema versioning and forward/backward compatibility
- Type safety and validation
- Better performance than text-based formats like JSON

## Vault File Format

The vault file uses a fully protobuf-based format:

```
+-----------------------------------------------------------------------------+
| Protobuf-serialized Vault message (variable length)                        |
| - Contains magic, version, salt, and encrypted entries                     |
+-----------------------------------------------------------------------------+
```

### Protobuf Schema

```protobuf
message Credential {
    string title = 1;
    string username = 2;
    string password = 3;
    string url = 4;
    string notes = 5;
}

message EncryptedEntry {
    bytes nonce = 1;        // 12-byte random nonce
    bytes ciphertext = 2;   // ChaCha20-Poly1305 encrypted credential + auth tag
}

message Vault {
    string magic = 1;                      // "TUIPASS2" identifier
    uint32 version = 2;                    // Format version (currently 2)
    bytes salt = 3;                        // 16-byte random salt for key derivation
    repeated EncryptedEntry entries = 4;   // Encrypted credential entries
}
```

### Field Descriptions

1. **Magic** (string): `TUIPASS2` - Identifies the file as a tui-pass vault (version 2)
2. **Version** (uint32): Format version number (currently 2) for future compatibility
3. **Salt** (bytes): 16-byte unique random salt for Argon2id key derivation
4. **Entries** (repeated EncryptedEntry): Array of encrypted credential entries

## Key Derivation Process

1. User enters master password
2. Random 16-byte salt is generated (on vault creation) or loaded from vault header
3. Argon2id derives a 32-byte master key using:
   - Master password as input
   - Random salt
   - Fixed memory, time, and parallelism parameters
4. **Master password is immediately zeroized from memory**
5. The 32-byte master key is stored in the Vault struct and used for all subsequent encryption/decryption operations
6. The master key is automatically zeroized when the Vault is dropped

```
Vault Creation:
    Master Password → Random Salt (16 bytes)
                              ↓
    Master Password + Salt → Argon2id → Master Key (32 bytes)
                                            ↓
                                  [Password Zeroized]
                                            ↓
    Master Key + Nonce → ChaCha20-Poly1305 → Encrypted Entry

Vault Opening:
    Vault File → Extract Salt (16 bytes)
                       ↓
    Master Password + Salt → Argon2id → Master Key (32 bytes)
                                            ↓
                                  [Password Zeroized]
                                            ↓
    Master Key + Nonce → ChaCha20-Poly1305 → Decrypt Entry (on-demand)
```
    Master Key + Nonce → ChaCha20-Poly1305 → Decrypt Entry (on-demand)
```

## Encryption Process

When creating or updating a vault:

1. Generate a random 16-byte salt (on vault creation) or use existing salt
2. Derive master key from password using Argon2id with the salt
3. For each credential to encrypt:
   a. Serialize credential to protobuf format
   b. Generate a random 12-byte nonce
   c. Encrypt protobuf data using ChaCha20-Poly1305 with master key and nonce
   d. Store nonce and ciphertext in an EncryptedEntry message
4. Create Vault protobuf message containing magic, version, salt, and all EncryptedEntry messages
5. Serialize Vault message to protobuf format
6. Write the serialized protobuf data to file

## Decryption Process

When opening a vault:

1. Read and deserialize protobuf Vault message from file
2. Verify magic number matches "TUIPASS2"
3. Verify version is supported (currently version 2)
4. Extract salt from Vault message
5. Prompt user for master password
6. Derive master key from password using Argon2id with salt
7. Store encrypted entries without decrypting them

**On-Demand Entry Decryption**:
When a specific credential is accessed:
1. Retrieve the EncryptedEntry for that credential
2. Extract the nonce from the entry
3. Decrypt ciphertext using ChaCha20-Poly1305 with master key and nonce
4. Verify authentication tag (automatic in ChaCha20-Poly1305)
5. Deserialize protobuf to Credential object
6. Cache the decrypted credential (optional)

If the password is incorrect or an entry has been tampered with, the authentication tag verification will fail, and decryption will return an error.

## Security Considerations

### Memory Security

- **Password Handling**: Master password is only kept in memory temporarily during vault opening/creation
- **Password Zeroization**: Password is immediately zeroized after deriving the master key
- **Master Key Storage**: Only the derived master key is stored in memory, not the password
- Master keys are stored using the `zeroize` crate and automatically cleared on drop
- All sensitive data structures implement `Zeroize` to clear memory on drop
- Keys are cleared from memory as soon as they're no longer needed
- **Individual Entry Decryption**: Only accessed credentials are decrypted into memory, minimizing plaintext exposure
- **Cache Management**: Decrypted credentials can be cleared from cache when not needed

### Side-Channel Resistance

- All cryptographic operations use constant-time implementations
- Argon2id provides resistance against timing attacks
- ChaCha20-Poly1305 is designed for constant-time execution

### Salt and Nonce Management

- Each vault has a unique random salt generated at creation time and stored in the header
- Each credential entry has its own unique random nonce
- Nonce reuse is prevented by generating a new nonce whenever the vault is saved

### Password Requirements

While the cryptographic scheme is robust, users should still follow password best practices:
- Use long, unique passwords (recommended: 16+ characters)
- Avoid dictionary words and common patterns
- Consider using passphrases for better memorability and security

### Attack Surface Reduction

- **Individual Entry Encryption**: Each credential is independently encrypted, limiting the impact of any single decryption failure
- **On-Demand Decryption**: Credentials are only decrypted when accessed, reducing the window of vulnerability
- **Granular Authentication**: Each entry has its own authentication tag, enabling per-entry integrity verification
- **Master Password Not Stored**: Password is never stored in memory, only the derived master key, reducing password exposure risk

## Threat Model

### Protected Against

1. **Brute force attacks**: Argon2id makes password guessing computationally expensive
2. **Rainbow tables**: Unique random salts prevent precomputed attacks
3. **Tampering**: Poly1305 authentication tag on each entry detects any modifications
4. **Known-plaintext attacks**: Modern AEAD cipher resistant to such attacks
5. **Memory dumps**: Minimized exposure through on-demand decryption and zeroization; password is never in memory after initial key derivation
6. **Selective decryption attacks**: Individual entry encryption prevents full vault exposure
7. **Parallel attacks**: Unique salts prevent attackers from amortizing brute-force costs across multiple vaults
8. **Password extraction from memory**: Master password is immediately zeroized after key derivation

### Not Protected Against

1. **Keyloggers**: Physical or software keyloggers capturing the master password
2. **Memory attacks on running process**: Advanced attacks on live process memory (though reduced exposure helps)
3. **Weak master passwords**: User choice of weak passwords undermines the scheme
4. **Compromised system**: Malware with root/admin access can potentially intercept data

## Advantages of Individual Entry Encryption

1. **Reduced Memory Footprint**: Only accessed credentials occupy memory in plaintext form
2. **Faster Initial Load**: Vault opens without decrypting all entries
3. **Better Security**: Limits exposure if memory is compromised during operation
4. **Scalability**: Performance scales better with large vaults (thousands of credentials)
5. **Granular Access Control**: Potential for future per-entry permissions or auditing

## Future Enhancements

Possible future improvements to consider:

1. **Key stretching on client**: Additional PBKDF2 iterations before Argon2id
2. **Hardware security module support**: Use HSM or TPM for key storage
3. **Multi-factor authentication**: Require additional factors beyond password
4. **Key rotation**: Periodic re-encryption with new keys
5. **Backup codes**: Recovery mechanism for forgotten passwords (with security trade-offs)
6. **Entry-level metadata**: Add timestamps, access counts without decryption

## References

1. Argon2: https://github.com/P-H-C/phc-winner-argon2
2. ChaCha20-Poly1305: RFC 8439
3. BLAKE3: https://github.com/BLAKE3-team/BLAKE3-specs
4. Protocol Buffers: https://protobuf.dev/
5. OWASP Password Storage Cheat Sheet
