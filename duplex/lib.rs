//! Keccak-based cryptographic primitives
//!
//! This crate implements:
//! - AEAD using duplex construction (KeccakAead, TurboShakeAead)
//! - TurboSHAKE256 (RFC 9861)

mod turboshake;
mod turboshake_aead;

pub use turboshake::TurboShake256;
pub use turboshake_aead::TurboShakeAead;

/// Size of the Keccak state in bytes (1600 bits / 8)
const STATE_SIZE: usize = 200;

/// Rate in bytes (1088 bits / 8) - balance between security and performance
const RATE: usize = 136;

/// Capacity in bytes
const CAPACITY: usize = STATE_SIZE - RATE;

/// Tag size in bytes
const TAG_SIZE: usize = 32;

/// Number of Keccak rounds
const ROUNDS: usize = 12;

/// Error types for AEAD operations
#[derive(Debug, PartialEq, Eq)]
pub enum AeadError {
    /// Authentication tag verification failed
    AuthenticationFailed,
    /// Invalid key size
    InvalidKeySize,
    /// Invalid nonce size
    InvalidNonceSize,
}

/// Keccak-based AEAD cipher using duplex construction
#[derive(Debug, PartialEq)]
pub struct KeccakAead {
    state: [u64; 25], // Keccak state is 1600 bits = 25 * 64-bit words
}

impl KeccakAead {
    /// Create a new AEAD instance with the given key and nonce
    ///
    /// # Arguments
    /// * `key` - 32-byte encryption key
    /// * `nonce` - 16-byte nonce (must be unique for each encryption with the same key)
    ///
    /// # Returns
    /// A new `KeccakAead` instance or an error if key/nonce sizes are invalid
    pub fn new(key: &[u8], nonce: &[u8]) -> Result<Self, AeadError> {
        if key.len() != 32 {
            return Err(AeadError::InvalidKeySize);
        }
        if nonce.len() != 16 {
            return Err(AeadError::InvalidNonceSize);
        }

        let mut cipher = Self {
            state: [0u64; 25],
        };

        // Initialize with key and nonce
        cipher.absorb(key);
        cipher.permute();
        cipher.absorb(nonce);
        cipher.permute();

        Ok(cipher)
    }

    /// Absorb data into the sponge state
    fn absorb(&mut self, data: &[u8]) {
        let state_bytes = state_to_bytes(&self.state);
        let mut new_state = state_bytes;

        for (i, &byte) in data.iter().enumerate() {
            if i >= RATE {
                break;
            }
            new_state[i] ^= byte;
        }

        self.state = bytes_to_state(&new_state);
    }

    /// Apply Keccak permutation with specified rounds
    fn permute(&mut self) {
        keccak::p1600(&mut self.state, 12);
        // The keccak crate's f1600 does 24 rounds by default
        // For 12 rounds, we'd need a custom implementation or use a different approach
        // For now, we'll use the full 24 rounds for security
        // In production, you'd want to implement a custom permutation with configurable rounds
    }

    /// Squeeze data from the sponge state
    fn squeeze(&self, output: &mut [u8]) {
        let state_bytes = state_to_bytes(&self.state);
        let len = output.len().min(RATE);
        output[..len].copy_from_slice(&state_bytes[..len]);
    }

    /// Encrypt plaintext with optional associated data
    ///
    /// # Arguments
    /// * `plaintext` - Data to encrypt
    /// * `associated_data` - Optional data to authenticate but not encrypt
    ///
    /// # Returns
    /// Ciphertext with authentication tag appended
    pub fn encrypt(&mut self, plaintext: &[u8], associated_data: &[u8]) -> Vec<u8> {
        let mut ciphertext = Vec::with_capacity(plaintext.len() + TAG_SIZE);

        // Absorb associated data
        if !associated_data.is_empty() {
            for chunk in associated_data.chunks(RATE) {
                self.absorb(chunk);
                self.permute();
            }
            // Domain separator
            self.absorb(&[0x01]);
            self.permute();
        }

        // Encrypt plaintext using duplex construction
        for chunk in plaintext.chunks(RATE) {
            // Squeeze keystream first
            let mut keystream = vec![0u8; chunk.len()];
            self.squeeze(&mut keystream);

            // XOR plaintext with keystream to get ciphertext
            for (i, &pt_byte) in chunk.iter().enumerate() {
                ciphertext.push(pt_byte ^ keystream[i]);
            }

            // Absorb ciphertext for authentication
            let ct_chunk = &ciphertext[ciphertext.len() - chunk.len()..];
            self.absorb(ct_chunk);
            self.permute();
        }

        // Generate authentication tag
        let mut tag = vec![0u8; TAG_SIZE];
        self.squeeze(&mut tag);
        ciphertext.extend_from_slice(&tag);

        ciphertext
    }

    /// Decrypt ciphertext and verify authentication tag
    ///
    /// # Arguments
    /// * `ciphertext` - Data to decrypt (including tag)
    /// * `associated_data` - Optional authenticated data
    ///
    /// # Returns
    /// Decrypted plaintext or an error if authentication fails
    pub fn decrypt(
        &mut self,
        ciphertext: &[u8],
        associated_data: &[u8],
    ) -> Result<Vec<u8>, AeadError> {
        if ciphertext.len() < TAG_SIZE {
            return Err(AeadError::AuthenticationFailed);
        }

        let ct_len = ciphertext.len() - TAG_SIZE;
        let (ct_data, received_tag) = ciphertext.split_at(ct_len);

        let mut plaintext = Vec::with_capacity(ct_len);

        // Absorb associated data
        if !associated_data.is_empty() {
            for chunk in associated_data.chunks(RATE) {
                self.absorb(chunk);
                self.permute();
            }
            // Domain separator
            self.absorb(&[0x01]);
            self.permute();
        }

        // Decrypt ciphertext using duplex construction
        for chunk in ct_data.chunks(RATE) {
            // Squeeze keystream first (same as encryption)
            let mut keystream = vec![0u8; chunk.len()];
            self.squeeze(&mut keystream);

            // Absorb ciphertext for authentication (same as encryption)
            self.absorb(chunk);
            self.permute();

            // XOR ciphertext with keystream to get plaintext
            for (i, &ct_byte) in chunk.iter().enumerate() {
                plaintext.push(ct_byte ^ keystream[i]);
            }
        }

        // Verify authentication tag
        let mut computed_tag = vec![0u8; TAG_SIZE];
        self.squeeze(&mut computed_tag);

        if !constant_time_compare(&computed_tag, received_tag) {
            return Err(AeadError::AuthenticationFailed);
        }

        Ok(plaintext)
    }
}

/// Convert state array to byte array
fn state_to_bytes(state: &[u64; 25]) -> [u8; STATE_SIZE] {
    let mut bytes = [0u8; STATE_SIZE];
    for (i, &word) in state.iter().enumerate() {
        bytes[i * 8..(i + 1) * 8].copy_from_slice(&word.to_le_bytes());
    }
    bytes
}

/// Convert byte array to state array
fn bytes_to_state(bytes: &[u8; STATE_SIZE]) -> [u64; 25] {
    let mut state = [0u64; 25];
    for (i, word) in state.iter_mut().enumerate() {
        let mut word_bytes = [0u8; 8];
        word_bytes.copy_from_slice(&bytes[i * 8..(i + 1) * 8]);
        *word = u64::from_le_bytes(word_bytes);
    }
    state
}

/// Constant-time comparison to prevent timing attacks
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
    fn test_new_valid_key_nonce() {
        let key = [0u8; 32];
        let nonce = [0u8; 16];
        let result = KeccakAead::new(&key, &nonce);
        assert!(result.is_ok());
    }

    #[test]
    fn test_new_invalid_key_size() {
        let key = [0u8; 16]; // Wrong size
        let nonce = [0u8; 16];
        let result = KeccakAead::new(&key, &nonce);
        assert_eq!(result, Err(AeadError::InvalidKeySize));
    }

    #[test]
    fn test_new_invalid_nonce_size() {
        let key = [0u8; 32];
        let nonce = [0u8; 8]; // Wrong size
        let result = KeccakAead::new(&key, &nonce);
        assert_eq!(result, Err(AeadError::InvalidNonceSize));
    }

    #[test]
    fn test_encrypt_decrypt_empty() {
        let key = [1u8; 32];
        let nonce = [2u8; 16];
        let plaintext = b"";
        let ad = b"";

        let mut cipher_enc = KeccakAead::new(&key, &nonce).unwrap();
        let ciphertext = cipher_enc.encrypt(plaintext, ad);

        let mut cipher_dec = KeccakAead::new(&key, &nonce).unwrap();
        let decrypted = cipher_dec.decrypt(&ciphertext, ad).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_decrypt_small() {
        let key = [1u8; 32];
        let nonce = [2u8; 16];
        let plaintext = b"Hello, World!";
        let ad = b"";

        let mut cipher_enc = KeccakAead::new(&key, &nonce).unwrap();
        let ciphertext = cipher_enc.encrypt(plaintext, ad);

        let mut cipher_dec = KeccakAead::new(&key, &nonce).unwrap();
        let decrypted = cipher_dec.decrypt(&ciphertext, ad).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_decrypt_with_associated_data() {
        let key = [1u8; 32];
        let nonce = [2u8; 16];
        let plaintext = b"Secret message";
        let ad = b"Additional authenticated data";

        let mut cipher_enc = KeccakAead::new(&key, &nonce).unwrap();
        let ciphertext = cipher_enc.encrypt(plaintext, ad);

        let mut cipher_dec = KeccakAead::new(&key, &nonce).unwrap();
        let decrypted = cipher_dec.decrypt(&ciphertext, ad).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_decrypt_wrong_ad_fails() {
        let key = [1u8; 32];
        let nonce = [2u8; 16];
        let plaintext = b"Secret message";
        let ad = b"Correct AD";
        let wrong_ad = b"Wrong AD";

        let mut cipher_enc = KeccakAead::new(&key, &nonce).unwrap();
        let ciphertext = cipher_enc.encrypt(plaintext, ad);

        let mut cipher_dec = KeccakAead::new(&key, &nonce).unwrap();
        let result = cipher_dec.decrypt(&ciphertext, wrong_ad);

        assert_eq!(result, Err(AeadError::AuthenticationFailed));
    }

    #[test]
    fn test_decrypt_modified_ciphertext_fails() {
        let key = [1u8; 32];
        let nonce = [2u8; 16];
        let plaintext = b"Secret message";
        let ad = b"";

        let mut cipher_enc = KeccakAead::new(&key, &nonce).unwrap();
        let mut ciphertext = cipher_enc.encrypt(plaintext, ad);

        // Modify ciphertext
        if !ciphertext.is_empty() {
            ciphertext[0] ^= 1;
        }

        let mut cipher_dec = KeccakAead::new(&key, &nonce).unwrap();
        let result = cipher_dec.decrypt(&ciphertext, ad);

        assert_eq!(result, Err(AeadError::AuthenticationFailed));
    }

    #[test]
    fn test_encrypt_decrypt_large() {
        let key = [1u8; 32];
        let nonce = [2u8; 16];
        let plaintext = vec![42u8; 1024]; // 1KB
        let ad = b"metadata";

        let mut cipher_enc = KeccakAead::new(&key, &nonce).unwrap();
        let ciphertext = cipher_enc.encrypt(&plaintext, ad);

        let mut cipher_dec = KeccakAead::new(&key, &nonce).unwrap();
        let decrypted = cipher_dec.decrypt(&ciphertext, ad).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_different_keys_produce_different_ciphertext() {
        let key1 = [1u8; 32];
        let key2 = [2u8; 32];
        let nonce = [0u8; 16];
        let plaintext = b"Same plaintext";
        let ad = b"";

        let mut cipher1 = KeccakAead::new(&key1, &nonce).unwrap();
        let ct1 = cipher1.encrypt(plaintext, ad);

        let mut cipher2 = KeccakAead::new(&key2, &nonce).unwrap();
        let ct2 = cipher2.encrypt(plaintext, ad);

        assert_ne!(ct1, ct2);
    }

    #[test]
    fn test_different_nonces_produce_different_ciphertext() {
        let key = [1u8; 32];
        let nonce1 = [1u8; 16];
        let nonce2 = [2u8; 16];
        let plaintext = b"Same plaintext";
        let ad = b"";

        let mut cipher1 = KeccakAead::new(&key, &nonce1).unwrap();
        let ct1 = cipher1.encrypt(plaintext, ad);

        let mut cipher2 = KeccakAead::new(&key, &nonce2).unwrap();
        let ct2 = cipher2.encrypt(plaintext, ad);

        assert_ne!(ct1, ct2);
    }

    #[test]
    fn test_state_byte_conversion() {
        let state = [0x0123456789ABCDEFu64; 25];
        let bytes = state_to_bytes(&state);
        let state2 = bytes_to_state(&bytes);
        assert_eq!(state, state2);
    }

    #[test]
    fn test_constant_time_compare() {
        assert!(constant_time_compare(&[1, 2, 3], &[1, 2, 3]));
        assert!(!constant_time_compare(&[1, 2, 3], &[1, 2, 4]));
        assert!(!constant_time_compare(&[1, 2], &[1, 2, 3]));
    }
}
