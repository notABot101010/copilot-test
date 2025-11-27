#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Test encode roundtrip
    let encoded = base32::encode(data, base32::ALPHABET_STANDARD, true);
    if let Ok(decoded) = base32::decode(&encoded, base32::ALPHABET_STANDARD) {
        assert_eq!(data, &decoded[..], "Roundtrip failed");
    }

    // Test without padding
    let encoded_no_pad = base32::encode(data, base32::ALPHABET_STANDARD, false);
    if let Ok(decoded) = base32::decode(&encoded_no_pad, base32::ALPHABET_STANDARD) {
        assert_eq!(data, &decoded[..], "Roundtrip without padding failed");
    }

    // Test AVX2 variants match scalar
    let avx2_encoded = base32::encode_avx2(data, base32::ALPHABET_STANDARD, true);
    assert_eq!(encoded, avx2_encoded, "AVX2 encode mismatch");

    if let Ok(avx2_decoded) = base32::decode_avx2(&encoded, base32::ALPHABET_STANDARD) {
        if let Ok(scalar_decoded) = base32::decode(&encoded, base32::ALPHABET_STANDARD) {
            assert_eq!(scalar_decoded, avx2_decoded, "AVX2 decode mismatch");
        }
    }

    // Conformance with external crate
    let external_encoded =
        base32_external::encode(base32_external::Alphabet::Rfc4648 { padding: true }, data);
    assert_eq!(
        encoded, external_encoded,
        "External crate encode mismatch"
    );
});
