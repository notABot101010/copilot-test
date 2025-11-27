//! A library for base32 encoding and decoding with AVX2 acceleration.
//!
//! This library provides functions to encode binary data to base32 strings
//! and decode base32 strings back to binary data using custom alphabets.

use std::fmt;

#[cfg(target_arch = "x86_64")]
#[path = "base32_avx2.rs"]
mod avx2;

#[cfg(test)]
#[path = "base32_tests.rs"]
mod tests;

/// RFC 4648 standard base32 alphabet (A-Z, 2-7).
const ALPHABET_STANDARD_BYTES: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

/// Extended hex base32 alphabet (0-9, A-V).
const ALPHABET_HEX_BYTES: &[u8; 32] = b"0123456789ABCDEFGHIJKLMNOPQRSTUV";

/// Pre-computed decode table for the standard alphabet (case-insensitive).
static DECODE_TABLE_STANDARD: [u8; 256] = build_decode_table_const(ALPHABET_STANDARD_BYTES);

/// Pre-computed decode table for the hex alphabet (case-insensitive).
static DECODE_TABLE_HEX: [u8; 256] = build_decode_table_const(ALPHABET_HEX_BYTES);

/// Alphabet for base32 encoding and decoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alphabet<'a> {
    /// RFC 4648 standard base32 alphabet: A-Z, 2-7
    Standard,
    /// Extended hex base32 alphabet: 0-9, A-V
    Hex,
    /// Custom 32-character alphabet.
    Custom(&'a [u8; 32]),
}

impl<'a> Alphabet<'a> {
    /// Returns the alphabet bytes.
    #[inline]
    pub fn bytes(&self) -> &[u8; 32] {
        match self {
            Alphabet::Standard => ALPHABET_STANDARD_BYTES,
            Alphabet::Hex => ALPHABET_HEX_BYTES,
            Alphabet::Custom(bytes) => bytes,
        }
    }

    /// Returns true if this is the standard alphabet.
    #[inline]
    fn is_standard(&self) -> bool {
        matches!(self, Alphabet::Standard)
    }

    /// Returns true if this is the hex alphabet.
    #[inline]
    fn is_hex(&self) -> bool {
        matches!(self, Alphabet::Hex)
    }
}

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
/// * `alphabet` - The alphabet to use for encoding.
/// * `padding` - Whether to add padding characters ('=') to the output.
///
/// # Returns
///
/// `Ok(())` if successful, or an error if the output buffer is too small.
///
/// # Example
///
/// ```
/// use base32::{encode_into, Alphabet};
///
/// let data = b"Hello";
/// let mut output = [0u8; 8];
/// encode_into(&mut output, data, Alphabet::Standard, true).unwrap();
/// assert_eq!(&output, b"JBSWY3DP");
/// ```
#[inline]
pub fn encode_into(
    output: &mut [u8],
    data: &[u8],
    alphabet: Alphabet,
    padding: bool,
) -> Result<(), Error> {
    let required_len = encoded_len(data.len(), padding);
    if output.len() < required_len {
        return Err(Error::OutputBufferTooSmall);
    }

    encode_into_unchecked(output, data, alphabet.bytes(), padding);
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
/// * `alphabet` - The alphabet to use for encoding.
/// * `padding` - Whether to add padding characters ('=') to the output.
///
/// # Returns
///
/// A base32-encoded string.
///
/// # Example
///
/// ```
/// use base32::{encode, Alphabet};
///
/// let encoded = encode(b"Hello", Alphabet::Standard, true);
/// assert_eq!(encoded, "JBSWY3DP");
/// ```
#[inline]
pub fn encode(data: &[u8], alphabet: Alphabet, padding: bool) -> String {
    if data.is_empty() {
        return String::new();
    }

    let output_len = encoded_len(data.len(), padding);
    let mut output = vec![0u8; output_len];

    encode_into_unchecked(&mut output, data, alphabet.bytes(), padding);

    // SAFETY: All bytes in output are valid ASCII (from alphabet or '=')
    // which is valid UTF-8
    String::from_utf8(output).expect("base32 output is always valid UTF-8")
}

/// Decodes a base32 string to binary data using the specified alphabet.
///
/// # Arguments
///
/// * `input` - The base32-encoded string to decode.
/// * `alphabet` - The alphabet to use for decoding (case-insensitive).
///
/// # Returns
///
/// A `Result` containing either the decoded binary data or an error.
///
/// # Example
///
/// ```
/// use base32::{decode, Alphabet};
///
/// let decoded = decode("JBSWY3DP", Alphabet::Standard).unwrap();
/// assert_eq!(decoded, b"Hello");
/// ```
#[inline]
pub fn decode(input: &str, alphabet: Alphabet) -> Result<Vec<u8>, Error> {
    if input.is_empty() {
        return Ok(Vec::new());
    }

    // Use pre-computed table for known alphabets, otherwise build dynamically
    let decode_table: &[u8; 256] = if alphabet.is_standard() {
        &DECODE_TABLE_STANDARD
    } else if alphabet.is_hex() {
        &DECODE_TABLE_HEX
    } else {
        // Scope owned_table to this branch only
        return decode_with_custom_alphabet(input, alphabet.bytes());
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

/// Encode binary data to a base32 string using AVX2 SIMD if available.
/// Falls back to scalar implementation if AVX2 is not available or for non-x86_64 architectures.
#[inline]
pub fn encode_avx2(data: &[u8], alphabet: Alphabet, padding: bool) -> String {
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
                avx2::encode_avx2(&mut output, data, alphabet.bytes(), padding);
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
pub fn decode_avx2(input: &str, alphabet: Alphabet) -> Result<Vec<u8>, Error> {
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
            let decoded_len = unsafe { avx2::decode_avx2(&mut output, input_bytes, alphabet.bytes())? };

            output.truncate(decoded_len);
            return Ok(output);
        }
    }

    // Fall back to scalar implementation
    decode(input, alphabet)
}
