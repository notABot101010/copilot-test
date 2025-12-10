//! WebAssembly bindings for hash function benchmarking.
//!
//! This module provides WASM-compatible wrappers around BLAKE3, SHA-256, and SHA-512
//! hash functions for performance comparison in the browser and Node.js.

use wasm_bindgen::prelude::*;
use blake3::Hasher as Blake3Hasher;
use sha2::{Sha256, Sha512, Digest};


/// Initialize the panic hook for better error messages in WASM.
/// This should be called once when the module is loaded.
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

/// Computes the BLAKE3 hash of the input data.
///
/// # Arguments
/// * `data` - The data to hash
///
/// # Returns
/// The 32-byte BLAKE3 hash as a Vec<u8>
#[wasm_bindgen]
pub fn hash_blake3(data: &[u8]) -> Vec<u8> {
    let hash = blake3::hash(data);
    hash.as_bytes().to_vec()
}

/// Computes the SHA-256 hash of the input data.
///
/// # Arguments
/// * `data` - The data to hash
///
/// # Returns
/// The 32-byte SHA-256 hash as a Vec<u8>
#[wasm_bindgen]
pub fn hash_sha256(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// Computes the SHA-512 hash of the input data.
///
/// # Arguments
/// * `data` - The data to hash
///
/// # Returns
/// The 64-byte SHA-512 hash as a Vec<u8>
#[wasm_bindgen]
pub fn hash_sha512(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha512::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// A BLAKE3 hasher instance for streaming hashing.
#[wasm_bindgen]
pub struct Blake3StreamHasher {
    inner: Blake3Hasher,
}

#[wasm_bindgen]
impl Blake3StreamHasher {
    /// Creates a new BLAKE3 hasher for streaming hashing.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Blake3StreamHasher {
        Blake3StreamHasher {
            inner: Blake3Hasher::new(),
        }
    }

    /// Updates the hasher with a chunk of data.
    pub fn update(&mut self, data: &[u8]) {
        self.inner.update(data);
    }

    /// Finalizes the hash and returns the result.
    pub fn finalize(&self) -> Vec<u8> {
        self.inner.finalize().as_bytes().to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blake3_hash() {
        let data = b"Hello, World!";
        let hash = hash_blake3(data);
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_sha256_hash() {
        let data = b"Hello, World!";
        let hash = hash_sha256(data);
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_sha512_hash() {
        let data = b"Hello, World!";
        let hash = hash_sha512(data);
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_blake3_streaming() {
        let data = b"Hello, World!";
        let mut hasher = Blake3StreamHasher::new();
        hasher.update(data);
        let hash = hasher.finalize();
        assert_eq!(hash.len(), 32);

        // Should match one-shot hash
        let one_shot = hash_blake3(data);
        assert_eq!(hash, one_shot);
    }

    #[test]
    fn test_empty_data() {
        let data: &[u8] = &[];

        let blake3_hash = hash_blake3(data);
        assert_eq!(blake3_hash.len(), 32);

        let sha256_hash = hash_sha256(data);
        assert_eq!(sha256_hash.len(), 32);

        let sha512_hash = hash_sha512(data);
        assert_eq!(sha512_hash.len(), 64);
    }
}
