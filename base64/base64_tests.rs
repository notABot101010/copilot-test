//! Tests for base64 encoding and decoding.

use super::*;
use rand::{Rng, SeedableRng};

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
    assert_eq!(
        encode_with(b"foobar", Alphabet::Standard, false),
        "Zm9vYmFy"
    );
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
    assert_eq!(
        decode_with("Zm9vYg==", Alphabet::Standard).unwrap(),
        b"foob"
    );
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
    assert_eq!(
        decode_with("Zm9vYmE", Alphabet::Standard).unwrap(),
        b"fooba"
    );
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

// Random vector tests
#[test]
fn test_random_roundtrip_various_sizes() {
    // Use a seeded RNG for reproducible tests
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);

    // Test various sizes including edge cases
    let sizes = [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 15, 16, 17, 23, 24, 25, 31, 32, 33, 47, 48, 49, 63, 64, 65, 100,
        127, 128, 255, 256, 500, 1000, 1024, 2048,
    ];

    for &size in &sizes {
        let data: Vec<u8> = (0..size).map(|_| rng.gen()).collect();

        // Test with padding
        let encoded = encode_with(&data, Alphabet::Standard, true);
        let decoded = decode_with(&encoded, Alphabet::Standard).unwrap();
        assert_eq!(
            decoded, data,
            "Roundtrip failed for random data of size {}",
            size
        );

        // Test without padding
        let encoded_no_pad = encode_with(&data, Alphabet::Standard, false);
        let decoded_no_pad = decode_with(&encoded_no_pad, Alphabet::Standard).unwrap();
        assert_eq!(
            decoded_no_pad, data,
            "Roundtrip without padding failed for random data of size {}",
            size
        );
    }
}

#[test]
fn test_random_roundtrip_url_alphabet() {
    let mut rng = rand::rngs::StdRng::seed_from_u64(123);

    for size in [0, 1, 10, 50, 100, 255, 1000] {
        let data: Vec<u8> = (0..size).map(|_| rng.gen()).collect();

        // Test with padding
        let encoded = encode_with(&data, Alphabet::Url, true);
        let decoded = decode_with(&encoded, Alphabet::Url).unwrap();
        assert_eq!(
            decoded, data,
            "URL alphabet roundtrip failed for random data of size {}",
            size
        );

        // Ensure no + or / characters in URL-safe encoding
        assert!(!encoded.contains('+') && !encoded.contains('/'));
    }
}

#[test]
fn test_random_avx2_roundtrip() {
    let mut rng = rand::rngs::StdRng::seed_from_u64(456);

    // Test sizes that exercise AVX2 code paths
    let sizes = [0, 1, 10, 27, 28, 29, 48, 50, 100, 256, 500, 1000, 2048];

    for &size in &sizes {
        let data: Vec<u8> = (0..size).map(|_| rng.gen()).collect();

        let encoded = encode_with_avx2(&data, Alphabet::Standard, true);
        let decoded = decode_with_avx2(&encoded, Alphabet::Standard).unwrap();
        assert_eq!(
            decoded, data,
            "AVX2 roundtrip failed for random data of size {}",
            size
        );
    }
}

#[test]
fn test_random_avx2_matches_scalar() {
    let mut rng = rand::rngs::StdRng::seed_from_u64(789);

    for size in [0, 1, 10, 28, 45, 50, 100, 256, 1000] {
        let data: Vec<u8> = (0..size).map(|_| rng.gen()).collect();

        // Test encoding
        let scalar_encoded = encode_with(&data, Alphabet::Standard, true);
        let avx2_encoded = encode_with_avx2(&data, Alphabet::Standard, true);
        assert_eq!(
            scalar_encoded, avx2_encoded,
            "AVX2 encode mismatch with scalar for random data of size {}",
            size
        );

        // Test decoding
        let encoded = encode_with(&data, Alphabet::Standard, true);
        let scalar_decoded = decode_with(&encoded, Alphabet::Standard).unwrap();
        let avx2_decoded = decode_with_avx2(&encoded, Alphabet::Standard).unwrap();
        assert_eq!(
            scalar_decoded, avx2_decoded,
            "AVX2 decode mismatch with scalar for random data of size {}",
            size
        );
    }
}

#[test]
fn test_random_conformance_with_external_crate() {
    use base64_external::{engine::general_purpose::STANDARD, Engine};

    let mut rng = rand::rngs::StdRng::seed_from_u64(999);

    for size in [0, 1, 5, 10, 50, 100, 255, 500, 1000] {
        let data: Vec<u8> = (0..size).map(|_| rng.gen()).collect();

        // Test encoding conformance
        let our_encoded = encode_with(&data, Alphabet::Standard, true);
        let external_encoded = STANDARD.encode(&data);
        assert_eq!(
            our_encoded, external_encoded,
            "Encoding mismatch with external crate for random data of size {}",
            size
        );

        // Test decoding conformance
        let our_decoded = decode_with(&our_encoded, Alphabet::Standard).unwrap();
        let external_decoded = STANDARD.decode(&external_encoded).unwrap();
        assert_eq!(
            our_decoded, external_decoded,
            "Decoding mismatch with external crate for random data of size {}",
            size
        );

        // Verify roundtrip
        assert_eq!(
            our_decoded, data,
            "Roundtrip failed for random data of size {}",
            size
        );
    }
}

#[test]
fn test_random_all_byte_values() {
    let mut rng = rand::rngs::StdRng::seed_from_u64(111);

    // Generate random data ensuring all byte values 0-255 are represented
    for _ in 0..10 {
        let mut data = Vec::new();

        // Add all byte values at least once
        for byte_val in 0..=255u8 {
            data.push(byte_val);
        }

        // Add random bytes
        for _ in 0..500 {
            data.push(rng.gen());
        }

        // Shuffle the data
        use rand::seq::SliceRandom;
        data.shuffle(&mut rng);

        // Test roundtrip
        let encoded = encode_with(&data, Alphabet::Standard, true);
        let decoded = decode_with(&encoded, Alphabet::Standard).unwrap();
        assert_eq!(decoded, data, "Failed for data with all byte values");

        // Test AVX2
        let avx2_encoded = encode_with_avx2(&data, Alphabet::Standard, true);
        let avx2_decoded = decode_with_avx2(&avx2_encoded, Alphabet::Standard).unwrap();
        assert_eq!(
            avx2_decoded, data,
            "AVX2 failed for data with all byte values"
        );
    }
}

#[test]
fn test_random_edge_case_lengths() {
    let mut rng = rand::rngs::StdRng::seed_from_u64(222);

    // Test lengths around multiples of 3 (base64 chunk size)
    for base in [3, 6, 9, 12, 24, 48, 96] {
        for offset in [-1, 0, 1] {
            let size = (base + offset).max(0) as usize;
            let data: Vec<u8> = (0..size).map(|_| rng.gen()).collect();

            let encoded = encode_with(&data, Alphabet::Standard, true);
            let decoded = decode_with(&encoded, Alphabet::Standard).unwrap();
            assert_eq!(
                decoded, data,
                "Failed for random data of size {} (base {} + offset {})",
                size, base, offset
            );
        }
    }
}

#[test]
fn test_random_with_custom_alphabet() {
    let mut rng = rand::rngs::StdRng::seed_from_u64(333);

    // Create a custom alphabet
    let custom_alphabet: [u8; 64] = [
        b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'A', b'B', b'C', b'D', b'E',
        b'F', b'G', b'H', b'I', b'J', b'K', b'L', b'M', b'N', b'O', b'P', b'Q', b'R', b'S', b'T',
        b'U', b'V', b'W', b'X', b'Y', b'Z', b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h', b'i',
        b'j', b'k', b'l', b'm', b'n', b'o', b'p', b'q', b'r', b's', b't', b'u', b'v', b'w', b'x',
        b'y', b'z', b'!', b'@',
    ];

    for size in [0, 1, 10, 50, 100, 255] {
        let data: Vec<u8> = (0..size).map(|_| rng.gen()).collect();

        let encoded = encode_with(&data, Alphabet::Custom(&custom_alphabet), true);
        let decoded = decode_with(&encoded, Alphabet::Custom(&custom_alphabet)).unwrap();
        assert_eq!(
            decoded, data,
            "Custom alphabet roundtrip failed for random data of size {}",
            size
        );
    }
}

#[test]
fn test_random_no_padding_various_remainders() {
    let mut rng = rand::rngs::StdRng::seed_from_u64(444);

    // Test sizes with different remainders when divided by 3
    // remainder 0: 3, 6, 9, 12...
    // remainder 1: 1, 4, 7, 10...
    // remainder 2: 2, 5, 8, 11...

    for remainder in 0..3 {
        for multiple in 0..20 {
            let size = multiple * 3 + remainder;
            let data: Vec<u8> = (0..size).map(|_| rng.gen()).collect();

            let encoded = encode_with(&data, Alphabet::Standard, false);
            let decoded = decode_with(&encoded, Alphabet::Standard).unwrap();
            assert_eq!(
                decoded, data,
                "No padding roundtrip failed for random data of size {} (remainder {})",
                size, remainder
            );
        }
    }
}

#[test]
fn test_random_stress_test() {
    let mut rng = rand::rngs::StdRng::seed_from_u64(555);

    // Perform many iterations with random sizes
    for _ in 0..100 {
        let size: usize = rng.gen_range(0..500);
        let data: Vec<u8> = (0..size).map(|_| rng.gen()).collect();

        // Test with standard alphabet
        let encoded = encode_with(&data, Alphabet::Standard, true);
        let decoded = decode_with(&encoded, Alphabet::Standard).unwrap();
        assert_eq!(decoded, data);

        // Test with URL alphabet
        let encoded_url = encode_with(&data, Alphabet::Url, true);
        let decoded_url = decode_with(&encoded_url, Alphabet::Url).unwrap();
        assert_eq!(decoded_url, data);

        // Test without padding
        let encoded_no_pad = encode_with(&data, Alphabet::Standard, false);
        let decoded_no_pad = decode_with(&encoded_no_pad, Alphabet::Standard).unwrap();
        assert_eq!(decoded_no_pad, data);

        // Test AVX2
        let avx2_encoded = encode_with_avx2(&data, Alphabet::Standard, true);
        let avx2_decoded = decode_with_avx2(&avx2_encoded, Alphabet::Standard).unwrap();
        assert_eq!(avx2_decoded, data);
    }
}
