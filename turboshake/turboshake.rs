//! TurboSHAKE128 and TurboSHAKE256 implementation (RFC 9861)
//!
//! TurboSHAKE is a family of hash functions based on the Keccak permutation
//! reduced to 12 rounds for improved performance.

use core::mem;

// Compile-time check for little-endian architecture
// Keccak state interpretation assumes little-endian byte order
#[cfg(not(target_endian = "little"))]
compile_error!("This crate requires a little-endian architecture");

/// State size in bytes (1600 bits)
const STATE_SIZE: usize = 200;

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

/// XOR data into state at given position
#[inline(always)]
fn xor_into_state(state: &mut [u64; 25], data: &[u8], position: usize) {
    let state_bytes = state_as_bytes_mut(state);
    for (i, &byte) in data.iter().enumerate() {
        state_bytes[position + i] ^= byte;
    }
}

/// TurboSHAKE128 hasher (RFC 9861)
/// Rate: 168 bytes, Capacity: 32 bytes
pub struct TurboShake128 {
    state: [u64; 25],
    position: usize,
    domain_sep: u8,
}

impl TurboShake128 {
    /// Rate in bytes (1344 bits / 8)
    pub const RATE: usize = 168;

    /// Create a new TurboSHAKE128 instance with default domain separation (0x1F)
    pub fn new() -> Self {
        Self::with_domain_sep(0x1F)
    }

    /// Create a new TurboSHAKE128 instance with custom domain separation byte
    /// Domain separation byte must be in range 0x01..=0x7F
    pub fn with_domain_sep(domain_sep: u8) -> Self {
        debug_assert!(
            domain_sep >= 0x01 && domain_sep <= 0x7F,
            "Domain separation byte must be in range 0x01..=0x7F"
        );
        Self {
            state: [0u64; 25],
            position: 0,
            domain_sep,
        }
    }

    /// Absorb input data
    #[inline]
    pub fn update(&mut self, data: &[u8]) {
        let mut offset = 0;
        while offset < data.len() {
            let to_copy = (data.len() - offset).min(Self::RATE - self.position);
            xor_into_state(&mut self.state, &data[offset..offset + to_copy], self.position);
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
        // Apply domain separation byte and padding
        let state_bytes = state_as_bytes_mut(&mut self.state);
        state_bytes[self.position] ^= self.domain_sep;
        state_bytes[Self::RATE - 1] ^= 0x80;

        keccak::p1600(&mut self.state, 12);

        // Squeeze output
        let mut offset = 0;
        while offset < output.len() {
            let state_bytes = state_as_bytes(&self.state);
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

    /// Hash with custom domain separation byte
    pub fn hash_with_domain_sep(data: &[u8], domain_sep: u8, output: &mut [u8]) {
        let mut hasher = Self::with_domain_sep(domain_sep);
        hasher.update(data);
        hasher.finalize(output);
    }
}

impl Default for TurboShake128 {
    fn default() -> Self {
        Self::new()
    }
}

/// TurboSHAKE256 hasher (RFC 9861)
/// Rate: 136 bytes, Capacity: 64 bytes
pub struct TurboShake256 {
    state: [u64; 25],
    position: usize,
    domain_sep: u8,
}

impl TurboShake256 {
    /// Rate in bytes (1088 bits / 8)
    pub const RATE: usize = 136;

    /// Create a new TurboSHAKE256 instance with default domain separation (0x1F)
    pub fn new() -> Self {
        Self::with_domain_sep(0x1F)
    }

    /// Create a new TurboSHAKE256 instance with custom domain separation byte
    /// Domain separation byte must be in range 0x01..=0x7F
    pub fn with_domain_sep(domain_sep: u8) -> Self {
        debug_assert!(
            domain_sep >= 0x01 && domain_sep <= 0x7F,
            "Domain separation byte must be in range 0x01..=0x7F"
        );
        Self {
            state: [0u64; 25],
            position: 0,
            domain_sep,
        }
    }

    /// Absorb input data
    #[inline]
    pub fn update(&mut self, data: &[u8]) {
        let mut offset = 0;
        while offset < data.len() {
            let to_copy = (data.len() - offset).min(Self::RATE - self.position);
            xor_into_state(&mut self.state, &data[offset..offset + to_copy], self.position);
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
        // Apply domain separation byte and padding
        let state_bytes = state_as_bytes_mut(&mut self.state);
        state_bytes[self.position] ^= self.domain_sep;
        state_bytes[Self::RATE - 1] ^= 0x80;

        keccak::p1600(&mut self.state, 12);

        // Squeeze output
        let mut offset = 0;
        while offset < output.len() {
            let state_bytes = state_as_bytes(&self.state);
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

    /// Hash with custom domain separation byte
    pub fn hash_with_domain_sep(data: &[u8], domain_sep: u8, output: &mut [u8]) {
        let mut hasher = Self::with_domain_sep(domain_sep);
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

    // Helper function to generate pattern data as per RFC 9861
    fn pattern(len: usize) -> Vec<u8> {
        (0..len).map(|i| (i % 251) as u8).collect()
    }

    // ===== TurboSHAKE128 tests from RFC 9861 =====

    #[test]
    fn test_turboshake128_empty() {
        // RFC 9861: TurboSHAKE128(M=`00`^0, D=`1F`, 32)
        // 1E 41 5F 1C 59 83 AF F2 16 92 17 27 7D 17 BB 53
        // 8C D9 45 A3 97 DD EC 54 1F 1C E4 1A F2 C1 B7 4C
        let expected: [u8; 32] = [
            0x1E, 0x41, 0x5F, 0x1C, 0x59, 0x83, 0xAF, 0xF2,
            0x16, 0x92, 0x17, 0x27, 0x7D, 0x17, 0xBB, 0x53,
            0x8C, 0xD9, 0x45, 0xA3, 0x97, 0xDD, 0xEC, 0x54,
            0x1F, 0x1C, 0xE4, 0x1A, 0xF2, 0xC1, 0xB7, 0x4C,
        ];
        let mut output = [0u8; 32];
        TurboShake128::hash(&[], &mut output);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_turboshake128_ptn_1() {
        // RFC 9861: TurboSHAKE128(M=ptn(17**0 bytes), D=`1F`, 32)
        // 55 CE DD 6F 60 AF 7B B2 9A 40 42 AE 83 2E F3 F5
        // 8D B7 29 9F 89 3E BB 92 47 24 7D 85 69 58 DA A9
        let expected: [u8; 32] = [
            0x55, 0xCE, 0xDD, 0x6F, 0x60, 0xAF, 0x7B, 0xB2,
            0x9A, 0x40, 0x42, 0xAE, 0x83, 0x2E, 0xF3, 0xF5,
            0x8D, 0xB7, 0x29, 0x9F, 0x89, 0x3E, 0xBB, 0x92,
            0x47, 0x24, 0x7D, 0x85, 0x69, 0x58, 0xDA, 0xA9,
        ];
        let mut output = [0u8; 32];
        TurboShake128::hash(&pattern(1), &mut output);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_turboshake128_ptn_17() {
        // RFC 9861: TurboSHAKE128(M=ptn(17**1 bytes), D=`1F`, 32)
        // 9C 97 D0 36 A3 BA C8 19 DB 70 ED E0 CA 55 4E C6
        // E4 C2 A1 A4 FF BF D9 EC 26 9C A6 A1 11 16 12 33
        let expected: [u8; 32] = [
            0x9C, 0x97, 0xD0, 0x36, 0xA3, 0xBA, 0xC8, 0x19,
            0xDB, 0x70, 0xED, 0xE0, 0xCA, 0x55, 0x4E, 0xC6,
            0xE4, 0xC2, 0xA1, 0xA4, 0xFF, 0xBF, 0xD9, 0xEC,
            0x26, 0x9C, 0xA6, 0xA1, 0x11, 0x16, 0x12, 0x33,
        ];
        let mut output = [0u8; 32];
        TurboShake128::hash(&pattern(17), &mut output);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_turboshake128_ptn_289() {
        // RFC 9861: TurboSHAKE128(M=ptn(17**2 bytes), D=`1F`, 32)
        // 96 C7 7C 27 9E 01 26 F7 FC 07 C9 B0 7F 5C DA E1
        // E0 BE 60 BD BE 10 62 00 40 E7 5D 72 23 A6 24 D2
        let expected: [u8; 32] = [
            0x96, 0xC7, 0x7C, 0x27, 0x9E, 0x01, 0x26, 0xF7,
            0xFC, 0x07, 0xC9, 0xB0, 0x7F, 0x5C, 0xDA, 0xE1,
            0xE0, 0xBE, 0x60, 0xBD, 0xBE, 0x10, 0x62, 0x00,
            0x40, 0xE7, 0x5D, 0x72, 0x23, 0xA6, 0x24, 0xD2,
        ];
        let mut output = [0u8; 32];
        TurboShake128::hash(&pattern(289), &mut output);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_turboshake128_domain_sep_01() {
        // RFC 9861: TurboSHAKE128(M=`FF FF FF`, D=`01`, 32)
        // BF 32 3F 94 04 94 E8 8E E1 C5 40 FE 66 0B E8 A0
        // C9 3F 43 D1 5E C0 06 99 84 62 FA 99 4E ED 5D AB
        let expected: [u8; 32] = [
            0xBF, 0x32, 0x3F, 0x94, 0x04, 0x94, 0xE8, 0x8E,
            0xE1, 0xC5, 0x40, 0xFE, 0x66, 0x0B, 0xE8, 0xA0,
            0xC9, 0x3F, 0x43, 0xD1, 0x5E, 0xC0, 0x06, 0x99,
            0x84, 0x62, 0xFA, 0x99, 0x4E, 0xED, 0x5D, 0xAB,
        ];
        let mut output = [0u8; 32];
        TurboShake128::hash_with_domain_sep(&[0xFF, 0xFF, 0xFF], 0x01, &mut output);
        assert_eq!(output, expected);
    }

    // ===== TurboSHAKE256 tests from RFC 9861 =====

    #[test]
    fn test_turboshake256_empty_64() {
        // RFC 9861: TurboSHAKE256(M=`00`^0, D=`1F`, 64)
        // 36 7A 32 9D AF EA 87 1C 78 02 EC 67 F9 05 AE 13
        // C5 76 95 DC 2C 66 63 C6 10 35 F5 9A 18 F8 E7 DB
        // 11 ED C0 E1 2E 91 EA 60 EB 6B 32 DF 06 DD 7F 00
        // 2F BA FA BB 6E 13 EC 1C C2 0D 99 55 47 60 0D B0
        let expected: [u8; 64] = [
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
        assert_eq!(output, expected);
    }

    #[test]
    fn test_turboshake256_ptn_1() {
        // RFC 9861: TurboSHAKE256(M=ptn(17**0 bytes), D=`1F`, 64)
        let expected: [u8; 64] = [
            0x3E, 0x17, 0x12, 0xF9, 0x28, 0xF8, 0xEA, 0xF1,
            0x05, 0x46, 0x32, 0xB2, 0xAA, 0x0A, 0x24, 0x6E,
            0xD8, 0xB0, 0xC3, 0x78, 0x72, 0x8F, 0x60, 0xBC,
            0x97, 0x04, 0x10, 0x15, 0x5C, 0x28, 0x82, 0x0E,
            0x90, 0xCC, 0x90, 0xD8, 0xA3, 0x00, 0x6A, 0xA2,
            0x37, 0x2C, 0x5C, 0x5E, 0xA1, 0x76, 0xB0, 0x68,
            0x2B, 0xF2, 0x2B, 0xAE, 0x74, 0x67, 0xAC, 0x94,
            0xF7, 0x4D, 0x43, 0xD3, 0x9B, 0x04, 0x82, 0xE2,
        ];
        let mut output = [0u8; 64];
        TurboShake256::hash(&pattern(1), &mut output);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_turboshake256_ptn_17() {
        // RFC 9861: TurboSHAKE256(M=ptn(17**1 bytes), D=`1F`, 64)
        let expected: [u8; 64] = [
            0xB3, 0xBA, 0xB0, 0x30, 0x0E, 0x6A, 0x19, 0x1F,
            0xBE, 0x61, 0x37, 0x93, 0x98, 0x35, 0x92, 0x35,
            0x78, 0x79, 0x4E, 0xA5, 0x48, 0x43, 0xF5, 0x01,
            0x10, 0x90, 0xFA, 0x2F, 0x37, 0x80, 0xA9, 0xE5,
            0xCB, 0x22, 0xC5, 0x9D, 0x78, 0xB4, 0x0A, 0x0F,
            0xBF, 0xF9, 0xE6, 0x72, 0xC0, 0xFB, 0xE0, 0x97,
            0x0B, 0xD2, 0xC8, 0x45, 0x09, 0x1C, 0x60, 0x44,
            0xD6, 0x87, 0x05, 0x4D, 0xA5, 0xD8, 0xE9, 0xC7,
        ];
        let mut output = [0u8; 64];
        TurboShake256::hash(&pattern(17), &mut output);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_turboshake256_ptn_289() {
        // RFC 9861: TurboSHAKE256(M=ptn(17**2 bytes), D=`1F`, 64)
        let expected: [u8; 64] = [
            0x66, 0xB8, 0x10, 0xDB, 0x8E, 0x90, 0x78, 0x04,
            0x24, 0xC0, 0x84, 0x73, 0x72, 0xFD, 0xC9, 0x57,
            0x10, 0x88, 0x2F, 0xDE, 0x31, 0xC6, 0xDF, 0x75,
            0xBE, 0xB9, 0xD4, 0xCD, 0x93, 0x05, 0xCF, 0xCA,
            0xE3, 0x5E, 0x7B, 0x83, 0xE8, 0xB7, 0xE6, 0xEB,
            0x4B, 0x78, 0x60, 0x58, 0x80, 0x11, 0x63, 0x16,
            0xFE, 0x2C, 0x07, 0x8A, 0x09, 0xB9, 0x4A, 0xD7,
            0xB8, 0x21, 0x3C, 0x0A, 0x73, 0x8B, 0x65, 0xC0,
        ];
        let mut output = [0u8; 64];
        TurboShake256::hash(&pattern(289), &mut output);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_turboshake256_domain_sep_01() {
        // RFC 9861: TurboSHAKE256(M=`FF FF FF`, D=`01`, 64)
        let expected: [u8; 64] = [
            0xD2, 0x1C, 0x6F, 0xBB, 0xF5, 0x87, 0xFA, 0x22,
            0x82, 0xF2, 0x9A, 0xEA, 0x62, 0x01, 0x75, 0xFB,
            0x02, 0x57, 0x41, 0x3A, 0xF7, 0x8A, 0x0B, 0x1B,
            0x2A, 0x87, 0x41, 0x9C, 0xE0, 0x31, 0xD9, 0x33,
            0xAE, 0x7A, 0x4D, 0x38, 0x33, 0x27, 0xA8, 0xA1,
            0x76, 0x41, 0xA3, 0x4F, 0x8A, 0x1D, 0x10, 0x03,
            0xAD, 0x7D, 0xA6, 0xB7, 0x2D, 0xBA, 0x84, 0xBB,
            0x62, 0xFE, 0xF2, 0x8F, 0x62, 0xF1, 0x24, 0x24,
        ];
        let mut output = [0u8; 64];
        TurboShake256::hash_with_domain_sep(&[0xFF, 0xFF, 0xFF], 0x01, &mut output);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_turboshake256_domain_sep_0b() {
        // RFC 9861: TurboSHAKE256(M=`FF FF FF FF FF FF FF`, D=`0B`, 64)
        let expected: [u8; 64] = [
            0xBB, 0x36, 0x76, 0x49, 0x51, 0xEC, 0x97, 0xE9,
            0xD8, 0x5F, 0x7E, 0xE9, 0xA6, 0x7A, 0x77, 0x18,
            0xFC, 0x00, 0x5C, 0xF4, 0x25, 0x56, 0xBE, 0x79,
            0xCE, 0x12, 0xC0, 0xBD, 0xE5, 0x0E, 0x57, 0x36,
            0xD6, 0x63, 0x2B, 0x0D, 0x0D, 0xFB, 0x20, 0x2D,
            0x1B, 0xBB, 0x8F, 0xFE, 0x3D, 0xD7, 0x4C, 0xB0,
            0x08, 0x34, 0xFA, 0x75, 0x6C, 0xB0, 0x34, 0x71,
            0xBA, 0xB1, 0x3A, 0x1E, 0x2C, 0x16, 0xB3, 0xC0,
        ];
        let mut output = [0u8; 64];
        TurboShake256::hash_with_domain_sep(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF], 0x0B, &mut output);
        assert_eq!(output, expected);
    }

    // ===== Additional tests =====

    #[test]
    fn test_turboshake256_incremental() {
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
    }

    #[test]
    fn test_turboshake128_incremental() {
        let data = b"The quick brown fox jumps over the lazy dog";

        let mut output1 = [0u8; 128];
        TurboShake128::hash(data, &mut output1);

        let mut hasher = TurboShake128::new();
        hasher.update(&data[..20]);
        hasher.update(&data[20..]);
        let mut output2 = [0u8; 128];
        hasher.finalize(&mut output2);

        assert_eq!(output1, output2, "Incremental hashing should match one-shot");
    }

    #[test]
    fn test_turboshake_different_outputs() {
        let data = b"test";
        let mut out128 = [0u8; 32];
        let mut out256 = [0u8; 32];

        TurboShake128::hash(data, &mut out128);
        TurboShake256::hash(data, &mut out256);

        assert_ne!(out128, out256, "TurboSHAKE128 and 256 should produce different outputs");
    }

    #[test]
    fn test_turboshake_large_input() {
        let data = vec![0x42u8; 10000];
        let mut output = [0u8; 64];

        TurboShake256::hash(&data, &mut output);

        // Verify it doesn't panic and produces non-zero output
        assert!(output.iter().any(|&x| x != 0));
    }
}
