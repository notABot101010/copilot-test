//! Tests for base64 encoding and decoding.

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
    assert_eq!(encode_with(b"", Alphabet::Standard, true), "");
    assert_eq!(encode_with(b"", Alphabet::Standard, false), "");
}

#[test]
fn test_encode_with_padding() {
    assert_eq!(encode_with(b"f", Alphabet::Standard, true), "Zg==");
    assert_eq!(encode_with(b"fo", Alphabet::Standard, true), "Zm8=");
    assert_eq!(encode_with(b"foo", Alphabet::Standard, true), "Zm9v");
    assert_eq!(encode_with(b"foob", Alphabet::Standard, true), "Zm9vYg==");
    assert_eq!(encode_with(b"fooba", Alphabet::Standard, true), "Zm9vYmE=");
    assert_eq!(encode_with(b"foobar", Alphabet::Standard, true), "Zm9vYmFy");
}

#[test]
fn test_encode_without_padding() {
    assert_eq!(encode_with(b"f", Alphabet::Standard, false), "Zg");
    assert_eq!(encode_with(b"fo", Alphabet::Standard, false), "Zm8");
    assert_eq!(encode_with(b"foo", Alphabet::Standard, false), "Zm9v");
    assert_eq!(encode_with(b"foob", Alphabet::Standard, false), "Zm9vYg");
    assert_eq!(encode_with(b"fooba", Alphabet::Standard, false), "Zm9vYmE");
    assert_eq!(encode_with(b"foobar", Alphabet::Standard, false), "Zm9vYmFy");
}

#[test]
fn test_encode_hello() {
    assert_eq!(encode_with(b"Hello", Alphabet::Standard, true), "SGVsbG8=");
    assert_eq!(
        encode_with(b"Hello, World!", Alphabet::Standard, true),
        "SGVsbG8sIFdvcmxkIQ=="
    );
}

#[test]
fn test_encode_url_safe() {
    // Test data that would produce + or / in standard base64
    let data = [0xfb, 0xff, 0xfe];
    let standard = encode_with(&data, Alphabet::Standard, true);
    let url_safe = encode_with(&data, Alphabet::Url, true);
    assert!(standard.contains('+') || standard.contains('/'));
    assert!(!url_safe.contains('+') && !url_safe.contains('/'));
}

#[test]
fn test_decode_empty() {
    assert_eq!(
        decode_with("", Alphabet::Standard).unwrap(),
        Vec::<u8>::new()
    );
}

#[test]
fn test_decode_with_padding() {
    assert_eq!(decode_with("Zg==", Alphabet::Standard).unwrap(), b"f");
    assert_eq!(decode_with("Zm8=", Alphabet::Standard).unwrap(), b"fo");
    assert_eq!(decode_with("Zm9v", Alphabet::Standard).unwrap(), b"foo");
    assert_eq!(decode_with("Zm9vYg==", Alphabet::Standard).unwrap(), b"foob");
    assert_eq!(
        decode_with("Zm9vYmE=", Alphabet::Standard).unwrap(),
        b"fooba"
    );
    assert_eq!(
        decode_with("Zm9vYmFy", Alphabet::Standard).unwrap(),
        b"foobar"
    );
}

#[test]
fn test_decode_without_padding() {
    assert_eq!(decode_with("Zg", Alphabet::Standard).unwrap(), b"f");
    assert_eq!(decode_with("Zm8", Alphabet::Standard).unwrap(), b"fo");
    assert_eq!(decode_with("Zm9v", Alphabet::Standard).unwrap(), b"foo");
    assert_eq!(decode_with("Zm9vYg", Alphabet::Standard).unwrap(), b"foob");
    assert_eq!(decode_with("Zm9vYmE", Alphabet::Standard).unwrap(), b"fooba");
    assert_eq!(
        decode_with("Zm9vYmFy", Alphabet::Standard).unwrap(),
        b"foobar"
    );
}

#[test]
fn test_decode_hello() {
    assert_eq!(
        decode_with("SGVsbG8=", Alphabet::Standard).unwrap(),
        b"Hello"
    );
    assert_eq!(
        decode_with("SGVsbG8sIFdvcmxkIQ==", Alphabet::Standard).unwrap(),
        b"Hello, World!"
    );
}

#[test]
fn test_decode_invalid_character() {
    let result = decode_with("!!!!", Alphabet::Standard);
    assert!(matches!(result, Err(Error::InvalidCharacter('!'))));
}

#[test]
fn test_decode_invalid_length() {
    let result = decode_with("Z", Alphabet::Standard);
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
        let encoded = encode_with(&data, Alphabet::Standard, true);
        let decoded = decode_with(&encoded, Alphabet::Standard).unwrap();
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
        let encoded = encode_with(&data, Alphabet::Standard, false);
        let decoded = decode_with(&encoded, Alphabet::Standard).unwrap();
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
    let encoded = encode_with(&data, Alphabet::Url, true);
    let decoded = decode_with(&encoded, Alphabet::Url).unwrap();
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
    let encoded = encode_with(data, Alphabet::Standard, true);
    assert_eq!(encoded, "44GT44KT44Gr44Gh44Gv");

    let decoded = decode_with(&encoded, Alphabet::Standard).unwrap();
    assert_eq!(decoded, data);
}

#[test]
fn test_encode_emoji() {
    // Test encoding emojis
    let data = "🎉🚀✨".as_bytes();
    let encoded = encode_with(data, Alphabet::Standard, true);
    let decoded = decode_with(&encoded, Alphabet::Standard).unwrap();
    assert_eq!(decoded, data);
    assert_eq!(String::from_utf8(decoded).unwrap(), "🎉🚀✨");
}

#[test]
fn test_encode_mixed_ascii_non_ascii() {
    // Test encoding mixed ASCII and non-ASCII characters
    let data = "Hello, 世界! Привет!".as_bytes();
    let encoded = encode_with(data, Alphabet::Standard, true);
    let decoded = decode_with(&encoded, Alphabet::Standard).unwrap();
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
        let encoded = encode_with(data, Alphabet::Standard, true);
        let decoded = decode_with(&encoded, Alphabet::Standard).unwrap();
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
    let encoded = encode_with(&data, Alphabet::Standard, true);
    let decoded = decode_with(&encoded, Alphabet::Standard).unwrap();
    assert_eq!(decoded, data);
}

#[test]
fn test_encode_null_and_control_chars() {
    // Test encoding data with null bytes and control characters
    let data = b"\x00\x01\x02\x1f\x7f\xff";
    let encoded = encode_with(data, Alphabet::Standard, true);
    let decoded = decode_with(&encoded, Alphabet::Standard).unwrap();
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
        let scalar_result = encode_with(&data, Alphabet::Standard, true);
        let avx2_result = encode_with_avx2(&data, Alphabet::Standard, true);
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
        let scalar_result = decode_with(encoded, Alphabet::Standard);
        let avx2_result = decode_with_avx2(encoded, Alphabet::Standard);
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
        let encoded = encode_with_avx2(&data, Alphabet::Standard, true);
        let decoded = decode_with_avx2(&encoded, Alphabet::Standard).unwrap();
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
fn test_large_data_roundtrip() {
    // Test with larger data sizes
    for size in [1000, 5000, 10000] {
        let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
        let encoded = encode_with(&data, Alphabet::Standard, true);
        let decoded = decode_with(&encoded, Alphabet::Standard).unwrap();
        assert_eq!(decoded, data, "Roundtrip failed for size {}", size);
    }
}

#[test]
fn test_avx2_large_data_roundtrip() {
    // Test AVX2 with larger data sizes
    for size in [1000, 5000, 10000] {
        let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
        let encoded = encode_with_avx2(&data, Alphabet::Standard, true);
        let decoded = decode_with_avx2(&encoded, Alphabet::Standard).unwrap();
        assert_eq!(decoded, data, "AVX2 roundtrip failed for size {}", size);
    }
}

#[test]
fn test_various_lengths() {
    // Test various input lengths to catch edge cases
    for len in 0..50 {
        let data: Vec<u8> = (0..len as u8).collect();
        let encoded = encode_with(&data, Alphabet::Standard, true);
        let decoded = decode_with(&encoded, Alphabet::Standard).unwrap();
        assert_eq!(decoded, data, "Failed for length {}", len);
    }
}

#[test]
fn test_avx2_various_lengths() {
    // Test AVX2 with various input lengths to catch edge cases
    for len in 0..100 {
        let data: Vec<u8> = (0..len).map(|i| i as u8).collect();
        let encoded = encode_with_avx2(&data, Alphabet::Standard, true);
        let decoded = decode_with_avx2(&encoded, Alphabet::Standard).unwrap();
        assert_eq!(decoded, data, "AVX2 failed for length {}", len);
    }
}

#[test]
fn test_url_safe_full() {
    // More comprehensive URL-safe alphabet tests
    let test_cases = [
        b"".to_vec(),
        b"Hello".to_vec(),
        (0..=255).collect::<Vec<u8>>(),
        (0..1000).map(|i| (i % 256) as u8).collect::<Vec<u8>>(),
    ];

    for data in test_cases {
        let encoded = encode_with(&data, Alphabet::Url, true);
        let decoded = decode_with(&encoded, Alphabet::Url).unwrap();
        assert_eq!(decoded, data);

        // Also test without padding
        let encoded_no_pad = encode_with(&data, Alphabet::Url, false);
        let decoded_no_pad = decode_with(&encoded_no_pad, Alphabet::Url).unwrap();
        assert_eq!(decoded_no_pad, data);
    }
}

// Conformance tests against external base64 crate
#[test]
fn test_conformance_with_external_crate_encode() {
    use base64_external::{engine::general_purpose::STANDARD, Engine};

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
        let our_result = encode_with(data, Alphabet::Standard, true);
        let external_result = STANDARD.encode(data);
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
    use base64_external::{engine::general_purpose::STANDARD, Engine};

    let test_cases = ["SGVsbG8=", "Zm9vYmFy", "SGVsbG8sIFdvcmxkIQ=="];

    for encoded in &test_cases {
        let our_result = decode_with(encoded, Alphabet::Standard).unwrap();
        let external_result = STANDARD.decode(encoded).unwrap();
        assert_eq!(
            our_result, external_result,
            "Decode mismatch for '{}'",
            encoded
        );
    }
}

#[test]
fn test_conformance_roundtrip_with_external_crate() {
    use base64_external::{engine::general_purpose::STANDARD, Engine};

    let test_cases = [
        b"Hello, World!".to_vec(),
        (0..=255).collect::<Vec<u8>>(),
        (0..1000).map(|i| (i % 256) as u8).collect::<Vec<u8>>(),
    ];

    for data in &test_cases {
        // Our encode -> external decode
        let our_encoded = encode_with(data, Alphabet::Standard, true);
        let external_decoded = STANDARD.decode(&our_encoded).unwrap();
        assert_eq!(
            data,
            &external_decoded,
            "Our encode -> external decode failed for len {}",
            data.len()
        );

        // External encode -> our decode
        let external_encoded = STANDARD.encode(data);
        let our_decoded = decode_with(&external_encoded, Alphabet::Standard).unwrap();
        assert_eq!(
            data,
            &our_decoded,
            "External encode -> our decode failed for len {}",
            data.len()
        );
    }
}
