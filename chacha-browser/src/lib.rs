//! WebAssembly bindings for the ChaCha stream cipher with WASM SIMD acceleration.
//!
//! This module provides WASM-compatible wrappers around the ChaCha cipher
//! for encrypting files in the browser. Uses the chacha12 crate which implements
//! WASM SIMD128 acceleration for improved performance.
//!
//! This implements the original DJB ChaCha variant with a 64-bit counter and
//! 64-bit (8-byte) nonce.

use wasm_bindgen::prelude::*;
use chacha12::{ChaCha8 as ChaCha8Impl, ChaCha12 as ChaCha12Impl, ChaCha20 as ChaCha20Impl};

/// A ChaCha8 cipher instance for streaming encryption/decryption.
#[wasm_bindgen]
pub struct ChaCha8Cipher {
    inner: ChaCha8Impl,
}

#[wasm_bindgen]
impl ChaCha8Cipher {
    /// Creates a new ChaCha8 cipher for streaming encryption/decryption.
    #[wasm_bindgen(constructor)]
    pub fn new(key: &[u8], nonce: &[u8]) -> Result<ChaCha8Cipher, JsError> {
        console_error_panic_hook::set_once();
        let (key_array, nonce_array) = validate_inputs(key, nonce)?;
        Ok(ChaCha8Cipher {
            inner: ChaCha8Impl::new(&key_array, &nonce_array),
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
    inner: ChaCha12Impl,
}

#[wasm_bindgen]
impl ChaCha12Cipher {
    /// Creates a new ChaCha12 cipher for streaming encryption/decryption.
    #[wasm_bindgen(constructor)]
    pub fn new(key: &[u8], nonce: &[u8]) -> Result<ChaCha12Cipher, JsError> {
        console_error_panic_hook::set_once();
        let (key_array, nonce_array) = validate_inputs(key, nonce)?;
        Ok(ChaCha12Cipher {
            inner: ChaCha12Impl::new(&key_array, &nonce_array),
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
#[wasm_bindgen]
pub struct ChaCha20Cipher {
    inner: ChaCha20Impl,
}

#[wasm_bindgen]
impl ChaCha20Cipher {
    /// Creates a new ChaCha20 cipher for streaming encryption/decryption.
    #[wasm_bindgen(constructor)]
    pub fn new(key: &[u8], nonce: &[u8]) -> Result<ChaCha20Cipher, JsError> {
        console_error_panic_hook::set_once();
        let (key_array, nonce_array) = validate_inputs(key, nonce)?;
        Ok(ChaCha20Cipher {
            inner: ChaCha20Impl::new(&key_array, &nonce_array),
        })
    }

    /// Process a chunk of data for streaming encryption/decryption.
    pub fn process_chunk(&mut self, chunk: &[u8]) -> Vec<u8> {
        let mut result = chunk.to_vec();
        self.inner.xor_keystream(&mut result);
        result
    }
}

/// Helper function to validate and convert key/nonce inputs
fn validate_inputs(key: &[u8], nonce: &[u8]) -> Result<([u8; 32], [u8; 8]), JsError> {
    if key.len() != 32 {
        return Err(JsError::new("Key must be exactly 32 bytes"));
    }
    if nonce.len() != 8 {
        return Err(JsError::new("Nonce must be exactly 8 bytes"));
    }

    let key_array: [u8; 32] = key.try_into().map_err(|_| JsError::new("Invalid key"))?;
    let nonce_array: [u8; 8] = nonce.try_into().map_err(|_| JsError::new("Invalid nonce"))?;

    Ok((key_array, nonce_array))
}

/// Encrypts data using ChaCha8 (8 rounds) with the provided key and nonce.
///
/// # Arguments
/// * `data` - The data to encrypt
/// * `key` - A 32-byte encryption key
/// * `nonce` - An 8-byte nonce
///
/// # Returns
/// The encrypted data
#[wasm_bindgen]
pub fn encrypt_chacha8(data: &[u8], key: &[u8], nonce: &[u8]) -> Result<Vec<u8>, JsError> {
    console_error_panic_hook::set_once();
    let (key_array, nonce_array) = validate_inputs(key, nonce)?;

    let mut cipher = ChaCha8Impl::new(&key_array, &nonce_array);
    let mut result = data.to_vec();
    cipher.xor_keystream(&mut result);

    Ok(result)
}

/// Encrypts data using ChaCha12 (12 rounds) with the provided key and nonce.
///
/// # Arguments
/// * `data` - The data to encrypt
/// * `key` - A 32-byte encryption key
/// * `nonce` - An 8-byte nonce
///
/// # Returns
/// The encrypted data
#[wasm_bindgen]
pub fn encrypt_chacha12(data: &[u8], key: &[u8], nonce: &[u8]) -> Result<Vec<u8>, JsError> {
    console_error_panic_hook::set_once();
    let (key_array, nonce_array) = validate_inputs(key, nonce)?;

    let mut cipher = ChaCha12Impl::new(&key_array, &nonce_array);
    let mut result = data.to_vec();
    cipher.xor_keystream(&mut result);

    Ok(result)
}

/// Encrypts data using ChaCha20 (20 rounds) with the provided key and nonce.
///
/// # Arguments
/// * `data` - The data to encrypt
/// * `key` - A 32-byte encryption key
/// * `nonce` - An 8-byte nonce
///
/// # Returns
/// The encrypted data
#[wasm_bindgen]
pub fn encrypt(data: &[u8], key: &[u8], nonce: &[u8]) -> Result<Vec<u8>, JsError> {
    console_error_panic_hook::set_once();
    let (key_array, nonce_array) = validate_inputs(key, nonce)?;

    let mut cipher = ChaCha20Impl::new(&key_array, &nonce_array);
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
    encrypt(data, key, nonce)
}

/// Generates a random 32-byte key for ChaCha encryption.
///
/// # Returns
/// A 32-byte random key
#[wasm_bindgen]
pub fn generate_key() -> Result<Vec<u8>, JsError> {
    let mut key = [0u8; 32];
    getrandom::getrandom(&mut key).map_err(|err| JsError::new(&format!("Failed to generate random key: {}", err)))?;
    Ok(key.to_vec())
}

/// Generates a random 8-byte nonce for ChaCha encryption.
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
    fn test_empty_data() {
        let data: &[u8] = &[];
        let key = [0u8; 32];
        let nonce = [0u8; 8];

        let result = encrypt(data, &key, &nonce).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_streaming_encryption() {
        let plaintext: Vec<u8> = (0..1000).map(|i| i as u8).collect();
        let key = [1u8; 32];
        let nonce = [2u8; 8];

        // Single-shot encryption
        let single_shot = encrypt(&plaintext, &key, &nonce).unwrap();

        // Streaming encryption in chunks
        let mut cipher = ChaCha20Impl::new(&key, &nonce);
        let chunk_size = 100;
        let mut streaming_result = Vec::new();
        
        for chunk in plaintext.chunks(chunk_size) {
            let mut chunk_data = chunk.to_vec();
            cipher.xor_keystream(&mut chunk_data);
            streaming_result.extend(chunk_data);
        }

        assert_eq!(single_shot, streaming_result, "Streaming encryption should match single-shot");
    }
}
