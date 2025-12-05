//! WebAssembly bindings for the ChaCha stream cipher.
//!
//! This module provides WASM-compatible wrappers around the ChaCha cipher
//! for encrypting files in the browser. Supports streaming encryption for
//! handling large files without loading them entirely into memory.
//!
//! This implements the original DJB ChaCha variant with a 64-bit counter and
//! 64-bit (8-byte) nonce, as opposed to the IETF variant (RFC 8439) which uses
//! a 32-bit counter and 96-bit (12-byte) nonce.

use wasm_bindgen::prelude::*;

/// ChaCha block size in bytes (512 bits = 64 bytes)
const BLOCK_SIZE: usize = 64;

/// Constants for ChaCha: "expand 32-byte k" in little-endian
const CONSTANTS: [u32; 4] = [0x61707865, 0x3320646e, 0x79622d32, 0x6b206574];

/// ChaCha stream cipher with parametrized rounds.
///
/// The `remaining_keystream` field stores unused keystream bytes from the previous block.
/// Index 0 contains the count of remaining bytes (0-63), and the actual remaining bytes
/// are stored at the end of the array (positions 64 - remaining_count to 63).
struct ChaChaInner<const ROUNDS: usize> {
    state: [u32; 16],
    remaining_keystream: [u8; BLOCK_SIZE],
}

/// The ChaCha quarter round operation.
#[inline]
fn quarter_round(state: &mut [u32; 16], a: usize, b: usize, c: usize, d: usize) {
    state[a] = state[a].wrapping_add(state[b]);
    state[d] ^= state[a];
    state[d] = state[d].rotate_left(16);

    state[c] = state[c].wrapping_add(state[d]);
    state[b] ^= state[c];
    state[b] = state[b].rotate_left(12);

    state[a] = state[a].wrapping_add(state[b]);
    state[d] ^= state[a];
    state[d] = state[d].rotate_left(8);

    state[c] = state[c].wrapping_add(state[d]);
    state[b] ^= state[c];
    state[b] = state[b].rotate_left(7);
}

/// Perform the ChaCha block function with parametrized rounds.
#[inline]
fn chacha_block<const ROUNDS: usize>(state: &[u32; 16], keystream: &mut [u8; BLOCK_SIZE]) {
    let mut working_state = *state;

    // ROUNDS / 2 double rounds
    for _ in 0..(ROUNDS / 2) {
        // Column rounds
        quarter_round(&mut working_state, 0, 4, 8, 12);
        quarter_round(&mut working_state, 1, 5, 9, 13);
        quarter_round(&mut working_state, 2, 6, 10, 14);
        quarter_round(&mut working_state, 3, 7, 11, 15);
        // Diagonal rounds
        quarter_round(&mut working_state, 0, 5, 10, 15);
        quarter_round(&mut working_state, 1, 6, 11, 12);
        quarter_round(&mut working_state, 2, 7, 8, 13);
        quarter_round(&mut working_state, 3, 4, 9, 14);
    }

    // Add the original state to the working state and serialize into keystream
    for word_index in 0..16 {
        working_state[word_index] = working_state[word_index].wrapping_add(state[word_index]);
        let bytes = working_state[word_index].to_le_bytes();
        keystream[word_index * 4..word_index * 4 + 4].copy_from_slice(&bytes);
    }
}

impl<const ROUNDS: usize> ChaChaInner<ROUNDS> {
    /// Creates a new ChaCha cipher instance with the given key and nonce.
    fn new(key: &[u8; 32], nonce: &[u8; 8]) -> ChaChaInner<ROUNDS> {
        let mut state = [0u32; 16];

        // Set constants
        state[0..4].copy_from_slice(&CONSTANTS);

        // Set key (8 x 32-bit words)
        for (state_word, key_chunk) in state[4..12].iter_mut().zip(key.chunks_exact(4)) {
            *state_word = u32::from_le_bytes(key_chunk.try_into().unwrap());
        }

        // Counter starts at 0
        state[12] = 0;
        state[13] = 0;

        // Set nonce (2 x 32-bit words)
        state[14] = u32::from_le_bytes([nonce[0], nonce[1], nonce[2], nonce[3]]);
        state[15] = u32::from_le_bytes([nonce[4], nonce[5], nonce[6], nonce[7]]);

        ChaChaInner {
            state,
            remaining_keystream: [0u8; BLOCK_SIZE],
        }
    }

    /// Returns the current counter value.
    #[inline(always)]
    fn counter(&self) -> u64 {
        ((self.state[13] as u64) << 32) | (self.state[12] as u64)
    }

    /// Increments the 64-bit counter by the given amount.
    #[inline(always)]
    fn increment_counter(&mut self, amount: u64) {
        let counter = self.counter();
        let new_counter = counter.wrapping_add(amount);
        self.state[12] = new_counter as u32;
        self.state[13] = (new_counter >> 32) as u32;
    }

    /// Generates the next keystream block and returns it.
    #[inline]
    fn next_keystream_block(&mut self) -> [u8; BLOCK_SIZE] {
        let mut keystream = [0u8; BLOCK_SIZE];
        chacha_block::<ROUNDS>(&self.state, &mut keystream);
        self.increment_counter(1);
        keystream
    }

    /// XORs the data with the keystream to encrypt/decrypt.
    fn xor_keystream(&mut self, data: &mut [u8]) {
        if data.is_empty() {
            return;
        }

        let mut offset = 0;
        let data_len = data.len();

        // First, use any remaining keystream bytes from the previous block
        let remaining = self.remaining_keystream[0] as usize;
        if remaining > 0 {
            let use_bytes = remaining.min(data_len);
            let start_idx = BLOCK_SIZE - remaining;
            data[..use_bytes]
                .iter_mut()
                .zip(&self.remaining_keystream[start_idx..start_idx + use_bytes])
                .for_each(|(d, k)| *d ^= k);
            offset = use_bytes;

            if use_bytes < remaining {
                self.remaining_keystream[0] = (remaining - use_bytes) as u8;
            } else {
                self.remaining_keystream[0] = 0;
            }
        }

        // Process full blocks
        while offset + BLOCK_SIZE <= data_len {
            let keystream = self.next_keystream_block();
            data[offset..offset + BLOCK_SIZE]
                .iter_mut()
                .zip(&keystream)
                .for_each(|(d, k)| *d ^= k);
            offset += BLOCK_SIZE;
        }

        // Handle remaining bytes (partial block)
        let remaining_data = data_len - offset;
        if remaining_data > 0 {
            let keystream = self.next_keystream_block();
            data[offset..]
                .iter_mut()
                .zip(&keystream[..remaining_data])
                .for_each(|(d, k)| *d ^= k);

            // Store the remaining keystream bytes for later use
            let remaining_keystream_length = BLOCK_SIZE - remaining_data;
            self.remaining_keystream[0] = remaining_keystream_length as u8;
            self.remaining_keystream[BLOCK_SIZE - remaining_keystream_length..BLOCK_SIZE]
                .copy_from_slice(&keystream[remaining_data..]);
        }
    }
}

/// Type aliases for common round counts
type ChaCha8Inner = ChaChaInner<8>;
type ChaCha12Inner = ChaChaInner<12>;
type ChaCha20Inner = ChaChaInner<20>;

/// A ChaCha8 cipher instance for streaming encryption/decryption.
#[wasm_bindgen]
pub struct ChaCha8Cipher {
    inner: ChaCha8Inner,
}

#[wasm_bindgen]
impl ChaCha8Cipher {
    /// Creates a new ChaCha8 cipher for streaming encryption/decryption.
    #[wasm_bindgen(constructor)]
    pub fn new(key: &[u8], nonce: &[u8]) -> Result<ChaCha8Cipher, JsError> {
        console_error_panic_hook::set_once();

        if key.len() != 32 {
            return Err(JsError::new("Key must be exactly 32 bytes"));
        }
        if nonce.len() != 8 {
            return Err(JsError::new("Nonce must be exactly 8 bytes"));
        }

        let key_array: [u8; 32] = key.try_into().map_err(|_| JsError::new("Invalid key"))?;
        let nonce_array: [u8; 8] = nonce.try_into().map_err(|_| JsError::new("Invalid nonce"))?;

        Ok(ChaCha8Cipher {
            inner: ChaCha8Inner::new(&key_array, &nonce_array),
        })
    }

    /// Process a chunk of data for streaming encryption/decryption.
    pub fn process_chunk(&mut self, chunk: &[u8]) -> Vec<u8> {
        let mut result = chunk.to_vec();
        self.inner.xor_keystream(&mut result);
        result
    }
}

/// A ChaCha12 cipher instance for streaming encryption/decryption.
#[wasm_bindgen]
pub struct ChaCha12Cipher {
    inner: ChaCha12Inner,
}

#[wasm_bindgen]
impl ChaCha12Cipher {
    /// Creates a new ChaCha12 cipher for streaming encryption/decryption.
    #[wasm_bindgen(constructor)]
    pub fn new(key: &[u8], nonce: &[u8]) -> Result<ChaCha12Cipher, JsError> {
        console_error_panic_hook::set_once();

        if key.len() != 32 {
            return Err(JsError::new("Key must be exactly 32 bytes"));
        }
        if nonce.len() != 8 {
            return Err(JsError::new("Nonce must be exactly 8 bytes"));
        }

        let key_array: [u8; 32] = key.try_into().map_err(|_| JsError::new("Invalid key"))?;
        let nonce_array: [u8; 8] = nonce.try_into().map_err(|_| JsError::new("Invalid nonce"))?;

        Ok(ChaCha12Cipher {
            inner: ChaCha12Inner::new(&key_array, &nonce_array),
        })
    }

    /// Process a chunk of data for streaming encryption/decryption.
    pub fn process_chunk(&mut self, chunk: &[u8]) -> Vec<u8> {
        let mut result = chunk.to_vec();
        self.inner.xor_keystream(&mut result);
        result
    }
}

/// A ChaCha20 cipher instance for streaming encryption/decryption.
/// 
/// This allows encrypting data in chunks without loading the entire file
/// into memory. Create an instance, then call `process_chunk` repeatedly
/// with chunks of data.
#[wasm_bindgen]
pub struct ChaCha20Cipher {
    inner: ChaCha20Inner,
}

#[wasm_bindgen]
impl ChaCha20Cipher {
    /// Creates a new ChaCha20 cipher for streaming encryption/decryption.
    ///
    /// # Arguments
    /// * `key` - A 32-byte encryption key
    /// * `nonce` - An 8-byte nonce
    #[wasm_bindgen(constructor)]
    pub fn new(key: &[u8], nonce: &[u8]) -> Result<ChaCha20Cipher, JsError> {
        console_error_panic_hook::set_once();

        if key.len() != 32 {
            return Err(JsError::new("Key must be exactly 32 bytes"));
        }
        if nonce.len() != 8 {
            return Err(JsError::new("Nonce must be exactly 8 bytes"));
        }

        let key_array: [u8; 32] = key.try_into().map_err(|_| JsError::new("Invalid key"))?;
        let nonce_array: [u8; 8] = nonce.try_into().map_err(|_| JsError::new("Invalid nonce"))?;

        Ok(ChaCha20Cipher {
            inner: ChaCha20Inner::new(&key_array, &nonce_array),
        })
    }

    /// Process a chunk of data for streaming encryption/decryption.
    /// 
    /// This method can be called multiple times with consecutive chunks of data.
    /// The cipher maintains its state between calls, so you can encrypt a large
    /// file by reading it in chunks (e.g., 1MB at a time) and calling this method
    /// for each chunk.
    ///
    /// # Arguments
    /// * `chunk` - The data chunk to encrypt/decrypt
    ///
    /// # Returns
    /// The encrypted/decrypted chunk
    pub fn process_chunk(&mut self, chunk: &[u8]) -> Vec<u8> {
        let mut result = chunk.to_vec();
        self.inner.xor_keystream(&mut result);
        result
    }
}

/// Encrypts data using ChaCha8 with the provided key and nonce.
#[wasm_bindgen]
pub fn encrypt_chacha8(data: &[u8], key: &[u8], nonce: &[u8]) -> Result<Vec<u8>, JsError> {
    console_error_panic_hook::set_once();

    if key.len() != 32 {
        return Err(JsError::new("Key must be exactly 32 bytes"));
    }
    if nonce.len() != 8 {
        return Err(JsError::new("Nonce must be exactly 8 bytes"));
    }

    let key_array: [u8; 32] = key.try_into().map_err(|_| JsError::new("Invalid key"))?;
    let nonce_array: [u8; 8] = nonce.try_into().map_err(|_| JsError::new("Invalid nonce"))?;

    let mut cipher = ChaCha8Inner::new(&key_array, &nonce_array);
    let mut result = data.to_vec();
    cipher.xor_keystream(&mut result);

    Ok(result)
}

/// Encrypts data using ChaCha12 with the provided key and nonce.
#[wasm_bindgen]
pub fn encrypt_chacha12(data: &[u8], key: &[u8], nonce: &[u8]) -> Result<Vec<u8>, JsError> {
    console_error_panic_hook::set_once();

    if key.len() != 32 {
        return Err(JsError::new("Key must be exactly 32 bytes"));
    }
    if nonce.len() != 8 {
        return Err(JsError::new("Nonce must be exactly 8 bytes"));
    }

    let key_array: [u8; 32] = key.try_into().map_err(|_| JsError::new("Invalid key"))?;
    let nonce_array: [u8; 8] = nonce.try_into().map_err(|_| JsError::new("Invalid nonce"))?;

    let mut cipher = ChaCha12Inner::new(&key_array, &nonce_array);
    let mut result = data.to_vec();
    cipher.xor_keystream(&mut result);

    Ok(result)
}

/// Encrypts data using ChaCha20 with the provided key and nonce.
///
/// # Arguments
/// * `data` - The data to encrypt (modified in place)
/// * `key` - A 32-byte encryption key
/// * `nonce` - An 8-byte nonce
///
/// # Returns
/// The encrypted data
#[wasm_bindgen]
pub fn encrypt(data: &[u8], key: &[u8], nonce: &[u8]) -> Result<Vec<u8>, JsError> {
    console_error_panic_hook::set_once();

    if key.len() != 32 {
        return Err(JsError::new("Key must be exactly 32 bytes"));
    }
    if nonce.len() != 8 {
        return Err(JsError::new("Nonce must be exactly 8 bytes"));
    }

    let key_array: [u8; 32] = key.try_into().map_err(|_| JsError::new("Invalid key"))?;
    let nonce_array: [u8; 8] = nonce.try_into().map_err(|_| JsError::new("Invalid nonce"))?;

    let mut cipher = ChaCha20Inner::new(&key_array, &nonce_array);
    let mut result = data.to_vec();
    cipher.xor_keystream(&mut result);

    Ok(result)
}

/// Decrypts data using ChaCha20 with the provided key and nonce.
/// Since ChaCha20 is a stream cipher, encryption and decryption are the same operation.
///
/// # Arguments
/// * `data` - The data to decrypt
/// * `key` - A 32-byte encryption key
/// * `nonce` - An 8-byte nonce
///
/// # Returns
/// The decrypted data
#[wasm_bindgen]
pub fn decrypt(data: &[u8], key: &[u8], nonce: &[u8]) -> Result<Vec<u8>, JsError> {
    // ChaCha20 encryption and decryption are identical
    encrypt(data, key, nonce)
}

/// Generates a random 32-byte key for ChaCha20 encryption.
///
/// # Returns
/// A 32-byte random key
#[wasm_bindgen]
pub fn generate_key() -> Result<Vec<u8>, JsError> {
    let mut key = [0u8; 32];
    getrandom::getrandom(&mut key).map_err(|err| JsError::new(&format!("Failed to generate random key: {}", err)))?;
    Ok(key.to_vec())
}

/// Generates a random 8-byte nonce for ChaCha20 encryption.
///
/// # Returns
/// An 8-byte random nonce
#[wasm_bindgen]
pub fn generate_nonce() -> Result<Vec<u8>, JsError> {
    let mut nonce = [0u8; 8];
    getrandom::getrandom(&mut nonce).map_err(|err| JsError::new(&format!("Failed to generate random nonce: {}", err)))?;
    Ok(nonce.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let plaintext = b"Hello, World! This is a test message.";
        let key = [1u8; 32];
        let nonce = [2u8; 8];

        let ciphertext = encrypt(plaintext, &key, &nonce).unwrap();
        assert_ne!(&ciphertext[..], &plaintext[..], "Ciphertext should differ from plaintext");

        let decrypted = decrypt(&ciphertext, &key, &nonce).unwrap();
        assert_eq!(&decrypted[..], &plaintext[..], "Decrypted should match original");
    }

    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_invalid_key_length() {
        let data = b"test";
        let key = [0u8; 16]; // Wrong size
        let nonce = [0u8; 8];

        let result = encrypt(data, &key, &nonce);
        assert!(result.is_err());
    }

    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_invalid_nonce_length() {
        let data = b"test";
        let key = [0u8; 32];
        let nonce = [0u8; 4]; // Wrong size

        let result = encrypt(data, &key, &nonce);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_data() {
        let data: &[u8] = &[];
        let key = [0u8; 32];
        let nonce = [0u8; 8];

        let result = encrypt(data, &key, &nonce).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_streaming_encryption() {
        // Test that streaming encryption produces the same result as single-shot encryption
        let plaintext: Vec<u8> = (0..1000).map(|i| i as u8).collect();
        let key = [1u8; 32];
        let nonce = [2u8; 8];

        // Single-shot encryption
        let single_shot = encrypt(&plaintext, &key, &nonce).unwrap();

        // Streaming encryption in chunks of various sizes
        let mut cipher = ChaCha20Inner::new(&key, &nonce);
        let chunk_size = 100;
        let mut streaming_result = Vec::new();
        
        for chunk in plaintext.chunks(chunk_size) {
            let mut chunk_data = chunk.to_vec();
            cipher.xor_keystream(&mut chunk_data);
            streaming_result.extend(chunk_data);
        }

        assert_eq!(single_shot, streaming_result, "Streaming encryption should match single-shot");
    }

    #[test]
    fn test_streaming_with_uneven_chunks() {
        // Test streaming with chunks that don't align with block boundaries
        let plaintext: Vec<u8> = (0..500).map(|i| i as u8).collect();
        let key = [3u8; 32];
        let nonce = [4u8; 8];

        // Single-shot encryption
        let single_shot = encrypt(&plaintext, &key, &nonce).unwrap();

        // Streaming with uneven chunk sizes (17, 23, 47, etc.)
        let chunk_sizes = [17, 23, 47, 100, 200, 113];
        let mut cipher = ChaCha20Inner::new(&key, &nonce);
        let mut streaming_result = Vec::new();
        let mut offset = 0;
        
        for &size in chunk_sizes.iter().cycle() {
            if offset >= plaintext.len() {
                break;
            }
            let end = (offset + size).min(plaintext.len());
            let mut chunk_data = plaintext[offset..end].to_vec();
            cipher.xor_keystream(&mut chunk_data);
            streaming_result.extend(chunk_data);
            offset = end;
        }

        assert_eq!(single_shot, streaming_result, "Streaming with uneven chunks should match single-shot");
    }
}
