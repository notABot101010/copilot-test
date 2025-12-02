//! TurboShake-based AEAD using duplex construction with 12-round Keccak
//!
//! This implements an AEAD (Authenticated Encryption with Associated Data) scheme
//! based on TurboSHAKE256's core primitives using the duplex construction.

use core::mem;

// Compile-time check for little-endian architecture
// Keccak state interpretation assumes little-endian byte order
#[cfg(not(target_endian = "little"))]
compile_error!("This crate requires a little-endian architecture");

/// State size in bytes (1600 bits)
const STATE_SIZE: usize = 200;

/// Rate in bytes (same as TurboSHAKE256)
const RATE: usize = 136;

/// Tag size in bytes
const TAG_SIZE: usize = 32;

/// Get mutable reference to state as bytes using transmute (zero-copy)
///
/// # Safety
/// This is safe because:
/// - [u64; 25] and [u8; 200] have the same size (200 bytes)
/// - We require little-endian architecture (checked at compile time)
/// - The byte layout matches Keccak's expected lane ordering
#[inline(always)]
fn state_as_bytes_mut(state: &mut [u64; 25]) -> &mut [u8; STATE_SIZE] {
    // SAFETY: Size and alignment are compatible, and we enforce little-endian at compile time
    unsafe { mem::transmute(state) }
}

/// Get reference to state as bytes using transmute (zero-copy)
///
/// # Safety
/// This is safe because:
/// - [u64; 25] and [u8; 200] have the same size (200 bytes)
/// - We require little-endian architecture (checked at compile time)
/// - The byte layout matches Keccak's expected lane ordering
#[inline(always)]
fn state_as_bytes(state: &[u64; 25]) -> &[u8; STATE_SIZE] {
    // SAFETY: Size and alignment are compatible, and we enforce little-endian at compile time
    unsafe { mem::transmute(state) }
}

/// Error types for AEAD operations
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AeadError {
    /// Authentication tag verification failed
    AuthenticationFailed,
    /// Invalid key size
    InvalidKeySize,
    /// Invalid nonce size
    InvalidNonceSize,
}

/// TurboShake-based AEAD using duplex construction
///
/// This implements the duplex construction where we can interleave
/// absorption and squeezing, unlike the standard sponge which is
/// absorb-all-then-squeeze.
///
/// The duplex pattern for encryption is:
/// 1. Initialize: absorb(key), permute, absorb(nonce), permute
/// 2. AD phase: for each block: absorb(ad), permute
/// 3. Encryption: for each block: squeeze(keystream), absorb(ciphertext), permute
/// 4. Finalization: squeeze(tag)
#[derive(Debug, PartialEq)]
pub struct TurboShakeAead {
    state: [u64; 25],
}

impl TurboShakeAead {
    const DOMAIN_SEP_AD: u8 = 0x01;
    const DOMAIN_SEP_MSG: u8 = 0x02;

    /// Create a new AEAD instance
    ///
    /// # Arguments
    /// * `key` - 32-byte key
    /// * `nonce` - 16-byte nonce (must be unique per encryption)
    pub fn new(key: &[u8], nonce: &[u8]) -> Result<Self, AeadError> {
        if key.len() != 32 {
            return Err(AeadError::InvalidKeySize);
        }
        if nonce.len() != 16 {
            return Err(AeadError::InvalidNonceSize);
        }

        let mut aead = Self { state: [0u64; 25] };

        // Initialize with key
        aead.absorb_block(key);
        keccak::p1600(&mut aead.state, 12);

        // Absorb nonce
        aead.absorb_block(nonce);
        keccak::p1600(&mut aead.state, 12);

        Ok(aead)
    }

    /// Absorb a block of data (up to rate bytes) - zero-copy version
    #[inline(always)]
    fn absorb_block(&mut self, data: &[u8]) {
        let state_bytes = state_as_bytes_mut(&mut self.state);
        let len = data.len().min(RATE);
        for i in 0..len {
            state_bytes[i] ^= data[i];
        }
    }

    /// Squeeze a block of data (up to rate bytes) - zero-copy version
    #[inline(always)]
    fn squeeze_block(&self, output: &mut [u8]) {
        let state_bytes = state_as_bytes(&self.state);
        let len = output.len().min(RATE);
        output[..len].copy_from_slice(&state_bytes[..len]);
    }

    /// Process associated data
    fn process_ad(&mut self, ad: &[u8]) {
        if ad.is_empty() {
            return;
        }

        // Process full blocks
        for chunk in ad.chunks(RATE) {
            self.absorb_block(chunk);
            keccak::p1600(&mut self.state, 12);
        }

        // Domain separation after AD
        let state_bytes = state_as_bytes_mut(&mut self.state);
        state_bytes[0] ^= Self::DOMAIN_SEP_AD;
        keccak::p1600(&mut self.state, 12);
    }

    /// Encrypt plaintext
    ///
    /// The duplex encryption pattern per block:
    /// 1. Squeeze keystream from current state
    /// 2. XOR plaintext with keystream → ciphertext
    /// 3. Absorb ciphertext back into state
    /// 4. Apply permutation
    pub fn encrypt(&mut self, plaintext: &[u8], ad: &[u8]) -> Vec<u8> {
        // Process associated data first
        self.process_ad(ad);

        let mut ciphertext = Vec::with_capacity(plaintext.len() + TAG_SIZE);

        // Domain separation for message phase
        let state_bytes = state_as_bytes_mut(&mut self.state);
        state_bytes[0] ^= Self::DOMAIN_SEP_MSG;
        keccak::p1600(&mut self.state, 12);

        // Encrypt each block using duplex pattern
        for chunk in plaintext.chunks(RATE) {
            let chunk_len = chunk.len();

            // 1. Squeeze keystream (use stack array to avoid allocation)
            let mut keystream = [0u8; RATE];
            self.squeeze_block(&mut keystream[..chunk_len]);

            // 2. XOR to produce ciphertext and append to result
            let start = ciphertext.len();
            ciphertext.extend(chunk.iter().zip(keystream.iter()).map(|(&p, &k)| p ^ k));

            // 3. Absorb ciphertext (authenticates the ciphertext)
            self.absorb_block(&ciphertext[start..]);

            // 4. Permute
            keccak::p1600(&mut self.state, 12);
        }

        // Generate authentication tag
        let mut tag = [0u8; TAG_SIZE];
        self.squeeze_block(&mut tag);
        ciphertext.extend_from_slice(&tag);

        ciphertext
    }

    /// Encrypt plaintext in-place
    ///
    /// This version modifies the plaintext buffer directly and appends the tag.
    /// More efficient when you already have a mutable buffer.
    pub fn encrypt_in_place(&mut self, buffer: &mut Vec<u8>, ad: &[u8]) {
        // Process associated data first
        self.process_ad(ad);

        // Domain separation for message phase
        let state_bytes = state_as_bytes_mut(&mut self.state);
        state_bytes[0] ^= Self::DOMAIN_SEP_MSG;
        keccak::p1600(&mut self.state, 12);

        // Encrypt each block using duplex pattern
        let plaintext_len = buffer.len();
        let mut offset = 0;

        while offset < plaintext_len {
            let chunk_len = (plaintext_len - offset).min(RATE);

            // 1. Squeeze keystream
            let mut keystream = [0u8; RATE];
            self.squeeze_block(&mut keystream[..chunk_len]);

            // 2. XOR in-place to produce ciphertext
            for i in 0..chunk_len {
                buffer[offset + i] ^= keystream[i];
            }

            // 3. Absorb ciphertext
            self.absorb_block(&buffer[offset..offset + chunk_len]);

            // 4. Permute
            keccak::p1600(&mut self.state, 12);

            offset += chunk_len;
        }

        // Generate and append authentication tag
        let mut tag = [0u8; TAG_SIZE];
        self.squeeze_block(&mut tag);
        buffer.extend_from_slice(&tag);
    }

    /// Decrypt ciphertext
    ///
    /// The duplex decryption pattern per block:
    /// 1. Squeeze keystream from current state
    /// 2. Absorb ciphertext into state (before decrypting!)
    /// 3. Apply permutation
    /// 4. XOR ciphertext with keystream → plaintext
    pub fn decrypt(&mut self, ciphertext: &[u8], ad: &[u8]) -> Result<Vec<u8>, AeadError> {
        if ciphertext.len() < TAG_SIZE {
            return Err(AeadError::AuthenticationFailed);
        }

        let ct_len = ciphertext.len() - TAG_SIZE;
        let (ct_data, received_tag) = ciphertext.split_at(ct_len);

        // Process associated data (must match encryption)
        self.process_ad(ad);

        let mut plaintext = Vec::with_capacity(ct_len);

        // Domain separation for message phase
        let state_bytes = state_as_bytes_mut(&mut self.state);
        state_bytes[0] ^= Self::DOMAIN_SEP_MSG;
        keccak::p1600(&mut self.state, 12);

        // Decrypt each block using duplex pattern
        for chunk in ct_data.chunks(RATE) {
            let chunk_len = chunk.len();

            // 1. Squeeze keystream (use stack array to avoid allocation)
            let mut keystream = [0u8; RATE];
            self.squeeze_block(&mut keystream[..chunk_len]);

            // 2. Absorb ciphertext (BEFORE decrypting)
            self.absorb_block(chunk);

            // 3. Permute
            keccak::p1600(&mut self.state, 12);

            // 4. XOR to recover plaintext
            plaintext.extend(chunk.iter().zip(keystream.iter()).map(|(&c, &k)| c ^ k));
        }

        // Verify authentication tag
        let mut computed_tag = [0u8; TAG_SIZE];
        self.squeeze_block(&mut computed_tag);

        // Constant-time comparison
        if !constant_time_compare(&computed_tag, received_tag) {
            return Err(AeadError::AuthenticationFailed);
        }

        Ok(plaintext)
    }

    /// Decrypt ciphertext in-place
    ///
    /// This version modifies the ciphertext buffer directly.
    /// Returns the length of the plaintext (buffer is truncated to remove tag).
    ///
    /// Note: This uses a two-pass approach (verify-then-decrypt) for security:
    /// 1. First pass: verify the authentication tag before any decryption
    /// 2. Second pass: decrypt the data only if verification succeeds
    ///
    /// A single-pass approach could leak plaintext on authentication failure,
    /// which violates the security guarantees of authenticated encryption.
    /// The two-pass approach ensures no plaintext is revealed if the tag is invalid.
    pub fn decrypt_in_place(&mut self, buffer: &mut Vec<u8>, ad: &[u8]) -> Result<(), AeadError> {
        if buffer.len() < TAG_SIZE {
            return Err(AeadError::AuthenticationFailed);
        }

        let ct_len = buffer.len() - TAG_SIZE;

        // Process associated data (must match encryption)
        self.process_ad(ad);

        // Domain separation for message phase
        let state_bytes = state_as_bytes_mut(&mut self.state);
        state_bytes[0] ^= Self::DOMAIN_SEP_MSG;
        keccak::p1600(&mut self.state, 12);

        // First pass: absorb all ciphertext to verify tag
        // We need to clone state to verify before decrypting
        let mut verify_state = self.state;
        let mut offset = 0;

        while offset < ct_len {
            let chunk_len = (ct_len - offset).min(RATE);
            let chunk = &buffer[offset..offset + chunk_len];

            // Absorb ciphertext
            let state_bytes = state_as_bytes_mut(&mut verify_state);
            for i in 0..chunk_len {
                state_bytes[i] ^= chunk[i];
            }
            keccak::p1600(&mut verify_state, 12);

            offset += chunk_len;
        }

        // Verify tag
        let verify_bytes = state_as_bytes(&verify_state);
        let received_tag = &buffer[ct_len..];
        if !constant_time_compare(&verify_bytes[..TAG_SIZE], received_tag) {
            return Err(AeadError::AuthenticationFailed);
        }

        // Second pass: decrypt (only reached if tag verification succeeded)
        offset = 0;
        while offset < ct_len {
            let chunk_len = (ct_len - offset).min(RATE);

            // 1. Squeeze keystream (extract bytes from sponge state)
            // This is equivalent to squeeze_block but inlined for efficiency
            let mut keystream = [0u8; RATE];
            let state_bytes = state_as_bytes(&self.state);
            keystream[..chunk_len].copy_from_slice(&state_bytes[..chunk_len]);

            // 2. Absorb ciphertext (BEFORE decrypting)
            let state_bytes = state_as_bytes_mut(&mut self.state);
            for i in 0..chunk_len {
                state_bytes[i] ^= buffer[offset + i];
            }

            // 3. Permute
            keccak::p1600(&mut self.state, 12);

            // 4. XOR in-place to recover plaintext
            for i in 0..chunk_len {
                buffer[offset + i] ^= keystream[i];
            }

            offset += chunk_len;
        }

        // Truncate to remove tag
        buffer.truncate(ct_len);

        Ok(())
    }
}

/// Constant-time comparison to prevent timing attacks
#[inline(always)]
fn constant_time_compare(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }

    result == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_empty() {
        let key = [0x42u8; 32];
        let nonce = [0x13u8; 16];
        let plaintext = b"";
        let ad = b"";

        let mut enc = TurboShakeAead::new(&key, &nonce).unwrap();
        let ciphertext = enc.encrypt(plaintext, ad);

        let mut dec = TurboShakeAead::new(&key, &nonce).unwrap();
        let decrypted = dec.decrypt(&ciphertext, ad).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_decrypt_basic() {
        let key = [0x42u8; 32];
        let nonce = [0x13u8; 16];
        let plaintext = b"Hello, TurboSHAKE AEAD!";
        let ad = b"";

        let mut enc = TurboShakeAead::new(&key, &nonce).unwrap();
        let ciphertext = enc.encrypt(plaintext, ad);

        // Verify ciphertext is different from plaintext
        assert_ne!(&ciphertext[..plaintext.len()], plaintext);

        // Verify tag is appended
        assert_eq!(ciphertext.len(), plaintext.len() + TAG_SIZE);

        let mut dec = TurboShakeAead::new(&key, &nonce).unwrap();
        let decrypted = dec.decrypt(&ciphertext, ad).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_with_associated_data() {
        let key = [0x01u8; 32];
        let nonce = [0x02u8; 16];
        let plaintext = b"Secret message";
        let ad = b"Public metadata that must be authenticated";

        let mut enc = TurboShakeAead::new(&key, &nonce).unwrap();
        let ciphertext = enc.encrypt(plaintext, ad);

        let mut dec = TurboShakeAead::new(&key, &nonce).unwrap();
        let decrypted = dec.decrypt(&ciphertext, ad).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_wrong_ad_fails() {
        let key = [0x01u8; 32];
        let nonce = [0x02u8; 16];
        let plaintext = b"Secret";
        let ad = b"Correct AD";

        let mut enc = TurboShakeAead::new(&key, &nonce).unwrap();
        let ciphertext = enc.encrypt(plaintext, ad);

        let mut dec = TurboShakeAead::new(&key, &nonce).unwrap();
        let result = dec.decrypt(&ciphertext, b"Wrong AD");

        assert_eq!(result, Err(AeadError::AuthenticationFailed));
    }

    #[test]
    fn test_modified_ciphertext_fails() {
        let key = [0x01u8; 32];
        let nonce = [0x02u8; 16];
        let plaintext = b"Secret message";
        let ad = b"";

        let mut enc = TurboShakeAead::new(&key, &nonce).unwrap();
        let mut ciphertext = enc.encrypt(plaintext, ad);

        // Flip a bit in the ciphertext
        ciphertext[0] ^= 1;

        let mut dec = TurboShakeAead::new(&key, &nonce).unwrap();
        let result = dec.decrypt(&ciphertext, ad);

        assert_eq!(result, Err(AeadError::AuthenticationFailed));
    }

    #[test]
    fn test_large_message() {
        let key = [0xAAu8; 32];
        let nonce = [0x55u8; 16];
        let plaintext = vec![0x42u8; 1000];
        let ad = b"metadata";

        let mut enc = TurboShakeAead::new(&key, &nonce).unwrap();
        let ciphertext = enc.encrypt(&plaintext, ad);

        let mut dec = TurboShakeAead::new(&key, &nonce).unwrap();
        let decrypted = dec.decrypt(&ciphertext, ad).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_different_keys_different_output() {
        let nonce = [0x02u8; 16];
        let plaintext = b"Same plaintext";
        let ad = b"";

        let mut enc1 = TurboShakeAead::new(&[0x01u8; 32], &nonce).unwrap();
        let ct1 = enc1.encrypt(plaintext, ad);

        let mut enc2 = TurboShakeAead::new(&[0x02u8; 32], &nonce).unwrap();
        let ct2 = enc2.encrypt(plaintext, ad);

        assert_ne!(ct1, ct2);
    }

    #[test]
    fn test_different_nonces_different_output() {
        let key = [0x01u8; 32];
        let plaintext = b"Same plaintext";
        let ad = b"";

        let mut enc1 = TurboShakeAead::new(&key, &[0x01u8; 16]).unwrap();
        let ct1 = enc1.encrypt(plaintext, ad);

        let mut enc2 = TurboShakeAead::new(&key, &[0x02u8; 16]).unwrap();
        let ct2 = enc2.encrypt(plaintext, ad);

        assert_ne!(ct1, ct2);
    }

    #[test]
    fn test_deterministic() {
        let key = [0xFFu8; 32];
        let nonce = [0x00u8; 16];
        let plaintext = b"Deterministic test";
        let ad = b"metadata";

        let mut enc1 = TurboShakeAead::new(&key, &nonce).unwrap();
        let ct1 = enc1.encrypt(plaintext, ad);

        let mut enc2 = TurboShakeAead::new(&key, &nonce).unwrap();
        let ct2 = enc2.encrypt(plaintext, ad);

        assert_eq!(ct1, ct2);
    }

    #[test]
    fn test_invalid_key_size() {
        let key = [0x00u8; 16]; // Wrong size
        let nonce = [0x00u8; 16];
        let result = TurboShakeAead::new(&key, &nonce);
        assert_eq!(result, Err(AeadError::InvalidKeySize));
    }

    #[test]
    fn test_invalid_nonce_size() {
        let key = [0x00u8; 32];
        let nonce = [0x00u8; 8]; // Wrong size
        let result = TurboShakeAead::new(&key, &nonce);
        assert_eq!(result, Err(AeadError::InvalidNonceSize));
    }

    #[test]
    fn test_encrypt_in_place() {
        let key = [0x42u8; 32];
        let nonce = [0x13u8; 16];
        let plaintext = b"Hello, in-place encryption!";
        let ad = b"test ad";

        // Standard encryption
        let mut enc1 = TurboShakeAead::new(&key, &nonce).unwrap();
        let ciphertext = enc1.encrypt(plaintext, ad);

        // In-place encryption
        let mut buffer = plaintext.to_vec();
        let mut enc2 = TurboShakeAead::new(&key, &nonce).unwrap();
        enc2.encrypt_in_place(&mut buffer, ad);

        assert_eq!(buffer, ciphertext);
    }

    #[test]
    fn test_decrypt_in_place() {
        let key = [0x42u8; 32];
        let nonce = [0x13u8; 16];
        let plaintext = b"Hello, in-place decryption!";
        let ad = b"test ad";

        // Encrypt
        let mut enc = TurboShakeAead::new(&key, &nonce).unwrap();
        let ciphertext = enc.encrypt(plaintext, ad);

        // Decrypt in-place
        let mut buffer = ciphertext.clone();
        let mut dec = TurboShakeAead::new(&key, &nonce).unwrap();
        dec.decrypt_in_place(&mut buffer, ad).unwrap();

        assert_eq!(buffer, plaintext);
    }

    #[test]
    fn test_various_message_sizes() {
        let key = [0x33u8; 32];
        let nonce = [0x44u8; 16];
        let ad = b"test";

        for size in [1, 10, 50, 100, 135, 136, 137, 200, 272, 500, 1000] {
            let plaintext = vec![0xABu8; size];

            let mut enc = TurboShakeAead::new(&key, &nonce).unwrap();
            let ciphertext = enc.encrypt(&plaintext, ad);

            let mut dec = TurboShakeAead::new(&key, &nonce).unwrap();
            let decrypted = dec.decrypt(&ciphertext, ad).unwrap();

            assert_eq!(decrypted, plaintext, "Failed for size {}", size);
        }
    }
}
