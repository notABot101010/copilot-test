//! A library for hexadecimal encoding and decoding.
//!
//! This library provides functions to encode binary data to hexadecimal strings
//! and decode hexadecimal strings back to binary data using custom alphabets.

use std::fmt;
use std::sync::LazyLock;

/// Standard lowercase hexadecimal alphabet.
pub const ALPHABET_LOWER: &[u8; 16] = b"0123456789abcdef";

/// Standard uppercase hexadecimal alphabet.
pub const ALPHABET_UPPER: &[u8; 16] = b"0123456789ABCDEF";

/// Pre-computed encode table for lowercase (each byte maps to 2 hex chars).
static ENCODE_TABLE_LOWER: LazyLock<[[u8; 2]; 256]> = LazyLock::new(|| build_encode_table(ALPHABET_LOWER));

/// Pre-computed encode table for uppercase (each byte maps to 2 hex chars).
static ENCODE_TABLE_UPPER: LazyLock<[[u8; 2]; 256]> = LazyLock::new(|| build_encode_table(ALPHABET_UPPER));

/// Builds an encode lookup table for the given alphabet.
fn build_encode_table(alphabet: &[u8; 16]) -> [[u8; 2]; 256] {
    let mut table = [[0u8; 2]; 256];
    for i in 0..256 {
        table[i][0] = alphabet[i >> 4];
        table[i][1] = alphabet[i & 0x0F];
    }
    table
}

/// Pre-computed decode table for hexadecimal (handles both cases).
static DECODE_TABLE: LazyLock<[u8; 256]> = LazyLock::new(|| {
    let mut table = [255u8; 256];
    for (i, c) in b"0123456789".iter().enumerate() {
        table[*c as usize] = i as u8;
    }
    for (i, c) in b"abcdef".iter().enumerate() {
        table[*c as usize] = (i + 10) as u8;
    }
    for (i, c) in b"ABCDEF".iter().enumerate() {
        table[*c as usize] = (i + 10) as u8;
    }
    table
});

/// Error type for hexadecimal decoding operations.
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Invalid character found in the input.
    InvalidCharacter(char),
    /// Invalid input length (must be even).
    InvalidLength,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidCharacter(c) => write!(f, "invalid character: '{}'", c),
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
///
/// # Returns
///
/// The length of the hexadecimal-encoded output string.
///
/// # Example
///
/// ```
/// use hex::encoded_len;
///
/// assert_eq!(encoded_len(1), 2);
/// assert_eq!(encoded_len(3), 6);
/// ```
#[inline]
pub fn encoded_len(len: usize) -> usize {
    len * 2
}

/// Encodes binary data to a hexadecimal string using the specified alphabet.
///
/// # Arguments
///
/// * `data` - The binary data to encode.
/// * `alphabet` - A 16-character alphabet used for encoding.
///
/// # Returns
///
/// A hexadecimal-encoded string.
///
/// # Example
///
/// ```
/// use hex::{encode_with, ALPHABET_LOWER, ALPHABET_UPPER};
///
/// let encoded = encode_with(b"Hello", ALPHABET_LOWER);
/// assert_eq!(encoded, "48656c6c6f");
///
/// let encoded_upper = encode_with(b"Hello", ALPHABET_UPPER);
/// assert_eq!(encoded_upper, "48656C6C6F");
/// ```
#[inline]
pub fn encode_with(data: &[u8], alphabet: &[u8; 16]) -> String {
    if data.is_empty() {
        return String::new();
    }

    let output_len = encoded_len(data.len());
    let mut output = Vec::with_capacity(output_len);

    // Use pre-computed table for known alphabets
    let owned_table;
    let encode_table: &[[u8; 2]; 256] = if alphabet == ALPHABET_LOWER {
        &ENCODE_TABLE_LOWER
    } else if alphabet == ALPHABET_UPPER {
        &ENCODE_TABLE_UPPER
    } else {
        owned_table = build_encode_table(alphabet);
        &owned_table
    };

    for &byte in data {
        let pair = encode_table[byte as usize];
        output.push(pair[0]);
        output.push(pair[1]);
    }

    // All bytes in output are valid ASCII (from alphabet), so this won't fail
    String::from_utf8(output).expect("hex output is always valid UTF-8")
}

/// Decodes a hexadecimal string to binary data.
///
/// # Arguments
///
/// * `hex_input` - The hexadecimal-encoded string to decode.
/// * `_alphabet` - Unused, kept for API consistency (decoding handles both cases).
///
/// # Returns
///
/// A `Result` containing either the decoded binary data or an error.
///
/// # Example
///
/// ```
/// use hex::{decode_with, ALPHABET_LOWER};
///
/// let decoded = decode_with("48656c6c6f", ALPHABET_LOWER).unwrap();
/// assert_eq!(decoded, b"Hello");
///
/// // Both cases work regardless of alphabet
/// let decoded = decode_with("48656C6C6F", ALPHABET_LOWER).unwrap();
/// assert_eq!(decoded, b"Hello");
/// ```
#[inline]
pub fn decode_with(hex_input: &str, _alphabet: &[u8; 16]) -> Result<Vec<u8>, Error> {
    if hex_input.is_empty() {
        return Ok(Vec::new());
    }

    let input_bytes = hex_input.as_bytes();

    // Hex input must have even length
    if input_bytes.len() % 2 != 0 {
        return Err(Error::InvalidLength);
    }

    let output_len = input_bytes.len() / 2;
    let mut result = Vec::with_capacity(output_len);
    let decode_table = &*DECODE_TABLE;

    let mut i = 0;
    while i < input_bytes.len() {
        let hi = input_bytes[i];
        let lo = input_bytes[i + 1];

        let hi_val = decode_table[hi as usize];
        let lo_val = decode_table[lo as usize];

        if hi_val == 255 {
            return Err(Error::InvalidCharacter(hi as char));
        }
        if lo_val == 255 {
            return Err(Error::InvalidCharacter(lo as char));
        }

        result.push((hi_val << 4) | lo_val);
        i += 2;
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoded_len() {
        assert_eq!(encoded_len(0), 0);
        assert_eq!(encoded_len(1), 2);
        assert_eq!(encoded_len(2), 4);
        assert_eq!(encoded_len(3), 6);
        assert_eq!(encoded_len(16), 32);
    }

    #[test]
    fn test_encode_empty() {
        assert_eq!(encode_with(b"", ALPHABET_LOWER), "");
        assert_eq!(encode_with(b"", ALPHABET_UPPER), "");
    }

    #[test]
    fn test_encode_lowercase() {
        assert_eq!(encode_with(b"Hello", ALPHABET_LOWER), "48656c6c6f");
        assert_eq!(encode_with(b"\x00\xff", ALPHABET_LOWER), "00ff");
    }

    #[test]
    fn test_encode_uppercase() {
        assert_eq!(encode_with(b"Hello", ALPHABET_UPPER), "48656C6C6F");
        assert_eq!(encode_with(b"\x00\xff", ALPHABET_UPPER), "00FF");
    }

    #[test]
    fn test_decode_empty() {
        assert_eq!(decode_with("", ALPHABET_LOWER).unwrap(), Vec::<u8>::new());
    }

    #[test]
    fn test_decode_lowercase() {
        assert_eq!(decode_with("48656c6c6f", ALPHABET_LOWER).unwrap(), b"Hello");
        assert_eq!(decode_with("00ff", ALPHABET_LOWER).unwrap(), b"\x00\xff");
    }

    #[test]
    fn test_decode_uppercase() {
        assert_eq!(decode_with("48656C6C6F", ALPHABET_LOWER).unwrap(), b"Hello");
        assert_eq!(decode_with("00FF", ALPHABET_LOWER).unwrap(), b"\x00\xff");
    }

    #[test]
    fn test_decode_mixed_case() {
        assert_eq!(decode_with("48656C6c6F", ALPHABET_LOWER).unwrap(), b"Hello");
    }

    #[test]
    fn test_decode_invalid_character() {
        let result = decode_with("zz", ALPHABET_LOWER);
        assert!(matches!(result, Err(Error::InvalidCharacter('z'))));

        let result = decode_with("gg", ALPHABET_LOWER);
        assert!(matches!(result, Err(Error::InvalidCharacter('g'))));
    }

    #[test]
    fn test_decode_invalid_length() {
        let result = decode_with("abc", ALPHABET_LOWER);
        assert!(matches!(result, Err(Error::InvalidLength)));

        let result = decode_with("a", ALPHABET_LOWER);
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
            let encoded = encode_with(&data, ALPHABET_LOWER);
            let decoded = decode_with(&encoded, ALPHABET_LOWER).unwrap();
            assert_eq!(decoded, data, "Roundtrip failed for {:?}", data);

            let encoded_upper = encode_with(&data, ALPHABET_UPPER);
            let decoded_upper = decode_with(&encoded_upper, ALPHABET_UPPER).unwrap();
            assert_eq!(decoded_upper, data, "Roundtrip (upper) failed for {:?}", data);
        }
    }

    #[test]
    fn test_error_display() {
        assert_eq!(
            format!("{}", Error::InvalidCharacter('z')),
            "invalid character: 'z'"
        );
        assert_eq!(format!("{}", Error::InvalidLength), "invalid input length");
    }

    #[test]
    fn test_encode_binary_with_high_bytes() {
        let data: Vec<u8> = (128..=255).collect();
        let encoded = encode_with(&data, ALPHABET_LOWER);
        let decoded = decode_with(&encoded, ALPHABET_LOWER).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_encode_null_and_control_chars() {
        let data = b"\x00\x01\x02\x1f\x7f\xff";
        let encoded = encode_with(data, ALPHABET_LOWER);
        let decoded = decode_with(&encoded, ALPHABET_LOWER).unwrap();
        assert_eq!(decoded, data);
    }
}
