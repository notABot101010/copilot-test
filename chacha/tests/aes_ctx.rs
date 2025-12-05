use aws_lc_rs::aead::{Aad, LessSafeKey, Nonce, Tag, UnboundKey, AES_256_GCM};
use aws_lc_rs::digest::{Context, SHA256};

fn encrypt_ctx(
    key: &[u8; 32],
    nonce: &[u8; 12],
    ad: &[u8],
    plaintext: &[u8],
) -> (Vec<u8>, Tag, Vec<u8>) {
    let unbound_key = match UnboundKey::new(&AES_256_GCM, key) {
        Ok(key_material) => key_material,
        Err(err) => panic!("failed to initialize AES-256-GCM key: {err:?}"),
    };

    let sealing_key = LessSafeKey::new(unbound_key);
    let nonce_value = Nonce::assume_unique_for_key(*nonce);
    let mut in_out = plaintext.to_vec();
    let tag = match sealing_key.seal_in_place_separate_tag(nonce_value, Aad::from(ad), &mut in_out) {
        Ok(tag) => tag,
        Err(err) => panic!("AES-256-GCM seal failed: {err:?}"),
    };

    let mut ctx = Context::new(&SHA256);
    ctx.update(nonce);
    ctx.update(ad);
    ctx.update(tag.as_ref());
    ctx.update(key);
    let secondary_tag = ctx.finish().as_ref().to_vec();

    (in_out, tag, secondary_tag)
}

#[test]
fn aes256_gcm_ctx_roundtrip() {
    let key = [0x11u8; 32];
    let nonce = [0xA5u8; 12];
    let associated_data = b"bench-ctx";
    let plaintext = b"ctx roundtrip validation data";

    let (ciphertext, tag, secondary_tag) =
        encrypt_ctx(&key, &nonce, associated_data, plaintext);

    let unbound_key = match UnboundKey::new(&AES_256_GCM, &key) {
        Ok(key_material) => key_material,
        Err(err) => panic!("failed to initialize AES-256-GCM key: {err:?}"),
    };
    let opening_key = LessSafeKey::new(unbound_key);
    let nonce_value = Nonce::assume_unique_for_key(nonce);
    let aad = Aad::from(associated_data.as_slice());

    let mut combined = ciphertext.clone();
    combined.extend_from_slice(tag.as_ref());
    let opened = match opening_key.open_in_place(nonce_value, aad, &mut combined) {
        Ok(result) => result,
        Err(err) => panic!("AES-256-GCM open failed: {err:?}"),
    };

    assert_eq!(opened, plaintext);

    let mut ctx = Context::new(&SHA256);
    ctx.update(&key);
    ctx.update(&nonce);
    ctx.update(associated_data);
    ctx.update(tag.as_ref());
    let recomputed = ctx.finish().as_ref().to_vec();

    assert_eq!(secondary_tag, recomputed);
}
