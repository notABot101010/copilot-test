# Building Duplex AEAD from TurboSHAKE256 Core

## Overview

This document explains how to construct an AEAD (Authenticated Encryption with Associated Data) scheme using the duplex construction, starting from TurboSHAKE256's core primitives.

## Core Concepts

### Sponge vs Duplex

**Sponge Construction (TurboSHAKE256)**:
```
absorb(data1) → permute → absorb(data2) → permute → ... → squeeze(output)
```
- One-way: absorb all input, then squeeze output
- Cannot go back to absorbing after squeezing starts
- Good for hashing

**Duplex Construction (AEAD)**:
```
absorb(data) → permute → squeeze(keystream) → absorb(ciphertext) → permute → ...
```
- Two-way: can interleave absorption and squeezing
- Allows authenticated encryption patterns
- State depends on both input and output

## Key Design Decisions

### 1. Initialization Phase

```rust
// Absorb key
absorb(key) → permute

// Absorb nonce
absorb(nonce) → permute
```

**Why separate permutations?**
- Ensures key and nonce each get their own permutation
- Prevents length-extension attacks
- Provides good domain separation

### 2. Associated Data Phase

```rust
for each AD block:
    absorb(ad_block) → permute

// Domain separation
absorb(DOMAIN_SEP_AD) → permute
```

**Why domain separation?**
- Prevents confusion between AD and message data
- Ensures AD=empty vs AD=missing produce different states
- Critical for security: `Encrypt(M, AD="") ≠ Encrypt(M, no AD)`

### 3. Encryption Phase - The Duplex Pattern

This is where the magic happens:

```rust
for each plaintext block:
    1. keystream = squeeze()          // Get keystream from CURRENT state
    2. ciphertext = plaintext ⊕ keystream
    3. absorb(ciphertext)             // Absorb result back
    4. permute()                      // Mix for next iteration
```

**Why absorb ciphertext (not plaintext)?**
- The ciphertext is what will be transmitted
- Absorbing it authenticates what the receiver will see
- Prevents attackers from modifying ciphertext without detection
- Creates binding: `state[i+1] = f(state[i], ciphertext[i])`

**Why this order matters:**
1. **Squeeze first**: Keystream must come from state that doesn't include current plaintext
2. **Absorb after encrypt**: Ciphertext depends on keystream, then gets absorbed
3. **Permute last**: Mixes ciphertext into state for next block

### 4. Decryption Phase - Mirror Pattern

```rust
for each ciphertext block:
    1. keystream = squeeze()          // Get keystream (must match encryption)
    2. absorb(ciphertext)             // Absorb BEFORE decrypting
    3. permute()                      // Same state evolution as encryption
    4. plaintext = ciphertext ⊕ keystream
```

**Critical insight**: Order is different from encryption!
- Encryption: squeeze → XOR → absorb → permute
- Decryption: squeeze → absorb → permute → XOR

**Why?** To maintain the same state evolution:
- Both paths must have: `absorb(ciphertext)` before `permute()`
- This ensures the authentication tag will match
- The XOR (which produces plaintext) doesn't affect state

### 5. Tag Generation

```rust
tag = squeeze(TAG_SIZE)
```

**Why this works:**
- State has absorbed: key, nonce, AD, all ciphertext blocks
- Any modification to ciphertext changes the state
- Tag verification automatically fails if anything was tampered with

## Comparison: TurboSHAKE256 vs TurboShakeAead

| Feature | TurboSHAKE256 | TurboShakeAead |
|---------|---------------|----------------|
| **Pattern** | Absorb-all → Squeeze | Interleaved absorb/squeeze |
| **Use case** | Hashing | AEAD |
| **Rate** | 136 bytes | 136 bytes (same) |
| **Permutation** | Keccak-p[1600,12] | Keccak-p[1600,12] (same) |
| **Domain sep** | 0x1F (single) | 0x01 (AD), 0x02 (message) |
| **State tracking** | position only | position (unused currently) |
| **Finalization** | Pad + permute | Just squeeze (already mixed) |

## Security Properties

### What TurboShakeAead Provides

1. **Confidentiality**: Plaintext is XORed with unpredictable keystream
2. **Authenticity**: Tag depends on all absorbed data (key, nonce, AD, ciphertext)
3. **Integrity**: Any ciphertext modification changes state → wrong tag
4. **Replay protection**: Nonce must be unique per message
5. **AD authentication**: Associated data is authenticated but not encrypted

### Security Level

- **Confidentiality**: ~128 bits (capacity = 64 bytes = 512 bits, but 12 rounds)
- **Authentication**: 256 bits (tag size = 32 bytes)
- **Collision resistance**: Reduced due to 12 rounds vs 24

⚠️ **Note**: 12 rounds provides less security margin than full Keccak (24 rounds). TurboSHAKE is designed for performance in non-adversarial contexts. For high-security applications, consider using full-round Keccak.

## Implementation Highlights

### From TurboSHAKE256 Core

We reuse these primitives:
```rust
state_to_bytes()     // Convert state to byte array
bytes_to_state()     // Convert byte array to state
keccak::p1600(state, 12)  // 12-round permutation
```

### Key Differences

**TurboSHAKE256**:
```rust
pub struct TurboShake256 {
    state: [u64; 25],
    position: usize,      // Track position in rate
    absorbing: bool,      // Track phase transition
}
```

**TurboShakeAead**:
```rust
pub struct TurboShakeAead {
    state: [u64; 25],
    position: usize,      // Not actively used (full blocks)
}
```

Why simpler?
- AEAD processes in full blocks more naturally
- No need to track absorbing/squeezing phase
- Each operation is explicit (absorb_block, squeeze_block)

## Example Usage

```rust
// Encryption
let key = [0x42u8; 32];
let nonce = [0x13u8; 16];
let plaintext = b"Secret message";
let ad = b"Public metadata";

let mut enc = TurboShakeAead::new(&key, &nonce)?;
let ciphertext = enc.encrypt(plaintext, ad);

// Decryption
let mut dec = TurboShakeAead::new(&key, &nonce)?;
let recovered = dec.decrypt(&ciphertext, ad)?;
assert_eq!(recovered, plaintext);
```

## Performance Characteristics

### Per-Block Cost

Each plaintext/ciphertext block requires:
1. One `squeeze_block()` - O(rate) XOR operations
2. One `absorb_block()` - O(rate) XOR operations
3. One `keccak::p1600()` - 12 rounds of Keccak-f

**Comparison to TurboSHAKE256 hash**:
- Hashing: 1 permutation per rate block
- AEAD encryption: 1 permutation per rate block (same!)
- AEAD decryption: 1 permutation per rate block (same!)

The duplex construction doesn't add permutation overhead - we get authentication "for free" by reusing the permutations we'd need anyway.

### Versus 24-Round AEAD

TurboShakeAead with 12 rounds should be ~2x faster than full-round Keccak AEAD:
- 12 rounds vs 24 rounds per block
- Same number of blocks to process
- All other operations (XOR, absorb) are identical

## Common Pitfalls

### ❌ Wrong: Absorb plaintext instead of ciphertext

```rust
// DON'T DO THIS
ciphertext = plaintext ⊕ keystream
absorb(plaintext)  // WRONG - authenticates plaintext, not what's sent
```

**Problem**: Attacker can modify ciphertext without changing plaintext in state.

### ❌ Wrong: Decrypt before absorbing

```rust
// DON'T DO THIS
keystream = squeeze()
plaintext = ciphertext ⊕ keystream  // Decrypt first
absorb(ciphertext)                  // Then absorb
permute()
```

**Problem**: Works but violates the pattern - harder to reason about security.

### ❌ Wrong: No domain separation

```rust
// DON'T DO THIS
// Just absorb AD, then immediately absorb message
absorb(ad)
permute()
absorb(message)  // No separator
```

**Problem**: `Encrypt("AB", "")` might equal `Encrypt("A", "B")`.

## Conclusion

The duplex construction is elegant:
- Start with sponge primitives (absorb, squeeze, permute)
- Allow interleaving absorption and squeezing
- Use the pattern: squeeze → encrypt → absorb → permute
- Authentication comes from absorbing ciphertext back into state

This gives you AEAD with minimal overhead beyond basic hashing, using the exact same permutation (Keccak-p[1600,12]) as TurboSHAKE256.
