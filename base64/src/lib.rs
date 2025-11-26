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
pub fn encode_with(data: &[u8], alphabet: &[u8; 64], padding: bool) -> String {
    if data.is_empty() {
        return String::new();
    }

    let mut result = String::with_capacity(encoded_len(data.len(), padding));

    // Process complete 3-byte groups
    let chunks = data.chunks_exact(3);
    let remainder = chunks.remainder();

    for chunk in chunks {
        let n = ((chunk[0] as u32) << 16) | ((chunk[1] as u32) << 8) | (chunk[2] as u32);

        result.push(alphabet[((n >> 18) & 0x3F) as usize] as char);
        result.push(alphabet[((n >> 12) & 0x3F) as usize] as char);
        result.push(alphabet[((n >> 6) & 0x3F) as usize] as char);
        result.push(alphabet[(n & 0x3F) as usize] as char);
    }

    // Handle remaining bytes
    match remainder.len() {
        1 => {
            let n = (remainder[0] as u32) << 16;
            result.push(alphabet[((n >> 18) & 0x3F) as usize] as char);
            result.push(alphabet[((n >> 12) & 0x3F) as usize] as char);
            if padding {
                result.push('=');
                result.push('=');
            }
        }
        2 => {
            let n = ((remainder[0] as u32) << 16) | ((remainder[1] as u32) << 8);
            result.push(alphabet[((n >> 18) & 0x3F) as usize] as char);
            result.push(alphabet[((n >> 12) & 0x3F) as usize] as char);
            result.push(alphabet[((n >> 6) & 0x3F) as usize] as char);
            if padding {
                result.push('=');
            }
        }
        _ => {}
    }

    result
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
pub fn decode_with(base64_input: &str, alphabet: &[u8; 64]) -> Result<Vec<u8>, Error> {
    if base64_input.is_empty() {
        return Ok(Vec::new());
    }

    // Build reverse lookup table
    let mut decode_table = [255u8; 256];
    for (i, &c) in alphabet.iter().enumerate() {
        decode_table[c as usize] = i as u8;
    }

    // Remove padding and calculate expected output size
    let input = base64_input.trim_end_matches('=');
    let padding_len = base64_input.len() - input.len();

    // Validate padding
    if padding_len > 2 {
        return Err(Error::InvalidPadding);
    }

    // Validate input length (with padding should be multiple of 4)
    if !base64_input.is_empty() && padding_len > 0 && !base64_input.len().is_multiple_of(4) {
        return Err(Error::InvalidLength);
    }

    let input_bytes: Vec<u8> = input.bytes().collect();
    let input_len = input_bytes.len();

    // Calculate output size
    let output_len = (input_len * 3) / 4;
    let mut result = Vec::with_capacity(output_len);

    // Process complete 4-character groups
    let chunks = input_bytes.chunks_exact(4);
    let remainder = chunks.remainder();

    for chunk in chunks {
        let mut values = [0u8; 4];
        for (i, &c) in chunk.iter().enumerate() {
            let val = decode_table[c as usize];
            if val == 255 {
                return Err(Error::InvalidCharacter(c as char));
            }
            values[i] = val;
        }

        result.push((values[0] << 2) | (values[1] >> 4));
        result.push((values[1] << 4) | (values[2] >> 2));
        result.push((values[2] << 6) | values[3]);
    }

    // Handle remaining characters
    match remainder.len() {
        2 => {
            let val0 = decode_table[remainder[0] as usize];
            let val1 = decode_table[remainder[1] as usize];
            if val0 == 255 {
                return Err(Error::InvalidCharacter(remainder[0] as char));
            }
            if val1 == 255 {
                return Err(Error::InvalidCharacter(remainder[1] as char));
            }
            result.push((val0 << 2) | (val1 >> 4));
        }
        3 => {
            let val0 = decode_table[remainder[0] as usize];
            let val1 = decode_table[remainder[1] as usize];
            let val2 = decode_table[remainder[2] as usize];
            if val0 == 255 {
                return Err(Error::InvalidCharacter(remainder[0] as char));
            }
            if val1 == 255 {
                return Err(Error::InvalidCharacter(remainder[1] as char));
            }
            if val2 == 255 {
                return Err(Error::InvalidCharacter(remainder[2] as char));
            }
            result.push((val0 << 2) | (val1 >> 4));
            result.push((val1 << 4) | (val2 >> 2));
        }
        1 => {
            // Single character is invalid for base64
            return Err(Error::InvalidLength);
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
}
