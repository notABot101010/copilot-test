//! A library for base32 encoding and decoding with AVX2 acceleration.
//!
//! This library provides functions to encode binary data to base32 strings
//! and decode base32 strings back to binary data using custom alphabets.

use std::fmt;

/// RFC 4648 standard base32 alphabet (A-Z, 2-7).
pub const ALPHABET_STANDARD: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

/// Extended hex base32 alphabet (0-9, A-V).
pub const ALPHABET_HEX: &[u8; 32] = b"0123456789ABCDEFGHIJKLMNOPQRSTUV";

/// Pre-computed decode table for the standard alphabet (case-insensitive).
static DECODE_TABLE_STANDARD: [u8; 256] = build_decode_table_const(ALPHABET_STANDARD);

/// Pre-computed decode table for the hex alphabet (case-insensitive).
static DECODE_TABLE_HEX: [u8; 256] = build_decode_table_const(ALPHABET_HEX);

/// Builds a decode lookup table for the given alphabet at compile time.
/// Accepts both upper and lower case for letters.
const fn build_decode_table_const(alphabet: &[u8; 32]) -> [u8; 256] {
    let mut table = [255u8; 256];
    let mut i = 0;
    while i < 32 {
        let c = alphabet[i];
        table[c as usize] = i as u8;
        // Accept lowercase for letters
        if c >= b'A' && c <= b'Z' {
            table[(c + 32) as usize] = i as u8;
        }
        i += 1;
    }
    table
}

/// Builds a decode lookup table for the given alphabet.
fn build_decode_table(alphabet: &[u8; 32]) -> [u8; 256] {
    let mut table = [255u8; 256];
    for (i, &c) in alphabet.iter().enumerate() {
        table[c as usize] = i as u8;
        // Accept lowercase for letters
        if c.is_ascii_uppercase() {
            table[c.to_ascii_lowercase() as usize] = i as u8;
        }
    }
    table
}

/// Error type for base32 encoding/decoding operations.
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Invalid character found in the input.
    InvalidCharacter(char),
    /// Invalid padding in the input.
    InvalidPadding,
    /// Invalid input length.
    InvalidLength,
    /// Output buffer too small.
    OutputBufferTooSmall,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidCharacter(c) => write!(f, "invalid character: '{}'", c),
            Error::InvalidPadding => write!(f, "invalid padding"),
            Error::InvalidLength => write!(f, "invalid input length"),
            Error::OutputBufferTooSmall => write!(f, "output buffer too small"),
        }
    }
}

impl std::error::Error for Error {}

/// Calculates the encoded length for a given input length.
///
/// # Arguments
///
/// * `len` - The length of the input data in bytes.
/// * `padding` - Whether to include padding characters ('=').
///
/// # Returns
///
/// The length of the base32-encoded output string.
///
/// # Example
///
/// ```
/// use base32::encoded_len;
///
/// assert_eq!(encoded_len(5, true), 8);
/// assert_eq!(encoded_len(1, true), 8);
/// assert_eq!(encoded_len(1, false), 2);
/// ```
pub fn encoded_len(len: usize, padding: bool) -> usize {
    if len == 0 {
        return 0;
    }

    if padding {
        // With padding: ceil(len / 5) * 8
        len.div_ceil(5) * 8
    } else {
        // Without padding: ceil(len * 8 / 5)
        let full_groups = len / 5;
        let remainder = len % 5;
        // Each 5-byte group produces 8 output characters
        // Remainder bytes produce: 1->2, 2->4, 3->5, 4->7
        full_groups * 8
            + match remainder {
                0 => 0,
                1 => 2,
                2 => 4,
                3 => 5,
                4 => 7,
                _ => unreachable!(),
            }
    }
}

/// Calculates the decoded length for a given input length.
///
/// Returns the maximum decoded length (actual may be less due to padding).
#[inline]
pub fn decoded_len(len: usize) -> usize {
    // Each 8 characters decode to 5 bytes
    len / 8 * 5
        + match len % 8 {
            0 => 0,
            2 => 1,
            4 => 2,
            5 => 3,
            7 => 4,
            _ => 0, // Invalid length, will be caught during decoding
        }
}

/// Encodes binary data into a pre-allocated output buffer.
///
/// # Arguments
///
/// * `output` - The output buffer to write the encoded data to.
/// * `data` - The binary data to encode.
/// * `alphabet` - A 32-character alphabet used for encoding.
/// * `padding` - Whether to add padding characters ('=') to the output.
///
/// # Returns
///
/// `Ok(())` if successful, or an error if the output buffer is too small.
///
/// # Example
///
/// ```
/// use base32::{encode_into, ALPHABET_STANDARD};
///
/// let data = b"Hello";
/// let mut output = [0u8; 8];
/// encode_into(&mut output, data, ALPHABET_STANDARD, true).unwrap();
/// assert_eq!(&output, b"JBSWY3DP");
/// ```
#[inline]
pub fn encode_into(
    output: &mut [u8],
    data: &[u8],
    alphabet: &[u8; 32],
    padding: bool,
) -> Result<(), Error> {
    let required_len = encoded_len(data.len(), padding);
    if output.len() < required_len {
        return Err(Error::OutputBufferTooSmall);
    }

    encode_into_unchecked(output, data, alphabet, padding);
    Ok(())
}

/// Encodes binary data into a pre-allocated output buffer without bounds checking.
fn encode_into_unchecked(output: &mut [u8], data: &[u8], alphabet: &[u8; 32], padding: bool) {
    let full_chunks = data.len() / 5;
    let remainder_len = data.len() % 5;

    let mut out_idx = 0;
    let mut in_idx = 0;

    // Process complete 5-byte groups
    for _ in 0..full_chunks {
        let b0 = data[in_idx];
        let b1 = data[in_idx + 1];
        let b2 = data[in_idx + 2];
        let b3 = data[in_idx + 3];
        let b4 = data[in_idx + 4];

        // 5 bytes = 40 bits -> 8 base32 characters (5 bits each)
        output[out_idx] = alphabet[(b0 >> 3) as usize];
        output[out_idx + 1] = alphabet[(((b0 & 0x07) << 2) | (b1 >> 6)) as usize];
        output[out_idx + 2] = alphabet[((b1 >> 1) & 0x1F) as usize];
        output[out_idx + 3] = alphabet[(((b1 & 0x01) << 4) | (b2 >> 4)) as usize];
        output[out_idx + 4] = alphabet[(((b2 & 0x0F) << 1) | (b3 >> 7)) as usize];
        output[out_idx + 5] = alphabet[((b3 >> 2) & 0x1F) as usize];
        output[out_idx + 6] = alphabet[(((b3 & 0x03) << 3) | (b4 >> 5)) as usize];
        output[out_idx + 7] = alphabet[(b4 & 0x1F) as usize];

        in_idx += 5;
        out_idx += 8;
    }

    // Handle remaining bytes
    match remainder_len {
        1 => {
            let b0 = data[in_idx];
            output[out_idx] = alphabet[(b0 >> 3) as usize];
            output[out_idx + 1] = alphabet[((b0 & 0x07) << 2) as usize];
            if padding {
                output[out_idx + 2] = b'=';
                output[out_idx + 3] = b'=';
                output[out_idx + 4] = b'=';
                output[out_idx + 5] = b'=';
                output[out_idx + 6] = b'=';
                output[out_idx + 7] = b'=';
            }
        }
        2 => {
            let b0 = data[in_idx];
            let b1 = data[in_idx + 1];
            output[out_idx] = alphabet[(b0 >> 3) as usize];
            output[out_idx + 1] = alphabet[(((b0 & 0x07) << 2) | (b1 >> 6)) as usize];
            output[out_idx + 2] = alphabet[((b1 >> 1) & 0x1F) as usize];
            output[out_idx + 3] = alphabet[((b1 & 0x01) << 4) as usize];
            if padding {
                output[out_idx + 4] = b'=';
                output[out_idx + 5] = b'=';
                output[out_idx + 6] = b'=';
                output[out_idx + 7] = b'=';
            }
        }
        3 => {
            let b0 = data[in_idx];
            let b1 = data[in_idx + 1];
            let b2 = data[in_idx + 2];
            output[out_idx] = alphabet[(b0 >> 3) as usize];
            output[out_idx + 1] = alphabet[(((b0 & 0x07) << 2) | (b1 >> 6)) as usize];
            output[out_idx + 2] = alphabet[((b1 >> 1) & 0x1F) as usize];
            output[out_idx + 3] = alphabet[(((b1 & 0x01) << 4) | (b2 >> 4)) as usize];
            output[out_idx + 4] = alphabet[((b2 & 0x0F) << 1) as usize];
            if padding {
                output[out_idx + 5] = b'=';
                output[out_idx + 6] = b'=';
                output[out_idx + 7] = b'=';
            }
        }
        4 => {
            let b0 = data[in_idx];
            let b1 = data[in_idx + 1];
            let b2 = data[in_idx + 2];
            let b3 = data[in_idx + 3];
            output[out_idx] = alphabet[(b0 >> 3) as usize];
            output[out_idx + 1] = alphabet[(((b0 & 0x07) << 2) | (b1 >> 6)) as usize];
            output[out_idx + 2] = alphabet[((b1 >> 1) & 0x1F) as usize];
            output[out_idx + 3] = alphabet[(((b1 & 0x01) << 4) | (b2 >> 4)) as usize];
            output[out_idx + 4] = alphabet[(((b2 & 0x0F) << 1) | (b3 >> 7)) as usize];
            output[out_idx + 5] = alphabet[((b3 >> 2) & 0x1F) as usize];
            output[out_idx + 6] = alphabet[((b3 & 0x03) << 3) as usize];
            if padding {
                output[out_idx + 7] = b'=';
            }
        }
        _ => {}
    }
}

/// Encodes binary data to a base32 string using the specified alphabet.
///
/// # Arguments
///
/// * `data` - The binary data to encode.
/// * `alphabet` - A 32-character alphabet used for encoding.
/// * `padding` - Whether to add padding characters ('=') to the output.
///
/// # Returns
///
/// A base32-encoded string.
///
/// # Example
///
/// ```
/// use base32::{encode, ALPHABET_STANDARD};
///
/// let encoded = encode(b"Hello", ALPHABET_STANDARD, true);
/// assert_eq!(encoded, "JBSWY3DP");
/// ```
#[inline]
pub fn encode(data: &[u8], alphabet: &[u8; 32], padding: bool) -> String {
    if data.is_empty() {
        return String::new();
    }

    let output_len = encoded_len(data.len(), padding);
    let mut output = vec![0u8; output_len];

    encode_into_unchecked(&mut output, data, alphabet, padding);

    // SAFETY: All bytes in output are valid ASCII (from alphabet or '=')
    // which is valid UTF-8
    String::from_utf8(output).expect("base32 output is always valid UTF-8")
}

/// Decodes a base32 string to binary data using the specified alphabet.
///
/// # Arguments
///
/// * `input` - The base32-encoded string to decode.
/// * `alphabet` - A 32-character alphabet used for decoding (case-insensitive).
///
/// # Returns
///
/// A `Result` containing either the decoded binary data or an error.
///
/// # Example
///
/// ```
/// use base32::{decode, ALPHABET_STANDARD};
///
/// let decoded = decode("JBSWY3DP", ALPHABET_STANDARD).unwrap();
/// assert_eq!(decoded, b"Hello");
/// ```
#[inline]
pub fn decode(input: &str, alphabet: &[u8; 32]) -> Result<Vec<u8>, Error> {
    if input.is_empty() {
        return Ok(Vec::new());
    }

    // Use pre-computed table for known alphabets, otherwise build dynamically
    let decode_table: &[u8; 256] = if alphabet == ALPHABET_STANDARD {
        &DECODE_TABLE_STANDARD
    } else if alphabet == ALPHABET_HEX {
        &DECODE_TABLE_HEX
    } else {
        // Scope owned_table to this branch only
        return decode_with_custom_alphabet(input, alphabet);
    };

    decode_with_table(input, decode_table)
}

/// Decode with a custom alphabet (builds table on the fly).
fn decode_with_custom_alphabet(input: &str, alphabet: &[u8; 32]) -> Result<Vec<u8>, Error> {
    let table = build_decode_table(alphabet);
    decode_with_table(input, &table)
}

/// Core decode implementation using a pre-built decode table.
fn decode_with_table(input: &str, decode_table: &[u8; 256]) -> Result<Vec<u8>, Error> {
    let input_bytes = input.as_bytes();

    // Count and validate padding
    let padding_len = input_bytes.iter().rev().take_while(|&&b| b == b'=').count();
    if padding_len > 6 {
        return Err(Error::InvalidPadding);
    }

    // Validate input length (with padding should be multiple of 8)
    if padding_len > 0 && !input_bytes.len().is_multiple_of(8) {
        return Err(Error::InvalidLength);
    }

    let input_len = input_bytes.len() - padding_len;

    // Calculate output size: each 8 input chars = 5 output bytes
    // Partial groups: 2->1, 4->2, 5->3, 7->4
    let full_groups = input_len / 8;
    let remainder_len = input_len % 8;
    let output_len = full_groups * 5
        + match remainder_len {
            0 => 0,
            2 => 1,
            4 => 2,
            5 => 3,
            7 => 4,
            _ => return Err(Error::InvalidLength),
        };

    let mut result = Vec::with_capacity(output_len);

    // Process complete 8-character groups
    let mut in_idx = 0;
    for _ in 0..full_groups {
        let c0 = input_bytes[in_idx];
        let c1 = input_bytes[in_idx + 1];
        let c2 = input_bytes[in_idx + 2];
        let c3 = input_bytes[in_idx + 3];
        let c4 = input_bytes[in_idx + 4];
        let c5 = input_bytes[in_idx + 5];
        let c6 = input_bytes[in_idx + 6];
        let c7 = input_bytes[in_idx + 7];

        let v0 = decode_table[c0 as usize];
        let v1 = decode_table[c1 as usize];
        let v2 = decode_table[c2 as usize];
        let v3 = decode_table[c3 as usize];
        let v4 = decode_table[c4 as usize];
        let v5 = decode_table[c5 as usize];
        let v6 = decode_table[c6 as usize];
        let v7 = decode_table[c7 as usize];

        // Fast path: check all values at once
        if (v0 | v1 | v2 | v3 | v4 | v5 | v6 | v7) > 31 {
            // Slow path: find which character is invalid
            for i in 0..8 {
                let c = input_bytes[in_idx + i];
                if decode_table[c as usize] == 255 {
                    return Err(Error::InvalidCharacter(c as char));
                }
            }
        }

        // Decode 8 base32 chars (40 bits) to 5 bytes using extend_from_slice
        result.extend_from_slice(&[
            (v0 << 3) | (v1 >> 2),
            (v1 << 6) | (v2 << 1) | (v3 >> 4),
            (v3 << 4) | (v4 >> 1),
            (v4 << 7) | (v5 << 2) | (v6 >> 3),
            (v6 << 5) | v7,
        ]);

        in_idx += 8;
    }

    // Handle remaining characters
    if remainder_len > 0 {
        let mut values = [0u8; 8];
        for i in 0..remainder_len {
            let c = input_bytes[in_idx + i];
            let v = decode_table[c as usize];
            if v == 255 {
                return Err(Error::InvalidCharacter(c as char));
            }
            values[i] = v;
        }

        match remainder_len {
            2 => {
                result.push((values[0] << 3) | (values[1] >> 2));
            }
            4 => {
                result.extend_from_slice(&[
                    (values[0] << 3) | (values[1] >> 2),
                    (values[1] << 6) | (values[2] << 1) | (values[3] >> 4),
                ]);
            }
            5 => {
                result.extend_from_slice(&[
                    (values[0] << 3) | (values[1] >> 2),
                    (values[1] << 6) | (values[2] << 1) | (values[3] >> 4),
                    (values[3] << 4) | (values[4] >> 1),
                ]);
            }
            7 => {
                result.extend_from_slice(&[
                    (values[0] << 3) | (values[1] >> 2),
                    (values[1] << 6) | (values[2] << 1) | (values[3] >> 4),
                    (values[3] << 4) | (values[4] >> 1),
                    (values[4] << 7) | (values[5] << 2) | (values[6] >> 3),
                ]);
            }
            _ => {}
        }
    }

    Ok(result)
}

// =============================================================================
// AVX2 SIMD Implementation
// =============================================================================

#[cfg(target_arch = "x86_64")]
mod avx2 {
    use super::*;
    use std::arch::x86_64::*;

    /// Check if AVX2 is available at runtime.
    #[inline]
    pub fn is_available() -> bool {
        is_x86_feature_detected!("avx2")
    }

    /// Encode using AVX2 SIMD instructions.
    ///
    /// Processes 10 input bytes at a time, producing 16 output bytes.
    /// Then processes a second group to get 20 bytes -> 32 chars.
    ///
    /// # Safety
    /// Caller must ensure AVX2 is available (check with `is_available()`).
    #[target_feature(enable = "avx2")]
    pub unsafe fn encode_avx2(output: &mut [u8], data: &[u8], alphabet: &[u8; 32], padding: bool) {
        // Only use SIMD for standard alphabet
        let use_simd = alphabet == ALPHABET_STANDARD || alphabet == ALPHABET_HEX;

        if !use_simd || data.len() < 10 {
            super::encode_into_unchecked(output, data, alphabet, padding);
            return;
        }

        let is_hex = alphabet == ALPHABET_HEX;

        // Process 10 bytes at a time (2 groups of 5 bytes = 16 output chars)
        // This allows us to use 128-bit operations for better efficiency
        let full_chunks = data.len() / 10;

        let mut in_idx = 0;
        let mut out_idx = 0;

        // Process pairs of 10-byte chunks when possible for AVX2 efficiency
        let chunk_pairs = full_chunks / 2;
        for _ in 0..chunk_pairs {
            // Load 20 bytes (will be processed as two 10-byte groups)
            let mut input_buf = [0u8; 32];
            input_buf[..20].copy_from_slice(&data[in_idx..in_idx + 20]);

            // Process 20 bytes -> 32 characters using optimized bit extraction
            let result = encode_20_bytes_avx2(&input_buf, is_hex);

            // Store 32 bytes of output
            _mm256_storeu_si256(output.as_mut_ptr().add(out_idx) as *mut __m256i, result);

            in_idx += 20;
            out_idx += 32;
        }

        // Handle remaining 10-byte chunk if odd number
        if full_chunks % 2 == 1 && in_idx + 10 <= data.len() {
            let mut input_buf = [0u8; 16];
            input_buf[..10].copy_from_slice(&data[in_idx..in_idx + 10]);

            let result = encode_10_bytes_sse(&input_buf, is_hex);
            _mm_storeu_si128(output.as_mut_ptr().add(out_idx) as *mut __m128i, result);

            in_idx += 10;
            out_idx += 16;
        }

        // Handle remaining bytes with scalar code
        if in_idx < data.len() {
            let remaining_data = &data[in_idx..];
            let remaining_output = &mut output[out_idx..];
            super::encode_into_unchecked(remaining_output, remaining_data, alphabet, padding);
        }
    }

    /// Encode 20 bytes to 32 base32 characters using AVX2.
    /// Uses optimized bit manipulation with SIMD.
    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn encode_20_bytes_avx2(input: &[u8; 32], is_hex: bool) -> __m256i {
        // For base32, each 5 bytes becomes 8 characters
        // We process 4 groups of 5 bytes -> 32 characters

        // The bit layout for 5 bytes (40 bits) -> 8 x 5-bit values:
        // Byte 0: bits 7-3 -> char 0, bits 2-0 -> part of char 1
        // Byte 1: bits 7-6 -> part of char 1, bits 5-1 -> char 2, bit 0 -> part of char 3
        // Byte 2: bits 7-4 -> part of char 3, bits 3-0 -> part of char 4
        // Byte 3: bit 7 -> part of char 4, bits 6-2 -> char 5, bits 1-0 -> part of char 6
        // Byte 4: bits 7-5 -> part of char 6, bits 4-0 -> char 7

        // Use scalar extraction (optimized with loop unrolling) then SIMD for ASCII conversion
        let mut values = [0u8; 32];

        // Process 4 groups of 5 bytes
        for g in 0..4 {
            let i = g * 5;
            let b0 = input[i];
            let b1 = input[i + 1];
            let b2 = input[i + 2];
            let b3 = input[i + 3];
            let b4 = input[i + 4];

            let o = g * 8;
            values[o] = b0 >> 3;
            values[o + 1] = ((b0 & 0x07) << 2) | (b1 >> 6);
            values[o + 2] = (b1 >> 1) & 0x1F;
            values[o + 3] = ((b1 & 0x01) << 4) | (b2 >> 4);
            values[o + 4] = ((b2 & 0x0F) << 1) | (b3 >> 7);
            values[o + 5] = (b3 >> 2) & 0x1F;
            values[o + 6] = ((b3 & 0x03) << 3) | (b4 >> 5);
            values[o + 7] = b4 & 0x1F;
        }

        // Now use SIMD to convert 5-bit values to ASCII
        let vals = _mm256_loadu_si256(values.as_ptr() as *const __m256i);
        values_to_ascii_avx2(vals, is_hex)
    }

    /// Encode 10 bytes to 16 base32 characters using SSE.
    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn encode_10_bytes_sse(input: &[u8; 16], is_hex: bool) -> __m128i {
        // Process 2 groups of 5 bytes -> 16 characters
        let mut values = [0u8; 16];

        for g in 0..2 {
            let i = g * 5;
            let b0 = input[i];
            let b1 = input[i + 1];
            let b2 = input[i + 2];
            let b3 = input[i + 3];
            let b4 = input[i + 4];

            let o = g * 8;
            values[o] = b0 >> 3;
            values[o + 1] = ((b0 & 0x07) << 2) | (b1 >> 6);
            values[o + 2] = (b1 >> 1) & 0x1F;
            values[o + 3] = ((b1 & 0x01) << 4) | (b2 >> 4);
            values[o + 4] = ((b2 & 0x0F) << 1) | (b3 >> 7);
            values[o + 5] = (b3 >> 2) & 0x1F;
            values[o + 6] = ((b3 & 0x03) << 3) | (b4 >> 5);
            values[o + 7] = b4 & 0x1F;
        }

        let vals = _mm_loadu_si128(values.as_ptr() as *const __m128i);
        values_to_ascii_sse(vals, is_hex)
    }

    /// Convert 5-bit values to ASCII using AVX2.
    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn values_to_ascii_avx2(values: __m256i, is_hex: bool) -> __m256i {
        if is_hex {
            // Hex alphabet: 0-9 -> '0'-'9', 10-31 -> 'A'-'V'
            let ten = _mm256_set1_epi8(10);
            let is_digit = _mm256_cmpgt_epi8(ten, values);

            let digit_offset = _mm256_set1_epi8(b'0' as i8);
            let letter_offset = _mm256_set1_epi8((b'A' as i8) - 10);

            let offsets = _mm256_blendv_epi8(letter_offset, digit_offset, is_digit);
            _mm256_add_epi8(values, offsets)
        } else {
            // Standard alphabet: 0-25 -> 'A'-'Z', 26-31 -> '2'-'7'
            let twenty_six = _mm256_set1_epi8(26);
            let is_letter = _mm256_cmpgt_epi8(twenty_six, values);

            let letter_offset = _mm256_set1_epi8(b'A' as i8);
            let digit_offset = _mm256_set1_epi8((b'2' as i8) - 26);

            let offsets = _mm256_blendv_epi8(digit_offset, letter_offset, is_letter);
            _mm256_add_epi8(values, offsets)
        }
    }

    /// Convert 5-bit values to ASCII using SSE.
    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn values_to_ascii_sse(values: __m128i, is_hex: bool) -> __m128i {
        if is_hex {
            let ten = _mm_set1_epi8(10);
            let is_digit = _mm_cmplt_epi8(values, ten);

            let digit_offset = _mm_set1_epi8(b'0' as i8);
            let letter_offset = _mm_set1_epi8((b'A' as i8) - 10);

            let offsets = _mm_blendv_epi8(letter_offset, digit_offset, is_digit);
            _mm_add_epi8(values, offsets)
        } else {
            let twenty_six = _mm_set1_epi8(26);
            let is_letter = _mm_cmplt_epi8(values, twenty_six);

            let letter_offset = _mm_set1_epi8(b'A' as i8);
            let digit_offset = _mm_set1_epi8((b'2' as i8) - 26);

            let offsets = _mm_blendv_epi8(digit_offset, letter_offset, is_letter);
            _mm_add_epi8(values, offsets)
        }
    }

    /// Decode using AVX2 SIMD instructions.
    ///
    /// Processes 32 input characters at a time, producing 20 output bytes.
    ///
    /// # Safety
    /// Caller must ensure AVX2 is available (check with `is_available()`).
    #[target_feature(enable = "avx2")]
    pub unsafe fn decode_avx2(
        output: &mut [u8],
        input: &[u8],
        alphabet: &[u8; 32],
    ) -> Result<usize, Error> {
        let input_len = input.len();

        // Only use SIMD for standard alphabet and sufficient length
        let use_simd =
            (alphabet == ALPHABET_STANDARD || alphabet == ALPHABET_HEX) && input_len >= 16;

        if !use_simd {
            let input_str =
                std::str::from_utf8(input).map_err(|_| Error::InvalidCharacter('\0'))?;
            let decoded = super::decode(input_str, alphabet)?;
            output[..decoded.len()].copy_from_slice(&decoded);
            return Ok(decoded.len());
        }

        let is_hex = alphabet == ALPHABET_HEX;

        // Count padding
        let padding_len = input.iter().rev().take_while(|&&b| b == b'=').count();
        let effective_len = input_len - padding_len;

        // Process 16-character chunks (-> 10 bytes)
        let full_chunks = effective_len / 16;

        let mut in_idx = 0;
        let mut out_idx = 0;

        // Process pairs of 16-char chunks for AVX2 efficiency (32 chars -> 20 bytes)
        let chunk_pairs = full_chunks / 2;
        for _ in 0..chunk_pairs {
            let input_vec = _mm256_loadu_si256(input.as_ptr().add(in_idx) as *const __m256i);

            // Decode ASCII to 5-bit values
            let (values, valid) = ascii_to_values_avx2(input_vec, is_hex);

            if !valid {
                // Find the invalid character
                for i in 0..32 {
                    let c = input[in_idx + i];
                    if !is_valid_base32_char(c, is_hex) {
                        return Err(Error::InvalidCharacter(c as char));
                    }
                }
            }

            // Convert 32 5-bit values to 20 bytes
            let decoded = decode_32_values_to_20_bytes(values);

            // Store first 20 bytes (copy from array to avoid overwriting)
            std::ptr::copy_nonoverlapping(decoded.as_ptr(), output.as_mut_ptr().add(out_idx), 20);

            in_idx += 32;
            out_idx += 20;
        }

        // Handle remaining 16-char chunk if odd number
        if full_chunks % 2 == 1 && in_idx + 16 <= effective_len {
            let input_vec = _mm_loadu_si128(input.as_ptr().add(in_idx) as *const __m128i);

            let (values, valid) = ascii_to_values_sse(input_vec, is_hex);

            if !valid {
                for i in 0..16 {
                    let c = input[in_idx + i];
                    if !is_valid_base32_char(c, is_hex) {
                        return Err(Error::InvalidCharacter(c as char));
                    }
                }
            }

            let decoded = decode_16_values_to_10_bytes(values);
            std::ptr::copy_nonoverlapping(decoded.as_ptr(), output.as_mut_ptr().add(out_idx), 10);

            in_idx += 16;
            out_idx += 10;
        }

        // Handle remaining characters with scalar code
        if in_idx < effective_len {
            let remaining_input =
                std::str::from_utf8(&input[in_idx..]).map_err(|_| Error::InvalidCharacter('\0'))?;
            let decoded = super::decode(remaining_input, alphabet)?;
            output[out_idx..out_idx + decoded.len()].copy_from_slice(&decoded);
            out_idx += decoded.len();
        }

        Ok(out_idx)
    }

    /// Check if a character is valid for base32.
    #[inline]
    fn is_valid_base32_char(c: u8, is_hex: bool) -> bool {
        if is_hex {
            c.is_ascii_digit() || (b'A'..=b'V').contains(&c) || (b'a'..=b'v').contains(&c)
        } else {
            c.is_ascii_uppercase() || c.is_ascii_lowercase() || (b'2'..=b'7').contains(&c)
        }
    }

    /// Convert ASCII to 5-bit values using AVX2.
    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn ascii_to_values_avx2(input: __m256i, is_hex: bool) -> (__m256i, bool) {
        if is_hex {
            // Hex alphabet: '0'-'9' -> 0-9, 'A'-'V' -> 10-31, 'a'-'v' -> 10-31
            let ascii_zero = _mm256_set1_epi8(b'0' as i8);
            let ascii_a = _mm256_set1_epi8(b'A' as i8);
            let ascii_a_lower = _mm256_set1_epi8(b'a' as i8);
            let ten = _mm256_set1_epi8(10);
            let twenty_two = _mm256_set1_epi8(22);

            // Check if digit (0-9)
            let offset_digit = _mm256_sub_epi8(input, ascii_zero);
            let is_digit = _mm256_and_si256(
                _mm256_cmpgt_epi8(offset_digit, _mm256_set1_epi8(-1)),
                _mm256_cmpgt_epi8(ten, offset_digit),
            );

            // Check if uppercase letter (A-V)
            let offset_upper = _mm256_sub_epi8(input, ascii_a);
            let is_upper = _mm256_and_si256(
                _mm256_cmpgt_epi8(offset_upper, _mm256_set1_epi8(-1)),
                _mm256_cmpgt_epi8(twenty_two, offset_upper),
            );

            // Check if lowercase letter (a-v)
            let offset_lower = _mm256_sub_epi8(input, ascii_a_lower);
            let is_lower = _mm256_and_si256(
                _mm256_cmpgt_epi8(offset_lower, _mm256_set1_epi8(-1)),
                _mm256_cmpgt_epi8(twenty_two, offset_lower),
            );

            // Check validity
            let is_valid = _mm256_or_si256(_mm256_or_si256(is_digit, is_upper), is_lower);
            let all_valid = _mm256_movemask_epi8(is_valid) == -1i32;

            // Calculate values
            let digit_val = offset_digit;
            let upper_val = _mm256_add_epi8(offset_upper, ten);
            let lower_val = _mm256_add_epi8(offset_lower, ten);

            let result = _mm256_blendv_epi8(
                _mm256_blendv_epi8(lower_val, upper_val, is_upper),
                digit_val,
                is_digit,
            );

            (result, all_valid)
        } else {
            // Standard alphabet: 'A'-'Z' -> 0-25, '2'-'7' -> 26-31, also lowercase
            let ascii_a = _mm256_set1_epi8(b'A' as i8);
            let ascii_a_lower = _mm256_set1_epi8(b'a' as i8);
            let ascii_two = _mm256_set1_epi8(b'2' as i8);
            let twenty_six = _mm256_set1_epi8(26);
            let six = _mm256_set1_epi8(6);

            // Check if uppercase letter (A-Z)
            let offset_upper = _mm256_sub_epi8(input, ascii_a);
            let is_upper = _mm256_and_si256(
                _mm256_cmpgt_epi8(offset_upper, _mm256_set1_epi8(-1)),
                _mm256_cmpgt_epi8(twenty_six, offset_upper),
            );

            // Check if lowercase letter (a-z)
            let offset_lower = _mm256_sub_epi8(input, ascii_a_lower);
            let is_lower = _mm256_and_si256(
                _mm256_cmpgt_epi8(offset_lower, _mm256_set1_epi8(-1)),
                _mm256_cmpgt_epi8(twenty_six, offset_lower),
            );

            // Check if digit (2-7)
            let offset_digit = _mm256_sub_epi8(input, ascii_two);
            let is_digit = _mm256_and_si256(
                _mm256_cmpgt_epi8(offset_digit, _mm256_set1_epi8(-1)),
                _mm256_cmpgt_epi8(six, offset_digit),
            );

            // Check validity
            let is_valid = _mm256_or_si256(_mm256_or_si256(is_upper, is_lower), is_digit);
            let all_valid = _mm256_movemask_epi8(is_valid) == -1i32;

            // Calculate values
            let upper_val = offset_upper;
            let lower_val = offset_lower;
            let digit_val = _mm256_add_epi8(offset_digit, twenty_six);

            let result = _mm256_blendv_epi8(
                _mm256_blendv_epi8(lower_val, digit_val, is_digit),
                upper_val,
                is_upper,
            );

            (result, all_valid)
        }
    }

    /// Convert ASCII to 5-bit values using SSE.
    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn ascii_to_values_sse(input: __m128i, is_hex: bool) -> (__m128i, bool) {
        if is_hex {
            let ascii_zero = _mm_set1_epi8(b'0' as i8);
            let ascii_a = _mm_set1_epi8(b'A' as i8);
            let ascii_a_lower = _mm_set1_epi8(b'a' as i8);
            let ten = _mm_set1_epi8(10);
            let twenty_two = _mm_set1_epi8(22);

            let offset_digit = _mm_sub_epi8(input, ascii_zero);
            let is_digit = _mm_and_si128(
                _mm_cmpgt_epi8(offset_digit, _mm_set1_epi8(-1)),
                _mm_cmpgt_epi8(ten, offset_digit),
            );

            let offset_upper = _mm_sub_epi8(input, ascii_a);
            let is_upper = _mm_and_si128(
                _mm_cmpgt_epi8(offset_upper, _mm_set1_epi8(-1)),
                _mm_cmpgt_epi8(twenty_two, offset_upper),
            );

            let offset_lower = _mm_sub_epi8(input, ascii_a_lower);
            let is_lower = _mm_and_si128(
                _mm_cmpgt_epi8(offset_lower, _mm_set1_epi8(-1)),
                _mm_cmpgt_epi8(twenty_two, offset_lower),
            );

            let is_valid = _mm_or_si128(_mm_or_si128(is_digit, is_upper), is_lower);
            let all_valid = _mm_movemask_epi8(is_valid) == 0xFFFF;

            let digit_val = offset_digit;
            let upper_val = _mm_add_epi8(offset_upper, ten);
            let lower_val = _mm_add_epi8(offset_lower, ten);

            let result = _mm_blendv_epi8(
                _mm_blendv_epi8(lower_val, upper_val, is_upper),
                digit_val,
                is_digit,
            );

            (result, all_valid)
        } else {
            let ascii_a = _mm_set1_epi8(b'A' as i8);
            let ascii_a_lower = _mm_set1_epi8(b'a' as i8);
            let ascii_two = _mm_set1_epi8(b'2' as i8);
            let twenty_six = _mm_set1_epi8(26);
            let six = _mm_set1_epi8(6);

            let offset_upper = _mm_sub_epi8(input, ascii_a);
            let is_upper = _mm_and_si128(
                _mm_cmpgt_epi8(offset_upper, _mm_set1_epi8(-1)),
                _mm_cmpgt_epi8(twenty_six, offset_upper),
            );

            let offset_lower = _mm_sub_epi8(input, ascii_a_lower);
            let is_lower = _mm_and_si128(
                _mm_cmpgt_epi8(offset_lower, _mm_set1_epi8(-1)),
                _mm_cmpgt_epi8(twenty_six, offset_lower),
            );

            let offset_digit = _mm_sub_epi8(input, ascii_two);
            let is_digit = _mm_and_si128(
                _mm_cmpgt_epi8(offset_digit, _mm_set1_epi8(-1)),
                _mm_cmpgt_epi8(six, offset_digit),
            );

            let is_valid = _mm_or_si128(_mm_or_si128(is_upper, is_lower), is_digit);
            let all_valid = _mm_movemask_epi8(is_valid) == 0xFFFF;

            let upper_val = offset_upper;
            let lower_val = offset_lower;
            let digit_val = _mm_add_epi8(offset_digit, twenty_six);

            let result = _mm_blendv_epi8(
                _mm_blendv_epi8(lower_val, digit_val, is_digit),
                upper_val,
                is_upper,
            );

            (result, all_valid)
        }
    }

    /// Convert 32 5-bit values to 20 bytes.
    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn decode_32_values_to_20_bytes(values: __m256i) -> [u8; 20] {
        let val_bytes: [u8; 32] = std::mem::transmute(values);
        let mut out_bytes = [0u8; 20];

        // Process 4 groups of 8 values -> 5 bytes each
        for g in 0..4 {
            let i = g * 8;
            let v0 = val_bytes[i];
            let v1 = val_bytes[i + 1];
            let v2 = val_bytes[i + 2];
            let v3 = val_bytes[i + 3];
            let v4 = val_bytes[i + 4];
            let v5 = val_bytes[i + 5];
            let v6 = val_bytes[i + 6];
            let v7 = val_bytes[i + 7];

            let o = g * 5;
            out_bytes[o] = (v0 << 3) | (v1 >> 2);
            out_bytes[o + 1] = (v1 << 6) | (v2 << 1) | (v3 >> 4);
            out_bytes[o + 2] = (v3 << 4) | (v4 >> 1);
            out_bytes[o + 3] = (v4 << 7) | (v5 << 2) | (v6 >> 3);
            out_bytes[o + 4] = (v6 << 5) | v7;
        }

        out_bytes
    }

    /// Convert 16 5-bit values to 10 bytes.
    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn decode_16_values_to_10_bytes(values: __m128i) -> [u8; 10] {
        let val_bytes: [u8; 16] = std::mem::transmute(values);
        let mut out_bytes = [0u8; 10];

        // Process 2 groups of 8 values -> 5 bytes each
        for g in 0..2 {
            let i = g * 8;
            let v0 = val_bytes[i];
            let v1 = val_bytes[i + 1];
            let v2 = val_bytes[i + 2];
            let v3 = val_bytes[i + 3];
            let v4 = val_bytes[i + 4];
            let v5 = val_bytes[i + 5];
            let v6 = val_bytes[i + 6];
            let v7 = val_bytes[i + 7];

            let o = g * 5;
            out_bytes[o] = (v0 << 3) | (v1 >> 2);
            out_bytes[o + 1] = (v1 << 6) | (v2 << 1) | (v3 >> 4);
            out_bytes[o + 2] = (v3 << 4) | (v4 >> 1);
            out_bytes[o + 3] = (v4 << 7) | (v5 << 2) | (v6 >> 3);
            out_bytes[o + 4] = (v6 << 5) | v7;
        }

        out_bytes
    }
}

/// Encode binary data to a base32 string using AVX2 SIMD if available.
/// Falls back to scalar implementation if AVX2 is not available or for non-x86_64 architectures.
#[inline]
pub fn encode_avx2(data: &[u8], alphabet: &[u8; 32], padding: bool) -> String {
    if data.is_empty() {
        return String::new();
    }

    #[cfg(target_arch = "x86_64")]
    {
        // Use AVX2 for sufficiently large inputs
        if avx2::is_available() && data.len() >= 20 {
            let output_len = encoded_len(data.len(), padding);
            let mut output = vec![0u8; output_len];

            // SAFETY: We just checked that AVX2 is available
            unsafe {
                avx2::encode_avx2(&mut output, data, alphabet, padding);
            }

            return String::from_utf8(output).expect("base32 output is always valid UTF-8");
        }
    }

    // Fall back to scalar implementation
    encode(data, alphabet, padding)
}

/// Decode a base32 string using AVX2 SIMD if available.
/// Falls back to scalar implementation if AVX2 is not available or for non-x86_64 architectures.
#[inline]
pub fn decode_avx2(input: &str, alphabet: &[u8; 32]) -> Result<Vec<u8>, Error> {
    if input.is_empty() {
        return Ok(Vec::new());
    }

    #[cfg(target_arch = "x86_64")]
    {
        // Use AVX2 for sufficiently large inputs
        if avx2::is_available() && input.len() >= 32 {
            let input_bytes = input.as_bytes();
            let output_len = decoded_len(input_bytes.len());
            let mut output = vec![0u8; output_len + 32]; // Extra space for SIMD writes

            // SAFETY: We just checked that AVX2 is available
            let decoded_len = unsafe { avx2::decode_avx2(&mut output, input_bytes, alphabet)? };

            output.truncate(decoded_len);
            return Ok(output);
        }
    }

    // Fall back to scalar implementation
    decode(input, alphabet)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoded_len() {
        assert_eq!(encoded_len(0, true), 0);
        assert_eq!(encoded_len(0, false), 0);
        assert_eq!(encoded_len(1, true), 8);
        assert_eq!(encoded_len(1, false), 2);
        assert_eq!(encoded_len(2, true), 8);
        assert_eq!(encoded_len(2, false), 4);
        assert_eq!(encoded_len(3, true), 8);
        assert_eq!(encoded_len(3, false), 5);
        assert_eq!(encoded_len(4, true), 8);
        assert_eq!(encoded_len(4, false), 7);
        assert_eq!(encoded_len(5, true), 8);
        assert_eq!(encoded_len(5, false), 8);
        assert_eq!(encoded_len(6, true), 16);
        assert_eq!(encoded_len(6, false), 10);
    }

    #[test]
    fn test_decoded_len() {
        assert_eq!(decoded_len(0), 0);
        assert_eq!(decoded_len(8), 5);
        assert_eq!(decoded_len(16), 10);
        assert_eq!(decoded_len(2), 1);
        assert_eq!(decoded_len(4), 2);
        assert_eq!(decoded_len(5), 3);
        assert_eq!(decoded_len(7), 4);
    }

    #[test]
    fn test_encode_empty() {
        assert_eq!(encode(b"", ALPHABET_STANDARD, true), "");
        assert_eq!(encode(b"", ALPHABET_STANDARD, false), "");
    }

    #[test]
    fn test_encode_with_padding() {
        // RFC 4648 test vectors
        assert_eq!(encode(b"f", ALPHABET_STANDARD, true), "MY======");
        assert_eq!(encode(b"fo", ALPHABET_STANDARD, true), "MZXQ====");
        assert_eq!(encode(b"foo", ALPHABET_STANDARD, true), "MZXW6===");
        assert_eq!(encode(b"foob", ALPHABET_STANDARD, true), "MZXW6YQ=");
        assert_eq!(encode(b"fooba", ALPHABET_STANDARD, true), "MZXW6YTB");
        assert_eq!(
            encode(b"foobar", ALPHABET_STANDARD, true),
            "MZXW6YTBOI======"
        );
    }

    #[test]
    fn test_encode_without_padding() {
        assert_eq!(encode(b"f", ALPHABET_STANDARD, false), "MY");
        assert_eq!(encode(b"fo", ALPHABET_STANDARD, false), "MZXQ");
        assert_eq!(encode(b"foo", ALPHABET_STANDARD, false), "MZXW6");
        assert_eq!(encode(b"foob", ALPHABET_STANDARD, false), "MZXW6YQ");
        assert_eq!(encode(b"fooba", ALPHABET_STANDARD, false), "MZXW6YTB");
        assert_eq!(encode(b"foobar", ALPHABET_STANDARD, false), "MZXW6YTBOI");
    }

    #[test]
    fn test_encode_hello() {
        assert_eq!(encode(b"Hello", ALPHABET_STANDARD, true), "JBSWY3DP");
        assert_eq!(
            encode(b"Hello, World!", ALPHABET_STANDARD, true),
            "JBSWY3DPFQQFO33SNRSCC==="
        );
    }

    #[test]
    fn test_encode_into() {
        let data = b"Hello";
        let mut output = [0u8; 8];
        encode_into(&mut output, data, ALPHABET_STANDARD, true).unwrap();
        assert_eq!(&output, b"JBSWY3DP");
    }

    #[test]
    fn test_encode_into_buffer_too_small() {
        let data = b"Hello";
        let mut output = [0u8; 4];
        let result = encode_into(&mut output, data, ALPHABET_STANDARD, true);
        assert!(matches!(result, Err(Error::OutputBufferTooSmall)));
    }

    #[test]
    fn test_decode_empty() {
        assert_eq!(decode("", ALPHABET_STANDARD).unwrap(), Vec::<u8>::new());
    }

    #[test]
    fn test_decode_with_padding() {
        assert_eq!(decode("MY======", ALPHABET_STANDARD).unwrap(), b"f");
        assert_eq!(decode("MZXQ====", ALPHABET_STANDARD).unwrap(), b"fo");
        assert_eq!(decode("MZXW6===", ALPHABET_STANDARD).unwrap(), b"foo");
        assert_eq!(decode("MZXW6YQ=", ALPHABET_STANDARD).unwrap(), b"foob");
        assert_eq!(decode("MZXW6YTB", ALPHABET_STANDARD).unwrap(), b"fooba");
        assert_eq!(
            decode("MZXW6YTBOI======", ALPHABET_STANDARD).unwrap(),
            b"foobar"
        );
    }

    #[test]
    fn test_decode_without_padding() {
        assert_eq!(decode("MY", ALPHABET_STANDARD).unwrap(), b"f");
        assert_eq!(decode("MZXQ", ALPHABET_STANDARD).unwrap(), b"fo");
        assert_eq!(decode("MZXW6", ALPHABET_STANDARD).unwrap(), b"foo");
        assert_eq!(decode("MZXW6YQ", ALPHABET_STANDARD).unwrap(), b"foob");
        assert_eq!(decode("MZXW6YTB", ALPHABET_STANDARD).unwrap(), b"fooba");
        assert_eq!(decode("MZXW6YTBOI", ALPHABET_STANDARD).unwrap(), b"foobar");
    }

    #[test]
    fn test_decode_hello() {
        assert_eq!(decode("JBSWY3DP", ALPHABET_STANDARD).unwrap(), b"Hello");
        assert_eq!(
            decode("JBSWY3DPFQQFO33SNRSCC===", ALPHABET_STANDARD).unwrap(),
            b"Hello, World!"
        );
    }

    #[test]
    fn test_decode_lowercase() {
        // Should accept lowercase
        assert_eq!(decode("jbswy3dp", ALPHABET_STANDARD).unwrap(), b"Hello");
        assert_eq!(decode("mzxw6ytboi", ALPHABET_STANDARD).unwrap(), b"foobar");
    }

    #[test]
    fn test_decode_mixed_case() {
        assert_eq!(decode("JbSwY3dP", ALPHABET_STANDARD).unwrap(), b"Hello");
    }

    #[test]
    fn test_decode_invalid_character() {
        let result = decode("!INVALID", ALPHABET_STANDARD);
        assert!(matches!(result, Err(Error::InvalidCharacter('!'))));

        // 0, 1, 8, 9 are not valid in standard base32
        let result = decode("01ABCDEF", ALPHABET_STANDARD);
        assert!(matches!(result, Err(Error::InvalidCharacter('0'))));
    }

    #[test]
    fn test_decode_invalid_length() {
        // Length 1, 3, 6 are invalid
        let result = decode("A", ALPHABET_STANDARD);
        assert!(matches!(result, Err(Error::InvalidLength)));

        let result = decode("ABC", ALPHABET_STANDARD);
        assert!(matches!(result, Err(Error::InvalidLength)));

        let result = decode("ABCDEF", ALPHABET_STANDARD);
        assert!(matches!(result, Err(Error::InvalidLength)));
    }

    #[test]
    fn test_roundtrip() {
        let test_cases = [
            b"".to_vec(),
            b"a".to_vec(),
            b"ab".to_vec(),
            b"abc".to_vec(),
            b"abcd".to_vec(),
            b"abcde".to_vec(),
            b"Hello, World!".to_vec(),
            (0..=255).collect::<Vec<u8>>(),
        ];

        for data in test_cases {
            let encoded = encode(&data, ALPHABET_STANDARD, true);
            let decoded = decode(&encoded, ALPHABET_STANDARD).unwrap();
            assert_eq!(decoded, data, "Roundtrip failed for {:?}", data);
        }
    }

    #[test]
    fn test_roundtrip_no_padding() {
        let test_cases = [
            b"".to_vec(),
            b"a".to_vec(),
            b"ab".to_vec(),
            b"abc".to_vec(),
            b"abcd".to_vec(),
            b"abcde".to_vec(),
            b"Hello, World!".to_vec(),
        ];

        for data in test_cases {
            let encoded = encode(&data, ALPHABET_STANDARD, false);
            let decoded = decode(&encoded, ALPHABET_STANDARD).unwrap();
            assert_eq!(
                decoded, data,
                "Roundtrip without padding failed for {:?}",
                data
            );
        }
    }

    #[test]
    fn test_hex_alphabet() {
        // Test with hex alphabet
        let data = b"test";
        let encoded = encode(data, ALPHABET_HEX, false);
        let decoded = decode(&encoded, ALPHABET_HEX).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_error_display() {
        assert_eq!(
            format!("{}", Error::InvalidCharacter('!')),
            "invalid character: '!'"
        );
        assert_eq!(format!("{}", Error::InvalidPadding), "invalid padding");
        assert_eq!(format!("{}", Error::InvalidLength), "invalid input length");
        assert_eq!(
            format!("{}", Error::OutputBufferTooSmall),
            "output buffer too small"
        );
    }

    // AVX2 tests
    #[test]
    fn test_encode_avx2_matches_scalar() {
        let test_cases = [
            b"".to_vec(),
            b"a".to_vec(),
            b"Hello, World!".to_vec(),
            (0..20).collect::<Vec<u8>>(), // Exactly 20 bytes (AVX2 block size)
            (0..40).collect::<Vec<u8>>(), // Two AVX2 blocks
            (0..100).collect::<Vec<u8>>(), // Multiple blocks + remainder
            (0..=255).collect::<Vec<u8>>(), // All byte values
        ];

        for data in test_cases {
            let scalar_result = encode(&data, ALPHABET_STANDARD, true);
            let avx2_result = encode_avx2(&data, ALPHABET_STANDARD, true);
            assert_eq!(
                scalar_result,
                avx2_result,
                "AVX2 encode mismatch for data len {}",
                data.len()
            );
        }
    }

    #[test]
    fn test_decode_avx2_matches_scalar() {
        let test_cases = [
            "",
            "MY======",
            "JBSWY3DP",
            "MZXW6YTBOI======",
            "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ", // 32 chars
            "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ", // 64 chars
        ];

        for encoded in test_cases {
            let scalar_result = decode(encoded, ALPHABET_STANDARD);
            let avx2_result = decode_avx2(encoded, ALPHABET_STANDARD);
            match (scalar_result, avx2_result) {
                (Ok(scalar), Ok(avx2)) => {
                    assert_eq!(scalar, avx2, "AVX2 decode mismatch for '{}'", encoded);
                }
                (Err(_), Err(_)) => {} // Both failed, OK
                (Ok(scalar), Err(e)) => {
                    panic!(
                        "Scalar succeeded but AVX2 failed for '{}': {:?} vs {:?}",
                        encoded, scalar, e
                    );
                }
                (Err(e), Ok(avx2)) => {
                    panic!(
                        "AVX2 succeeded but scalar failed for '{}': {:?} vs {:?}",
                        encoded, e, avx2
                    );
                }
            }
        }
    }

    #[test]
    fn test_avx2_roundtrip() {
        let test_cases = [
            b"".to_vec(),
            b"a".to_vec(),
            b"Hello, World!".to_vec(),
            (0..20).collect::<Vec<u8>>(),
            (0..40).collect::<Vec<u8>>(),
            (0..100).collect::<Vec<u8>>(),
            (0..=255).collect::<Vec<u8>>(),
        ];

        for data in test_cases {
            let encoded = encode_avx2(&data, ALPHABET_STANDARD, true);
            let decoded = decode_avx2(&encoded, ALPHABET_STANDARD).unwrap();
            assert_eq!(
                decoded,
                data,
                "AVX2 roundtrip failed for data len {}",
                data.len()
            );
        }
    }

    // Additional comprehensive tests
    #[test]
    fn test_encode_binary_with_high_bytes() {
        // Test binary data with bytes > 127 (non-ASCII range)
        let data: Vec<u8> = (128..=255).collect();
        let encoded = encode(&data, ALPHABET_STANDARD, true);
        let decoded = decode(&encoded, ALPHABET_STANDARD).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_encode_null_and_control_chars() {
        // Test encoding data with null bytes and control characters
        let data = b"\x00\x01\x02\x1f\x7f\xff";
        let encoded = encode(data, ALPHABET_STANDARD, true);
        let decoded = decode(&encoded, ALPHABET_STANDARD).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_large_data_roundtrip() {
        // Test with larger data sizes
        for size in [1000, 5000, 10000] {
            let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
            let encoded = encode(&data, ALPHABET_STANDARD, true);
            let decoded = decode(&encoded, ALPHABET_STANDARD).unwrap();
            assert_eq!(decoded, data, "Roundtrip failed for size {}", size);
        }
    }

    #[test]
    fn test_avx2_large_data_roundtrip() {
        // Test AVX2 with larger data sizes
        for size in [1000, 5000, 10000] {
            let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
            let encoded = encode_avx2(&data, ALPHABET_STANDARD, true);
            let decoded = decode_avx2(&encoded, ALPHABET_STANDARD).unwrap();
            assert_eq!(decoded, data, "AVX2 roundtrip failed for size {}", size);
        }
    }

    #[test]
    fn test_all_byte_values() {
        // Test all 256 byte values
        let data: Vec<u8> = (0..=255).collect();

        // Test with standard alphabet
        let encoded = encode(&data, ALPHABET_STANDARD, true);
        let decoded = decode(&encoded, ALPHABET_STANDARD).unwrap();
        assert_eq!(decoded, data);

        // Test with hex alphabet
        let encoded = encode(&data, ALPHABET_HEX, true);
        let decoded = decode(&encoded, ALPHABET_HEX).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_various_lengths() {
        // Test various input lengths to catch edge cases
        for len in 0..50 {
            let data: Vec<u8> = (0..len as u8).collect();
            let encoded = encode(&data, ALPHABET_STANDARD, true);
            let decoded = decode(&encoded, ALPHABET_STANDARD).unwrap();
            assert_eq!(decoded, data, "Failed for length {}", len);
        }
    }

    #[test]
    fn test_avx2_various_lengths() {
        // Test AVX2 with various input lengths to catch edge cases
        for len in 0..100 {
            let data: Vec<u8> = (0..len).map(|i| i as u8).collect();
            let encoded = encode_avx2(&data, ALPHABET_STANDARD, true);
            let decoded = decode_avx2(&encoded, ALPHABET_STANDARD).unwrap();
            assert_eq!(decoded, data, "AVX2 failed for length {}", len);
        }
    }

    #[test]
    fn test_hex_alphabet_full() {
        // More comprehensive hex alphabet tests
        let test_cases = [
            b"".to_vec(),
            b"Hello".to_vec(),
            (0..=255).collect::<Vec<u8>>(),
            (0..1000).map(|i| (i % 256) as u8).collect::<Vec<u8>>(),
        ];

        for data in test_cases {
            let encoded = encode(&data, ALPHABET_HEX, true);
            let decoded = decode(&encoded, ALPHABET_HEX).unwrap();
            assert_eq!(decoded, data);

            // Also test without padding
            let encoded_no_pad = encode(&data, ALPHABET_HEX, false);
            let decoded_no_pad = decode(&encoded_no_pad, ALPHABET_HEX).unwrap();
            assert_eq!(decoded_no_pad, data);
        }
    }

    // Conformance tests against external base32 crate
    #[test]
    fn test_conformance_with_external_crate_encode() {
        let test_cases = [
            b"".to_vec(),
            b"f".to_vec(),
            b"fo".to_vec(),
            b"foo".to_vec(),
            b"foob".to_vec(),
            b"fooba".to_vec(),
            b"foobar".to_vec(),
            b"Hello, World!".to_vec(),
            (0..=255).collect::<Vec<u8>>(),
            (0..1000).map(|i| (i % 256) as u8).collect::<Vec<u8>>(),
        ];

        for data in &test_cases {
            let our_result = encode(data, ALPHABET_STANDARD, true);
            let external_result =
                base32_external::encode(base32_external::Alphabet::Rfc4648 { padding: true }, data);
            assert_eq!(
                our_result,
                external_result,
                "Encode mismatch for data len {}",
                data.len()
            );
        }
    }

    #[test]
    fn test_conformance_with_external_crate_decode() {
        let test_cases = [
            "JBSWY3DP",         // "Hello"
            "MZXW6YTBOI======", // "foobar"
            "GEZDGNBVGY3TQOJQ", // "12345678"
        ];

        for encoded in &test_cases {
            let our_result = decode(encoded, ALPHABET_STANDARD).unwrap();
            let external_result = base32_external::decode(
                base32_external::Alphabet::Rfc4648 { padding: true },
                encoded,
            )
            .unwrap();
            assert_eq!(
                our_result, external_result,
                "Decode mismatch for '{}'",
                encoded
            );
        }
    }

    #[test]
    fn test_conformance_roundtrip_with_external_crate() {
        // Encode with ours, decode with external
        let test_cases = [
            b"Hello, World!".to_vec(),
            (0..=255).collect::<Vec<u8>>(),
            (0..1000).map(|i| (i % 256) as u8).collect::<Vec<u8>>(),
        ];

        for data in &test_cases {
            // Our encode -> external decode
            let our_encoded = encode(data, ALPHABET_STANDARD, true);
            let external_decoded = base32_external::decode(
                base32_external::Alphabet::Rfc4648 { padding: true },
                &our_encoded,
            )
            .unwrap();
            assert_eq!(
                data,
                &external_decoded,
                "Our encode -> external decode failed for len {}",
                data.len()
            );

            // External encode -> our decode
            let external_encoded =
                base32_external::encode(base32_external::Alphabet::Rfc4648 { padding: true }, data);
            let our_decoded = decode(&external_encoded, ALPHABET_STANDARD).unwrap();
            assert_eq!(
                data,
                &our_decoded,
                "External encode -> our decode failed for len {}",
                data.len()
            );
        }
    }
}
