//! A library for base32 encoding and decoding.
//!
//! This library provides functions to encode binary data to base32 strings
//! and decode base32 strings back to binary data using custom alphabets.

use std::fmt;
use std::sync::LazyLock;

/// Standard base32 alphabet (RFC 4648).
pub const ALPHABET_STANDARD: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

/// Extended hex base32 alphabet (RFC 4648).
pub const ALPHABET_HEX: &[u8; 32] = b"0123456789ABCDEFGHIJKLMNOPQRSTUV";

/// Crockford's base32 alphabet.
pub const ALPHABET_CROCKFORD: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

/// Pre-computed decode table for the standard alphabet.
static DECODE_TABLE_STANDARD: LazyLock<[u8; 256]> =
    LazyLock::new(|| build_decode_table(ALPHABET_STANDARD));

/// Pre-computed decode table for the hex alphabet.
static DECODE_TABLE_HEX: LazyLock<[u8; 256]> = LazyLock::new(|| build_decode_table(ALPHABET_HEX));

/// Pre-computed decode table for the Crockford alphabet.
static DECODE_TABLE_CROCKFORD: LazyLock<[u8; 256]> =
    LazyLock::new(|| build_decode_table(ALPHABET_CROCKFORD));

/// Builds a decode lookup table for the given alphabet.
fn build_decode_table(alphabet: &[u8; 32]) -> [u8; 256] {
    let mut table = [255u8; 256];
    for (i, &c) in alphabet.iter().enumerate() {
        table[c as usize] = i as u8;
    }
    table
}

/// Error type for base32 decoding operations.
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
#[inline]
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
        let remainder_chars = match remainder {
            0 => 0,
            1 => 2,
            2 => 4,
            3 => 5,
            4 => 7,
            _ => unreachable!(),
        };
        full_groups * 8 + remainder_chars
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
/// use base32::encode_with;
///
/// const ALPHABET_STANDARD: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
///
/// let encoded = encode_with(b"Hello", ALPHABET_STANDARD, true);
/// assert_eq!(encoded, "JBSWY3DP");
/// ```
#[inline]
pub fn encode_with(data: &[u8], alphabet: &[u8; 32], padding: bool) -> String {
    if data.is_empty() {
        return String::new();
    }

    let output_len = encoded_len(data.len(), padding);
    let mut output = Vec::with_capacity(output_len);

    // Process complete 5-byte groups (40 bits = 8 base32 characters)
    let chunks = data.chunks_exact(5);
    let remainder = chunks.remainder();

    for chunk in chunks {
        // Combine 5 bytes into a 40-bit number
        let n = ((chunk[0] as u64) << 32)
            | ((chunk[1] as u64) << 24)
            | ((chunk[2] as u64) << 16)
            | ((chunk[3] as u64) << 8)
            | (chunk[4] as u64);

        // Extract 8 5-bit groups and encode
        output.push(alphabet[((n >> 35) & 0x1F) as usize]);
        output.push(alphabet[((n >> 30) & 0x1F) as usize]);
        output.push(alphabet[((n >> 25) & 0x1F) as usize]);
        output.push(alphabet[((n >> 20) & 0x1F) as usize]);
        output.push(alphabet[((n >> 15) & 0x1F) as usize]);
        output.push(alphabet[((n >> 10) & 0x1F) as usize]);
        output.push(alphabet[((n >> 5) & 0x1F) as usize]);
        output.push(alphabet[(n & 0x1F) as usize]);
    }

    // Handle remaining bytes
    if !remainder.is_empty() {
        let mut n: u64 = 0;
        for (i, &byte) in remainder.iter().enumerate() {
            n |= (byte as u64) << (32 - i * 8);
        }

        // Output characters based on remainder length
        // 1 byte = 8 bits = 2 chars (covers 10 bits, 2 bits unused)
        // 2 bytes = 16 bits = 4 chars (covers 20 bits, 4 bits unused)
        // 3 bytes = 24 bits = 5 chars (covers 25 bits, 1 bit unused)
        // 4 bytes = 32 bits = 7 chars (covers 35 bits, 3 bits unused)
        let chars_to_output = match remainder.len() {
            1 => 2,
            2 => 4,
            3 => 5,
            4 => 7,
            _ => unreachable!(),
        };

        let shifts = [35, 30, 25, 20, 15, 10, 5, 0];
        for &shift in &shifts[..chars_to_output] {
            output.push(alphabet[((n >> shift) & 0x1F) as usize]);
        }

        if padding {
            let padding_chars = 8 - chars_to_output;
            for _ in 0..padding_chars {
                output.push(b'=');
            }
        }
    }

    // All bytes in output are valid ASCII (from alphabet or '='), so this won't fail
    String::from_utf8(output).expect("base32 output is always valid UTF-8")
}

/// Decodes a base32 string to binary data using the specified alphabet.
///
/// # Arguments
///
/// * `base32_input` - The base32-encoded string to decode.
/// * `alphabet` - A 32-character alphabet used for decoding.
///
/// # Returns
///
/// A `Result` containing either the decoded binary data or an error.
///
/// # Example
///
/// ```
/// use base32::decode_with;
///
/// const ALPHABET_STANDARD: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
///
/// let decoded = decode_with("JBSWY3DP", ALPHABET_STANDARD).unwrap();
/// assert_eq!(decoded, b"Hello");
/// ```
#[inline]
pub fn decode_with(base32_input: &str, alphabet: &[u8; 32]) -> Result<Vec<u8>, Error> {
    if base32_input.is_empty() {
        return Ok(Vec::new());
    }

    // Use pre-computed table for known alphabets, otherwise build dynamically
    let owned_table;
    let decode_table: &[u8; 256] = if alphabet == ALPHABET_STANDARD {
        &DECODE_TABLE_STANDARD
    } else if alphabet == ALPHABET_HEX {
        &DECODE_TABLE_HEX
    } else if alphabet == ALPHABET_CROCKFORD {
        &DECODE_TABLE_CROCKFORD
    } else {
        owned_table = build_decode_table(alphabet);
        &owned_table
    };

    let input_bytes = base32_input.as_bytes();

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

    // Validate unpadded length
    let remainder = input_len % 8;
    if remainder == 1 || remainder == 3 || remainder == 6 {
        return Err(Error::InvalidLength);
    }

    // Calculate output size
    let full_groups = input_len / 8;
    let output_len = full_groups * 5
        + match remainder {
            0 => 0,
            2 => 1,
            4 => 2,
            5 => 3,
            7 => 4,
            _ => return Err(Error::InvalidLength),
        };

    let mut result = Vec::with_capacity(output_len);

    // Process complete 8-character groups with unrolled loop
    let mut i = 0;
    while i + 8 <= input_len {
        // Read all 8 characters at once
        let chars = [
            input_bytes[i],
            input_bytes[i + 1],
            input_bytes[i + 2],
            input_bytes[i + 3],
            input_bytes[i + 4],
            input_bytes[i + 5],
            input_bytes[i + 6],
            input_bytes[i + 7],
        ];

        // Decode and validate all 8 characters
        let mut vals = [0u8; 8];
        for j in 0..8 {
            vals[j] = decode_table[chars[j] as usize];
            if vals[j] == 255 {
                return Err(Error::InvalidCharacter(chars[j] as char));
            }
        }

        let n = ((vals[0] as u64) << 35)
            | ((vals[1] as u64) << 30)
            | ((vals[2] as u64) << 25)
            | ((vals[3] as u64) << 20)
            | ((vals[4] as u64) << 15)
            | ((vals[5] as u64) << 10)
            | ((vals[6] as u64) << 5)
            | (vals[7] as u64);

        result.push(((n >> 32) & 0xFF) as u8);
        result.push(((n >> 24) & 0xFF) as u8);
        result.push(((n >> 16) & 0xFF) as u8);
        result.push(((n >> 8) & 0xFF) as u8);
        result.push((n & 0xFF) as u8);

        i += 8;
    }

    // Handle remaining characters
    let remaining = input_len - i;
    if remaining > 0 {
        let mut n: u64 = 0;
        for j in 0..remaining {
            let c = input_bytes[i + j];
            let v = decode_table[c as usize];
            if v == 255 {
                return Err(Error::InvalidCharacter(c as char));
            }
            n = (n << 5) | (v as u64);
        }

        // Shift to align properly
        n <<= (8 - remaining) * 5;

        // Output bytes based on remaining character count
        match remaining {
            2 => {
                result.push(((n >> 32) & 0xFF) as u8);
            }
            4 => {
                result.push(((n >> 32) & 0xFF) as u8);
                result.push(((n >> 24) & 0xFF) as u8);
            }
            5 => {
                result.push(((n >> 32) & 0xFF) as u8);
                result.push(((n >> 24) & 0xFF) as u8);
                result.push(((n >> 16) & 0xFF) as u8);
            }
            7 => {
                result.push(((n >> 32) & 0xFF) as u8);
                result.push(((n >> 24) & 0xFF) as u8);
                result.push(((n >> 16) & 0xFF) as u8);
                result.push(((n >> 8) & 0xFF) as u8);
            }
            _ => {}
        }
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
    fn test_encode_empty() {
        assert_eq!(encode_with(b"", ALPHABET_STANDARD, true), "");
        assert_eq!(encode_with(b"", ALPHABET_STANDARD, false), "");
    }

    #[test]
    fn test_encode_with_padding() {
        // RFC 4648 test vectors
        assert_eq!(encode_with(b"f", ALPHABET_STANDARD, true), "MY======");
        assert_eq!(encode_with(b"fo", ALPHABET_STANDARD, true), "MZXQ====");
        assert_eq!(encode_with(b"foo", ALPHABET_STANDARD, true), "MZXW6===");
        assert_eq!(encode_with(b"foob", ALPHABET_STANDARD, true), "MZXW6YQ=");
        assert_eq!(encode_with(b"fooba", ALPHABET_STANDARD, true), "MZXW6YTB");
        assert_eq!(encode_with(b"foobar", ALPHABET_STANDARD, true), "MZXW6YTBOI======");
    }

    #[test]
    fn test_encode_without_padding() {
        assert_eq!(encode_with(b"f", ALPHABET_STANDARD, false), "MY");
        assert_eq!(encode_with(b"fo", ALPHABET_STANDARD, false), "MZXQ");
        assert_eq!(encode_with(b"foo", ALPHABET_STANDARD, false), "MZXW6");
        assert_eq!(encode_with(b"foob", ALPHABET_STANDARD, false), "MZXW6YQ");
        assert_eq!(encode_with(b"fooba", ALPHABET_STANDARD, false), "MZXW6YTB");
        assert_eq!(encode_with(b"foobar", ALPHABET_STANDARD, false), "MZXW6YTBOI");
    }

    #[test]
    fn test_encode_hello() {
        assert_eq!(encode_with(b"Hello", ALPHABET_STANDARD, true), "JBSWY3DP");
        assert_eq!(encode_with(b"Hello, World!", ALPHABET_STANDARD, true), "JBSWY3DPFQQFO33SNRSCC===");
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
        assert_eq!(decode_with("MY======", ALPHABET_STANDARD).unwrap(), b"f");
        assert_eq!(decode_with("MZXQ====", ALPHABET_STANDARD).unwrap(), b"fo");
        assert_eq!(decode_with("MZXW6===", ALPHABET_STANDARD).unwrap(), b"foo");
        assert_eq!(decode_with("MZXW6YQ=", ALPHABET_STANDARD).unwrap(), b"foob");
        assert_eq!(decode_with("MZXW6YTB", ALPHABET_STANDARD).unwrap(), b"fooba");
        assert_eq!(decode_with("MZXW6YTBOI======", ALPHABET_STANDARD).unwrap(), b"foobar");
    }

    #[test]
    fn test_decode_without_padding() {
        assert_eq!(decode_with("MY", ALPHABET_STANDARD).unwrap(), b"f");
        assert_eq!(decode_with("MZXQ", ALPHABET_STANDARD).unwrap(), b"fo");
        assert_eq!(decode_with("MZXW6", ALPHABET_STANDARD).unwrap(), b"foo");
        assert_eq!(decode_with("MZXW6YQ", ALPHABET_STANDARD).unwrap(), b"foob");
        assert_eq!(decode_with("MZXW6YTB", ALPHABET_STANDARD).unwrap(), b"fooba");
        assert_eq!(decode_with("MZXW6YTBOI", ALPHABET_STANDARD).unwrap(), b"foobar");
    }

    #[test]
    fn test_decode_hello() {
        assert_eq!(
            decode_with("JBSWY3DP", ALPHABET_STANDARD).unwrap(),
            b"Hello"
        );
        assert_eq!(
            decode_with("JBSWY3DPFQQFO33SNRSCC===", ALPHABET_STANDARD).unwrap(),
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
        // Single character is invalid
        let result = decode_with("M", ALPHABET_STANDARD);
        assert!(matches!(result, Err(Error::InvalidLength)));
        
        // 3 characters is invalid
        let result = decode_with("MZX", ALPHABET_STANDARD);
        assert!(matches!(result, Err(Error::InvalidLength)));
        
        // 6 characters is invalid
        let result = decode_with("MZXW6Y", ALPHABET_STANDARD);
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
            b"abcdef".to_vec(),
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
            b"abcde".to_vec(),
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
    fn test_hex_alphabet() {
        let data = b"foobar";
        let encoded = encode_with(data, ALPHABET_HEX, true);
        let decoded = decode_with(&encoded, ALPHABET_HEX).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_crockford_alphabet() {
        let data = b"foobar";
        let encoded = encode_with(data, ALPHABET_CROCKFORD, true);
        let decoded = decode_with(&encoded, ALPHABET_CROCKFORD).unwrap();
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
    fn test_encode_binary_with_high_bytes() {
        let data: Vec<u8> = (128..=255).collect();
        let encoded = encode_with(&data, ALPHABET_STANDARD, true);
        let decoded = decode_with(&encoded, ALPHABET_STANDARD).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_encode_null_and_control_chars() {
        let data = b"\x00\x01\x02\x1f\x7f\xff";
        let encoded = encode_with(data, ALPHABET_STANDARD, true);
        let decoded = decode_with(&encoded, ALPHABET_STANDARD).unwrap();
        assert_eq!(decoded, data);
    }
}
