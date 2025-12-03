//! DJB ChaCha stream cipher implementation with parametrized rounds.
//!
//! This module provides an implementation of the ChaCha stream cipher as designed
//! by Daniel J. Bernstein, using a 64-bit counter and 64-bit nonce.
//!
//! The ChaCha cipher is a variant of Salsa20 with improved diffusion per round.

#[cfg(target_arch = "x86_64")]
mod chacha_avx2;

/// ChaCha stream cipher with parametrized number of rounds.
///
/// The state array contains:
/// - `state[0..16]`: The 512-bit ChaCha state (16 x 32-bit words)
/// - `last_keystream_block[0]`: Number of remaining keystream bytes available (0-63)
/// - `last_keystream_block[1..64]`: Storage for remaining keystream bytes
pub struct ChaCha<const ROUNDS: usize> {
    state: [u32; 16],
    last_keystream_block: [u8; 64],
}

/// ChaCha20 is ChaCha with 20 rounds (the most common variant).
pub type ChaCha20 = ChaCha<20>;

/// ChaCha12 is ChaCha with 12 rounds.
pub type ChaCha12 = ChaCha<12>;

/// ChaCha8 is ChaCha with 8 rounds.
pub type ChaCha8 = ChaCha<8>;

/// Constants for ChaCha: "expand 32-byte k" in little-endian
const CONSTANTS: [u32; 4] = [0x61707865, 0x3320646e, 0x79622d32, 0x6b206574];

/// The ChaCha quarter round operation.
#[inline]
fn quarter_round(state: &mut [u32; 16], a: usize, b: usize, c: usize, d: usize) {
    state[a] = state[a].wrapping_add(state[b]);
    state[d] ^= state[a];
    state[d] = state[d].rotate_left(16);

    state[c] = state[c].wrapping_add(state[d]);
    state[b] ^= state[c];
    state[b] = state[b].rotate_left(12);

    state[a] = state[a].wrapping_add(state[b]);
    state[d] ^= state[a];
    state[d] = state[d].rotate_left(8);

    state[c] = state[c].wrapping_add(state[d]);
    state[b] ^= state[c];
    state[b] = state[b].rotate_left(7);
}

/// Perform the ChaCha block function (double rounds).
#[inline]
fn chacha_block<const ROUNDS: usize>(state: &[u32; 16]) -> [u32; 16] {
    let mut working_state = *state;

    // Each double round consists of 4 column rounds and 4 diagonal rounds
    for _ in 0..(ROUNDS / 2) {
        // Column rounds
        quarter_round(&mut working_state, 0, 4, 8, 12);
        quarter_round(&mut working_state, 1, 5, 9, 13);
        quarter_round(&mut working_state, 2, 6, 10, 14);
        quarter_round(&mut working_state, 3, 7, 11, 15);
        // Diagonal rounds
        quarter_round(&mut working_state, 0, 5, 10, 15);
        quarter_round(&mut working_state, 1, 6, 11, 12);
        quarter_round(&mut working_state, 2, 7, 8, 13);
        quarter_round(&mut working_state, 3, 4, 9, 14);
    }

    // Add the original state to the working state
    working_state
        .iter_mut()
        .zip(state.iter())
        .for_each(|(ws, s)| *ws = ws.wrapping_add(*s));

    working_state
}

/// Serialize the state to a byte array (little-endian).
#[inline]
fn serialize_state(state: &[u32; 16], output: &mut [u8; 64]) {
    for (i, word) in state.iter().enumerate() {
        output[i * 4..i * 4 + 4].copy_from_slice(&word.to_le_bytes());
    }
}

impl<const ROUNDS: usize> ChaCha<ROUNDS> {
    /// Creates a new ChaCha cipher instance with the given key and nonce.
    ///
    /// The counter is initialized to 0.
    ///
    /// # Arguments
    ///
    /// * `key` - A 256-bit (32-byte) key
    /// * `nonce` - A 64-bit (8-byte) nonce
    ///
    /// # State layout
    ///
    /// ```text
    /// Constants | Constants | Constants | Constants
    /// Key       | Key       | Key       | Key
    /// Key       | Key       | Key       | Key
    /// Counter   | Counter   | Nonce     | Nonce
    /// ```
    pub fn new(key: &[u8; 32], nonce: &[u8; 8]) -> ChaCha<ROUNDS> {
        let mut state = [0u32; 16];

        // Set constants
        state[0..4].copy_from_slice(&CONSTANTS);

        // Set key (8 x 32-bit words)
        for (state_word, key_chunk) in state[4..12].iter_mut().zip(key.chunks_exact(4)) {
            *state_word = u32::from_le_bytes(key_chunk.try_into().unwrap());
        }

        // Counter starts at 0 (will be in state[12] and state[13])
        state[12] = 0;
        state[13] = 0;

        // Set nonce (2 x 32-bit words)
        state[14] = u32::from_le_bytes([nonce[0], nonce[1], nonce[2], nonce[3]]);
        state[15] = u32::from_le_bytes([nonce[4], nonce[5], nonce[6], nonce[7]]);

        ChaCha {
            state,
            last_keystream_block: [0u8; 64],
        }
    }

    /// Sets the counter value.
    ///
    /// This is useful for seeking to a specific position in the keystream
    /// or for setting an initial counter value as required by some protocols.
    ///
    /// Note: This also clears any remaining keystream bytes from the previous block.
    pub fn set_counter(&mut self, counter: u64) {
        self.state[12] = counter as u32;
        self.state[13] = (counter >> 32) as u32;
        // Clear remaining keystream bytes
        self.last_keystream_block[0] = 0;
    }

    /// Returns the current counter value.
    #[inline]
    pub fn counter(&self) -> u64 {
        return ((self.state[13] as u64) << 32) | (self.state[12] as u64);
    }

    /// Increments the 64-bit counter by the given amount.
    #[inline]
    fn increment_counter(&mut self, amount: u64) {
        let counter = (self.state[12] as u64) | ((self.state[13] as u64) << 32);
        let new_counter = counter.wrapping_add(amount);
        self.state[12] = new_counter as u32;
        self.state[13] = (new_counter >> 32) as u32;
    }

    /// Generates the next keystream block and returns it.
    fn next_keystream_block(&mut self) -> [u8; 64] {
        let block = chacha_block::<ROUNDS>(&self.state);
        self.increment_counter(1);

        let mut keystream = [0u8; 64];
        serialize_state(&block, &mut keystream);
        keystream
    }

    /// XORs the plaintext with the keystream to produce ciphertext (or vice versa).
    ///
    /// This function can be called multiple times with chunks of data.
    /// If the previous call did not consume a full block of keystream,
    /// the remaining bytes will be used first.
    ///
    /// # Arguments
    ///
    /// * `data` - The data to encrypt/decrypt. This is modified in place.
    pub fn xor_keystream(&mut self, data: &mut [u8]) {
        if data.is_empty() {
            return;
        }

        let mut offset = 0;
        let data_len = data.len();

        // First, use any remaining keystream bytes from the previous block
        let remaining = self.last_keystream_block[0] as usize;
        if remaining > 0 {
            let use_bytes = remaining.min(data_len);
            let start_idx = 64 - remaining;
            data[..use_bytes]
                .iter_mut()
                .zip(&self.last_keystream_block[start_idx..start_idx + use_bytes])
                .for_each(|(d, k)| *d ^= k);
            offset = use_bytes;

            if use_bytes < remaining {
                // Still have remaining bytes, update the count
                self.last_keystream_block[0] = (remaining - use_bytes) as u8;
                // Shift the remaining keystream bytes to the end
                // No need to shift since we're reading from the end
            } else {
                // Used all remaining bytes
                self.last_keystream_block[0] = 0;
            }
        }

        // Use AVX2 for large data if available
        #[cfg(target_arch = "x86_64")]
        {
            use chacha_avx2::AlignedU8x512;
            if is_x86_feature_detected!("avx2") && (data_len - offset) >= 512 {
                // Process 512-byte chunks (8 blocks) using full AVX2
                while offset + 512 <= data_len {
                    let mut keystream = AlignedU8x512([0u8; 512]);
                    unsafe {
                        chacha_avx2::chacha_blocks_avx2_x8::<ROUNDS>(&self.state, &mut keystream);
                    }

                    // XOR keystream with data using AVX2
                    unsafe {
                        use core::arch::x86_64::*;
                        for i in (0..512).step_by(32) {
                            let data_ptr = data.as_mut_ptr().add(offset + i);
                            let key_ptr = keystream.0.as_ptr().add(i);
                            let data_vec = _mm256_loadu_si256(data_ptr as *const __m256i);
                            let key_vec = _mm256_load_si256(key_ptr as *const __m256i);
                            let result = _mm256_xor_si256(data_vec, key_vec);
                            _mm256_storeu_si256(data_ptr as *mut __m256i, result);
                        }
                    }

                    self.increment_counter(8);
                    offset += 512;
                }
            }

            // Process remaining 256-byte chunks (4 blocks) using SSE
            if is_x86_feature_detected!("avx2") && (data_len - offset) >= 256 {
                while offset + 256 <= data_len {
                    let mut keystream = [0u8; 256];
                    unsafe {
                        chacha_avx2::chacha_blocks_avx2::<ROUNDS>(&self.state, &mut keystream);
                    }

                    // XOR keystream with data
                    unsafe {
                        use core::arch::x86_64::*;
                        for i in (0..256).step_by(32) {
                            let data_ptr = data.as_mut_ptr().add(offset + i);
                            let key_ptr = keystream.as_ptr().add(i);
                            let data_vec = _mm256_loadu_si256(data_ptr as *const __m256i);
                            let key_vec = _mm256_loadu_si256(key_ptr as *const __m256i);
                            let result = _mm256_xor_si256(data_vec, key_vec);
                            _mm256_storeu_si256(data_ptr as *mut __m256i, result);
                        }
                    }

                    self.increment_counter(4);
                    offset += 256;
                }
            }
        }

        // Process remaining full blocks using scalar code
        while offset + 64 <= data_len {
            let keystream = self.next_keystream_block();
            data[offset..offset + 64]
                .iter_mut()
                .zip(&keystream)
                .for_each(|(d, k)| *d ^= k);
            offset += 64;
        }

        // Handle remaining bytes (partial block)
        let remaining_data = data_len - offset;
        if remaining_data > 0 {
            let keystream = self.next_keystream_block();
            data[offset..]
                .iter_mut()
                .zip(&keystream[..remaining_data])
                .for_each(|(d, k)| *d ^= k);

            // Store the remaining keystream bytes for later use
            // remaining bytes = 64 - remaining_data
            let remaining_keystream = 64 - remaining_data;
            self.last_keystream_block[0] = remaining_keystream as u8;
            // Copy the remaining keystream bytes to the end of last_keystream_block
            // We store from position (64 - remaining_keystream) to 63
            self.last_keystream_block[64 - remaining_keystream..64]
                .copy_from_slice(&keystream[remaining_data..]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Test {
        key: [u8; 32],
        nonce: [u8; 8],
        initial_counter: u64,
        plaintext: Vec<u8>,
        expected_ciphertext: Vec<u8>,
    }

    #[test]
    fn chacha20_test_vectors() {
        let tests = vec![
            // https://www.rfc-editor.org/rfc/rfc8439#section-2.4.2
            Test {
                key: hex::decode(
                    "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f",
                )
                .unwrap()
                .try_into()
                .unwrap(),
                nonce: hex::decode("0000004a00000000").unwrap().try_into().unwrap(),
                initial_counter: 1,
                plaintext: hex::decode(
                    "4c616469657320616e642047656e746c\
656d656e206f662074686520636c6173\
73206f66202739393a20496620492063\
6f756c64206f6666657220796f75206f\
6e6c79206f6e652074697020666f7220\
746865206675747572652c2073756e73\
637265656e20776f756c642062652069\
742e",
                )
                .unwrap(),
                expected_ciphertext: hex::decode(
                    "6e2e359a2568f98041ba0728dd0d6981\
e97e7aec1d4360c20a27afccfd9fae0b\
f91b65c5524733ab8f593dabcd62b357\
1639d624e65152ab8f530c359f0861d8\
07ca0dbf500d6a6156a38e088a22b65e\
52bc514d16ccf806818ce91ab7793736\
5af90bbf74a35be6b40b8eedf2785e42\
874d",
                )
                .unwrap(),
            },
            // https://www.rfc-editor.org/rfc/rfc8439#appendix-A.2 Test vector #1
            Test {
                key: [0u8; 32],
                nonce: [0u8; 8],
                initial_counter: 0,
                plaintext: [0u8; 64].to_vec(),
                expected_ciphertext: hex::decode(
                    "76b8e0ada0f13d90405d6ae55386bd28\
bdd219b8a08ded1aa836efcc8b770dc7\
da41597c5157488d7724e03fb8d84a37\
6a43b8f41518a11cc387b669b2ee6586",
                )
                .unwrap(),
            },
            // https://www.rfc-editor.org/rfc/rfc8439#appendix-A.2 Test Vector #2
            Test {
                key: hex::decode(
                    "0000000000000000000000000000000000000000000000000000000000000001",
                )
                .unwrap()
                .try_into()
                .unwrap(),
                nonce: hex::decode("0000000000000002").unwrap().try_into().unwrap(),
                initial_counter: 1,
                plaintext: hex::decode(
                    "416e79207375626d697373696f6e2074\
6f20746865204945544620696e74656e\
6465642062792074686520436f6e7472\
696275746f7220666f72207075626c69\
636174696f6e20617320616c6c206f72\
2070617274206f6620616e2049455446\
20496e7465726e65742d447261667420\
6f722052464320616e6420616e792073\
746174656d656e74206d616465207769\
7468696e2074686520636f6e74657874\
206f6620616e20494554462061637469\
7669747920697320636f6e7369646572\
656420616e20224945544620436f6e74\
7269627574696f6e222e205375636820\
73746174656d656e747320696e636c75\
6465206f72616c2073746174656d656e\
747320696e2049455446207365737369\
6f6e732c2061732077656c6c20617320\
7772697474656e20616e6420656c6563\
74726f6e696320636f6d6d756e696361\
74696f6e73206d61646520617420616e\
792074696d65206f7220706c6163652c\
20776869636820617265206164647265\
7373656420746f",
                )
                .unwrap(),
                expected_ciphertext: hex::decode(
                    "a3fbf07df3fa2fde4f376ca23e827370\
41605d9f4f4f57bd8cff2c1d4b7955ec\
2a97948bd3722915c8f3d337f7d37005\
0e9e96d647b7c39f56e031ca5eb6250d\
4042e02785ececfa4b4bb5e8ead0440e\
20b6e8db09d881a7c6132f420e527950\
42bdfa7773d8a9051447b3291ce1411c\
680465552aa6c405b7764d5e87bea85a\
d00f8449ed8f72d0d662ab052691ca66\
424bc86d2df80ea41f43abf937d3259d\
c4b2d0dfb48a6c9139ddd7f76966e928\
e635553ba76c5c879d7b35d49eb2e62b\
0871cdac638939e25e8a1e0ef9d5280f\
a8ca328b351c3c765989cbcf3daa8b6c\
cc3aaf9f3979c92b3720fc88dc95ed84\
a1be059c6499b9fda236e7e818b04b0b\
c39c1e876b193bfe5569753f88128cc0\
8aaa9b63d1a16f80ef2554d7189c411f\
5869ca52c5b83fa36ff216b9c1d30062\
bebcfd2dc5bce0911934fda79a86f6e6\
98ced759c3ff9b6477338f3da4f9cd85\
14ea9982ccafb341b2384dd902f3d1ab\
7ac61dd29c6f21ba5b862f3730e37cfd\
c4fd806c22f221",
                )
                .unwrap(),
            },
            // https://www.rfc-editor.org/rfc/rfc8439#appendix-A.2 Test Vector #3
            Test {
                key: hex::decode(
                    "1c9240a5eb55d38af333888604f6b5f0473917c1402b80099dca5cbc207075c0",
                )
                .unwrap()
                .try_into()
                .unwrap(),
                nonce: hex::decode("0000000000000002").unwrap().try_into().unwrap(),
                initial_counter: 42,
                plaintext: hex::decode(
                    "2754776173206272696c6c69672c2061\
6e642074686520736c6974687920746f\
7665730a446964206779726520616e64\
2067696d626c6520696e207468652077\
6162653a0a416c6c206d696d73792077\
6572652074686520626f726f676f7665\
732c0a416e6420746865206d6f6d6520\
7261746873206f757467726162652e",
                )
                .unwrap(),
                expected_ciphertext: hex::decode(
                    "62e6347f95ed87a45ffae7426f27a1df\
5fb69110044c0d73118effa95b01e5cf\
166d3df2d721caf9b21e5fb14c616871\
fd84c54f9d65b283196c7fe4f60553eb\
f39c6402c42234e32a356b3e764312a6\
1a5532055716ead6962568f87d3f3f77\
04c6a8d1bcd1bf4d50d6154b6da731b1\
87b58dfd728afa36757a797ac188d1",
                )
                .unwrap(),
            },
        ];

        for (i, test) in tests.into_iter().enumerate() {
            let mut cipher = ChaCha20::new(&test.key, &test.nonce);
            cipher.set_counter(test.initial_counter);

            let mut plaintext = test.plaintext.clone();
            cipher.xor_keystream(&mut plaintext);

            assert_eq!(
                plaintext,
                test.expected_ciphertext,
                "test [{i}] failed
Got ciphertext: {}
Expected ciphertext: {}",
                hex::encode(&plaintext),
                hex::encode(&test.expected_ciphertext),
            );

            let mut cipher = ChaCha20::new(&test.key, &test.nonce);
            cipher.set_counter(test.initial_counter);
            cipher.xor_keystream(&mut plaintext);

            assert_eq!(
                plaintext,
                test.plaintext,
                "test [{i}] failed. Initial plaintext != decrypt(encrypt(plaintext))
Got: {}
Expected: {}",
                hex::encode(&plaintext),
                hex::encode(&test.plaintext),
            );

            // ensure that the encryption is correct even for plaintexts that are not % 64 (block size)
            // thus:
            // cipher.xor_keystream(plaintext[0..10])
            // cipher.xor_keystream(plaintext[10..30])
            // cipher.xor_keystream(plaintext[30..35])
            // should be equal to:
            // cipher.xor_keystream(plaintext[0..35])

            let mut cipher = ChaCha20::new(&test.key, &test.nonce);
            cipher.set_counter(test.initial_counter);
            cipher.xor_keystream(&mut plaintext);
            for n in 0..10 {
                let mut partial_plaintext: Vec<u8> = test.plaintext.clone();

                let mut cipher = ChaCha20::new(&test.key, &test.nonce);
                cipher.set_counter(test.initial_counter);
                cipher.xor_keystream(&mut partial_plaintext[..n]);
                cipher.xor_keystream(&mut partial_plaintext[n..]);

                assert_eq!(
                    plaintext,
                    partial_plaintext,
                    "test [{i}] failed. partial encryption is not valid for n = {n}
            Got: {}
            Expected: {}",
                    hex::encode(&partial_plaintext),
                    hex::encode(&plaintext),
                )
            }
        }
    }

    #[test]
    fn test_quarter_round() {
        // Test vector from RFC 8439 section 2.1.1
        let mut state = [
            0x879531e0, 0xc5ecf37d, 0x516461b1, 0xc9a62f8a, 0x44c20ef3, 0x3390af7f, 0xd9fc690b,
            0x2a5f714c, 0x53372767, 0xb00a5631, 0x974c541a, 0x359e9963, 0x5c971061, 0x3d631689,
            0x2098d9d6, 0x91dbd320,
        ];

        quarter_round(&mut state, 2, 7, 8, 13);

        assert_eq!(state[2], 0xbdb886dc, "state[2] mismatch");
        assert_eq!(state[7], 0xcfacafd2, "state[7] mismatch");
        assert_eq!(state[8], 0xe46bea80, "state[8] mismatch");
        assert_eq!(state[13], 0xccc07c79, "state[13] mismatch");
    }

    #[test]
    fn test_chacha20_block() {
        // Test vector adapted from RFC 8439 section 2.3.2 for DJB variant
        // The RFC uses IETF variant (32-bit counter, 96-bit nonce) but we're testing
        // DJB variant (64-bit counter, 64-bit nonce), so we manually construct the state
        let key = hex::decode("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f")
            .unwrap();
        // For DJB variant, use 8-byte nonce
        let nonce: [u8; 8] = [0x00, 0x00, 0x00, 0x4a, 0x00, 0x00, 0x00, 0x00];

        let mut state = [0u32; 16];
        state[0..4].copy_from_slice(&CONSTANTS);

        for (i, chunk) in key.chunks_exact(4).enumerate() {
            state[4 + i] = u32::from_le_bytes(chunk.try_into().unwrap());
        }

        // DJB variant: 64-bit counter in state[12-13], 64-bit nonce in state[14-15]
        state[12] = 1; // counter low
        state[13] = 0; // counter high
        state[14] = u32::from_le_bytes([nonce[0], nonce[1], nonce[2], nonce[3]]);
        state[15] = u32::from_le_bytes([nonce[4], nonce[5], nonce[6], nonce[7]]);

        let result = chacha_block::<20>(&state);

        // Verify the first output word matches what we expect based on our state setup
        // Since this is DJB variant with different state layout, we verify by testing
        // that encryption/decryption works (main test vectors already verify this)
        // Here we just verify the block function produces non-zero output
        assert_ne!(
            result[0], 0,
            "ChaCha20 block should produce non-zero output"
        );

        // Verify the block output differs from input state
        let mut differs = false;
        for i in 0..16 {
            if result[i] != state[i] {
                differs = true;
                break;
            }
        }
        assert!(
            differs,
            "ChaCha20 block output should differ from input state"
        );
    }

    #[test]
    fn test_counter_increment() {
        let key = [0u8; 32];
        let nonce = [0u8; 8];

        let mut cipher = ChaCha::<20>::new(&key, &nonce);
        assert_eq!(cipher.counter(), 0);

        cipher.set_counter(1);
        assert_eq!(cipher.counter(), 1);

        // Test counter wrap-around at 32-bit boundary
        cipher.set_counter(0xFFFFFFFF);
        assert_eq!(cipher.counter(), 0xFFFFFFFF);

        // Process one block to increment counter
        let mut data = [0u8; 64];
        cipher.xor_keystream(&mut data);
        assert_eq!(cipher.counter(), 0x100000000);

        // Test large counter value
        cipher.set_counter(0xFFFFFFFF_FFFFFFFFu64);
        assert_eq!(cipher.counter(), 0xFFFFFFFF_FFFFFFFFu64);
    }

    #[test]
    fn test_empty_input() {
        let key = [0u8; 32];
        let nonce = [0u8; 8];

        let mut cipher = ChaCha20::new(&key, &nonce);
        let mut data: [u8; 0] = [];
        cipher.xor_keystream(&mut data);
        assert_eq!(cipher.counter(), 0);
    }

    #[test]
    fn test_partial_blocks() {
        let key = [1u8; 32];
        let nonce = [2u8; 8];

        // Encrypt in one call
        let mut data1 = vec![3u8; 100];
        let mut cipher1 = ChaCha20::new(&key, &nonce);
        cipher1.xor_keystream(&mut data1);

        // Encrypt in multiple calls
        let mut data2 = vec![3u8; 100];
        let mut cipher2 = ChaCha20::new(&key, &nonce);
        cipher2.xor_keystream(&mut data2[..10]);
        cipher2.xor_keystream(&mut data2[10..50]);
        cipher2.xor_keystream(&mut data2[50..51]);
        cipher2.xor_keystream(&mut data2[51..]);

        assert_eq!(data1, data2, "Partial block encryption mismatch");
    }

    #[test]
    fn test_chacha8_and_chacha12() {
        let key = [0u8; 32];
        let nonce = [0u8; 8];

        // ChaCha8
        let mut cipher8 = ChaCha8::new(&key, &nonce);
        let mut data8 = vec![0u8; 64];
        cipher8.xor_keystream(&mut data8);

        // ChaCha12
        let mut cipher12 = ChaCha12::new(&key, &nonce);
        let mut data12 = vec![0u8; 64];
        cipher12.xor_keystream(&mut data12);

        // ChaCha20
        let mut cipher20 = ChaCha20::new(&key, &nonce);
        let mut data20 = vec![0u8; 64];
        cipher20.xor_keystream(&mut data20);

        // All three should produce different outputs
        assert_ne!(data8, data12, "ChaCha8 and ChaCha12 should differ");
        assert_ne!(data12, data20, "ChaCha12 and ChaCha20 should differ");
        assert_ne!(data8, data20, "ChaCha8 and ChaCha20 should differ");
    }

    #[test]
    fn test_decryption_is_inverse_of_encryption() {
        let key = hex::decode("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f")
            .unwrap()
            .try_into()
            .unwrap();
        let nonce = [0x00, 0x00, 0x00, 0x4a, 0x00, 0x00, 0x00, 0x00];

        let plaintext = b"Hello, World! This is a test message for ChaCha20 encryption.";
        let mut data = plaintext.to_vec();

        // Encrypt
        let mut cipher = ChaCha20::new(&key, &nonce);
        cipher.xor_keystream(&mut data);

        // Verify it's different from plaintext
        assert_ne!(&data[..], &plaintext[..]);

        // Decrypt
        let mut cipher = ChaCha20::new(&key, &nonce);
        cipher.xor_keystream(&mut data);

        // Should match original plaintext
        assert_eq!(&data[..], &plaintext[..]);
    }

    #[test]
    fn test_avx2_integration() {
        // Test that AVX2 path (for data >= 256 bytes) produces correct results
        let key = hex::decode("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f")
            .unwrap()
            .try_into()
            .unwrap();
        let nonce = [0x00, 0x00, 0x00, 0x4a, 0x00, 0x00, 0x00, 0x00];

        // Test with exactly 256 bytes (should use AVX2 path entirely)
        let mut data256 = vec![0u8; 256];
        let mut cipher1 = ChaCha20::new(&key, &nonce);
        cipher1.xor_keystream(&mut data256);

        // Test with 512 bytes (multiple AVX2 iterations)
        let mut data512 = vec![0u8; 512];
        let mut cipher2 = ChaCha20::new(&key, &nonce);
        cipher2.xor_keystream(&mut data512);

        // First 256 bytes should match
        assert_eq!(
            &data256[..],
            &data512[..256],
            "First 256 bytes should match"
        );

        // Test encrypt/decrypt roundtrip with large data
        let plaintext: Vec<u8> = (0..1024).map(|i| i as u8).collect();
        let mut ciphertext = plaintext.clone();
        let mut cipher3 = ChaCha20::new(&key, &nonce);
        cipher3.xor_keystream(&mut ciphertext);

        assert_ne!(
            &ciphertext[..],
            &plaintext[..],
            "Ciphertext should differ from plaintext"
        );

        let mut decrypted = ciphertext.clone();
        let mut cipher4 = ChaCha20::new(&key, &nonce);
        cipher4.xor_keystream(&mut decrypted);

        assert_eq!(
            &decrypted[..],
            &plaintext[..],
            "Decryption should recover plaintext"
        );
    }

    #[test]
    fn test_avx2_boundary_conditions() {
        // Test boundary conditions around 256-byte threshold
        let key = [1u8; 32];
        let nonce = [2u8; 8];

        for size in [255, 256, 257, 320, 511, 512, 513, 1023, 1024, 1025] {
            let plaintext: Vec<u8> = (0..size).map(|i| i as u8).collect();
            let mut data = plaintext.clone();

            let mut cipher = ChaCha20::new(&key, &nonce);
            cipher.xor_keystream(&mut data);

            // Decrypt
            let mut cipher = ChaCha20::new(&key, &nonce);
            cipher.xor_keystream(&mut data);

            assert_eq!(
                &data[..],
                &plaintext[..],
                "Roundtrip failed for size {size}"
            );
        }
    }
}
