//! KangarooTwelve (KT128 and KT256) implementation (RFC 9861)
//!
//! KangarooTwelve applies tree hashing on top of TurboSHAKE for parallel processing.

use crate::turboshake::{TurboShake128, TurboShake256};

/// Encode length as per RFC 9861 section 3.3
/// Returns the encoded length and the number of bytes used
pub fn length_encode(x: usize) -> ([u8; 9], usize) {
    let mut result = [0u8; 9];
    if x == 0 {
        result[0] = 0x00;
        return (result, 1);
    }

    let mut n = 0;
    let mut val = x;
    while val > 0 {
        result[n] = (val & 0xFF) as u8;
        val >>= 8;
        n += 1;
    }
    // Reverse to get big-endian order for the length bytes
    result[..n].reverse();
    result[n] = n as u8;
    (result, n + 1)
}

/// KangarooTwelve 128-bit security (KT128) - RFC 9861
/// Uses TurboSHAKE128 with tree hashing for parallel processing
pub struct KT128;

impl KT128 {
    /// Chunk size for tree hashing (8192 bytes)
    const CHUNK_SIZE: usize = 8192;
    /// Chaining value size (32 bytes)
    const CV_SIZE: usize = 32;

    /// Hash with customization string
    pub fn hash(message: &[u8], custom: &[u8], output: &mut [u8]) {
        // S = M || C || length_encode(|C|)
        let (len_enc, len_enc_size) = length_encode(custom.len());
        let s_len = message.len() + custom.len() + len_enc_size;

        if s_len <= Self::CHUNK_SIZE {
            // Single node mode
            let mut hasher = TurboShake128::with_domain_sep(0x07);
            hasher.update(message);
            hasher.update(custom);
            hasher.update(&len_enc[..len_enc_size]);
            hasher.finalize(output);
        } else {
            // Tree hashing mode
            Self::tree_hash(message, custom, &len_enc[..len_enc_size], output);
        }
    }

    fn tree_hash(message: &[u8], custom: &[u8], len_enc: &[u8], output: &mut [u8]) {
        // Build S conceptually as: message || custom || len_enc
        let total_len = message.len() + custom.len() + len_enc.len();

        // Calculate number of chunks
        let num_chunks = (total_len + Self::CHUNK_SIZE - 1) / Self::CHUNK_SIZE;

        // FinalNode = S_0 || 0x03 || 0x00^7
        let mut final_hasher = TurboShake128::with_domain_sep(0x06);

        // Process first chunk (S_0)
        let first_chunk_end = Self::CHUNK_SIZE.min(total_len);
        Self::feed_s_range(&mut final_hasher, message, custom, len_enc, 0, first_chunk_end);

        // Add marker after first chunk
        final_hasher.update(&[0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

        // Process remaining chunks and compute chaining values
        for i in 1..num_chunks {
            let start = i * Self::CHUNK_SIZE;
            let end = ((i + 1) * Self::CHUNK_SIZE).min(total_len);

            // CV_i = TurboSHAKE128(S_i, 0x0B, 32)
            let mut cv_hasher = TurboShake128::with_domain_sep(0x0B);
            Self::feed_s_range(&mut cv_hasher, message, custom, len_enc, start, end);
            let mut cv = [0u8; Self::CV_SIZE];
            cv_hasher.finalize(&mut cv);

            final_hasher.update(&cv);
        }

        // Add length_encode(n-1) || 0xFF 0xFF
        let (num_blocks_enc, num_blocks_enc_size) = length_encode(num_chunks - 1);
        final_hasher.update(&num_blocks_enc[..num_blocks_enc_size]);
        final_hasher.update(&[0xFF, 0xFF]);

        final_hasher.finalize(output);
    }

    /// Feed a range from the conceptual S = message || custom || len_enc into hasher
    fn feed_s_range(
        hasher: &mut TurboShake128,
        message: &[u8],
        custom: &[u8],
        len_enc: &[u8],
        start: usize,
        end: usize,
    ) {
        let msg_len = message.len();
        let custom_len = custom.len();
        let len_enc_len = len_enc.len();

        let mut pos = start;
        while pos < end {
            if pos < msg_len {
                // Still in message part
                let chunk_end = end.min(msg_len);
                hasher.update(&message[pos..chunk_end]);
                pos = chunk_end;
            } else if pos < msg_len + custom_len {
                // In custom part
                let offset = pos - msg_len;
                let chunk_end = (end - msg_len).min(custom_len);
                hasher.update(&custom[offset..chunk_end]);
                pos = msg_len + chunk_end;
            } else {
                // In len_enc part
                let offset = pos - msg_len - custom_len;
                let chunk_end = (end - msg_len - custom_len).min(len_enc_len);
                hasher.update(&len_enc[offset..chunk_end]);
                pos = msg_len + custom_len + chunk_end;
            }
        }
    }
}

/// KangarooTwelve 256-bit security (KT256) - RFC 9861
/// Uses TurboSHAKE256 with tree hashing for parallel processing
pub struct KT256;

impl KT256 {
    /// Chunk size for tree hashing (8192 bytes)
    const CHUNK_SIZE: usize = 8192;
    /// Chaining value size (64 bytes for 256-bit security)
    const CV_SIZE: usize = 64;

    /// Hash with customization string
    pub fn hash(message: &[u8], custom: &[u8], output: &mut [u8]) {
        // S = M || C || length_encode(|C|)
        let (len_enc, len_enc_size) = length_encode(custom.len());
        let s_len = message.len() + custom.len() + len_enc_size;

        if s_len <= Self::CHUNK_SIZE {
            // Single node mode
            let mut hasher = TurboShake256::with_domain_sep(0x07);
            hasher.update(message);
            hasher.update(custom);
            hasher.update(&len_enc[..len_enc_size]);
            hasher.finalize(output);
        } else {
            // Tree hashing mode
            Self::tree_hash(message, custom, &len_enc[..len_enc_size], output);
        }
    }

    fn tree_hash(message: &[u8], custom: &[u8], len_enc: &[u8], output: &mut [u8]) {
        // Build S conceptually as: message || custom || len_enc
        let total_len = message.len() + custom.len() + len_enc.len();

        // Calculate number of chunks
        let num_chunks = (total_len + Self::CHUNK_SIZE - 1) / Self::CHUNK_SIZE;

        // FinalNode = S_0 || 0x03 || 0x00^7
        let mut final_hasher = TurboShake256::with_domain_sep(0x06);

        // Process first chunk (S_0)
        let first_chunk_end = Self::CHUNK_SIZE.min(total_len);
        Self::feed_s_range(&mut final_hasher, message, custom, len_enc, 0, first_chunk_end);

        // Add marker after first chunk
        final_hasher.update(&[0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

        // Process remaining chunks and compute chaining values
        for i in 1..num_chunks {
            let start = i * Self::CHUNK_SIZE;
            let end = ((i + 1) * Self::CHUNK_SIZE).min(total_len);

            // CV_i = TurboSHAKE256(S_i, 0x0B, 64)
            let mut cv_hasher = TurboShake256::with_domain_sep(0x0B);
            Self::feed_s_range(&mut cv_hasher, message, custom, len_enc, start, end);
            let mut cv = [0u8; Self::CV_SIZE];
            cv_hasher.finalize(&mut cv);

            final_hasher.update(&cv);
        }

        // Add length_encode(n-1) || 0xFF 0xFF
        let (num_blocks_enc, num_blocks_enc_size) = length_encode(num_chunks - 1);
        final_hasher.update(&num_blocks_enc[..num_blocks_enc_size]);
        final_hasher.update(&[0xFF, 0xFF]);

        final_hasher.finalize(output);
    }

    /// Feed a range from the conceptual S = message || custom || len_enc into hasher
    fn feed_s_range(
        hasher: &mut TurboShake256,
        message: &[u8],
        custom: &[u8],
        len_enc: &[u8],
        start: usize,
        end: usize,
    ) {
        let msg_len = message.len();
        let custom_len = custom.len();
        let len_enc_len = len_enc.len();

        let mut pos = start;
        while pos < end {
            if pos < msg_len {
                // Still in message part
                let chunk_end = end.min(msg_len);
                hasher.update(&message[pos..chunk_end]);
                pos = chunk_end;
            } else if pos < msg_len + custom_len {
                // In custom part
                let offset = pos - msg_len;
                let chunk_end = (end - msg_len).min(custom_len);
                hasher.update(&custom[offset..chunk_end]);
                pos = msg_len + chunk_end;
            } else {
                // In len_enc part
                let offset = pos - msg_len - custom_len;
                let chunk_end = (end - msg_len - custom_len).min(len_enc_len);
                hasher.update(&len_enc[offset..chunk_end]);
                pos = msg_len + custom_len + chunk_end;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to generate pattern data as per RFC 9861
    fn pattern(len: usize) -> Vec<u8> {
        (0..len).map(|i| (i % 251) as u8).collect()
    }

    #[test]
    fn test_length_encode() {
        // length_encode(0) = `00`
        let (enc, len) = length_encode(0);
        assert_eq!(len, 1);
        assert_eq!(enc[0], 0x00);

        // length_encode(12) = `0C 01`
        let (enc, len) = length_encode(12);
        assert_eq!(len, 2);
        assert_eq!(&enc[..len], &[0x0C, 0x01]);

        // length_encode(65538) = `01 00 02 03`
        let (enc, len) = length_encode(65538);
        assert_eq!(len, 4);
        assert_eq!(&enc[..len], &[0x01, 0x00, 0x02, 0x03]);
    }

    // ===== KT128 tests from RFC 9861 =====

    #[test]
    fn test_kt128_empty() {
        // RFC 9861: KT128(M=`00`^0, C=`00`^0, 32)
        // 1A C2 D4 50 FC 3B 42 05 D1 9D A7 BF CA 1B 37 51
        // 3C 08 03 57 7A C7 16 7F 06 FE 2C E1 F0 EF 39 E5
        let expected: [u8; 32] = [
            0x1A, 0xC2, 0xD4, 0x50, 0xFC, 0x3B, 0x42, 0x05,
            0xD1, 0x9D, 0xA7, 0xBF, 0xCA, 0x1B, 0x37, 0x51,
            0x3C, 0x08, 0x03, 0x57, 0x7A, 0xC7, 0x16, 0x7F,
            0x06, 0xFE, 0x2C, 0xE1, 0xF0, 0xEF, 0x39, 0xE5,
        ];
        let mut output = [0u8; 32];
        KT128::hash(&[], &[], &mut output);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_kt128_ptn_1() {
        // RFC 9861: KT128(M=ptn(1 bytes), C=`00`^0, 32)
        // 2B DA 92 45 0E 8B 14 7F 8A 7C B6 29 E7 84 A0 58
        // EF CA 7C F7 D8 21 8E 02 D3 45 DF AA 65 24 4A 1F
        let expected: [u8; 32] = [
            0x2B, 0xDA, 0x92, 0x45, 0x0E, 0x8B, 0x14, 0x7F,
            0x8A, 0x7C, 0xB6, 0x29, 0xE7, 0x84, 0xA0, 0x58,
            0xEF, 0xCA, 0x7C, 0xF7, 0xD8, 0x21, 0x8E, 0x02,
            0xD3, 0x45, 0xDF, 0xAA, 0x65, 0x24, 0x4A, 0x1F,
        ];
        let mut output = [0u8; 32];
        KT128::hash(&pattern(1), &[], &mut output);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_kt128_ptn_17() {
        // RFC 9861: KT128(M=ptn(17 bytes), C=`00`^0, 32)
        // 6B F7 5F A2 23 91 98 DB 47 72 E3 64 78 F8 E1 9B
        // 0F 37 12 05 F6 A9 A9 3A 27 3F 51 DF 37 12 28 88
        let expected: [u8; 32] = [
            0x6B, 0xF7, 0x5F, 0xA2, 0x23, 0x91, 0x98, 0xDB,
            0x47, 0x72, 0xE3, 0x64, 0x78, 0xF8, 0xE1, 0x9B,
            0x0F, 0x37, 0x12, 0x05, 0xF6, 0xA9, 0xA9, 0x3A,
            0x27, 0x3F, 0x51, 0xDF, 0x37, 0x12, 0x28, 0x88,
        ];
        let mut output = [0u8; 32];
        KT128::hash(&pattern(17), &[], &mut output);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_kt128_ptn_289() {
        // RFC 9861: KT128(M=ptn(17**2 bytes), C=`00`^0, 32)
        // 0C 31 5E BC DE DB F6 14 26 DE 7D CF 8F B7 25 D1
        // E7 46 75 D7 F5 32 7A 50 67 F3 67 B1 08 EC B6 7C
        let expected: [u8; 32] = [
            0x0C, 0x31, 0x5E, 0xBC, 0xDE, 0xDB, 0xF6, 0x14,
            0x26, 0xDE, 0x7D, 0xCF, 0x8F, 0xB7, 0x25, 0xD1,
            0xE7, 0x46, 0x75, 0xD7, 0xF5, 0x32, 0x7A, 0x50,
            0x67, 0xF3, 0x67, 0xB1, 0x08, 0xEC, 0xB6, 0x7C,
        ];
        let mut output = [0u8; 32];
        KT128::hash(&pattern(289), &[], &mut output);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_kt128_ptn_4913() {
        // RFC 9861: KT128(M=ptn(17**3 bytes), C=`00`^0, 32)
        // CB 55 2E 2E C7 7D 99 10 70 1D 57 8B 45 7D DF 77
        // 2C 12 E3 22 E4 EE 7F E4 17 F9 2C 75 8F 0D 59 D0
        let expected: [u8; 32] = [
            0xCB, 0x55, 0x2E, 0x2E, 0xC7, 0x7D, 0x99, 0x10,
            0x70, 0x1D, 0x57, 0x8B, 0x45, 0x7D, 0xDF, 0x77,
            0x2C, 0x12, 0xE3, 0x22, 0xE4, 0xEE, 0x7F, 0xE4,
            0x17, 0xF9, 0x2C, 0x75, 0x8F, 0x0D, 0x59, 0xD0,
        ];
        let mut output = [0u8; 32];
        KT128::hash(&pattern(4913), &[], &mut output);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_kt128_ptn_83521() {
        // RFC 9861: KT128(M=ptn(17**4 bytes), C=`00`^0, 32)
        // 87 01 04 5E 22 20 53 45 FF 4D DA 05 55 5C BB 5C
        // 3A F1 A7 71 C2 B8 9B AE F3 7D B4 3D 99 98 B9 FE
        let expected: [u8; 32] = [
            0x87, 0x01, 0x04, 0x5E, 0x22, 0x20, 0x53, 0x45,
            0xFF, 0x4D, 0xDA, 0x05, 0x55, 0x5C, 0xBB, 0x5C,
            0x3A, 0xF1, 0xA7, 0x71, 0xC2, 0xB8, 0x9B, 0xAE,
            0xF3, 0x7D, 0xB4, 0x3D, 0x99, 0x98, 0xB9, 0xFE,
        ];
        let mut output = [0u8; 32];
        KT128::hash(&pattern(83521), &[], &mut output);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_kt128_with_custom_1() {
        // RFC 9861: KT128(`00`^0, C=ptn(1 bytes), 32)
        // FA B6 58 DB 63 E9 4A 24 61 88 BF 7A F6 9A 13 30
        // 45 F4 6E E9 84 C5 6E 3C 33 28 CA AF 1A A1 A5 83
        let expected: [u8; 32] = [
            0xFA, 0xB6, 0x58, 0xDB, 0x63, 0xE9, 0x4A, 0x24,
            0x61, 0x88, 0xBF, 0x7A, 0xF6, 0x9A, 0x13, 0x30,
            0x45, 0xF4, 0x6E, 0xE9, 0x84, 0xC5, 0x6E, 0x3C,
            0x33, 0x28, 0xCA, 0xAF, 0x1A, 0xA1, 0xA5, 0x83,
        ];
        let mut output = [0u8; 32];
        KT128::hash(&[], &pattern(1), &mut output);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_kt128_with_custom_41() {
        // RFC 9861: KT128(`FF`, C=ptn(41 bytes), 32)
        // D8 48 C5 06 8C ED 73 6F 44 62 15 9B 98 67 FD 4C
        // 20 B8 08 AC C3 D5 BC 48 E0 B0 6B A0 A3 76 2E C4
        let expected: [u8; 32] = [
            0xD8, 0x48, 0xC5, 0x06, 0x8C, 0xED, 0x73, 0x6F,
            0x44, 0x62, 0x15, 0x9B, 0x98, 0x67, 0xFD, 0x4C,
            0x20, 0xB8, 0x08, 0xAC, 0xC3, 0xD5, 0xBC, 0x48,
            0xE0, 0xB0, 0x6B, 0xA0, 0xA3, 0x76, 0x2E, 0xC4,
        ];
        let mut output = [0u8; 32];
        KT128::hash(&[0xFF], &pattern(41), &mut output);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_kt128_ptn_8191() {
        // RFC 9861: KT128(M=ptn(8191 bytes), C=`00`^0, 32)
        // 1B 57 76 36 F7 23 64 3E 99 0C C7 D6 A6 59 83 74
        // 36 FD 6A 10 36 26 60 0E B8 30 1C D1 DB E5 53 D6
        let expected: [u8; 32] = [
            0x1B, 0x57, 0x76, 0x36, 0xF7, 0x23, 0x64, 0x3E,
            0x99, 0x0C, 0xC7, 0xD6, 0xA6, 0x59, 0x83, 0x74,
            0x36, 0xFD, 0x6A, 0x10, 0x36, 0x26, 0x60, 0x0E,
            0xB8, 0x30, 0x1C, 0xD1, 0xDB, 0xE5, 0x53, 0xD6,
        ];
        let mut output = [0u8; 32];
        KT128::hash(&pattern(8191), &[], &mut output);
        assert_eq!(output, expected);
    }

    // ===== Additional tests =====

    #[test]
    fn test_kt128_deterministic() {
        let data = b"deterministic test";
        let mut out1 = [0u8; 64];
        let mut out2 = [0u8; 64];

        KT128::hash(data, &[], &mut out1);
        KT128::hash(data, &[], &mut out2);

        assert_eq!(out1, out2);
    }

    #[test]
    fn test_kt256_empty() {
        // KT256 with empty input should work
        let mut output = [0u8; 64];
        KT256::hash(&[], &[], &mut output);
        // Just verify it doesn't panic and produces non-zero output
        assert!(output.iter().any(|&x| x != 0));
    }

    #[test]
    fn test_kt256_deterministic() {
        let data = b"deterministic test";
        let mut out1 = [0u8; 64];
        let mut out2 = [0u8; 64];

        KT256::hash(data, &[], &mut out1);
        KT256::hash(data, &[], &mut out2);

        assert_eq!(out1, out2);
    }

    #[test]
    fn test_kt128_large_message_tree_hash() {
        // Test with message larger than chunk size to trigger tree hashing
        let data = vec![0x42u8; 10000];
        let mut output = [0u8; 32];
        KT128::hash(&data, &[], &mut output);
        // Just verify it doesn't panic and produces non-zero output
        assert!(output.iter().any(|&x| x != 0));
    }

    #[test]
    fn test_kt256_large_message_tree_hash() {
        // Test with message larger than chunk size to trigger tree hashing
        let data = vec![0x42u8; 10000];
        let mut output = [0u8; 64];
        KT256::hash(&data, &[], &mut output);
        // Just verify it doesn't panic and produces non-zero output
        assert!(output.iter().any(|&x| x != 0));
    }

    #[test]
    fn test_kt128_vs_kt256_different_output() {
        let data = b"same input data";
        let mut out128 = [0u8; 32];
        let mut out256 = [0u8; 32];

        KT128::hash(data, &[], &mut out128);
        KT256::hash(data, &[], &mut out256);

        assert_ne!(out128, out256, "KT128 and KT256 should produce different outputs");
    }
}
