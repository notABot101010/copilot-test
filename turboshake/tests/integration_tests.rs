//! Integration tests for TurboSHAKE, KangarooTwelve, and AEAD

use turboshake::{AeadError, TurboShake128, TurboShake256, TurboShakeAead, KT128, KT256};

// ===== TurboSHAKE integration tests =====

#[test]
fn test_turboshake128_vs_256_different_output() {
    let data = b"same input data";
    let mut out128 = [0u8; 32];
    let mut out256 = [0u8; 32];

    TurboShake128::hash(data, &mut out128);
    TurboShake256::hash(data, &mut out256);

    assert_ne!(
        out128, out256,
        "TurboSHAKE128 and 256 should produce different outputs"
    );
}

#[test]
fn test_turboshake_large_output() {
    let data = b"test";
    let mut output = vec![0u8; 10000];

    TurboShake256::hash(data, &mut output);

    // Verify output is non-zero and varies
    assert!(output.iter().any(|&x| x != 0));
    assert!(output[0..100] != output[100..200]);
}

#[test]
fn test_turboshake_incremental_boundary() {
    // Test incremental hashing at various boundaries
    let data = vec![0xABu8; 500];

    let mut output1 = [0u8; 64];
    TurboShake256::hash(&data, &mut output1);

    // Split at rate boundary (136 bytes)
    let mut hasher = TurboShake256::new();
    hasher.update(&data[..136]);
    hasher.update(&data[136..]);
    let mut output2 = [0u8; 64];
    hasher.finalize(&mut output2);

    assert_eq!(output1, output2);
}

// ===== KangarooTwelve integration tests =====

#[test]
fn test_kt128_vs_kt256_different_output() {
    let data = b"same input data";
    let mut out128 = [0u8; 32];
    let mut out256 = [0u8; 32];

    KT128::hash(data, &[], &mut out128);
    KT256::hash(data, &[], &mut out256);

    assert_ne!(
        out128, out256,
        "KT128 and KT256 should produce different outputs"
    );
}

#[test]
fn test_kt128_with_various_custom_strings() {
    let data = b"message";
    let mut out1 = [0u8; 32];
    let mut out2 = [0u8; 32];
    let mut out3 = [0u8; 32];

    KT128::hash(data, &[], &mut out1);
    KT128::hash(data, b"custom1", &mut out2);
    KT128::hash(data, b"custom2", &mut out3);

    assert_ne!(out1, out2);
    assert_ne!(out2, out3);
    assert_ne!(out1, out3);
}

#[test]
fn test_kt128_tree_hashing_boundary() {
    // Test at the boundary of tree hashing (8192 bytes)
    let data_small = vec![0xAAu8; 8191];
    let data_large = vec![0xAAu8; 8193];

    let mut out_small = [0u8; 32];
    let mut out_large = [0u8; 32];

    KT128::hash(&data_small, &[], &mut out_small);
    KT128::hash(&data_large, &[], &mut out_large);

    // Different sizes should produce different outputs
    assert_ne!(out_small, out_large);

    // Both should produce non-zero output
    assert!(out_small.iter().any(|&x| x != 0));
    assert!(out_large.iter().any(|&x| x != 0));
}

#[test]
fn test_kt128_multiple_chunks() {
    // Test with message spanning multiple chunks
    let data = vec![0x42u8; 25000]; // About 3 chunks
    let mut output = [0u8; 32];

    KT128::hash(&data, &[], &mut output);

    // Verify non-zero and deterministic
    assert!(output.iter().any(|&x| x != 0));

    let mut output2 = [0u8; 32];
    KT128::hash(&data, &[], &mut output2);
    assert_eq!(output, output2);
}

// ===== AEAD integration tests =====

#[test]
fn test_aead_basic_roundtrip() {
    let key = [42u8; 32];
    let nonce = [17u8; 16];
    let plaintext = b"This is a test message for integration testing";
    let ad = b"";

    let mut cipher_enc = TurboShakeAead::new(&key, &nonce).unwrap();
    let ciphertext = cipher_enc.encrypt(plaintext, ad);

    let mut cipher_dec = TurboShakeAead::new(&key, &nonce).unwrap();
    let decrypted = cipher_dec.decrypt(&ciphertext, ad).unwrap();

    assert_eq!(decrypted, plaintext);
    assert_ne!(&ciphertext[..plaintext.len()], plaintext);
}

#[test]
fn test_aead_with_associated_data() {
    let key = [1u8; 32];
    let nonce = [2u8; 16];
    let plaintext = b"Secret data";
    let ad = b"This is public metadata that should be authenticated";

    let mut cipher_enc = TurboShakeAead::new(&key, &nonce).unwrap();
    let ciphertext = cipher_enc.encrypt(plaintext, ad);

    let mut cipher_dec = TurboShakeAead::new(&key, &nonce).unwrap();
    let decrypted = cipher_dec.decrypt(&ciphertext, ad).unwrap();

    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_aead_multiple_messages_same_key_different_nonces() {
    let key = [99u8; 32];
    let nonce1 = [1u8; 16];
    let nonce2 = [2u8; 16];
    let nonce3 = [3u8; 16];

    let messages = [
        b"First message".as_slice(),
        b"Second message with more content".as_slice(),
        b"Third message is even longer than the previous ones".as_slice(),
    ];

    let nonces = [nonce1, nonce2, nonce3];
    let mut ciphertexts = Vec::new();

    // Encrypt all messages
    for (msg, nonce) in messages.iter().zip(nonces.iter()) {
        let mut cipher = TurboShakeAead::new(&key, nonce).unwrap();
        ciphertexts.push(cipher.encrypt(msg, b""));
    }

    // Decrypt all messages
    for (ct, (msg, nonce)) in ciphertexts.iter().zip(messages.iter().zip(nonces.iter())) {
        let mut cipher = TurboShakeAead::new(&key, nonce).unwrap();
        let decrypted = cipher.decrypt(ct, b"").unwrap();
        assert_eq!(&decrypted, msg);
    }

    // Verify all ciphertexts are different
    assert_ne!(ciphertexts[0], ciphertexts[1]);
    assert_ne!(ciphertexts[1], ciphertexts[2]);
    assert_ne!(ciphertexts[0], ciphertexts[2]);
}

#[test]
fn test_aead_wrong_key_fails() {
    let key1 = [1u8; 32];
    let key2 = [2u8; 32];
    let nonce = [0u8; 16];
    let plaintext = b"Secret message";
    let ad = b"";

    let mut cipher_enc = TurboShakeAead::new(&key1, &nonce).unwrap();
    let ciphertext = cipher_enc.encrypt(plaintext, ad);

    let mut cipher_dec = TurboShakeAead::new(&key2, &nonce).unwrap();
    let result = cipher_dec.decrypt(&ciphertext, ad);

    assert_eq!(result, Err(AeadError::AuthenticationFailed));
}

#[test]
fn test_aead_wrong_nonce_fails() {
    let key = [1u8; 32];
    let nonce1 = [1u8; 16];
    let nonce2 = [2u8; 16];
    let plaintext = b"Secret message";
    let ad = b"";

    let mut cipher_enc = TurboShakeAead::new(&key, &nonce1).unwrap();
    let ciphertext = cipher_enc.encrypt(plaintext, ad);

    let mut cipher_dec = TurboShakeAead::new(&key, &nonce2).unwrap();
    let result = cipher_dec.decrypt(&ciphertext, ad);

    assert_eq!(result, Err(AeadError::AuthenticationFailed));
}

#[test]
fn test_aead_tampered_ciphertext_fails() {
    let key = [1u8; 32];
    let nonce = [2u8; 16];
    let plaintext = b"Original message that should not be tampered with";
    let ad = b"";

    let mut cipher_enc = TurboShakeAead::new(&key, &nonce).unwrap();
    let mut ciphertext = cipher_enc.encrypt(plaintext, ad);

    // Tamper with the ciphertext (flip a bit in the middle)
    let middle = ciphertext.len() / 2;
    ciphertext[middle] ^= 0x01;

    let mut cipher_dec = TurboShakeAead::new(&key, &nonce).unwrap();
    let result = cipher_dec.decrypt(&ciphertext, ad);

    assert_eq!(result, Err(AeadError::AuthenticationFailed));
}

#[test]
fn test_aead_tampered_tag_fails() {
    let key = [1u8; 32];
    let nonce = [2u8; 16];
    let plaintext = b"Message with tag";
    let ad = b"";

    let mut cipher_enc = TurboShakeAead::new(&key, &nonce).unwrap();
    let mut ciphertext = cipher_enc.encrypt(plaintext, ad);

    // Tamper with the tag (last 32 bytes)
    let tag_start = ciphertext.len() - 32;
    ciphertext[tag_start] ^= 0xFF;

    let mut cipher_dec = TurboShakeAead::new(&key, &nonce).unwrap();
    let result = cipher_dec.decrypt(&ciphertext, ad);

    assert_eq!(result, Err(AeadError::AuthenticationFailed));
}

#[test]
fn test_aead_large_message() {
    let key = [0x55u8; 32];
    let nonce = [0xAAu8; 16];
    let plaintext = vec![0x42u8; 10000]; // 10KB message
    let ad = b"Large message metadata";

    let mut cipher_enc = TurboShakeAead::new(&key, &nonce).unwrap();
    let ciphertext = cipher_enc.encrypt(&plaintext, ad);

    let mut cipher_dec = TurboShakeAead::new(&key, &nonce).unwrap();
    let decrypted = cipher_dec.decrypt(&ciphertext, ad).unwrap();

    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_aead_empty_plaintext_with_ad() {
    let key = [7u8; 32];
    let nonce = [13u8; 16];
    let plaintext = b"";
    let ad = b"Only authenticating this associated data";

    let mut cipher_enc = TurboShakeAead::new(&key, &nonce).unwrap();
    let ciphertext = cipher_enc.encrypt(plaintext, ad);

    // Ciphertext should only contain the tag (32 bytes)
    assert_eq!(ciphertext.len(), 32);

    let mut cipher_dec = TurboShakeAead::new(&key, &nonce).unwrap();
    let decrypted = cipher_dec.decrypt(&ciphertext, ad).unwrap();

    assert_eq!(decrypted, plaintext);

    // Wrong AD should fail
    let mut cipher_dec2 = TurboShakeAead::new(&key, &nonce).unwrap();
    let result = cipher_dec2.decrypt(&ciphertext, b"Wrong AD");
    assert_eq!(result, Err(AeadError::AuthenticationFailed));
}

#[test]
fn test_aead_various_message_sizes() {
    let key = [3u8; 32];
    let nonce = [9u8; 16];
    let ad = b"test";

    // Test various sizes around the rate boundary (136 bytes)
    for size in [1, 10, 50, 100, 135, 136, 137, 200, 272, 500] {
        let plaintext = vec![0xABu8; size];

        let mut cipher_enc = TurboShakeAead::new(&key, &nonce).unwrap();
        let ciphertext = cipher_enc.encrypt(&plaintext, ad);

        let mut cipher_dec = TurboShakeAead::new(&key, &nonce).unwrap();
        let decrypted = cipher_dec.decrypt(&ciphertext, ad).unwrap();

        assert_eq!(decrypted, plaintext, "Failed for message size {}", size);
    }
}

#[test]
fn test_aead_in_place_roundtrip() {
    let key = [0x42u8; 32];
    let nonce = [0x13u8; 16];
    let plaintext = b"In-place encryption and decryption test";
    let ad = b"metadata";

    // Encrypt in-place
    let mut buffer = plaintext.to_vec();
    let mut enc = TurboShakeAead::new(&key, &nonce).unwrap();
    enc.encrypt_in_place(&mut buffer, ad);

    // Decrypt in-place
    let mut dec = TurboShakeAead::new(&key, &nonce).unwrap();
    dec.decrypt_in_place(&mut buffer, ad).unwrap();

    assert_eq!(buffer, plaintext);
}

#[test]
fn test_aead_truncated_ciphertext_fails() {
    let key = [1u8; 32];
    let nonce = [2u8; 16];
    let plaintext = b"Test message";
    let ad = b"";

    let mut cipher_enc = TurboShakeAead::new(&key, &nonce).unwrap();
    let ciphertext = cipher_enc.encrypt(plaintext, ad);

    // Try to decrypt truncated ciphertext (remove some bytes)
    let truncated = &ciphertext[..ciphertext.len() - 5];

    let mut cipher_dec = TurboShakeAead::new(&key, &nonce).unwrap();
    let result = cipher_dec.decrypt(truncated, ad);

    assert_eq!(result, Err(AeadError::AuthenticationFailed));
}

#[test]
fn test_aead_ciphertext_too_short() {
    let key = [1u8; 32];
    let nonce = [2u8; 16];
    let ad = b"";

    // Ciphertext shorter than tag size (32 bytes)
    let short_ct = vec![0u8; 20];

    let mut cipher_dec = TurboShakeAead::new(&key, &nonce).unwrap();
    let result = cipher_dec.decrypt(&short_ct, ad);

    assert_eq!(result, Err(AeadError::AuthenticationFailed));
}

#[test]
fn test_aead_deterministic_encryption() {
    let key = [5u8; 32];
    let nonce = [10u8; 16];
    let plaintext = b"Same input should produce same output";
    let ad = b"metadata";

    let mut cipher1 = TurboShakeAead::new(&key, &nonce).unwrap();
    let ct1 = cipher1.encrypt(plaintext, ad);

    let mut cipher2 = TurboShakeAead::new(&key, &nonce).unwrap();
    let ct2 = cipher2.encrypt(plaintext, ad);

    // With same key and nonce, encryption should be deterministic
    assert_eq!(ct1, ct2);
}
