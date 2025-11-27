//! A library for hex encoding and decoding with AVX2 acceleration.
//!
//! This library provides functions to encode binary data to hexadecimal strings
//! and decode hexadecimal strings back to binary data using custom alphabets.

use std::fmt;

#[cfg(target_arch = "x86_64")]
#[path = "hex_avx2.rs"]
mod avx2;

#[cfg(test)]
#[path = "hex_tests.rs"]
mod tests;

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

    if !input_bytes.len().is_multiple_of(2) {
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
