//! Tests for base32 encoding and decoding.

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
fn test_decoded_len() {
    assert_eq!(decoded_len(0), 0);
    assert_eq!(decoded_len(8), 5);
    assert_eq!(decoded_len(16), 10);
    assert_eq!(decoded_len(2), 1);
    assert_eq!(decoded_len(4), 2);
    assert_eq!(decoded_len(5), 3);
    assert_eq!(decoded_len(7), 4);
}

#[test]
fn test_encode_empty() {
    assert_eq!(encode(b"", ALPHABET_STANDARD, true), "");
    assert_eq!(encode(b"", ALPHABET_STANDARD, false), "");
}

#[test]
fn test_encode_with_padding() {
    // RFC 4648 test vectors
    assert_eq!(encode(b"f", ALPHABET_STANDARD, true), "MY======");
    assert_eq!(encode(b"fo", ALPHABET_STANDARD, true), "MZXQ====");
    assert_eq!(encode(b"foo", ALPHABET_STANDARD, true), "MZXW6===");
    assert_eq!(encode(b"foob", ALPHABET_STANDARD, true), "MZXW6YQ=");
    assert_eq!(encode(b"fooba", ALPHABET_STANDARD, true), "MZXW6YTB");
    assert_eq!(
        encode(b"foobar", ALPHABET_STANDARD, true),
        "MZXW6YTBOI======"
    );
}

#[test]
fn test_encode_without_padding() {
    assert_eq!(encode(b"f", ALPHABET_STANDARD, false), "MY");
    assert_eq!(encode(b"fo", ALPHABET_STANDARD, false), "MZXQ");
    assert_eq!(encode(b"foo", ALPHABET_STANDARD, false), "MZXW6");
    assert_eq!(encode(b"foob", ALPHABET_STANDARD, false), "MZXW6YQ");
    assert_eq!(encode(b"fooba", ALPHABET_STANDARD, false), "MZXW6YTB");
    assert_eq!(encode(b"foobar", ALPHABET_STANDARD, false), "MZXW6YTBOI");
}

#[test]
fn test_encode_hello() {
    assert_eq!(encode(b"Hello", ALPHABET_STANDARD, true), "JBSWY3DP");
    assert_eq!(
        encode(b"Hello, World!", ALPHABET_STANDARD, true),
        "JBSWY3DPFQQFO33SNRSCC==="
    );
}

#[test]
fn test_encode_into() {
    let data = b"Hello";
    let mut output = [0u8; 8];
    encode_into(&mut output, data, ALPHABET_STANDARD, true).unwrap();
    assert_eq!(&output, b"JBSWY3DP");
}

#[test]
fn test_encode_into_buffer_too_small() {
    let data = b"Hello";
    let mut output = [0u8; 4];
    let result = encode_into(&mut output, data, ALPHABET_STANDARD, true);
    assert!(matches!(result, Err(Error::OutputBufferTooSmall)));
}

#[test]
fn test_decode_empty() {
    assert_eq!(decode("", ALPHABET_STANDARD).unwrap(), Vec::<u8>::new());
}

#[test]
fn test_decode_with_padding() {
    assert_eq!(decode("MY======", ALPHABET_STANDARD).unwrap(), b"f");
    assert_eq!(decode("MZXQ====", ALPHABET_STANDARD).unwrap(), b"fo");
    assert_eq!(decode("MZXW6===", ALPHABET_STANDARD).unwrap(), b"foo");
    assert_eq!(decode("MZXW6YQ=", ALPHABET_STANDARD).unwrap(), b"foob");
    assert_eq!(decode("MZXW6YTB", ALPHABET_STANDARD).unwrap(), b"fooba");
    assert_eq!(
        decode("MZXW6YTBOI======", ALPHABET_STANDARD).unwrap(),
        b"foobar"
    );
}

#[test]
fn test_decode_without_padding() {
    assert_eq!(decode("MY", ALPHABET_STANDARD).unwrap(), b"f");
    assert_eq!(decode("MZXQ", ALPHABET_STANDARD).unwrap(), b"fo");
    assert_eq!(decode("MZXW6", ALPHABET_STANDARD).unwrap(), b"foo");
    assert_eq!(decode("MZXW6YQ", ALPHABET_STANDARD).unwrap(), b"foob");
    assert_eq!(decode("MZXW6YTB", ALPHABET_STANDARD).unwrap(), b"fooba");
    assert_eq!(decode("MZXW6YTBOI", ALPHABET_STANDARD).unwrap(), b"foobar");
}

#[test]
fn test_decode_hello() {
    assert_eq!(decode("JBSWY3DP", ALPHABET_STANDARD).unwrap(), b"Hello");
    assert_eq!(
        decode("JBSWY3DPFQQFO33SNRSCC===", ALPHABET_STANDARD).unwrap(),
        b"Hello, World!"
    );
}

#[test]
fn test_decode_lowercase() {
    // Should accept lowercase
    assert_eq!(decode("jbswy3dp", ALPHABET_STANDARD).unwrap(), b"Hello");
    assert_eq!(decode("mzxw6ytboi", ALPHABET_STANDARD).unwrap(), b"foobar");
}

#[test]
fn test_decode_mixed_case() {
    assert_eq!(decode("JbSwY3dP", ALPHABET_STANDARD).unwrap(), b"Hello");
}

#[test]
fn test_decode_invalid_character() {
    let result = decode("!INVALID", ALPHABET_STANDARD);
    assert!(matches!(result, Err(Error::InvalidCharacter('!'))));

    // 0, 1, 8, 9 are not valid in standard base32
    let result = decode("01ABCDEF", ALPHABET_STANDARD);
    assert!(matches!(result, Err(Error::InvalidCharacter('0'))));
}

#[test]
fn test_decode_invalid_length() {
    // Length 1, 3, 6 are invalid
    let result = decode("A", ALPHABET_STANDARD);
    assert!(matches!(result, Err(Error::InvalidLength)));

    let result = decode("ABC", ALPHABET_STANDARD);
    assert!(matches!(result, Err(Error::InvalidLength)));

    let result = decode("ABCDEF", ALPHABET_STANDARD);
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
        b"Hello, World!".to_vec(),
        (0..=255).collect::<Vec<u8>>(),
    ];

    for data in test_cases {
        let encoded = encode(&data, ALPHABET_STANDARD, true);
        let decoded = decode(&encoded, ALPHABET_STANDARD).unwrap();
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
        let encoded = encode(&data, ALPHABET_STANDARD, false);
        let decoded = decode(&encoded, ALPHABET_STANDARD).unwrap();
        assert_eq!(
            decoded, data,
            "Roundtrip without padding failed for {:?}",
            data
        );
    }
}

#[test]
fn test_hex_alphabet() {
    // Test with hex alphabet
    let data = b"test";
    let encoded = encode(data, ALPHABET_HEX, false);
    let decoded = decode(&encoded, ALPHABET_HEX).unwrap();
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
    assert_eq!(
        format!("{}", Error::OutputBufferTooSmall),
        "output buffer too small"
    );
}

// AVX2 tests
#[test]
fn test_encode_avx2_matches_scalar() {
    let test_cases = [
        b"".to_vec(),
        b"a".to_vec(),
        b"Hello, World!".to_vec(),
        (0..20).collect::<Vec<u8>>(), // Exactly 20 bytes (AVX2 block size)
        (0..40).collect::<Vec<u8>>(), // Two AVX2 blocks
        (0..100).collect::<Vec<u8>>(), // Multiple blocks + remainder
        (0..=255).collect::<Vec<u8>>(), // All byte values
    ];

    for data in test_cases {
        let scalar_result = encode(&data, ALPHABET_STANDARD, true);
        let avx2_result = encode_avx2(&data, ALPHABET_STANDARD, true);
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
        "MY======",
        "JBSWY3DP",
        "MZXW6YTBOI======",
        "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ", // 32 chars
        "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ", // 64 chars
    ];

    for encoded in test_cases {
        let scalar_result = decode(encoded, ALPHABET_STANDARD);
        let avx2_result = decode_avx2(encoded, ALPHABET_STANDARD);
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
        (0..20).collect::<Vec<u8>>(),
        (0..40).collect::<Vec<u8>>(),
        (0..100).collect::<Vec<u8>>(),
        (0..=255).collect::<Vec<u8>>(),
    ];

    for data in test_cases {
        let encoded = encode_avx2(&data, ALPHABET_STANDARD, true);
        let decoded = decode_avx2(&encoded, ALPHABET_STANDARD).unwrap();
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
    let encoded = encode(&data, ALPHABET_STANDARD, true);
    let decoded = decode(&encoded, ALPHABET_STANDARD).unwrap();
    assert_eq!(decoded, data);
}

#[test]
fn test_encode_null_and_control_chars() {
    // Test encoding data with null bytes and control characters
    let data = b"\x00\x01\x02\x1f\x7f\xff";
    let encoded = encode(data, ALPHABET_STANDARD, true);
    let decoded = decode(&encoded, ALPHABET_STANDARD).unwrap();
    assert_eq!(decoded, data);
}

#[test]
fn test_large_data_roundtrip() {
    // Test with larger data sizes
    for size in [1000, 5000, 10000] {
        let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
        let encoded = encode(&data, ALPHABET_STANDARD, true);
        let decoded = decode(&encoded, ALPHABET_STANDARD).unwrap();
        assert_eq!(decoded, data, "Roundtrip failed for size {}", size);
    }
}

#[test]
fn test_avx2_large_data_roundtrip() {
    // Test AVX2 with larger data sizes
    for size in [1000, 5000, 10000] {
        let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
        let encoded = encode_avx2(&data, ALPHABET_STANDARD, true);
        let decoded = decode_avx2(&encoded, ALPHABET_STANDARD).unwrap();
        assert_eq!(decoded, data, "AVX2 roundtrip failed for size {}", size);
    }
}

#[test]
fn test_all_byte_values() {
    // Test all 256 byte values
    let data: Vec<u8> = (0..=255).collect();

    // Test with standard alphabet
    let encoded = encode(&data, ALPHABET_STANDARD, true);
    let decoded = decode(&encoded, ALPHABET_STANDARD).unwrap();
    assert_eq!(decoded, data);

    // Test with hex alphabet
    let encoded = encode(&data, ALPHABET_HEX, true);
    let decoded = decode(&encoded, ALPHABET_HEX).unwrap();
    assert_eq!(decoded, data);
}

#[test]
fn test_various_lengths() {
    // Test various input lengths to catch edge cases
    for len in 0..50 {
        let data: Vec<u8> = (0..len as u8).collect();
        let encoded = encode(&data, ALPHABET_STANDARD, true);
        let decoded = decode(&encoded, ALPHABET_STANDARD).unwrap();
        assert_eq!(decoded, data, "Failed for length {}", len);
    }
}

#[test]
fn test_avx2_various_lengths() {
    // Test AVX2 with various input lengths to catch edge cases
    for len in 0..100 {
        let data: Vec<u8> = (0..len).map(|i| i as u8).collect();
        let encoded = encode_avx2(&data, ALPHABET_STANDARD, true);
        let decoded = decode_avx2(&encoded, ALPHABET_STANDARD).unwrap();
        assert_eq!(decoded, data, "AVX2 failed for length {}", len);
    }
}

#[test]
fn test_hex_alphabet_full() {
    // More comprehensive hex alphabet tests
    let test_cases = [
        b"".to_vec(),
        b"Hello".to_vec(),
        (0..=255).collect::<Vec<u8>>(),
        (0..1000).map(|i| (i % 256) as u8).collect::<Vec<u8>>(),
    ];

    for data in test_cases {
        let encoded = encode(&data, ALPHABET_HEX, true);
        let decoded = decode(&encoded, ALPHABET_HEX).unwrap();
        assert_eq!(decoded, data);

        // Also test without padding
        let encoded_no_pad = encode(&data, ALPHABET_HEX, false);
        let decoded_no_pad = decode(&encoded_no_pad, ALPHABET_HEX).unwrap();
        assert_eq!(decoded_no_pad, data);
    }
}

// Conformance tests against external base32 crate
#[test]
fn test_conformance_with_external_crate_encode() {
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
        let our_result = encode(data, ALPHABET_STANDARD, true);
        let external_result =
            base32_external::encode(base32_external::Alphabet::Rfc4648 { padding: true }, data);
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
    let test_cases = [
        "JBSWY3DP",         // "Hello"
        "MZXW6YTBOI======", // "foobar"
        "GEZDGNBVGY3TQOJQ", // "12345678"
    ];

    for encoded in &test_cases {
        let our_result = decode(encoded, ALPHABET_STANDARD).unwrap();
        let external_result = base32_external::decode(
            base32_external::Alphabet::Rfc4648 { padding: true },
            encoded,
        )
        .unwrap();
        assert_eq!(
            our_result, external_result,
            "Decode mismatch for '{}'",
            encoded
        );
    }
}

#[test]
fn test_conformance_roundtrip_with_external_crate() {
    // Encode with ours, decode with external
    let test_cases = [
        b"Hello, World!".to_vec(),
        (0..=255).collect::<Vec<u8>>(),
        (0..1000).map(|i| (i % 256) as u8).collect::<Vec<u8>>(),
    ];

    for data in &test_cases {
        // Our encode -> external decode
        let our_encoded = encode(data, ALPHABET_STANDARD, true);
        let external_decoded = base32_external::decode(
            base32_external::Alphabet::Rfc4648 { padding: true },
            &our_encoded,
        )
        .unwrap();
        assert_eq!(
            data,
            &external_decoded,
            "Our encode -> external decode failed for len {}",
            data.len()
        );

        // External encode -> our decode
        let external_encoded =
            base32_external::encode(base32_external::Alphabet::Rfc4648 { padding: true }, data);
        let our_decoded = decode(&external_encoded, ALPHABET_STANDARD).unwrap();
        assert_eq!(
            data,
            &our_decoded,
            "External encode -> our decode failed for len {}",
            data.len()
        );
    }
}
