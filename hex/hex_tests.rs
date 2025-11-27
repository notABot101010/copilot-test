//! Tests for hex encoding and decoding.

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
    assert_eq!(encode(b"", Alphabet::Lower), "");
    assert_eq!(encode(b"", Alphabet::Upper), "");
}

#[test]
fn test_encode_lower() {
    assert_eq!(encode(b"Hello", Alphabet::Lower), "48656c6c6f");
    assert_eq!(encode(b"\x00\xff", Alphabet::Lower), "00ff");
    assert_eq!(encode(b"abc", Alphabet::Lower), "616263");
}

#[test]
fn test_encode_upper() {
    assert_eq!(encode(b"Hello", Alphabet::Upper), "48656C6C6F");
    assert_eq!(encode(b"\x00\xff", Alphabet::Upper), "00FF");
    assert_eq!(encode(b"abc", Alphabet::Upper), "616263");
}

#[test]
fn test_encode_into() {
    let data = b"Hello";
    let mut output = [0u8; 10];
    encode_into(&mut output, data, Alphabet::Lower).unwrap();
    assert_eq!(&output, b"48656c6c6f");
}

#[test]
fn test_encode_into_buffer_too_small() {
    let data = b"Hello";
    let mut output = [0u8; 5];
    let result = encode_into(&mut output, data, Alphabet::Lower);
    assert!(matches!(result, Err(Error::OutputBufferTooSmall)));
}

#[test]
fn test_decode_empty() {
    assert_eq!(decode("", Alphabet::Lower), Vec::<u8>::new());
}

#[test]
fn test_decode_lower() {
    assert_eq!(decode("48656c6c6f", Alphabet::Lower), b"Hello");
    assert_eq!(decode("00ff", Alphabet::Lower), b"\x00\xff");
    assert_eq!(decode("616263", Alphabet::Lower), b"abc");
}

#[test]
fn test_decode_upper() {
    assert_eq!(decode("48656C6C6F", Alphabet::Upper), b"Hello");
    assert_eq!(decode("00FF", Alphabet::Upper), b"\x00\xff");
}

#[test]
fn test_decode_mixed_case() {
    // Should accept both cases
    assert_eq!(decode("48656C6c6F", Alphabet::Lower), b"Hello");
    assert_eq!(
        decode("aAbBcCdDeEfF", Alphabet::Lower),
        b"\xaa\xbb\xcc\xdd\xee\xff"
    );
}

#[test]
fn test_decode_invalid_character() {
    let result = decode_checked("ghij", Alphabet::Lower);
    assert!(matches!(result, Err(Error::InvalidCharacter('g'))));
}

#[test]
fn test_decode_invalid_length() {
    let result = decode_checked("abc", Alphabet::Lower);
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
        let encoded = encode(&data, Alphabet::Lower);
        let decoded = decode(&encoded, Alphabet::Lower);
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
        let encoded = encode(&data, Alphabet::Upper);
        let decoded = decode(&encoded, Alphabet::Upper);
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
    assert_eq!(format!("{}", Error::InvalidUtf8), "invalid UTF-8 in input");
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
        let scalar_result = encode(&data, Alphabet::Lower);
        let avx2_result = encode_avx2(&data, Alphabet::Lower);
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
    let test_cases = [
        "",
        "61",
        "48656c6c6f",
        "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f", // 32 bytes
        "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f", // 64 bytes
    ];

    for encoded in test_cases {
        let scalar_result = decode_checked(encoded, Alphabet::Lower);
        let avx2_result = decode_avx2_checked(encoded, Alphabet::Lower);
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
        let encoded = encode_avx2(&data, Alphabet::Lower);
        let decoded = decode_avx2(&encoded, Alphabet::Lower);
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
fn test_encode_binary_with_high_bytes() {
    // Test binary data with bytes > 127 (non-ASCII range)
    let data: Vec<u8> = (128..=255).collect();
    let encoded = encode(&data, Alphabet::Lower);
    let decoded = decode(&encoded, Alphabet::Lower);
    assert_eq!(decoded, data);
}

#[test]
fn test_encode_null_and_control_chars() {
    // Test encoding data with null bytes and control characters
    let data = b"\x00\x01\x02\x1f\x7f\xff";
    let encoded = encode(data, Alphabet::Lower);
    let decoded = decode(&encoded, Alphabet::Lower);
    assert_eq!(decoded, data);
}

#[test]
fn test_large_data_roundtrip() {
    // Test with larger data sizes
    for size in [1000, 5000, 10000] {
        let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
        let encoded = encode(&data, Alphabet::Lower);
        let decoded = decode(&encoded, Alphabet::Lower);
        assert_eq!(decoded, data, "Roundtrip failed for size {}", size);
    }
}

#[test]
fn test_avx2_large_data_roundtrip() {
    // Test AVX2 with larger data sizes
    for size in [1000, 5000, 10000] {
        let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
        let encoded = encode_avx2(&data, Alphabet::Lower);
        let decoded = decode_avx2(&encoded, Alphabet::Lower);
        assert_eq!(decoded, data, "AVX2 roundtrip failed for size {}", size);
    }
}

#[test]
fn test_all_byte_values() {
    // Test all 256 byte values
    let data: Vec<u8> = (0..=255).collect();

    // Test with lower alphabet
    let encoded = encode(&data, Alphabet::Lower);
    let decoded = decode(&encoded, Alphabet::Lower);
    assert_eq!(decoded, data);

    // Test with upper alphabet
    let encoded = encode(&data, Alphabet::Upper);
    let decoded = decode(&encoded, Alphabet::Upper);
    assert_eq!(decoded, data);
}

#[test]
fn test_various_lengths() {
    // Test various input lengths to catch edge cases
    for len in 0..50 {
        let data: Vec<u8> = (0..len as u8).collect();
        let encoded = encode(&data, Alphabet::Lower);
        let decoded = decode(&encoded, Alphabet::Lower);
        assert_eq!(decoded, data, "Failed for length {}", len);
    }
}

#[test]
fn test_avx2_various_lengths() {
    // Test AVX2 with various input lengths to catch edge cases
    for len in 0..100 {
        let data: Vec<u8> = (0..len).map(|i| i as u8).collect();
        let encoded = encode_avx2(&data, Alphabet::Lower);
        let decoded = decode_avx2(&encoded, Alphabet::Lower);
        assert_eq!(decoded, data, "AVX2 failed for length {}", len);
    }
}

#[test]
fn test_upper_alphabet_full() {
    // More comprehensive upper alphabet tests
    let test_cases = [
        b"".to_vec(),
        b"Hello".to_vec(),
        (0..=255).collect::<Vec<u8>>(),
        (0..1000).map(|i| (i % 256) as u8).collect::<Vec<u8>>(),
    ];

    for data in test_cases {
        let encoded = encode(&data, Alphabet::Upper);
        let decoded = decode(&encoded, Alphabet::Upper);
        assert_eq!(decoded, data);
    }
}

// Conformance tests against external hex crate
#[test]
fn test_conformance_with_external_crate_encode() {
    let test_cases = [
        b"".to_vec(),
        b"f".to_vec(),
        b"fo".to_vec(),
        b"foo".to_vec(),
        b"Hello, World!".to_vec(),
        (0..=255).collect::<Vec<u8>>(),
        (0..1000).map(|i| (i % 256) as u8).collect::<Vec<u8>>(),
    ];

    for data in &test_cases {
        let our_result = encode(data, Alphabet::Lower);
        let external_result = hex_external::encode(data);
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
    let test_cases = ["48656c6c6f", "666f6f626172", "48656c6c6f2c20576f726c6421"];

    for encoded in &test_cases {
        let our_result = decode(encoded, Alphabet::Lower);
        let external_result = hex_external::decode(encoded).unwrap();
        assert_eq!(
            our_result, external_result,
            "Decode mismatch for '{}'",
            encoded
        );
    }
}

#[test]
fn test_conformance_roundtrip_with_external_crate() {
    let test_cases = [
        b"Hello, World!".to_vec(),
        (0..=255).collect::<Vec<u8>>(),
        (0..1000).map(|i| (i % 256) as u8).collect::<Vec<u8>>(),
    ];

    for data in &test_cases {
        // Our encode -> external decode
        let our_encoded = encode(data, Alphabet::Lower);
        let external_decoded = hex_external::decode(&our_encoded).unwrap();
        assert_eq!(
            data,
            &external_decoded,
            "Our encode -> external decode failed for len {}",
            data.len()
        );

        // External encode -> our decode
        let external_encoded = hex_external::encode(data);
        let our_decoded = decode(&external_encoded, Alphabet::Lower);
        assert_eq!(
            data,
            &our_decoded,
            "External encode -> our decode failed for len {}",
            data.len()
        );
    }
}
