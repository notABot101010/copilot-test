use duplex::{AeadError, KeccakAead};

#[test]
fn test_basic_encryption_decryption() {
    let key = [42u8; 32];
    let nonce = [17u8; 16];
    let plaintext = b"This is a test message for integration testing";
    let ad = b"";

    let mut cipher_enc = KeccakAead::new(&key, &nonce).unwrap();
    let ciphertext = cipher_enc.encrypt(plaintext, ad);

    let mut cipher_dec = KeccakAead::new(&key, &nonce).unwrap();
    let decrypted = cipher_dec.decrypt(&ciphertext, ad).unwrap();

    assert_eq!(decrypted, plaintext);
    assert_ne!(&ciphertext[..plaintext.len()], plaintext);
}

#[test]
fn test_with_associated_data() {
    let key = [1u8; 32];
    let nonce = [2u8; 16];
    let plaintext = b"Secret data";
    let ad = b"This is public metadata that should be authenticated";

    let mut cipher_enc = KeccakAead::new(&key, &nonce).unwrap();
    let ciphertext = cipher_enc.encrypt(plaintext, ad);

    let mut cipher_dec = KeccakAead::new(&key, &nonce).unwrap();
    let decrypted = cipher_dec.decrypt(&ciphertext, ad).unwrap();

    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_multiple_messages_same_key_different_nonces() {
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
        let mut cipher = KeccakAead::new(&key, nonce).unwrap();
        ciphertexts.push(cipher.encrypt(msg, b""));
    }

    // Decrypt all messages
    for (ct, (msg, nonce)) in ciphertexts.iter().zip(messages.iter().zip(nonces.iter())) {
        let mut cipher = KeccakAead::new(&key, nonce).unwrap();
        let decrypted = cipher.decrypt(ct, b"").unwrap();
        assert_eq!(&decrypted, msg);
    }

    // Verify all ciphertexts are different
    assert_ne!(ciphertexts[0], ciphertexts[1]);
    assert_ne!(ciphertexts[1], ciphertexts[2]);
    assert_ne!(ciphertexts[0], ciphertexts[2]);
}

#[test]
fn test_wrong_key_fails() {
    let key1 = [1u8; 32];
    let key2 = [2u8; 32];
    let nonce = [0u8; 16];
    let plaintext = b"Secret message";
    let ad = b"";

    let mut cipher_enc = KeccakAead::new(&key1, &nonce).unwrap();
    let ciphertext = cipher_enc.encrypt(plaintext, ad);

    let mut cipher_dec = KeccakAead::new(&key2, &nonce).unwrap();
    let result = cipher_dec.decrypt(&ciphertext, ad);

    assert_eq!(result, Err(AeadError::AuthenticationFailed));
}

#[test]
fn test_wrong_nonce_fails() {
    let key = [1u8; 32];
    let nonce1 = [1u8; 16];
    let nonce2 = [2u8; 16];
    let plaintext = b"Secret message";
    let ad = b"";

    let mut cipher_enc = KeccakAead::new(&key, &nonce1).unwrap();
    let ciphertext = cipher_enc.encrypt(plaintext, ad);

    let mut cipher_dec = KeccakAead::new(&key, &nonce2).unwrap();
    let result = cipher_dec.decrypt(&ciphertext, ad);

    assert_eq!(result, Err(AeadError::AuthenticationFailed));
}

#[test]
fn test_tampered_ciphertext_fails() {
    let key = [1u8; 32];
    let nonce = [2u8; 16];
    let plaintext = b"Original message that should not be tampered with";
    let ad = b"";

    let mut cipher_enc = KeccakAead::new(&key, &nonce).unwrap();
    let mut ciphertext = cipher_enc.encrypt(plaintext, ad);

    // Tamper with the ciphertext (flip a bit in the middle)
    let middle = ciphertext.len() / 2;
    ciphertext[middle] ^= 0x01;

    let mut cipher_dec = KeccakAead::new(&key, &nonce).unwrap();
    let result = cipher_dec.decrypt(&ciphertext, ad);

    assert_eq!(result, Err(AeadError::AuthenticationFailed));
}

#[test]
fn test_tampered_tag_fails() {
    let key = [1u8; 32];
    let nonce = [2u8; 16];
    let plaintext = b"Message with tag";
    let ad = b"";

    let mut cipher_enc = KeccakAead::new(&key, &nonce).unwrap();
    let mut ciphertext = cipher_enc.encrypt(plaintext, ad);

    // Tamper with the tag (last 32 bytes)
    let tag_start = ciphertext.len() - 32;
    ciphertext[tag_start] ^= 0xFF;

    let mut cipher_dec = KeccakAead::new(&key, &nonce).unwrap();
    let result = cipher_dec.decrypt(&ciphertext, ad);

    assert_eq!(result, Err(AeadError::AuthenticationFailed));
}

#[test]
fn test_large_message() {
    let key = [0x55u8; 32];
    let nonce = [0xAAu8; 16];
    let plaintext = vec![0x42u8; 10000]; // 10KB message
    let ad = b"Large message metadata";

    let mut cipher_enc = KeccakAead::new(&key, &nonce).unwrap();
    let ciphertext = cipher_enc.encrypt(&plaintext, ad);

    let mut cipher_dec = KeccakAead::new(&key, &nonce).unwrap();
    let decrypted = cipher_dec.decrypt(&ciphertext, ad).unwrap();

    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_empty_plaintext_with_ad() {
    let key = [7u8; 32];
    let nonce = [13u8; 16];
    let plaintext = b"";
    let ad = b"Only authenticating this associated data";

    let mut cipher_enc = KeccakAead::new(&key, &nonce).unwrap();
    let ciphertext = cipher_enc.encrypt(plaintext, ad);

    // Ciphertext should only contain the tag (32 bytes)
    assert_eq!(ciphertext.len(), 32);

    let mut cipher_dec = KeccakAead::new(&key, &nonce).unwrap();
    let decrypted = cipher_dec.decrypt(&ciphertext, ad).unwrap();

    assert_eq!(decrypted, plaintext);

    // Wrong AD should fail
    let mut cipher_dec2 = KeccakAead::new(&key, &nonce).unwrap();
    let result = cipher_dec2.decrypt(&ciphertext, b"Wrong AD");
    assert_eq!(result, Err(AeadError::AuthenticationFailed));
}

#[test]
fn test_various_message_sizes() {
    let key = [3u8; 32];
    let nonce = [9u8; 16];
    let ad = b"test";

    // Test various sizes around the rate boundary (136 bytes)
    for size in [1, 10, 50, 100, 135, 136, 137, 200, 272, 500] {
        let plaintext = vec![0xABu8; size];

        let mut cipher_enc = KeccakAead::new(&key, &nonce).unwrap();
        let ciphertext = cipher_enc.encrypt(&plaintext, ad);

        let mut cipher_dec = KeccakAead::new(&key, &nonce).unwrap();
        let decrypted = cipher_dec.decrypt(&ciphertext, ad).unwrap();

        assert_eq!(
            decrypted, plaintext,
            "Failed for message size {}",
            size
        );
    }
}

#[test]
fn test_boundary_conditions() {
    let key = [11u8; 32];
    let nonce = [22u8; 16];

    // Test with exactly one rate-sized block (136 bytes)
    let plaintext = vec![0x77u8; 136];
    let ad = b"";

    let mut cipher_enc = KeccakAead::new(&key, &nonce).unwrap();
    let ciphertext = cipher_enc.encrypt(&plaintext, ad);

    let mut cipher_dec = KeccakAead::new(&key, &nonce).unwrap();
    let decrypted = cipher_dec.decrypt(&ciphertext, ad).unwrap();

    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_truncated_ciphertext_fails() {
    let key = [1u8; 32];
    let nonce = [2u8; 16];
    let plaintext = b"Test message";
    let ad = b"";

    let mut cipher_enc = KeccakAead::new(&key, &nonce).unwrap();
    let ciphertext = cipher_enc.encrypt(plaintext, ad);

    // Try to decrypt truncated ciphertext (remove some bytes)
    let truncated = &ciphertext[..ciphertext.len() - 5];

    let mut cipher_dec = KeccakAead::new(&key, &nonce).unwrap();
    let result = cipher_dec.decrypt(truncated, ad);

    assert_eq!(result, Err(AeadError::AuthenticationFailed));
}

#[test]
fn test_ciphertext_too_short() {
    let key = [1u8; 32];
    let nonce = [2u8; 16];
    let ad = b"";

    // Ciphertext shorter than tag size (32 bytes)
    let short_ct = vec![0u8; 20];

    let mut cipher_dec = KeccakAead::new(&key, &nonce).unwrap();
    let result = cipher_dec.decrypt(&short_ct, ad);

    assert_eq!(result, Err(AeadError::AuthenticationFailed));
}

#[test]
fn test_deterministic_encryption() {
    let key = [5u8; 32];
    let nonce = [10u8; 16];
    let plaintext = b"Same input should produce same output";
    let ad = b"metadata";

    let mut cipher1 = KeccakAead::new(&key, &nonce).unwrap();
    let ct1 = cipher1.encrypt(plaintext, ad);

    let mut cipher2 = KeccakAead::new(&key, &nonce).unwrap();
    let ct2 = cipher2.encrypt(plaintext, ad);

    // With same key and nonce, encryption should be deterministic
    assert_eq!(ct1, ct2);
}

#[test]
fn test_different_ad_produces_same_ciphertext_but_different_tag() {
    let key = [6u8; 32];
    let nonce = [12u8; 16];
    let plaintext = b"Same plaintext";
    let ad1 = b"First AD";
    let ad2 = b"Second AD";

    let mut cipher1 = KeccakAead::new(&key, &nonce).unwrap();
    let ct1 = cipher1.encrypt(plaintext, ad1);

    let mut cipher2 = KeccakAead::new(&key, &nonce).unwrap();
    let ct2 = cipher2.encrypt(plaintext, ad2);

    // Different AD should produce different results
    assert_ne!(ct1, ct2);

    // But they should both decrypt correctly with their respective AD
    let mut cipher_dec1 = KeccakAead::new(&key, &nonce).unwrap();
    assert_eq!(cipher_dec1.decrypt(&ct1, ad1).unwrap(), plaintext);

    let mut cipher_dec2 = KeccakAead::new(&key, &nonce).unwrap();
    assert_eq!(cipher_dec2.decrypt(&ct2, ad2).unwrap(), plaintext);
}
