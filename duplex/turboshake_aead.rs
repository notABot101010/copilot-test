//! Duplex AEAD based on TurboSHAKE256 core
//!
//! This demonstrates how to build an AEAD using the duplex construction
//! with the same 12-round Keccak permutation as TurboSHAKE256.

/// Convert state array to byte array
fn state_to_bytes(state: &[u64; 25]) -> [u8; 200] {
    let mut bytes = [0u8; 200];
    for (i, &word) in state.iter().enumerate() {
        bytes[i * 8..(i + 1) * 8].copy_from_slice(&word.to_le_bytes());
    }
    bytes
}

/// Convert byte array to state array
fn bytes_to_state(bytes: &[u8; 200]) -> [u64; 25] {
    let mut state = [0u64; 25];
    for (i, word) in state.iter_mut().enumerate() {
        let mut word_bytes = [0u8; 8];
        word_bytes.copy_from_slice(&bytes[i * 8..(i + 1) * 8]);
        *word = u64::from_le_bytes(word_bytes);
    }
    state
}

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

/// Duplex AEAD using TurboSHAKE256 core (12-round Keccak-p[1600])
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
pub struct TurboShakeAead {
    state: [u64; 25],
}

impl TurboShakeAead {
    const RATE: usize = 136; // Same as TurboSHAKE256
    const TAG_SIZE: usize = 32;
    const DOMAIN_SEP_AD: u8 = 0x01; // Domain separator for AD
    const DOMAIN_SEP_MSG: u8 = 0x02; // Domain separator for message

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

    /// Absorb a block of data (up to rate bytes)
    fn absorb_block(&mut self, data: &[u8]) {
        let state_bytes = state_to_bytes(&self.state);
        let mut new_state = state_bytes;

        let len = data.len().min(Self::RATE);
        for i in 0..len {
            new_state[i] ^= data[i];
        }

        self.state = bytes_to_state(&new_state);
    }

    /// Squeeze a block of data (up to rate bytes)
    fn squeeze_block(&self, output: &mut [u8]) {
        let state_bytes = state_to_bytes(&self.state);
        let len = output.len().min(Self::RATE);
        output[..len].copy_from_slice(&state_bytes[..len]);
    }

    /// Process associated data
    fn process_ad(&mut self, ad: &[u8]) {
        if ad.is_empty() {
            return;
        }

        // Process full blocks
        for chunk in ad.chunks(Self::RATE) {
            self.absorb_block(chunk);
            keccak::p1600(&mut self.state, 12);
        }

        // Domain separation after AD
        let mut sep = [0u8; 1];
        sep[0] = Self::DOMAIN_SEP_AD;
        self.absorb_block(&sep);
        keccak::p1600(&mut self.state, 12);
    }

    /// Encrypt plaintext
    ///
    /// The duplex encryption pattern per block:
    /// 1. Squeeze keystream from current state
    /// 2. XOR plaintext with keystream → ciphertext
    /// 3. Absorb ciphertext back into state
    /// 4. Apply permutation
    ///
    /// This ensures the ciphertext is authenticated because it's absorbed
    /// into the state that will be used to generate the tag.
    pub fn encrypt(&mut self, plaintext: &[u8], ad: &[u8]) -> Vec<u8> {
        // Process associated data first
        self.process_ad(ad);

        let mut ciphertext = Vec::with_capacity(plaintext.len() + Self::TAG_SIZE);

        // Domain separation for message phase
        let mut sep = [0u8; 1];
        sep[0] = Self::DOMAIN_SEP_MSG;
        self.absorb_block(&sep);
        keccak::p1600(&mut self.state, 12);

        // Encrypt each block using duplex pattern
        for chunk in plaintext.chunks(Self::RATE) {
            // 1. Squeeze keystream
            let mut keystream = vec![0u8; chunk.len()];
            self.squeeze_block(&mut keystream);

            // 2. XOR to produce ciphertext
            let mut ct_block = vec![0u8; chunk.len()];
            for i in 0..chunk.len() {
                ct_block[i] = chunk[i] ^ keystream[i];
            }

            // 3. Absorb ciphertext (authenticates the ciphertext)
            self.absorb_block(&ct_block);

            // 4. Permute
            keccak::p1600(&mut self.state, 12);

            ciphertext.extend_from_slice(&ct_block);
        }

        // Generate authentication tag
        let mut tag = vec![0u8; Self::TAG_SIZE];
        self.squeeze_block(&mut tag);
        ciphertext.extend_from_slice(&tag);

        ciphertext
    }

    /// Decrypt ciphertext
    ///
    /// The duplex decryption pattern per block:
    /// 1. Squeeze keystream from current state
    /// 2. Absorb ciphertext into state (before decrypting!)
    /// 3. Apply permutation
    /// 4. XOR ciphertext with keystream → plaintext
    ///
    /// Note: We absorb ciphertext BEFORE decrypting to match the
    /// encryption pattern (where we absorbed ciphertext).
    pub fn decrypt(&mut self, ciphertext: &[u8], ad: &[u8]) -> Result<Vec<u8>, AeadError> {
        if ciphertext.len() < Self::TAG_SIZE {
            return Err(AeadError::AuthenticationFailed);
        }

        let ct_len = ciphertext.len() - Self::TAG_SIZE;
        let (ct_data, received_tag) = ciphertext.split_at(ct_len);

        // Process associated data (must match encryption)
        self.process_ad(ad);

        let mut plaintext = Vec::with_capacity(ct_len);

        // Domain separation for message phase
        let mut sep = [0u8; 1];
        sep[0] = Self::DOMAIN_SEP_MSG;
        self.absorb_block(&sep);
        keccak::p1600(&mut self.state, 12);

        // Decrypt each block using duplex pattern
        for chunk in ct_data.chunks(Self::RATE) {
            // 1. Squeeze keystream
            let mut keystream = vec![0u8; chunk.len()];
            self.squeeze_block(&mut keystream);

            // 2. Absorb ciphertext (BEFORE decrypting)
            self.absorb_block(chunk);

            // 3. Permute
            keccak::p1600(&mut self.state, 12);

            // 4. XOR to recover plaintext
            let mut pt_block = vec![0u8; chunk.len()];
            for i in 0..chunk.len() {
                pt_block[i] = chunk[i] ^ keystream[i];
            }

            plaintext.extend_from_slice(&pt_block);
        }

        // Verify authentication tag
        let mut computed_tag = vec![0u8; Self::TAG_SIZE];
        self.squeeze_block(&mut computed_tag);

        // Constant-time comparison
        if !constant_time_compare(&computed_tag, received_tag) {
            return Err(AeadError::AuthenticationFailed);
        }

        Ok(plaintext)
    }
}

/// Constant-time comparison
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
        assert_eq!(ciphertext.len(), plaintext.len() + 32);

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
}
