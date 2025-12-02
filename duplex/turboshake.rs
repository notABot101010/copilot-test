//! TurboSHAKE256 implementation (RFC 9861)
//!
//! TurboSHAKE256 is a hash function based on the Keccak permutation
//! reduced to 12 rounds for improved performance.

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

/// TurboSHAKE256 hasher (RFC 9861)
pub struct TurboShake256 {
    state: [u64; 25],
    position: usize,
    absorbing: bool,
}

impl TurboShake256 {
    const RATE: usize = 136;
    const DOMAIN_SEP: u8 = 0x1F;

    /// Create a new TurboSHAKE256 instance
    pub fn new() -> Self {
        Self {
            state: [0u64; 25],
            position: 0,
            absorbing: true,
        }
    }

    /// Absorb input data
    pub fn update(&mut self, data: &[u8]) {
        assert!(self.absorbing, "Cannot absorb after squeezing");

        let mut offset = 0;
        while offset < data.len() {
            let to_copy = (data.len() - offset).min(Self::RATE - self.position);

            let state_bytes = state_to_bytes(&self.state);
            let mut new_state = state_bytes;

            for i in 0..to_copy {
                new_state[self.position + i] ^= data[offset + i];
            }

            self.state = bytes_to_state(&new_state);
            self.position += to_copy;
            offset += to_copy;

            if self.position == Self::RATE {
                keccak::p1600(&mut self.state, 12);
                self.position = 0;
            }
        }
    }

    /// Finalize and squeeze output
    pub fn finalize(mut self, output: &mut [u8]) {
        // Pad and apply domain separation
        let state_bytes = state_to_bytes(&self.state);
        let mut new_state = state_bytes;

        // Apply domain separation byte
        new_state[self.position] ^= Self::DOMAIN_SEP;
        new_state[Self::RATE - 1] ^= 0x80;

        self.state = bytes_to_state(&new_state);
        keccak::p1600(&mut self.state, 12);

        // Squeeze output
        let mut offset = 0;
        while offset < output.len() {
            let state_bytes = state_to_bytes(&self.state);
            let to_copy = (output.len() - offset).min(Self::RATE);
            output[offset..offset + to_copy].copy_from_slice(&state_bytes[..to_copy]);
            offset += to_copy;

            if offset < output.len() {
                keccak::p1600(&mut self.state, 12);
            }
        }
    }

    /// Convenience function to hash data in one call
    pub fn hash(data: &[u8], output: &mut [u8]) {
        let mut hasher = Self::new();
        hasher.update(data);
        hasher.finalize(output);
    }
}

impl Default for TurboShake256 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_turboshake256_empty_rfc9861() {
        // RFC 9861 test vector: empty message, 64-byte output
        let expected = [
            0x36, 0x7A, 0x32, 0x9D, 0xAF, 0xEA, 0x87, 0x1C,
            0x78, 0x02, 0xEC, 0x67, 0xF9, 0x05, 0xAE, 0x13,
            0xC5, 0x76, 0x95, 0xDC, 0x2C, 0x66, 0x63, 0xC6,
            0x10, 0x35, 0xF5, 0x9A, 0x18, 0xF8, 0xE7, 0xDB,
            0x11, 0xED, 0xC0, 0xE1, 0x2E, 0x91, 0xEA, 0x60,
            0xEB, 0x6B, 0x32, 0xDF, 0x06, 0xDD, 0x7F, 0x00,
            0x2F, 0xBA, 0xFA, 0xBB, 0x6E, 0x13, 0xEC, 0x1C,
            0xC2, 0x0D, 0x99, 0x55, 0x47, 0x60, 0x0D, 0xB0,
        ];

        let mut output = [0u8; 64];
        TurboShake256::hash(&[], &mut output);

        assert_eq!(output, expected, "TurboSHAKE256 empty message test failed");
    }

    #[test]
    fn test_turboshake256_incremental() {
        // Test that incremental hashing matches one-shot
        let data = b"The quick brown fox jumps over the lazy dog";

        let mut output1 = [0u8; 128];
        TurboShake256::hash(data, &mut output1);

        let mut hasher = TurboShake256::new();
        hasher.update(&data[..20]);
        hasher.update(&data[20..]);
        let mut output2 = [0u8; 128];
        hasher.finalize(&mut output2);

        assert_eq!(output1, output2, "Incremental hashing should match one-shot");
    }

    #[test]
    fn test_turboshake256_various_lengths() {
        // Test different output lengths
        let data = b"test data";

        let mut out16 = [0u8; 16];
        let mut out32 = [0u8; 32];
        let mut out128 = [0u8; 128];
        let mut out256 = [0u8; 256];

        TurboShake256::hash(data, &mut out16);
        TurboShake256::hash(data, &mut out32);
        TurboShake256::hash(data, &mut out128);
        TurboShake256::hash(data, &mut out256);

        // Shorter outputs should be prefixes of longer outputs
        assert_eq!(&out32[..16], &out16[..]);
        assert_eq!(&out128[..32], &out32[..]);
        assert_eq!(&out256[..128], &out128[..]);

        // Verify non-zero output
        assert!(out16.iter().any(|&x| x != 0));
        assert!(out256.iter().any(|&x| x != 0));
    }

    #[test]
    fn test_turboshake256_different_inputs() {
        // Different inputs should produce different outputs
        let mut out1 = [0u8; 64];
        let mut out2 = [0u8; 64];
        let mut out3 = [0u8; 64];

        TurboShake256::hash(b"", &mut out1);
        TurboShake256::hash(b"a", &mut out2);
        TurboShake256::hash(b"ab", &mut out3);

        assert_ne!(out1, out2);
        assert_ne!(out2, out3);
        assert_ne!(out1, out3);
    }

    #[test]
    fn test_turboshake256_deterministic() {
        // Same input should always produce same output
        let data = b"determinism test";
        let mut out1 = [0u8; 96];
        let mut out2 = [0u8; 96];

        TurboShake256::hash(data, &mut out1);
        TurboShake256::hash(data, &mut out2);

        assert_eq!(out1, out2);
    }

    #[test]
    fn test_turboshake256_large_input() {
        // Test with large input
        let data = vec![0x42u8; 10000];
        let mut output = [0u8; 64];

        TurboShake256::hash(&data, &mut output);

        // Just verify it doesn't panic and produces non-zero output
        assert!(output.iter().any(|&x| x != 0));
    }

    #[test]
    fn test_turboshake256_boundary_rate() {
        // Test input at rate boundary (136 bytes)
        let data = vec![0xAAu8; 136];
        let mut output1 = [0u8; 32];

        TurboShake256::hash(&data, &mut output1);

        // Test input just over rate boundary
        let data2 = vec![0xAAu8; 137];
        let mut output2 = [0u8; 32];

        TurboShake256::hash(&data2, &mut output2);

        assert_ne!(output1, output2);
    }

    #[test]
    fn test_turboshake256_multiple_blocks() {
        // Test hashing data that spans multiple rate blocks
        let data = vec![0x5Au8; 500];
        let mut output = [0u8; 64];

        TurboShake256::hash(&data, &mut output);

        // Verify non-zero and deterministic
        assert!(output.iter().any(|&x| x != 0));

        let mut output2 = [0u8; 64];
        TurboShake256::hash(&data, &mut output2);
        assert_eq!(output, output2);
    }

    #[test]
    fn test_turboshake256_squeeze_multiple_blocks() {
        // Test squeezing output larger than rate (136 bytes)
        let data = b"test";
        let mut output = [0u8; 400];

        TurboShake256::hash(data, &mut output);

        // Verify all bytes are filled and not all zero
        assert!(output.iter().any(|&x| x != 0));
        assert!(output[300..].iter().any(|&x| x != 0)); // Check end of output
    }
}
