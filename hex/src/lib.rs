//! A library for hex encoding and decoding with AVX2 acceleration.
//!
//! This library provides functions to encode binary data to hexadecimal strings
//! and decode hexadecimal strings back to binary data using custom alphabets.

use std::fmt;

/// Standard lowercase hex alphabet (0-9, a-f).
pub const ALPHABET_LOWER: &[u8; 16] = b"0123456789abcdef";

/// Standard uppercase hex alphabet (0-9, A-F).
pub const ALPHABET_UPPER: &[u8; 16] = b"0123456789ABCDEF";

/// Pre-computed decode table that accepts both lower and upper case.
static DECODE_TABLE_MIXED: [u8; 256] = build_mixed_decode_table();

/// Builds a decode lookup table that accepts both lower and upper case hex.
const fn build_mixed_decode_table() -> [u8; 256] {
    let mut table = [255u8; 256];
    let mut i = 0u8;
    while i < 10 {
        table[(b'0' + i) as usize] = i;
        i += 1;
    }
    let mut i = 0u8;
    while i < 6 {
        table[(b'a' + i) as usize] = 10 + i;
        table[(b'A' + i) as usize] = 10 + i;
        i += 1;
    }
    table
}

/// Builds a decode lookup table for the given alphabet.
fn build_decode_table(alphabet: &[u8; 16]) -> [u8; 256] {
    let mut table = [255u8; 256];
    for (i, &c) in alphabet.iter().enumerate() {
        table[c as usize] = i as u8;
        // Also accept opposite case for letters
        if c.is_ascii_lowercase() {
            table[c.to_ascii_uppercase() as usize] = i as u8;
        } else if c.is_ascii_uppercase() {
            table[c.to_ascii_lowercase() as usize] = i as u8;
        }
    }
    table
}

/// Error type for hex encoding/decoding operations.
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Invalid character found in the input.
    InvalidCharacter(char),
    /// Invalid input length (must be even for decoding).
    InvalidLength,
    /// Output buffer too small.
    OutputBufferTooSmall,
    /// Invalid UTF-8 in input.
    InvalidUtf8,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidCharacter(c) => write!(f, "invalid character: '{}'", c),
            Error::InvalidLength => write!(f, "invalid input length (must be even)"),
            Error::OutputBufferTooSmall => write!(f, "output buffer too small"),
            Error::InvalidUtf8 => write!(f, "invalid UTF-8 in input"),
        }
    }
}

impl std::error::Error for Error {}

/// Calculates the encoded length for a given input length.
///
/// Each byte becomes 2 hex characters.
#[inline]
pub const fn encoded_len(len: usize) -> usize {
    len * 2
}

/// Calculates the decoded length for a given input length.
///
/// Each 2 hex characters become 1 byte.
#[inline]
pub const fn decoded_len(len: usize) -> usize {
    len / 2
}

/// Encodes binary data into a pre-allocated output buffer.
///
/// # Arguments
///
/// * `output` - The output buffer to write the encoded data to.
/// * `data` - The binary data to encode.
/// * `alphabet` - A 16-character alphabet used for encoding.
///
/// # Returns
///
/// `Ok(())` if successful, or an error if the output buffer is too small.
///
/// # Example
///
/// ```
/// use hex::{encode_into, ALPHABET_LOWER};
///
/// let data = b"Hello";
/// let mut output = [0u8; 10];
/// encode_into(&mut output, data, ALPHABET_LOWER).unwrap();
/// assert_eq!(&output, b"48656c6c6f");
/// ```
#[inline]
pub fn encode_into(output: &mut [u8], data: &[u8], alphabet: &[u8; 16]) -> Result<(), Error> {
    let required_len = encoded_len(data.len());
    if output.len() < required_len {
        return Err(Error::OutputBufferTooSmall);
    }

    encode_into_unchecked(output, data, alphabet);
    Ok(())
}

/// Encodes binary data into a pre-allocated output buffer without bounds checking.
#[inline]
fn encode_into_unchecked(output: &mut [u8], data: &[u8], alphabet: &[u8; 16]) {
    // Process 8 bytes at a time for better instruction-level parallelism
    let chunks_8 = data.len() / 8;
    let mut in_idx = 0;
    let mut out_idx = 0;

    for _ in 0..chunks_8 {
        // Unroll 8 bytes
        let b0 = data[in_idx];
        let b1 = data[in_idx + 1];
        let b2 = data[in_idx + 2];
        let b3 = data[in_idx + 3];
        let b4 = data[in_idx + 4];
        let b5 = data[in_idx + 5];
        let b6 = data[in_idx + 6];
        let b7 = data[in_idx + 7];

        output[out_idx] = alphabet[(b0 >> 4) as usize];
        output[out_idx + 1] = alphabet[(b0 & 0x0F) as usize];
        output[out_idx + 2] = alphabet[(b1 >> 4) as usize];
        output[out_idx + 3] = alphabet[(b1 & 0x0F) as usize];
        output[out_idx + 4] = alphabet[(b2 >> 4) as usize];
        output[out_idx + 5] = alphabet[(b2 & 0x0F) as usize];
        output[out_idx + 6] = alphabet[(b3 >> 4) as usize];
        output[out_idx + 7] = alphabet[(b3 & 0x0F) as usize];
        output[out_idx + 8] = alphabet[(b4 >> 4) as usize];
        output[out_idx + 9] = alphabet[(b4 & 0x0F) as usize];
        output[out_idx + 10] = alphabet[(b5 >> 4) as usize];
        output[out_idx + 11] = alphabet[(b5 & 0x0F) as usize];
        output[out_idx + 12] = alphabet[(b6 >> 4) as usize];
        output[out_idx + 13] = alphabet[(b6 & 0x0F) as usize];
        output[out_idx + 14] = alphabet[(b7 >> 4) as usize];
        output[out_idx + 15] = alphabet[(b7 & 0x0F) as usize];

        in_idx += 8;
        out_idx += 16;
    }

    // Handle remaining bytes
    for &byte in &data[in_idx..] {
        output[out_idx] = alphabet[(byte >> 4) as usize];
        output[out_idx + 1] = alphabet[(byte & 0x0F) as usize];
        out_idx += 2;
    }
}

/// Encodes binary data to a hex string using the specified alphabet.
///
/// # Arguments
///
/// * `data` - The binary data to encode.
/// * `alphabet` - A 16-character alphabet used for encoding.
///
/// # Returns
///
/// A hex-encoded string.
///
/// # Example
///
/// ```
/// use hex::{encode, ALPHABET_LOWER};
///
/// let encoded = encode(b"Hello", ALPHABET_LOWER);
/// assert_eq!(encoded, "48656c6c6f");
/// ```
#[inline]
pub fn encode(data: &[u8], alphabet: &[u8; 16]) -> String {
    if data.is_empty() {
        return String::new();
    }

    let output_len = encoded_len(data.len());
    let mut output = vec![0u8; output_len];

    encode_into_unchecked(&mut output, data, alphabet);

    // SAFETY: All bytes in output are valid ASCII characters from the hex alphabet,
    // which is a subset of valid UTF-8
    unsafe { String::from_utf8_unchecked(output) }
}

/// Decodes a hex string to binary data using the specified alphabet.
///
/// # Arguments
///
/// * `input` - The hex-encoded string to decode.
/// * `alphabet` - A 16-character alphabet used for decoding (case-insensitive).
///
/// # Returns
///
/// The decoded binary data as a `Vec<u8>`.
///
/// # Panics
///
/// Panics if the input contains invalid characters or has an odd length.
/// Use `decode_checked` for a non-panicking version.
///
/// # Example
///
/// ```
/// use hex::{decode, ALPHABET_LOWER};
///
/// let decoded = decode("48656c6c6f", ALPHABET_LOWER);
/// assert_eq!(decoded, b"Hello");
/// ```
#[inline]
pub fn decode(input: &str, alphabet: &[u8; 16]) -> Vec<u8> {
    decode_checked(input, alphabet).expect("invalid hex input")
}

/// Decodes a hex string to binary data, returning an error on invalid input.
///
/// # Arguments
///
/// * `input` - The hex-encoded string to decode.
/// * `alphabet` - A 16-character alphabet used for decoding (case-insensitive).
///
/// # Returns
///
/// A `Result` containing either the decoded binary data or an error.
#[inline]
pub fn decode_checked(input: &str, alphabet: &[u8; 16]) -> Result<Vec<u8>, Error> {
    if input.is_empty() {
        return Ok(Vec::new());
    }

    let input_bytes = input.as_bytes();

    if input_bytes.len() % 2 != 0 {
        return Err(Error::InvalidLength);
    }

    // Use pre-computed table for known alphabets, otherwise build dynamically
    let owned_table;
    let decode_table: &[u8; 256] = if alphabet == ALPHABET_LOWER || alphabet == ALPHABET_UPPER {
        &DECODE_TABLE_MIXED
    } else {
        owned_table = build_decode_table(alphabet);
        &owned_table
    };

    let output_len = decoded_len(input_bytes.len());
    let mut result = Vec::with_capacity(output_len);

    // Process 8 hex pairs at a time for better instruction-level parallelism
    let chunks_8 = input_bytes.len() / 16;
    let mut in_idx = 0;

    for _ in 0..chunks_8 {
        // Decode 8 pairs (16 hex chars -> 8 bytes)
        let mut valid = true;
        let mut bytes = [0u8; 8];

        for i in 0..8 {
            let hi = decode_table[input_bytes[in_idx + i * 2] as usize];
            let lo = decode_table[input_bytes[in_idx + i * 2 + 1] as usize];
            if hi == 255 || lo == 255 {
                valid = false;
                break;
            }
            bytes[i] = (hi << 4) | lo;
        }

        if !valid {
            // Find the invalid character
            for i in 0..16 {
                let c = input_bytes[in_idx + i];
                if decode_table[c as usize] == 255 {
                    return Err(Error::InvalidCharacter(c as char));
                }
            }
        }

        result.extend_from_slice(&bytes);
        in_idx += 16;
    }

    // Handle remaining pairs
    while in_idx < input_bytes.len() {
        let hi_char = input_bytes[in_idx];
        let lo_char = input_bytes[in_idx + 1];
        let hi = decode_table[hi_char as usize];
        let lo = decode_table[lo_char as usize];

        if hi == 255 {
            return Err(Error::InvalidCharacter(hi_char as char));
        }
        if lo == 255 {
            return Err(Error::InvalidCharacter(lo_char as char));
        }

        result.push((hi << 4) | lo);
        in_idx += 2;
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

    /// Extra bytes allocated for SIMD writes that may overshoot.
    pub const AVX2_EXTRA_BYTES: usize = 32;

    /// Check if AVX2 is available at runtime.
    #[inline]
    pub fn is_available() -> bool {
        is_x86_feature_detected!("avx2")
    }

    /// Encode using AVX2 SIMD instructions.
    ///
    /// Processes 32 input bytes at a time, producing 64 output bytes.
    ///
    /// # Safety
    /// Caller must ensure AVX2 is available (check with `is_available()`).
    #[target_feature(enable = "avx2")]
    pub unsafe fn encode_avx2(output: &mut [u8], data: &[u8], alphabet: &[u8; 16]) {
        // Only use SIMD for lowercase alphabet (we can optimize for specific cases)
        let use_simd = alphabet == ALPHABET_LOWER || alphabet == ALPHABET_UPPER;

        if !use_simd {
            super::encode_into_unchecked(output, data, alphabet);
            return;
        }

        let is_upper = alphabet == ALPHABET_UPPER;

        // Process 32 bytes at a time
        let full_chunks = data.len() / 32;
        let simd_input_len = full_chunks * 32;

        let mut in_idx = 0;
        let mut out_idx = 0;

        // ASCII offset for a-f (lowercase) or A-F (uppercase)
        let letter_offset = if is_upper { b'A' - 10 } else { b'a' - 10 };

        for _ in 0..full_chunks {
            // Load 32 bytes of input
            let input = _mm256_loadu_si256(data.as_ptr().add(in_idx) as *const __m256i);

            // Split into high and low nibbles
            let mask_0f = _mm256_set1_epi8(0x0F);
            let lo_nibbles = _mm256_and_si256(input, mask_0f);
            let hi_nibbles = _mm256_and_si256(_mm256_srli_epi16(input, 4), mask_0f);

            // Convert nibbles to ASCII hex characters
            // If nibble < 10: add '0' (0x30)
            // If nibble >= 10: add 'a' - 10 (0x57) or 'A' - 10 (0x37)
            let nine = _mm256_set1_epi8(9);
            let ascii_zero = _mm256_set1_epi8(b'0' as i8);
            let letter_off = _mm256_set1_epi8(letter_offset as i8);

            // High nibbles
            let hi_is_letter = _mm256_cmpgt_epi8(hi_nibbles, nine);
            let hi_offset = _mm256_blendv_epi8(ascii_zero, letter_off, hi_is_letter);
            let hi_ascii = _mm256_add_epi8(hi_nibbles, hi_offset);

            // Low nibbles
            let lo_is_letter = _mm256_cmpgt_epi8(lo_nibbles, nine);
            let lo_offset = _mm256_blendv_epi8(ascii_zero, letter_off, lo_is_letter);
            let lo_ascii = _mm256_add_epi8(lo_nibbles, lo_offset);

            // Interleave high and low nibbles
            // We need to produce: h0 l0 h1 l1 h2 l2 ...
            let lo_interleaved = _mm256_unpacklo_epi8(hi_ascii, lo_ascii);
            let hi_interleaved = _mm256_unpackhi_epi8(hi_ascii, lo_ascii);

            // The unpack operation works on 128-bit lanes, so we need to permute
            // to get the correct order across lanes
            let result_lo = _mm256_permute2x128_si256(lo_interleaved, hi_interleaved, 0x20);
            let result_hi = _mm256_permute2x128_si256(lo_interleaved, hi_interleaved, 0x31);

            // Store 64 bytes of output
            _mm256_storeu_si256(output.as_mut_ptr().add(out_idx) as *mut __m256i, result_lo);
            _mm256_storeu_si256(
                output.as_mut_ptr().add(out_idx + 32) as *mut __m256i,
                result_hi,
            );

            in_idx += 32;
            out_idx += 64;
        }

        // Handle remaining bytes with scalar code
        if simd_input_len < data.len() {
            let remaining_data = &data[simd_input_len..];
            let remaining_output = &mut output[out_idx..];
            super::encode_into_unchecked(remaining_output, remaining_data, alphabet);
        }
    }

    /// Decode using AVX2 SIMD instructions.
    ///
    /// Processes 64 input bytes (hex chars) at a time, producing 32 output bytes.
    ///
    /// # Safety
    /// Caller must ensure AVX2 is available (check with `is_available()`).
    #[target_feature(enable = "avx2")]
    pub unsafe fn decode_avx2(output: &mut [u8], input: &[u8]) -> Result<usize, Error> {
        let input_len = input.len();

        if input_len % 2 != 0 {
            return Err(Error::InvalidLength);
        }

        // Process 64 hex chars at a time (32 output bytes)
        let full_chunks = input_len / 64;

        let mut in_idx = 0;
        let mut out_idx = 0;

        for _ in 0..full_chunks {
            // Load 64 bytes of hex input (two 256-bit loads)
            let input_lo = _mm256_loadu_si256(input.as_ptr().add(in_idx) as *const __m256i);
            let input_hi = _mm256_loadu_si256(input.as_ptr().add(in_idx + 32) as *const __m256i);

            // Decode each 256-bit vector to nibbles
            let (nibbles_lo, valid_lo) = decode_hex_chars(input_lo);
            let (nibbles_hi, valid_hi) = decode_hex_chars(input_hi);

            // Check for invalid characters
            if !valid_lo || !valid_hi {
                // Fall back to scalar to find the invalid character
                for i in 0..64 {
                    let c = input[in_idx + i];
                    if !c.is_ascii_hexdigit() {
                        return Err(Error::InvalidCharacter(c as char));
                    }
                }
            }

            // Pack nibbles into bytes: each pair of nibbles becomes one byte
            // nibbles_lo: [n0, n1, n2, n3, ..., n31] - 32 nibbles from first 32 hex chars
            // nibbles_hi: [n32, n33, ..., n63] - 32 nibbles from next 32 hex chars
            // We need: [(n0<<4)|n1, (n2<<4)|n3, ...] -> 16 bytes from each vector

            // Use maddubs to combine pairs: multiply first by 16 and add second
            // maddubs: (a[0]*b[0] + a[1]*b[1], a[2]*b[2] + a[3]*b[3], ...)
            // We want: n0*16 + n1, n2*16 + n3, ...
            let mult = _mm256_set1_epi16(0x0110); // [16, 1, 16, 1, ...]
            
            let packed_lo = _mm256_maddubs_epi16(nibbles_lo, mult);
            let packed_hi = _mm256_maddubs_epi16(nibbles_hi, mult);
            
            // packed_lo now has 16 values in 16-bit slots: [b0, b1, ..., b15]
            // packed_hi now has 16 values in 16-bit slots: [b16, b17, ..., b31]
            
            // Pack 16-bit to 8-bit using packus
            // packus takes two vectors and packs them, but operates on 128-bit lanes separately
            // Result: [lo_lane0, hi_lane0, lo_lane1, hi_lane1]
            let packed = _mm256_packus_epi16(packed_lo, packed_hi);
            
            // Permute to get correct order across lanes
            // After packus: [b0-b7, b16-b23, b8-b15, b24-b31]
            // We want: [b0-b7, b8-b15, b16-b23, b24-b31]
            let result = _mm256_permute4x64_epi64(packed, 0b11011000);

            // Store 32 bytes of output
            _mm256_storeu_si256(output.as_mut_ptr().add(out_idx) as *mut __m256i, result);

            in_idx += 64;
            out_idx += 32;
        }

        // Handle remaining bytes with scalar code
        if in_idx < input_len {
            let remaining_input =
                std::str::from_utf8(&input[in_idx..]).map_err(|_| Error::InvalidUtf8)?;
            let decoded = super::decode_checked(remaining_input, ALPHABET_LOWER)?;
            output[out_idx..out_idx + decoded.len()].copy_from_slice(&decoded);
            out_idx += decoded.len();
        }

        Ok(out_idx)
    }

    /// Decode a vector of 32 hex characters to 32 nibbles (4-bit values).
    /// Returns (decoded_values, all_valid).
    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn decode_hex_chars(input: __m256i) -> (__m256i, bool) {
        // Character ranges:
        // '0'-'9' = 0x30-0x39 -> values 0-9
        // 'A'-'F' = 0x41-0x46 -> values 10-15
        // 'a'-'f' = 0x61-0x66 -> values 10-15

        let ascii_zero = _mm256_set1_epi8(b'0' as i8);
        let ascii_a = _mm256_set1_epi8(b'a' as i8);
        let ascii_upper_a = _mm256_set1_epi8(b'A' as i8);
        let six = _mm256_set1_epi8(6);
        let ten = _mm256_set1_epi8(10);

        // Check if in '0'-'9' range
        let offset_digit = _mm256_sub_epi8(input, ascii_zero);
        let is_digit = _mm256_and_si256(
            _mm256_cmpgt_epi8(offset_digit, _mm256_set1_epi8(-1)),
            _mm256_cmpgt_epi8(_mm256_set1_epi8(10), offset_digit),
        );

        // Check if in 'a'-'f' range
        let offset_lower = _mm256_sub_epi8(input, ascii_a);
        let is_lower = _mm256_and_si256(
            _mm256_cmpgt_epi8(offset_lower, _mm256_set1_epi8(-1)),
            _mm256_cmpgt_epi8(six, offset_lower),
        );

        // Check if in 'A'-'F' range
        let offset_upper = _mm256_sub_epi8(input, ascii_upper_a);
        let is_upper = _mm256_and_si256(
            _mm256_cmpgt_epi8(offset_upper, _mm256_set1_epi8(-1)),
            _mm256_cmpgt_epi8(six, offset_upper),
        );

        // Combine validity check
        let is_valid = _mm256_or_si256(_mm256_or_si256(is_digit, is_lower), is_upper);
        let all_valid = _mm256_movemask_epi8(is_valid) == -1i32;

        // Calculate nibble values
        // For digits: value = input - '0'
        // For lowercase: value = input - 'a' + 10
        // For uppercase: value = input - 'A' + 10
        let digit_val = offset_digit;
        let lower_val = _mm256_add_epi8(offset_lower, ten);
        let upper_val = _mm256_add_epi8(offset_upper, ten);

        // Select the correct value based on character type
        let result = _mm256_blendv_epi8(
            _mm256_blendv_epi8(upper_val, lower_val, is_lower),
            digit_val,
            is_digit,
        );

        (result, all_valid)
    }
}

/// Encode binary data to a hex string using AVX2 SIMD if available.
/// Falls back to scalar implementation if AVX2 is not available or for non-x86_64 architectures.
#[inline]
pub fn encode_avx2(data: &[u8], alphabet: &[u8; 16]) -> String {
    if data.is_empty() {
        return String::new();
    }

    #[cfg(target_arch = "x86_64")]
    {
        // Use AVX2 for sufficiently large inputs
        if avx2::is_available() && data.len() >= 32 {
            let output_len = encoded_len(data.len());
            let mut output = vec![0u8; output_len];

            // SAFETY: We just checked that AVX2 is available
            unsafe {
                avx2::encode_avx2(&mut output, data, alphabet);
            }

            return String::from_utf8(output).expect("hex output is always valid UTF-8");
        }
    }

    // Fall back to scalar implementation
    encode(data, alphabet)
}

/// Decode a hex string using AVX2 SIMD if available.
/// Falls back to scalar implementation if AVX2 is not available or for non-x86_64 architectures.
#[inline]
pub fn decode_avx2(input: &str, alphabet: &[u8; 16]) -> Vec<u8> {
    decode_avx2_checked(input, alphabet).expect("invalid hex input")
}

/// Decode a hex string using AVX2 SIMD if available, returning an error on invalid input.
#[inline]
pub fn decode_avx2_checked(input: &str, alphabet: &[u8; 16]) -> Result<Vec<u8>, Error> {
    if input.is_empty() {
        return Ok(Vec::new());
    }

    #[cfg(target_arch = "x86_64")]
    {
        // Use AVX2 for sufficiently large inputs
        if avx2::is_available() && input.len() >= 64 {
            let input_bytes = input.as_bytes();
            let output_len = decoded_len(input_bytes.len());
            let mut output = vec![0u8; output_len + avx2::AVX2_EXTRA_BYTES];

            // SAFETY: We just checked that AVX2 is available
            let decoded_len = unsafe { avx2::decode_avx2(&mut output, input_bytes)? };

            output.truncate(decoded_len);
            return Ok(output);
        }
    }

    // Fall back to scalar implementation
    decode_checked(input, alphabet)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoded_len() {
        assert_eq!(encoded_len(0), 0);
        assert_eq!(encoded_len(1), 2);
        assert_eq!(encoded_len(5), 10);
        assert_eq!(encoded_len(16), 32);
    }

    #[test]
    fn test_decoded_len() {
        assert_eq!(decoded_len(0), 0);
        assert_eq!(decoded_len(2), 1);
        assert_eq!(decoded_len(10), 5);
        assert_eq!(decoded_len(32), 16);
    }

    #[test]
    fn test_encode_empty() {
        assert_eq!(encode(b"", ALPHABET_LOWER), "");
        assert_eq!(encode(b"", ALPHABET_UPPER), "");
    }

    #[test]
    fn test_encode_lower() {
        assert_eq!(encode(b"Hello", ALPHABET_LOWER), "48656c6c6f");
        assert_eq!(encode(b"\x00\xff", ALPHABET_LOWER), "00ff");
        assert_eq!(encode(b"abc", ALPHABET_LOWER), "616263");
    }

    #[test]
    fn test_encode_upper() {
        assert_eq!(encode(b"Hello", ALPHABET_UPPER), "48656C6C6F");
        assert_eq!(encode(b"\x00\xff", ALPHABET_UPPER), "00FF");
        assert_eq!(encode(b"abc", ALPHABET_UPPER), "616263");
    }

    #[test]
    fn test_encode_into() {
        let data = b"Hello";
        let mut output = [0u8; 10];
        encode_into(&mut output, data, ALPHABET_LOWER).unwrap();
        assert_eq!(&output, b"48656c6c6f");
    }

    #[test]
    fn test_encode_into_buffer_too_small() {
        let data = b"Hello";
        let mut output = [0u8; 5];
        let result = encode_into(&mut output, data, ALPHABET_LOWER);
        assert!(matches!(result, Err(Error::OutputBufferTooSmall)));
    }

    #[test]
    fn test_decode_empty() {
        assert_eq!(decode("", ALPHABET_LOWER), Vec::<u8>::new());
    }

    #[test]
    fn test_decode_lower() {
        assert_eq!(decode("48656c6c6f", ALPHABET_LOWER), b"Hello");
        assert_eq!(decode("00ff", ALPHABET_LOWER), b"\x00\xff");
        assert_eq!(decode("616263", ALPHABET_LOWER), b"abc");
    }

    #[test]
    fn test_decode_upper() {
        assert_eq!(decode("48656C6C6F", ALPHABET_UPPER), b"Hello");
        assert_eq!(decode("00FF", ALPHABET_UPPER), b"\x00\xff");
    }

    #[test]
    fn test_decode_mixed_case() {
        // Should accept both cases
        assert_eq!(decode("48656C6c6F", ALPHABET_LOWER), b"Hello");
        assert_eq!(decode("aAbBcCdDeEfF", ALPHABET_LOWER), b"\xaa\xbb\xcc\xdd\xee\xff");
    }

    #[test]
    fn test_decode_invalid_character() {
        let result = decode_checked("ghij", ALPHABET_LOWER);
        assert!(matches!(result, Err(Error::InvalidCharacter('g'))));
    }

    #[test]
    fn test_decode_invalid_length() {
        let result = decode_checked("abc", ALPHABET_LOWER);
        assert!(matches!(result, Err(Error::InvalidLength)));
    }

    #[test]
    fn test_roundtrip() {
        let test_cases = [
            b"".to_vec(),
            b"a".to_vec(),
            b"ab".to_vec(),
            b"abc".to_vec(),
            b"Hello, World!".to_vec(),
            (0..=255).collect::<Vec<u8>>(),
        ];

        for data in test_cases {
            let encoded = encode(&data, ALPHABET_LOWER);
            let decoded = decode(&encoded, ALPHABET_LOWER);
            assert_eq!(decoded, data, "Roundtrip failed for {:?}", data);
        }
    }

    #[test]
    fn test_roundtrip_upper() {
        let test_cases = [
            b"".to_vec(),
            b"a".to_vec(),
            b"Hello, World!".to_vec(),
            (0..=255).collect::<Vec<u8>>(),
        ];

        for data in test_cases {
            let encoded = encode(&data, ALPHABET_UPPER);
            let decoded = decode(&encoded, ALPHABET_UPPER);
            assert_eq!(decoded, data, "Roundtrip failed for {:?}", data);
        }
    }

    #[test]
    fn test_error_display() {
        assert_eq!(
            format!("{}", Error::InvalidCharacter('g')),
            "invalid character: 'g'"
        );
        assert_eq!(
            format!("{}", Error::InvalidLength),
            "invalid input length (must be even)"
        );
        assert_eq!(
            format!("{}", Error::OutputBufferTooSmall),
            "output buffer too small"
        );
        assert_eq!(
            format!("{}", Error::InvalidUtf8),
            "invalid UTF-8 in input"
        );
    }

    // AVX2 tests
    #[test]
    fn test_encode_avx2_matches_scalar() {
        let test_cases = [
            b"".to_vec(),
            b"a".to_vec(),
            b"Hello, World!".to_vec(),
            (0..32).collect::<Vec<u8>>(), // Exactly 32 bytes (AVX2 block size)
            (0..64).collect::<Vec<u8>>(), // Two AVX2 blocks
            (0..100).collect::<Vec<u8>>(), // Multiple blocks + remainder
            (0..=255).collect::<Vec<u8>>(), // All byte values
        ];

        for data in test_cases {
            let scalar_result = encode(&data, ALPHABET_LOWER);
            let avx2_result = encode_avx2(&data, ALPHABET_LOWER);
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
            "61",
            "48656c6c6f",
            "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f", // 32 bytes
            "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f", // 64 bytes
        ];

        for encoded in test_cases {
            let scalar_result = decode_checked(encoded, ALPHABET_LOWER);
            let avx2_result = decode_avx2_checked(encoded, ALPHABET_LOWER);
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
            (0..32).collect::<Vec<u8>>(),
            (0..64).collect::<Vec<u8>>(),
            (0..100).collect::<Vec<u8>>(),
            (0..=255).collect::<Vec<u8>>(),
        ];

        for data in test_cases {
            let encoded = encode_avx2(&data, ALPHABET_LOWER);
            let decoded = decode_avx2(&encoded, ALPHABET_LOWER);
            assert_eq!(
                decoded, data,
                "AVX2 roundtrip failed for data len {}",
                data.len()
            );
        }
    }
}
