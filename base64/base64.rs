//! A library for base64 encoding and decoding.
//!
//! This library provides functions to encode binary data to base64 strings
//! and decode base64 strings back to binary data using custom alphabets.

use std::fmt;

#[cfg(target_arch = "x86_64")]
#[path = "base64_avx2.rs"]
mod avx2;

#[cfg(test)]
#[path = "base64_tests.rs"]
mod tests;

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
