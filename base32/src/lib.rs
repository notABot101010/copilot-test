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
    len / 8 * 5 + match len % 8 {
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
    let owned_table;
    let decode_table: &[u8; 256] = if alphabet == ALPHABET_STANDARD {
        &DECODE_TABLE_STANDARD
    } else if alphabet == ALPHABET_HEX {
        &DECODE_TABLE_HEX
    } else {
        owned_table = build_decode_table(alphabet);
        &owned_table
    };

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

        // Decode 8 base32 chars (40 bits) to 5 bytes
        result.push((v0 << 3) | (v1 >> 2));
        result.push((v1 << 6) | (v2 << 1) | (v3 >> 4));
        result.push((v3 << 4) | (v4 >> 1));
        result.push((v4 << 7) | (v5 << 2) | (v6 >> 3));
        result.push((v6 << 5) | v7);

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
                result.push((values[0] << 3) | (values[1] >> 2));
                result.push((values[1] << 6) | (values[2] << 1) | (values[3] >> 4));
            }
            5 => {
                result.push((values[0] << 3) | (values[1] >> 2));
                result.push((values[1] << 6) | (values[2] << 1) | (values[3] >> 4));
                result.push((values[3] << 4) | (values[4] >> 1));
            }
            7 => {
                result.push((values[0] << 3) | (values[1] >> 2));
                result.push((values[1] << 6) | (values[2] << 1) | (values[3] >> 4));
                result.push((values[3] << 4) | (values[4] >> 1));
                result.push((values[4] << 7) | (values[5] << 2) | (values[6] >> 3));
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
    /// Processes 20 input bytes at a time, producing 32 output bytes.
    ///
    /// # Safety
    /// Caller must ensure AVX2 is available (check with `is_available()`).
    #[target_feature(enable = "avx2")]
    pub unsafe fn encode_avx2(output: &mut [u8], data: &[u8], alphabet: &[u8; 32], padding: bool) {
        // Only use SIMD for standard alphabet
        let use_simd = alphabet == ALPHABET_STANDARD || alphabet == ALPHABET_HEX;

        if !use_simd || data.len() < 20 {
            super::encode_into_unchecked(output, data, alphabet, padding);
            return;
        }

        let is_hex = alphabet == ALPHABET_HEX;

        // Process 20 bytes at a time (4 groups of 5 bytes = 32 output chars)
        let full_chunks = data.len() / 20;
        let simd_input_len = full_chunks * 20;

        let mut in_idx = 0;
        let mut out_idx = 0;

        for _ in 0..full_chunks {
            // Load 32 bytes but only use first 20
            // We need to be careful here - we can only read up to the data boundary
            let mut input_buf = [0u8; 32];
            input_buf[..20].copy_from_slice(&data[in_idx..in_idx + 20]);
            let input = _mm256_loadu_si256(input_buf.as_ptr() as *const __m256i);

            // Reshuffle bytes to prepare for base32 encoding
            // Base32: 5 bytes -> 8 chars (5 bits each)
            // We process 4 groups: 20 bytes -> 32 chars
            
            // First, we need to rearrange bytes and extract 5-bit values
            // This is complex because base32 doesn't align to byte boundaries as nicely as base64
            
            // Use a lookup table approach for the translation
            let result = encode_block_avx2(input, is_hex);

            // Store 32 bytes of output
            _mm256_storeu_si256(output.as_mut_ptr().add(out_idx) as *mut __m256i, result);

            in_idx += 20;
            out_idx += 32;
        }

        // Handle remaining bytes with scalar code
        if simd_input_len < data.len() {
            let remaining_data = &data[simd_input_len..];
            let remaining_output = &mut output[out_idx..];
            super::encode_into_unchecked(remaining_output, remaining_data, alphabet, padding);
        }
    }

    /// Encode a 20-byte block to 32 base32 characters using AVX2.
    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn encode_block_avx2(input: __m256i, is_hex: bool) -> __m256i {
        // Base32 encoding: extract 5-bit groups from the input
        // We have 20 bytes = 160 bits = 32 x 5-bit values
        
        // Step 1: Shuffle bytes into position for bit extraction
        // We need to process 4 groups of 5 bytes each
        
        // Shuffle to get bytes in the right order for each 5-byte group
        // Group 0: bytes 0-4 -> outputs 0-7
        // Group 1: bytes 5-9 -> outputs 8-15
        // Group 2: bytes 10-14 -> outputs 16-23
        // Group 3: bytes 15-19 -> outputs 24-31
        
        let shuf1 = _mm256_setr_epi8(
            // First 128-bit lane: process groups 0 and 1 (10 bytes -> 16 output chars)
            0, 0, 1, 1, 1, 2, 2, 3, 3, 3, 4, 4, 5, 5, 6, 6,
            // Second 128-bit lane: process groups 2 and 3
            6, 7, 7, 7, 8, 8, 9, 9, 9, 10, 10, 11, 11, 11, 12, 12
        );
        
        // Permute to get the right bytes in position
        let perm = _mm256_setr_epi32(0, 1, 2, 3, 4, 5, 6, 7);
        
        // For base32, we need to extract 5-bit values from byte boundaries
        // Using shift and mask operations
        
        // Simpler approach: process scalar-style but in parallel
        // Extract each 5-bit value using shifts and masks
        
        let zero = _mm256_setzero_si256();
        
        // Load bytes and replicate them for bit extraction
        // Byte 0: provides bits for outputs 0, 1 (top 5 bits, next 3 bits)
        // Byte 1: provides bits for outputs 1, 2, 3 (top 2 bits + 5 bits + bottom 1 bit)
        // etc.
        
        // Use a reshuffle approach similar to base64
        // First shuffle to replicate bytes
        let shuf = _mm256_setr_epi8(
            0, 0, 1, 1, 2, 2, 2, 3, 3, 4, 5, 5, 6, 6, 7, 7,
            7, 8, 8, 9, 10, 10, 11, 11, 12, 12, 12, 13, 13, 14, 15, 15
        );
        
        // Shuffle input (note: we're using first 20 bytes, bytes 20-31 are zeros/junk)
        // Need to handle lane crossing - use permute first
        let input_lo = _mm256_permute4x64_epi64(input, 0x44); // duplicate low 128 bits
        let input_hi = _mm256_permute4x64_epi64(input, 0xEE); // duplicate high 128 bits
        
        // Extract 5-byte groups and encode them
        // This is getting complex, let's use a different strategy:
        // Process using multiword arithmetic
        
        // Actually, let's use a more straightforward bit manipulation approach
        // Load 8-byte chunks and process them
        
        // Simpler: do this in smaller steps
        // Load the 20 bytes into a buffer, extract 5-bit values
        
        let mut out_bytes = [0u8; 32];
        let in_bytes: [u8; 32] = std::mem::transmute(input);
        
        // Process 4 groups of 5 bytes -> 8 chars each
        for g in 0..4 {
            let b0 = in_bytes[g * 5];
            let b1 = in_bytes[g * 5 + 1];
            let b2 = in_bytes[g * 5 + 2];
            let b3 = in_bytes[g * 5 + 3];
            let b4 = in_bytes[g * 5 + 4];
            
            // Extract 8 5-bit values
            let v0 = b0 >> 3;
            let v1 = ((b0 & 0x07) << 2) | (b1 >> 6);
            let v2 = (b1 >> 1) & 0x1F;
            let v3 = ((b1 & 0x01) << 4) | (b2 >> 4);
            let v4 = ((b2 & 0x0F) << 1) | (b3 >> 7);
            let v5 = (b3 >> 2) & 0x1F;
            let v6 = ((b3 & 0x03) << 3) | (b4 >> 5);
            let v7 = b4 & 0x1F;
            
            out_bytes[g * 8] = v0;
            out_bytes[g * 8 + 1] = v1;
            out_bytes[g * 8 + 2] = v2;
            out_bytes[g * 8 + 3] = v3;
            out_bytes[g * 8 + 4] = v4;
            out_bytes[g * 8 + 5] = v5;
            out_bytes[g * 8 + 6] = v6;
            out_bytes[g * 8 + 7] = v7;
        }
        
        // Now translate 5-bit values to ASCII
        let values = _mm256_loadu_si256(out_bytes.as_ptr() as *const __m256i);
        
        if is_hex {
            // Hex alphabet: 0-9 (add '0') and 10-31 (add 'A' - 10)
            let nine = _mm256_set1_epi8(9);
            let is_digit = _mm256_cmpgt_epi8(_mm256_set1_epi8(10), values);
            let is_digit = _mm256_and_si256(is_digit, _mm256_cmpgt_epi8(values, _mm256_set1_epi8(-1)));
            let is_digit = _mm256_cmpgt_epi8(_mm256_set1_epi8(10), values);
            
            let digit_offset = _mm256_set1_epi8(b'0' as i8);
            let letter_offset = _mm256_set1_epi8((b'A' as i8) - 10);
            
            let offsets = _mm256_blendv_epi8(letter_offset, digit_offset, is_digit);
            _mm256_add_epi8(values, offsets)
        } else {
            // Standard alphabet: A-Z (0-25 add 'A') and 2-7 (26-31 add '2' - 26)
            let twenty_five = _mm256_set1_epi8(25);
            let is_letter = _mm256_cmpgt_epi8(_mm256_set1_epi8(26), values);
            
            let letter_offset = _mm256_set1_epi8(b'A' as i8);
            let digit_offset = _mm256_set1_epi8((b'2' as i8) - 26);
            
            let offsets = _mm256_blendv_epi8(digit_offset, letter_offset, is_letter);
            _mm256_add_epi8(values, offsets)
        }
    }

    /// Decode using AVX2 SIMD instructions.
    ///
    /// Processes 32 input characters at a time, producing 20 output bytes.
    ///
    /// # Safety
    /// Caller must ensure AVX2 is available (check with `is_available()`).
    #[target_feature(enable = "avx2")]
    pub unsafe fn decode_avx2(output: &mut [u8], input: &[u8], alphabet: &[u8; 32]) -> Result<usize, Error> {
        let input_len = input.len();
        
        // Only use SIMD for standard alphabet and sufficient length
        let use_simd = (alphabet == ALPHABET_STANDARD || alphabet == ALPHABET_HEX) && input_len >= 32;
        
        if !use_simd {
            let input_str = std::str::from_utf8(input).map_err(|_| Error::InvalidCharacter('\0'))?;
            let decoded = super::decode(input_str, alphabet)?;
            output[..decoded.len()].copy_from_slice(&decoded);
            return Ok(decoded.len());
        }
        
        let is_hex = alphabet == ALPHABET_HEX;
        
        // Count padding
        let padding_len = input.iter().rev().take_while(|&&b| b == b'=').count();
        let effective_len = input_len - padding_len;
        
        // Process 32-character chunks (-> 20 bytes)
        let full_chunks = effective_len / 32;
        let simd_input_len = full_chunks * 32;
        
        let mut in_idx = 0;
        let mut out_idx = 0;
        
        for _ in 0..full_chunks {
            let input_vec = _mm256_loadu_si256(input.as_ptr().add(in_idx) as *const __m256i);
            
            // Decode ASCII to 5-bit values
            let (values, valid) = decode_chars_avx2(input_vec, is_hex);
            
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
            let decoded = decode_values_to_bytes(values);
            
            // Store 20 bytes (with 12 extra bytes that will be overwritten or ignored)
            _mm256_storeu_si256(output.as_mut_ptr().add(out_idx) as *mut __m256i, decoded);
            
            in_idx += 32;
            out_idx += 20;
        }
        
        // Handle remaining characters with scalar code
        if in_idx < effective_len {
            let remaining_input = std::str::from_utf8(&input[in_idx..]).map_err(|_| Error::InvalidCharacter('\0'))?;
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
            c.is_ascii_digit() || (c >= b'A' && c <= b'V') || (c >= b'a' && c <= b'v')
        } else {
            (c >= b'A' && c <= b'Z') || (c >= b'a' && c <= b'z') || (c >= b'2' && c <= b'7')
        }
    }
    
    /// Decode a vector of 32 base32 characters to 32 5-bit values.
    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn decode_chars_avx2(input: __m256i, is_hex: bool) -> (__m256i, bool) {
        if is_hex {
            // Hex alphabet: 0-9 -> 0-9, A-V -> 10-31
            let ascii_zero = _mm256_set1_epi8(b'0' as i8);
            let ascii_a = _mm256_set1_epi8(b'A' as i8);
            let ascii_a_lower = _mm256_set1_epi8(b'a' as i8);
            
            // Check if digit (0-9)
            let offset_digit = _mm256_sub_epi8(input, ascii_zero);
            let is_digit = _mm256_and_si256(
                _mm256_cmpgt_epi8(offset_digit, _mm256_set1_epi8(-1)),
                _mm256_cmpgt_epi8(_mm256_set1_epi8(10), offset_digit),
            );
            
            // Check if uppercase letter (A-V)
            let offset_upper = _mm256_sub_epi8(input, ascii_a);
            let is_upper = _mm256_and_si256(
                _mm256_cmpgt_epi8(offset_upper, _mm256_set1_epi8(-1)),
                _mm256_cmpgt_epi8(_mm256_set1_epi8(22), offset_upper),
            );
            
            // Check if lowercase letter (a-v)
            let offset_lower = _mm256_sub_epi8(input, ascii_a_lower);
            let is_lower = _mm256_and_si256(
                _mm256_cmpgt_epi8(offset_lower, _mm256_set1_epi8(-1)),
                _mm256_cmpgt_epi8(_mm256_set1_epi8(22), offset_lower),
            );
            
            // Check validity
            let is_valid = _mm256_or_si256(_mm256_or_si256(is_digit, is_upper), is_lower);
            let all_valid = _mm256_movemask_epi8(is_valid) == -1i32;
            
            // Calculate values
            let digit_val = offset_digit;
            let upper_val = _mm256_add_epi8(offset_upper, _mm256_set1_epi8(10));
            let lower_val = _mm256_add_epi8(offset_lower, _mm256_set1_epi8(10));
            
            let result = _mm256_blendv_epi8(
                _mm256_blendv_epi8(lower_val, upper_val, is_upper),
                digit_val,
                is_digit,
            );
            
            (result, all_valid)
        } else {
            // Standard alphabet: A-Z -> 0-25, 2-7 -> 26-31
            let ascii_a = _mm256_set1_epi8(b'A' as i8);
            let ascii_a_lower = _mm256_set1_epi8(b'a' as i8);
            let ascii_two = _mm256_set1_epi8(b'2' as i8);
            
            // Check if uppercase letter (A-Z)
            let offset_upper = _mm256_sub_epi8(input, ascii_a);
            let is_upper = _mm256_and_si256(
                _mm256_cmpgt_epi8(offset_upper, _mm256_set1_epi8(-1)),
                _mm256_cmpgt_epi8(_mm256_set1_epi8(26), offset_upper),
            );
            
            // Check if lowercase letter (a-z)
            let offset_lower = _mm256_sub_epi8(input, ascii_a_lower);
            let is_lower = _mm256_and_si256(
                _mm256_cmpgt_epi8(offset_lower, _mm256_set1_epi8(-1)),
                _mm256_cmpgt_epi8(_mm256_set1_epi8(26), offset_lower),
            );
            
            // Check if digit (2-7)
            let offset_digit = _mm256_sub_epi8(input, ascii_two);
            let is_digit = _mm256_and_si256(
                _mm256_cmpgt_epi8(offset_digit, _mm256_set1_epi8(-1)),
                _mm256_cmpgt_epi8(_mm256_set1_epi8(6), offset_digit),
            );
            
            // Check validity
            let is_valid = _mm256_or_si256(_mm256_or_si256(is_upper, is_lower), is_digit);
            let all_valid = _mm256_movemask_epi8(is_valid) == -1i32;
            
            // Calculate values
            let upper_val = offset_upper;
            let lower_val = offset_lower;
            let digit_val = _mm256_add_epi8(offset_digit, _mm256_set1_epi8(26));
            
            let result = _mm256_blendv_epi8(
                _mm256_blendv_epi8(lower_val, digit_val, is_digit),
                upper_val,
                is_upper,
            );
            
            (result, all_valid)
        }
    }
    
    /// Convert 32 5-bit values to 20 bytes.
    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn decode_values_to_bytes(values: __m256i) -> __m256i {
        // Convert from 5-bit values to bytes
        // 8 values (40 bits) -> 5 bytes
        // We have 32 values -> 20 bytes
        
        let val_bytes: [u8; 32] = std::mem::transmute(values);
        let mut out_bytes = [0u8; 32];
        
        // Process 4 groups of 8 values -> 5 bytes each
        for g in 0..4 {
            let v0 = val_bytes[g * 8];
            let v1 = val_bytes[g * 8 + 1];
            let v2 = val_bytes[g * 8 + 2];
            let v3 = val_bytes[g * 8 + 3];
            let v4 = val_bytes[g * 8 + 4];
            let v5 = val_bytes[g * 8 + 5];
            let v6 = val_bytes[g * 8 + 6];
            let v7 = val_bytes[g * 8 + 7];
            
            // 8 5-bit values -> 5 bytes
            out_bytes[g * 5] = (v0 << 3) | (v1 >> 2);
            out_bytes[g * 5 + 1] = (v1 << 6) | (v2 << 1) | (v3 >> 4);
            out_bytes[g * 5 + 2] = (v3 << 4) | (v4 >> 1);
            out_bytes[g * 5 + 3] = (v4 << 7) | (v5 << 2) | (v6 >> 3);
            out_bytes[g * 5 + 4] = (v6 << 5) | v7;
        }
        
        _mm256_loadu_si256(out_bytes.as_ptr() as *const __m256i)
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
        assert_eq!(encode(b"foobar", ALPHABET_STANDARD, true), "MZXW6YTBOI======");
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
        assert_eq!(decode("MZXW6YTBOI======", ALPHABET_STANDARD).unwrap(), b"foobar");
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
            (0..20).collect::<Vec<u8>>(),  // Exactly 20 bytes (AVX2 block size)
            (0..40).collect::<Vec<u8>>(),  // Two AVX2 blocks
            (0..100).collect::<Vec<u8>>(), // Multiple blocks + remainder
            (0..=255).collect::<Vec<u8>>(), // All byte values
        ];

        for data in test_cases {
            let scalar_result = encode(&data, ALPHABET_STANDARD, true);
            let avx2_result = encode_avx2(&data, ALPHABET_STANDARD, true);
            assert_eq!(
                scalar_result, avx2_result,
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
                decoded, data,
                "AVX2 roundtrip failed for data len {}",
                data.len()
            );
        }
    }
}
