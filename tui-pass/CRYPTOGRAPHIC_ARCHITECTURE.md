# Cryptographic Architecture - TUI Password Manager

## Overview

This document describes the cryptographic architecture used in the TUI Password Manager to secure password vaults. The design follows modern cryptographic best practices and uses well-established, audited algorithms.

## Security Goals

1. **Confidentiality**: Vault contents must remain encrypted and unreadable without the master password
2. **Integrity**: Any tampering with the vault must be detected
3. **Authentication**: Only users with the correct master password can decrypt the vault
4. **Key Derivation**: Master password must be transformed into cryptographic keys using a memory-hard function

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

**Salt**: A unique 16-byte random salt is generated for each vault and stored alongside the encrypted data. This prevents rainbow table attacks and ensures that the same password produces different keys for different vaults.

### 2. Authenticated Encryption

**Algorithm**: ChaCha20-Poly1305

**Purpose**: Encrypt vault data with authenticated encryption

**Key derivation**: 
- Encryption key: 32-byte key derived from Argon2id output
- Nonce: 12-byte random nonce generated for each encryption operation

**Rationale**: ChaCha20-Poly1305 is a modern authenticated encryption with associated data (AEAD) cipher that provides:
- High performance on software implementations
- Constant-time operation (resistant to timing attacks)
- Built-in authentication via Poly1305 MAC
- No known practical attacks

The 12-byte nonce is sufficient for the birthday bound given the expected number of encryptions per vault.

### 3. Key Derivation and Message Authentication

**Algorithm**: BLAKE3

**Purpose**: Additional key derivation and integrity verification

**Usage**:
- Domain separation in key derivation contexts
- File integrity checksums
- Derive multiple keys from the master key

**Rationale**: BLAKE3 is significantly faster than traditional hash functions while providing the same security level. It's designed for parallelism and is resistant to length extension attacks.

## Vault File Format

The vault file is a binary format with the following structure:

```
+------------------+------------------+------------------+------------------+
| Magic Number     | Version          | Argon2 Salt      | ChaCha20 Nonce   |
| (8 bytes)        | (4 bytes)        | (16 bytes)       | (12 bytes)       |
+------------------+------------------+------------------+------------------+
| Argon2 Memory    | Argon2 Iterations| Argon2 Parallel  | Reserved         |
| (4 bytes)        | (4 bytes)        | (4 bytes)        | (4 bytes)        |
+------------------+------------------+------------------+------------------+
| Encrypted Data Length               | Encrypted Data + Auth Tag           |
| (8 bytes)                           | (variable length)                   |
+-------------------------------------+-------------------------------------+
```

### Field Descriptions

1. **Magic Number** (8 bytes): `TUIPASS1` - Identifies the file as a tui-pass vault
2. **Version** (4 bytes): Format version number (currently 1) for future compatibility
3. **Argon2 Salt** (16 bytes): Random salt for Argon2id key derivation
4. **ChaCha20 Nonce** (12 bytes): Random nonce for ChaCha20-Poly1305 encryption
5. **Argon2 Memory** (4 bytes): Memory cost in KiB (default: 65536)
6. **Argon2 Iterations** (4 bytes): Time cost (default: 3)
7. **Argon2 Parallel** (4 bytes): Parallelism degree (default: 4)
8. **Reserved** (4 bytes): Reserved for future use (set to 0)
9. **Encrypted Data Length** (8 bytes): Length of the encrypted payload
10. **Encrypted Data + Auth Tag**: ChaCha20-Poly1305 ciphertext with 16-byte Poly1305 authentication tag appended

## Key Derivation Process

1. User enters master password
2. Argon2id derives a 32-byte master key using:
   - Master password as input
   - Random salt from vault header
   - Memory, time, and parallelism parameters from vault header
3. The 32-byte master key is used directly as the ChaCha20-Poly1305 encryption key

```
Master Password + Salt → Argon2id → Master Key (32 bytes) → ChaCha20-Poly1305
```

## Encryption Process

When creating or updating a vault:

1. Generate a random 16-byte salt (on vault creation)
2. Derive master key from password using Argon2id
3. Serialize credentials to JSON format
4. Generate a random 12-byte nonce
5. Encrypt JSON data using ChaCha20-Poly1305 with master key and nonce
6. Write vault header and encrypted data to file

## Decryption Process

When opening a vault:

1. Read vault header to extract salt, nonce, and Argon2 parameters
2. Prompt user for master password
3. Derive master key using Argon2id with stored parameters
4. Decrypt data using ChaCha20-Poly1305 with derived key and stored nonce
5. Verify authentication tag (automatic in ChaCha20-Poly1305)
6. Deserialize JSON to credential objects

If the password is incorrect or the vault has been tampered with, the authentication tag verification will fail, and decryption will return an error.

## Security Considerations

### Memory Security

- Master keys and passwords are stored in memory using the `zeroize` crate
- All sensitive data structures implement `Zeroize` to clear memory on drop
- Keys are cleared from memory as soon as they're no longer needed

### Side-Channel Resistance

- All cryptographic operations use constant-time implementations
- Argon2id provides resistance against timing attacks
- ChaCha20-Poly1305 is designed for constant-time execution

### Salt and Nonce Management

- Each vault has a unique random salt generated at creation time
- Each encryption operation uses a fresh random nonce
- Nonce reuse is prevented by generating a new nonce whenever the vault is saved

### Password Requirements

While the cryptographic scheme is robust, users should still follow password best practices:
- Use long, unique passwords (recommended: 16+ characters)
- Avoid dictionary words and common patterns
- Consider using passphrases for better memorability and security

## Threat Model

### Protected Against

1. **Brute force attacks**: Argon2id makes password guessing computationally expensive
2. **Rainbow tables**: Unique salts prevent precomputed attacks
3. **Tampering**: Poly1305 authentication tag detects any modifications
4. **Known-plaintext attacks**: Modern AEAD cipher resistant to such attacks
5. **Memory dumps**: Zeroization of sensitive data in memory

### Not Protected Against

1. **Keyloggers**: Physical or software keyloggers capturing the master password
2. **Memory attacks on running process**: Advanced attacks on live process memory
3. **Weak master passwords**: User choice of weak passwords undermines the scheme
4. **Compromised system**: Malware with root/admin access can potentially intercept data

## Future Enhancements

Possible future improvements to consider:

1. **Key stretching on client**: Additional PBKDF2 iterations before Argon2id
2. **Hardware security module support**: Use HSM or TPM for key storage
3. **Multi-factor authentication**: Require additional factors beyond password
4. **Key rotation**: Periodic re-encryption with new keys
5. **Backup codes**: Recovery mechanism for forgotten passwords (with security trade-offs)

## References

1. Argon2: https://github.com/P-H-C/phc-winner-argon2
2. ChaCha20-Poly1305: RFC 8439
3. BLAKE3: https://github.com/BLAKE3-team/BLAKE3-specs
4. OWASP Password Storage Cheat Sheet
