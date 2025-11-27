#![no_main]

use libfuzzer_sys::fuzz_target;
use base64_external::{engine::general_purpose::STANDARD, Engine};

fuzz_target!(|data: &[u8]| {
    // Test encode roundtrip
    let encoded = base64::encode_with(data, base64::ALPHABET_STANDARD, true);
    if let Ok(decoded) = base64::decode_with(&encoded, base64::ALPHABET_STANDARD) {
        assert_eq!(data, &decoded[..], "Roundtrip failed");
    }

    // Test without padding
    let encoded_no_pad = base64::encode_with(data, base64::ALPHABET_STANDARD, false);
    if let Ok(decoded) = base64::decode_with(&encoded_no_pad, base64::ALPHABET_STANDARD) {
        assert_eq!(data, &decoded[..], "Roundtrip without padding failed");
    }

    // Test AVX2 variants match scalar
    let avx2_encoded = base64::encode_with_avx2(data, base64::ALPHABET_STANDARD, true);
    assert_eq!(encoded, avx2_encoded, "AVX2 encode mismatch");

    if let Ok(avx2_decoded) = base64::decode_with_avx2(&encoded, base64::ALPHABET_STANDARD) {
        if let Ok(scalar_decoded) = base64::decode_with(&encoded, base64::ALPHABET_STANDARD) {
            assert_eq!(scalar_decoded, avx2_decoded, "AVX2 decode mismatch");
        }
    }

    // Conformance with external crate
    let external_encoded = STANDARD.encode(data);
    assert_eq!(
        encoded, external_encoded,
        "External crate encode mismatch"
    );
});
