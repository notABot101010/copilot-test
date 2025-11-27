#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Test encode roundtrip
    let encoded = hex::encode(data, hex::ALPHABET_LOWER);
    let decoded = hex::decode(&encoded, hex::ALPHABET_LOWER);
    assert_eq!(data, &decoded[..], "Roundtrip failed");

    // Test with upper alphabet
    let encoded_upper = hex::encode(data, hex::ALPHABET_UPPER);
    let decoded_upper = hex::decode(&encoded_upper, hex::ALPHABET_UPPER);
    assert_eq!(data, &decoded_upper[..], "Upper roundtrip failed");

    // Test AVX2 variants match scalar
    let avx2_encoded = hex::encode_avx2(data, hex::ALPHABET_LOWER);
    assert_eq!(encoded, avx2_encoded, "AVX2 encode mismatch");

    let avx2_decoded = hex::decode_avx2(&encoded, hex::ALPHABET_LOWER);
    assert_eq!(decoded, avx2_decoded, "AVX2 decode mismatch");

    // Conformance with external crate
    let external_encoded = hex_external::encode(data);
    assert_eq!(
        encoded, external_encoded,
        "External crate encode mismatch"
    );
});
