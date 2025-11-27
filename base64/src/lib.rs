//! A library for base64 encoding and decoding.
//!
//! This library provides functions to encode binary data to base64 strings
//! and decode base64 strings back to binary data using custom alphabets.

use std::fmt;

/// Standard base64 alphabet (RFC 4648).
pub const ALPHABET_STANDARD: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// URL-safe base64 alphabet (RFC 4648).
pub const ALPHABET_URL: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

/// Pre-computed decode table for the standard alphabet.
static DECODE_TABLE_STANDARD: [u8; 256] = build_decode_table_const(ALPHABET_STANDARD);

/// Pre-computed decode table for the URL-safe alphabet.
static DECODE_TABLE_URL: [u8; 256] = build_decode_table_const(ALPHABET_URL);

/// Builds a decode lookup table for the given alphabet at compile time.
const fn build_decode_table_const(alphabet: &[u8; 64]) -> [u8; 256] {
    let mut table = [255u8; 256];
    let mut i = 0;
    while i < 64 {
        table[alphabet[i] as usize] = i as u8;
        i += 1;
    }
    table
}

/// Builds a decode lookup table for the given alphabet.
fn build_decode_table(alphabet: &[u8; 64]) -> [u8; 256] {
    let mut table = [255u8; 256];
    for (i, &c) in alphabet.iter().enumerate() {
        table[c as usize] = i as u8;
    }
    table
}

/// Error type for base64 decoding operations.
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Invalid character found in the input.
    InvalidCharacter(char),
    /// Invalid padding in the input.
    InvalidPadding,
    /// Invalid input length.
    InvalidLength,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidCharacter(c) => write!(f, "invalid character: '{}'", c),
            Error::InvalidPadding => write!(f, "invalid padding"),
            Error::InvalidLength => write!(f, "invalid input length"),
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
/// The length of the base64-encoded output string.
///
/// # Example
///
/// ```
/// use base64::encoded_len;
///
/// assert_eq!(encoded_len(3, true), 4);
/// assert_eq!(encoded_len(1, true), 4);
/// assert_eq!(encoded_len(1, false), 2);
/// ```
#[inline]
pub fn encoded_len(len: usize, padding: bool) -> usize {
    if len == 0 {
        return 0;
    }

    if padding {
        // With padding: ceil(len / 3) * 4
        len.div_ceil(3) * 4
    } else {
        // Without padding: ceil(len * 4 / 3)
        let full_groups = len / 3;
        let remainder = len % 3;
        full_groups * 4 + if remainder == 0 { 0 } else { remainder + 1 }
    }
}

/// Encodes binary data to a base64 string using the specified alphabet.
///
/// # Arguments
///
/// * `data` - The binary data to encode.
/// * `alphabet` - A 64-character alphabet used for encoding.
/// * `padding` - Whether to add padding characters ('=') to the output.
///
/// # Returns
///
/// A base64-encoded string.
///
/// # Example
///
/// ```
/// use base64::encode_with;
///
/// const ALPHABET_STANDARD: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
///
/// let encoded = encode_with(b"Hello", ALPHABET_STANDARD, true);
/// assert_eq!(encoded, "SGVsbG8=");
/// ```
#[inline]
pub fn encode_with(data: &[u8], alphabet: &[u8; 64], padding: bool) -> String {
    if data.is_empty() {
        return String::new();
    }

    let output_len = encoded_len(data.len(), padding);
    let mut output = vec![0u8; output_len];

    encode_to_slice(&mut output, data, alphabet, padding);

    // SAFETY: All bytes in output are valid ASCII (from alphabet or '=')
    // which is valid UTF-8
    String::from_utf8(output).expect("base64 output is always valid UTF-8")
}

/// Encodes data directly into a pre-allocated slice for maximum performance.
///
/// # Arguments
///
/// * `output` - The output buffer to write the encoded data to.
/// * `data` - The binary data to encode.
/// * `alphabet` - A 64-character alphabet used for encoding.
/// * `padding` - Whether to add padding characters ('=') to the output.
#[inline]
pub fn encode_to_slice(output: &mut [u8], data: &[u8], alphabet: &[u8; 64], padding: bool) {
    let full_chunks = data.len() / 3;
    let remainder_len = data.len() % 3;

    // Process complete 3-byte groups - write directly to output buffer
    let mut out_idx = 0;
    let mut in_idx = 0;

    // Process 4 groups at a time for better instruction-level parallelism
    let chunks_4 = full_chunks / 4;
    for _ in 0..chunks_4 {
        // Group 1
        let b0 = data[in_idx];
        let b1 = data[in_idx + 1];
        let b2 = data[in_idx + 2];
        output[out_idx] = alphabet[(b0 >> 2) as usize];
        output[out_idx + 1] = alphabet[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize];
        output[out_idx + 2] = alphabet[(((b1 & 0x0F) << 2) | (b2 >> 6)) as usize];
        output[out_idx + 3] = alphabet[(b2 & 0x3F) as usize];

        // Group 2
        let b0 = data[in_idx + 3];
        let b1 = data[in_idx + 4];
        let b2 = data[in_idx + 5];
        output[out_idx + 4] = alphabet[(b0 >> 2) as usize];
        output[out_idx + 5] = alphabet[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize];
        output[out_idx + 6] = alphabet[(((b1 & 0x0F) << 2) | (b2 >> 6)) as usize];
        output[out_idx + 7] = alphabet[(b2 & 0x3F) as usize];

        // Group 3
        let b0 = data[in_idx + 6];
        let b1 = data[in_idx + 7];
        let b2 = data[in_idx + 8];
        output[out_idx + 8] = alphabet[(b0 >> 2) as usize];
        output[out_idx + 9] = alphabet[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize];
        output[out_idx + 10] = alphabet[(((b1 & 0x0F) << 2) | (b2 >> 6)) as usize];
        output[out_idx + 11] = alphabet[(b2 & 0x3F) as usize];

        // Group 4
        let b0 = data[in_idx + 9];
        let b1 = data[in_idx + 10];
        let b2 = data[in_idx + 11];
        output[out_idx + 12] = alphabet[(b0 >> 2) as usize];
        output[out_idx + 13] = alphabet[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize];
        output[out_idx + 14] = alphabet[(((b1 & 0x0F) << 2) | (b2 >> 6)) as usize];
        output[out_idx + 15] = alphabet[(b2 & 0x3F) as usize];

        in_idx += 12;
        out_idx += 16;
    }

    // Process remaining complete 3-byte groups
    for _ in 0..(full_chunks % 4) {
        let b0 = data[in_idx];
        let b1 = data[in_idx + 1];
        let b2 = data[in_idx + 2];
        output[out_idx] = alphabet[(b0 >> 2) as usize];
        output[out_idx + 1] = alphabet[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize];
        output[out_idx + 2] = alphabet[(((b1 & 0x0F) << 2) | (b2 >> 6)) as usize];
        output[out_idx + 3] = alphabet[(b2 & 0x3F) as usize];
        in_idx += 3;
        out_idx += 4;
    }

    // Handle remaining bytes
    match remainder_len {
        1 => {
            let b0 = data[in_idx];
            output[out_idx] = alphabet[(b0 >> 2) as usize];
            output[out_idx + 1] = alphabet[((b0 & 0x03) << 4) as usize];
            if padding {
                output[out_idx + 2] = b'=';
                output[out_idx + 3] = b'=';
            }
        }
        2 => {
            let b0 = data[in_idx];
            let b1 = data[in_idx + 1];
            output[out_idx] = alphabet[(b0 >> 2) as usize];
            output[out_idx + 1] = alphabet[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize];
            output[out_idx + 2] = alphabet[((b1 & 0x0F) << 2) as usize];
            if padding {
                output[out_idx + 3] = b'=';
            }
        }
        _ => {}
    }
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

    /// Reshuffle bytes for encoding: takes 24 bytes and produces 32 6-bit values.
    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn enc_reshuffle(input: __m256i) -> __m256i {
        // translation from SSE into AVX2 of procedure
        // https://github.com/WojciechMula/base64simd/blob/master/encode/unpack_bigendian.cpp
        let input: __m256i = _mm256_shuffle_epi8(
            input,
            _mm256_set_epi8(
                10, 11, 9, 10, 7, 8, 6, 7, 4, 5, 3, 4, 1, 2, 0, 1, 14, 15, 13, 14, 11, 12, 10, 11,
                8, 9, 7, 8, 5, 6, 4, 5,
            ),
        );

        let t0: __m256i = _mm256_and_si256(input, _mm256_set1_epi32(0x0fc0fc00u32 as i32));
        let t1: __m256i = _mm256_mulhi_epu16(t0, _mm256_set1_epi32(0x04000040));

        let t2 = _mm256_and_si256(input, _mm256_set1_epi32(0x003f03f0));
        let t3 = _mm256_mullo_epi16(t2, _mm256_set1_epi32(0x01000010));

        _mm256_or_si256(t1, t3)
    }

    /// Translate 6-bit indices to ASCII characters for the standard alphabet.
    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn enc_translate(input: __m256i) -> __m256i {
        let lut: __m256i = _mm256_setr_epi8(
            65, 71, -4, -4, -4, -4, -4, -4, -4, -4, -4, -4, -19, -16, 0, 0, 65, 71, -4, -4, -4, -4,
            -4, -4, -4, -4, -4, -4, -19, -16, 0, 0,
        );
        let mut indices = _mm256_subs_epu8(input, _mm256_set1_epi8(51));
        let mask = _mm256_cmpgt_epi8(input, _mm256_set1_epi8(25));
        indices = _mm256_sub_epi8(indices, mask);

        _mm256_add_epi8(input, _mm256_shuffle_epi8(lut, indices))
    }

    /// Reshuffle decoded 6-bit values into bytes.
    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn dec_reshuffle(input: __m256i) -> __m256i {
        let merge_ab_and_bc: __m256i = _mm256_maddubs_epi16(input, _mm256_set1_epi32(0x01400140));
        let out: __m256i = _mm256_madd_epi16(merge_ab_and_bc, _mm256_set1_epi32(0x00011000));

        let out = _mm256_shuffle_epi8(
            out,
            _mm256_setr_epi8(
                2, 1, 0, 6, 5, 4, 10, 9, 8, 14, 13, 12, -1, -1, -1, -1, 2, 1, 0, 6, 5, 4, 10, 9, 8,
                14, 13, 12, -1, -1, -1, -1,
            ),
        );
        _mm256_permutevar8x32_epi32(out, _mm256_setr_epi32(0, 1, 2, 4, 5, 6, -1, -1))
    }

    /// Encode using AVX2 SIMD instructions.
    ///
    /// # Safety
    /// Caller must ensure AVX2 is available (check with `is_available()`).
    #[target_feature(enable = "avx2")]
    pub unsafe fn encode_avx2(output: &mut [u8], data: &[u8], padding: bool) {
        // Process 24-byte chunks using iterator, producing 32 bytes of output each
        let full_chunks = data.len() / 24;
        let simd_input_len = full_chunks * 24;

        // Use chunks_exact for the SIMD path
        data.chunks_exact(24)
            .zip(output.chunks_exact_mut(32))
            .for_each(|(input_chunk, output_chunk)| {
                // Create a properly aligned buffer with 4 padding bytes at the start
                let mut aligned_buf = [0u8; 32];
                aligned_buf[4..28].copy_from_slice(input_chunk);

                // Load from offset 0 of our aligned buffer (which has the data at offset 4)
                let inputvector = _mm256_loadu_si256(aligned_buf.as_ptr() as *const __m256i);
                let reshuffled = enc_reshuffle(inputvector);
                let translated = enc_translate(reshuffled);

                _mm256_storeu_si256(output_chunk.as_mut_ptr() as *mut __m256i, translated);
            });

        // Handle remaining bytes with scalar code
        let remaining_data = &data[simd_input_len..];
        if !remaining_data.is_empty() {
            let out_offset = full_chunks * 32;
            let remaining_output = &mut output[out_offset..];
            super::encode_to_slice(remaining_output, remaining_data, ALPHABET_STANDARD, padding);
        }
    }

    /// Decode using AVX2 SIMD instructions.
    ///
    /// Returns the number of bytes written to output, or an error.
    ///
    /// # Safety
    /// Caller must ensure AVX2 is available (check with `is_available()`).
    #[target_feature(enable = "avx2")]
    pub unsafe fn decode_avx2(output: &mut [u8], input: &[u8]) -> Result<usize, Error> {
        let input_len = input.len();

        // We need at least 45 bytes for the AVX2 path (32 for SIMD + 13 for safety margin)
        // Calculate how many full 32-byte chunks we can safely process
        let safe_len = if input_len >= 45 { input_len - 13 } else { 0 };
        let full_chunks = safe_len / 32;

        // Lookup tables for decoding (moved outside loop for clarity)
        let lut_lo: __m256i = _mm256_setr_epi8(
            0x15, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x13, 0x1A, 0x1B, 0x1B,
            0x1B, 0x1A, 0x15, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x13, 0x1A,
            0x1B, 0x1B, 0x1B, 0x1A,
        );
        let lut_hi: __m256i = _mm256_setr_epi8(
            0x10, 0x10, 0x01, 0x02, 0x04, 0x08, 0x04, 0x08, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
            0x10, 0x10, 0x10, 0x10, 0x01, 0x02, 0x04, 0x08, 0x04, 0x08, 0x10, 0x10, 0x10, 0x10,
            0x10, 0x10, 0x10, 0x10,
        );
        let lut_roll: __m256i = _mm256_setr_epi8(
            0, 16, 19, 4, -65, -65, -71, -71, 0, 0, 0, 0, 0, 0, 0, 0, 0, 16, 19, 4, -65, -65, -71,
            -71, 0, 0, 0, 0, 0, 0, 0, 0,
        );
        let mask_2f: __m256i = _mm256_set1_epi8(0x2f);

        // Process chunks using iterator with try_fold to handle early exit on invalid chars
        let simd_input = &input[..full_chunks * 32];
        let simd_output = &mut output[..full_chunks * 24];

        let processed = simd_input
            .chunks_exact(32)
            .zip(simd_output.chunks_exact_mut(24))
            .try_fold(0usize, |chunks_done, (input_chunk, output_chunk)| {
                let str_vec = _mm256_loadu_si256(input_chunk.as_ptr() as *const __m256i);

                // Lookup
                let hi_nibbles: __m256i = _mm256_srli_epi32(str_vec, 4);
                let lo_nibbles: __m256i = _mm256_and_si256(str_vec, mask_2f);

                let lo: __m256i = _mm256_shuffle_epi8(lut_lo, lo_nibbles);
                let eq_2f: __m256i = _mm256_cmpeq_epi8(str_vec, mask_2f);

                let hi_nibbles = _mm256_and_si256(hi_nibbles, mask_2f);
                let hi: __m256i = _mm256_shuffle_epi8(lut_hi, hi_nibbles);
                let roll: __m256i =
                    _mm256_shuffle_epi8(lut_roll, _mm256_add_epi8(eq_2f, hi_nibbles));

                // Check for invalid characters - return None to break iteration
                if _mm256_testz_si256(lo, hi) == 0 {
                    return None;
                }

                let str_vec = _mm256_add_epi8(str_vec, roll);

                // Reshuffle to packed output
                let result = dec_reshuffle(str_vec);
                _mm256_storeu_si256(output_chunk.as_mut_ptr() as *mut __m256i, result);

                Some(chunks_done + 1)
            })
            .unwrap_or(0);

        let in_offset = processed * 32;
        let out_offset = processed * 24;

        // Handle remaining bytes with scalar decoder
        if in_offset < input_len {
            let remaining_input = &input[in_offset..];
            let remaining_output = &mut output[out_offset..];

            // Decode remaining bytes using scalar implementation
            let remaining_str =
                std::str::from_utf8(remaining_input).map_err(|_| Error::InvalidCharacter('\0'))?;
            let decoded = super::decode_with(remaining_str, ALPHABET_STANDARD)?;
            remaining_output[..decoded.len()].copy_from_slice(&decoded);
            return Ok(out_offset + decoded.len());
        }

        Ok(out_offset)
    }
}

/// Encode binary data to a base64 string using AVX2 SIMD if available.
/// Falls back to scalar implementation if AVX2 is not available or for non-x86_64 architectures.
///
/// # Arguments
///
/// * `data` - The binary data to encode.
/// * `alphabet` - A 64-character alphabet used for encoding.
/// * `padding` - Whether to add padding characters ('=') to the output.
///
/// # Returns
///
/// A base64-encoded string.
#[inline]
pub fn encode_with_avx2(data: &[u8], alphabet: &[u8; 64], padding: bool) -> String {
    if data.is_empty() {
        return String::new();
    }

    #[cfg(target_arch = "x86_64")]
    {
        // Only use AVX2 for standard alphabet and sufficiently large inputs
        if avx2::is_available() && alphabet == ALPHABET_STANDARD && data.len() >= 28 {
            let output_len = encoded_len(data.len(), padding);
            let mut output = vec![0u8; output_len];

            // SAFETY: We just checked that AVX2 is available
            unsafe {
                avx2::encode_avx2(&mut output, data, padding);
            }

            return String::from_utf8(output).expect("base64 output is always valid UTF-8");
        }
    }

    // Fall back to scalar implementation
    encode_with(data, alphabet, padding)
}

/// Decode a base64 string using AVX2 SIMD if available.
/// Falls back to scalar implementation if AVX2 is not available or for non-x86_64 architectures.
///
/// # Arguments
///
/// * `base64_input` - The base64-encoded string to decode.
/// * `alphabet` - A 64-character alphabet used for decoding.
///
/// # Returns
///
/// A `Result` containing either the decoded binary data or an error.
#[inline]
pub fn decode_with_avx2(base64_input: &str, alphabet: &[u8; 64]) -> Result<Vec<u8>, Error> {
    if base64_input.is_empty() {
        return Ok(Vec::new());
    }

    #[cfg(target_arch = "x86_64")]
    {
        // Only use AVX2 for standard alphabet and sufficiently large inputs
        if avx2::is_available() && alphabet == ALPHABET_STANDARD && base64_input.len() >= 45 {
            let input_bytes = base64_input.as_bytes();

            // Count padding
            let padding_len = input_bytes.iter().rev().take_while(|&&b| b == b'=').count();
            if padding_len > 2 {
                return Err(Error::InvalidPadding);
            }

            let input_len = input_bytes.len() - padding_len;

            // Calculate output size
            let full_groups = input_len / 4;
            let remainder_len = input_len % 4;
            let output_len = full_groups * 3
                + match remainder_len {
                    0 => 0,
                    2 => 1,
                    3 => 2,
                    1 => return Err(Error::InvalidLength),
                    _ => unreachable!(),
                };

            let mut output = vec![0u8; output_len + 32]; // Extra space for SIMD writes

            // SAFETY: We just checked that AVX2 is available
            let decoded_len = unsafe { avx2::decode_avx2(&mut output, input_bytes)? };

            output.truncate(decoded_len);
            return Ok(output);
        }
    }

    // Fall back to scalar implementation
    decode_with(base64_input, alphabet)
}

/// Decodes a base64 string to binary data using the specified alphabet.
///
/// # Arguments
///
/// * `base64_input` - The base64-encoded string to decode.
/// * `alphabet` - A 64-character alphabet used for decoding.
///
/// # Returns
///
/// A `Result` containing either the decoded binary data or an error.
///
/// # Example
///
/// ```
/// use base64::decode_with;
///
/// const ALPHABET_STANDARD: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
///
/// let decoded = decode_with("SGVsbG8=", ALPHABET_STANDARD).unwrap();
/// assert_eq!(decoded, b"Hello");
/// ```
#[inline]
pub fn decode_with(base64_input: &str, alphabet: &[u8; 64]) -> Result<Vec<u8>, Error> {
    if base64_input.is_empty() {
        return Ok(Vec::new());
    }

    // Use pre-computed table for known alphabets, otherwise build dynamically
    let owned_table;
    let decode_table: &[u8; 256] = if alphabet == ALPHABET_STANDARD {
        &DECODE_TABLE_STANDARD
    } else if alphabet == ALPHABET_URL {
        &DECODE_TABLE_URL
    } else {
        owned_table = build_decode_table(alphabet);
        &owned_table
    };

    let input_bytes = base64_input.as_bytes();

    // Count and validate padding
    let padding_len = input_bytes.iter().rev().take_while(|&&b| b == b'=').count();
    if padding_len > 2 {
        return Err(Error::InvalidPadding);
    }

    // Validate input length (with padding should be multiple of 4)
    if padding_len > 0 && !input_bytes.len().is_multiple_of(4) {
        return Err(Error::InvalidLength);
    }

    let input_len = input_bytes.len() - padding_len;

    // Calculate output size: each 4 input chars = 3 output bytes
    // For partial groups: 2 chars = 1 byte, 3 chars = 2 bytes
    let full_groups = input_len / 4;
    let remainder_len = input_len % 4;
    let output_len = full_groups * 3
        + match remainder_len {
            0 => 0,
            2 => 1,
            3 => 2,
            1 => return Err(Error::InvalidLength),
            _ => unreachable!(),
        };

    let mut result = Vec::with_capacity(output_len);

    // Process complete 4-character groups using a while loop for better optimization
    debug_assert!(full_groups * 4 <= input_bytes.len(), "bounds check failed");
    let mut in_ptr = input_bytes.as_ptr();
    let in_end = input_bytes.as_ptr().wrapping_add(full_groups * 4);

    while in_ptr < in_end {
        // SAFETY: We've verified that full_groups * 4 <= input_bytes.len() above,
        // and in_ptr stays within bounds [input_bytes.as_ptr(), in_end)
        let (c0, c1, c2, c3) = unsafe { (*in_ptr, *in_ptr.add(1), *in_ptr.add(2), *in_ptr.add(3)) };

        let v0 = decode_table[c0 as usize];
        let v1 = decode_table[c1 as usize];
        let v2 = decode_table[c2 as usize];
        let v3 = decode_table[c3 as usize];

        // Fast path: check all values at once
        if (v0 | v1 | v2 | v3) > 63 {
            // Slow path: find which character is invalid
            if v0 == 255 {
                return Err(Error::InvalidCharacter(c0 as char));
            }
            if v1 == 255 {
                return Err(Error::InvalidCharacter(c1 as char));
            }
            if v2 == 255 {
                return Err(Error::InvalidCharacter(c2 as char));
            }
            return Err(Error::InvalidCharacter(c3 as char));
        }

        // Decode and write as a batch
        result.extend_from_slice(&[(v0 << 2) | (v1 >> 4), (v1 << 4) | (v2 >> 2), (v2 << 6) | v3]);

        in_ptr = in_ptr.wrapping_add(4);
    }

    // Handle remaining characters
    let in_idx = full_groups * 4;
    match remainder_len {
        2 => {
            let c0 = input_bytes[in_idx];
            let c1 = input_bytes[in_idx + 1];
            let v0 = decode_table[c0 as usize];
            let v1 = decode_table[c1 as usize];
            if v0 == 255 {
                return Err(Error::InvalidCharacter(c0 as char));
            }
            if v1 == 255 {
                return Err(Error::InvalidCharacter(c1 as char));
            }
            result.push((v0 << 2) | (v1 >> 4));
        }
        3 => {
            let c0 = input_bytes[in_idx];
            let c1 = input_bytes[in_idx + 1];
            let c2 = input_bytes[in_idx + 2];
            let v0 = decode_table[c0 as usize];
            let v1 = decode_table[c1 as usize];
            let v2 = decode_table[c2 as usize];
            if v0 == 255 {
                return Err(Error::InvalidCharacter(c0 as char));
            }
            if v1 == 255 {
                return Err(Error::InvalidCharacter(c1 as char));
            }
            if v2 == 255 {
                return Err(Error::InvalidCharacter(c2 as char));
            }
            result.extend_from_slice(&[(v0 << 2) | (v1 >> 4), (v1 << 4) | (v2 >> 2)]);
        }
        _ => {}
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoded_len() {
        assert_eq!(encoded_len(0, true), 0);
        assert_eq!(encoded_len(0, false), 0);
        assert_eq!(encoded_len(1, true), 4);
        assert_eq!(encoded_len(1, false), 2);
        assert_eq!(encoded_len(2, true), 4);
        assert_eq!(encoded_len(2, false), 3);
        assert_eq!(encoded_len(3, true), 4);
        assert_eq!(encoded_len(3, false), 4);
        assert_eq!(encoded_len(4, true), 8);
        assert_eq!(encoded_len(4, false), 6);
        assert_eq!(encoded_len(5, true), 8);
        assert_eq!(encoded_len(5, false), 7);
        assert_eq!(encoded_len(6, true), 8);
        assert_eq!(encoded_len(6, false), 8);
    }

    #[test]
    fn test_encode_empty() {
        assert_eq!(encode_with(b"", ALPHABET_STANDARD, true), "");
        assert_eq!(encode_with(b"", ALPHABET_STANDARD, false), "");
    }

    #[test]
    fn test_encode_with_padding() {
        assert_eq!(encode_with(b"f", ALPHABET_STANDARD, true), "Zg==");
        assert_eq!(encode_with(b"fo", ALPHABET_STANDARD, true), "Zm8=");
        assert_eq!(encode_with(b"foo", ALPHABET_STANDARD, true), "Zm9v");
        assert_eq!(encode_with(b"foob", ALPHABET_STANDARD, true), "Zm9vYg==");
        assert_eq!(encode_with(b"fooba", ALPHABET_STANDARD, true), "Zm9vYmE=");
        assert_eq!(encode_with(b"foobar", ALPHABET_STANDARD, true), "Zm9vYmFy");
    }

    #[test]
    fn test_encode_without_padding() {
        assert_eq!(encode_with(b"f", ALPHABET_STANDARD, false), "Zg");
        assert_eq!(encode_with(b"fo", ALPHABET_STANDARD, false), "Zm8");
        assert_eq!(encode_with(b"foo", ALPHABET_STANDARD, false), "Zm9v");
        assert_eq!(encode_with(b"foob", ALPHABET_STANDARD, false), "Zm9vYg");
        assert_eq!(encode_with(b"fooba", ALPHABET_STANDARD, false), "Zm9vYmE");
        assert_eq!(encode_with(b"foobar", ALPHABET_STANDARD, false), "Zm9vYmFy");
    }

    #[test]
    fn test_encode_hello() {
        assert_eq!(encode_with(b"Hello", ALPHABET_STANDARD, true), "SGVsbG8=");
        assert_eq!(
            encode_with(b"Hello, World!", ALPHABET_STANDARD, true),
            "SGVsbG8sIFdvcmxkIQ=="
        );
    }

    #[test]
    fn test_encode_url_safe() {
        // Test data that would produce + or / in standard base64
        let data = [0xfb, 0xff, 0xfe];
        let standard = encode_with(&data, ALPHABET_STANDARD, true);
        let url_safe = encode_with(&data, ALPHABET_URL, true);
        assert!(standard.contains('+') || standard.contains('/'));
        assert!(!url_safe.contains('+') && !url_safe.contains('/'));
    }

    #[test]
    fn test_decode_empty() {
        assert_eq!(
            decode_with("", ALPHABET_STANDARD).unwrap(),
            Vec::<u8>::new()
        );
    }

    #[test]
    fn test_decode_with_padding() {
        assert_eq!(decode_with("Zg==", ALPHABET_STANDARD).unwrap(), b"f");
        assert_eq!(decode_with("Zm8=", ALPHABET_STANDARD).unwrap(), b"fo");
        assert_eq!(decode_with("Zm9v", ALPHABET_STANDARD).unwrap(), b"foo");
        assert_eq!(decode_with("Zm9vYg==", ALPHABET_STANDARD).unwrap(), b"foob");
        assert_eq!(
            decode_with("Zm9vYmE=", ALPHABET_STANDARD).unwrap(),
            b"fooba"
        );
        assert_eq!(
            decode_with("Zm9vYmFy", ALPHABET_STANDARD).unwrap(),
            b"foobar"
        );
    }

    #[test]
    fn test_decode_without_padding() {
        assert_eq!(decode_with("Zg", ALPHABET_STANDARD).unwrap(), b"f");
        assert_eq!(decode_with("Zm8", ALPHABET_STANDARD).unwrap(), b"fo");
        assert_eq!(decode_with("Zm9v", ALPHABET_STANDARD).unwrap(), b"foo");
        assert_eq!(decode_with("Zm9vYg", ALPHABET_STANDARD).unwrap(), b"foob");
        assert_eq!(decode_with("Zm9vYmE", ALPHABET_STANDARD).unwrap(), b"fooba");
        assert_eq!(
            decode_with("Zm9vYmFy", ALPHABET_STANDARD).unwrap(),
            b"foobar"
        );
    }

    #[test]
    fn test_decode_hello() {
        assert_eq!(
            decode_with("SGVsbG8=", ALPHABET_STANDARD).unwrap(),
            b"Hello"
        );
        assert_eq!(
            decode_with("SGVsbG8sIFdvcmxkIQ==", ALPHABET_STANDARD).unwrap(),
            b"Hello, World!"
        );
    }

    #[test]
    fn test_decode_invalid_character() {
        let result = decode_with("!!!!", ALPHABET_STANDARD);
        assert!(matches!(result, Err(Error::InvalidCharacter('!'))));
    }

    #[test]
    fn test_decode_invalid_length() {
        let result = decode_with("Z", ALPHABET_STANDARD);
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
            b"Hello, World!".to_vec(),
            (0..=255).collect::<Vec<u8>>(),
        ];

        for data in test_cases {
            let encoded = encode_with(&data, ALPHABET_STANDARD, true);
            let decoded = decode_with(&encoded, ALPHABET_STANDARD).unwrap();
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
            b"Hello, World!".to_vec(),
        ];

        for data in test_cases {
            let encoded = encode_with(&data, ALPHABET_STANDARD, false);
            let decoded = decode_with(&encoded, ALPHABET_STANDARD).unwrap();
            assert_eq!(
                decoded, data,
                "Roundtrip without padding failed for {:?}",
                data
            );
        }
    }

    #[test]
    fn test_url_safe_roundtrip() {
        let data: Vec<u8> = (0..=255).collect();
        let encoded = encode_with(&data, ALPHABET_URL, true);
        let decoded = decode_with(&encoded, ALPHABET_URL).unwrap();
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
    }

    #[test]
    fn test_encode_non_ascii_utf8() {
        // Test encoding UTF-8 strings with non-ASCII characters
        let data = "こんにちは".as_bytes(); // Japanese "Hello"
        let encoded = encode_with(data, ALPHABET_STANDARD, true);
        assert_eq!(encoded, "44GT44KT44Gr44Gh44Gv");

        let decoded = decode_with(&encoded, ALPHABET_STANDARD).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_encode_emoji() {
        // Test encoding emojis
        let data = "🎉🚀✨".as_bytes();
        let encoded = encode_with(data, ALPHABET_STANDARD, true);
        let decoded = decode_with(&encoded, ALPHABET_STANDARD).unwrap();
        assert_eq!(decoded, data);
        assert_eq!(String::from_utf8(decoded).unwrap(), "🎉🚀✨");
    }

    #[test]
    fn test_encode_mixed_ascii_non_ascii() {
        // Test encoding mixed ASCII and non-ASCII characters
        let data = "Hello, 世界! Привет!".as_bytes();
        let encoded = encode_with(data, ALPHABET_STANDARD, true);
        let decoded = decode_with(&encoded, ALPHABET_STANDARD).unwrap();
        assert_eq!(decoded, data);
        assert_eq!(String::from_utf8(decoded).unwrap(), "Hello, 世界! Привет!");
    }

    #[test]
    fn test_encode_various_unicode() {
        // Test various Unicode characters from different scripts
        let test_cases = [
            "Ελληνικά",    // Greek
            "עברית",       // Hebrew
            "العربية",     // Arabic
            "हिन्दी",       // Hindi
            "한국어",      // Korean
            "ไทย",         // Thai
            "café naïve",  // Latin with accents
            "Ñoño",        // Spanish
            "Ümlauts äöü", // German
        ];

        for text in test_cases {
            let data = text.as_bytes();
            let encoded = encode_with(data, ALPHABET_STANDARD, true);
            let decoded = decode_with(&encoded, ALPHABET_STANDARD).unwrap();
            assert_eq!(decoded, data, "Roundtrip failed for: {}", text);
            assert_eq!(
                String::from_utf8(decoded).unwrap(),
                text,
                "UTF-8 conversion failed for: {}",
                text
            );
        }
    }

    #[test]
    fn test_encode_binary_with_high_bytes() {
        // Test binary data with bytes > 127 (non-ASCII range)
        let data: Vec<u8> = (128..=255).collect();
        let encoded = encode_with(&data, ALPHABET_STANDARD, true);
        let decoded = decode_with(&encoded, ALPHABET_STANDARD).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_encode_null_and_control_chars() {
        // Test encoding data with null bytes and control characters
        let data = b"\x00\x01\x02\x1f\x7f\xff";
        let encoded = encode_with(data, ALPHABET_STANDARD, true);
        let decoded = decode_with(&encoded, ALPHABET_STANDARD).unwrap();
        assert_eq!(decoded, data);
    }

    // AVX2 tests
    #[test]
    fn test_encode_avx2_matches_scalar() {
        // Test that AVX2 encoding produces the same results as scalar
        let test_cases = [
            b"".to_vec(),
            b"a".to_vec(),
            b"ab".to_vec(),
            b"abc".to_vec(),
            b"Hello, World!".to_vec(),
            (0..24).collect::<Vec<u8>>(), // Exactly 24 bytes (AVX2 block size)
            (0..48).collect::<Vec<u8>>(), // Two AVX2 blocks
            (0..100).collect::<Vec<u8>>(), // Multiple blocks + remainder
            (0..=255).collect::<Vec<u8>>(), // All byte values
        ];

        for data in test_cases {
            let scalar_result = encode_with(&data, ALPHABET_STANDARD, true);
            let avx2_result = encode_with_avx2(&data, ALPHABET_STANDARD, true);
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
        // Test that AVX2 decoding produces the same results as scalar
        let test_cases = [
            "",
            "YQ==",
            "YWI=",
            "YWJj",
            "SGVsbG8sIFdvcmxkIQ==",
            "AAECAwQFBgcICQoLDA0ODxAREhMUFRYX", // 24 bytes encoded (32 chars)
            "AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8gISIjJCUmJygpKissLS4v", // 48 bytes encoded (64 chars)
        ];

        for encoded in test_cases {
            let scalar_result = decode_with(encoded, ALPHABET_STANDARD);
            let avx2_result = decode_with_avx2(encoded, ALPHABET_STANDARD);
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
        // Test roundtrip with AVX2 encode/decode
        let test_cases = [
            b"".to_vec(),
            b"a".to_vec(),
            b"ab".to_vec(),
            b"abc".to_vec(),
            b"Hello, World!".to_vec(),
            (0..24).collect::<Vec<u8>>(),
            (0..48).collect::<Vec<u8>>(),
            (0..100).collect::<Vec<u8>>(),
            (0..=255).collect::<Vec<u8>>(),
        ];

        for data in test_cases {
            let encoded = encode_with_avx2(&data, ALPHABET_STANDARD, true);
            let decoded = decode_with_avx2(&encoded, ALPHABET_STANDARD).unwrap();
            assert_eq!(
                decoded,
                data,
                "AVX2 roundtrip failed for data len {}",
                data.len()
            );
        }
    }
}
