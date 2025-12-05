//! NEON-optimized ChaCha block function implementation for ARM64.
//!
//! This module provides a NEON-accelerated implementation of the ChaCha stream cipher
//! that processes 4 blocks in parallel using 128-bit SIMD registers.

use core::arch::aarch64::*;

use crate::AlignedU8;

/// how many ChaCha blocks we compute in parallel (depends on the size of the SIMD vectors, here 128 / 32 = 4)
pub const SIMD_LANES: usize = 4;

/// The number of 32-bit words that compose ChaCha's state.
const STATE_WORDS: usize = 16;

/// The size of a ChaCha block in bytes which is the size of the state in bytes
const BLOCK_SIZE: usize = 64;

/// Process 4 ChaCha blocks in parallel using NEON.
/// The state array represents the initial state with base counter.
/// Produces 4 keystream blocks (256 bytes total) for counter values:
/// base, base+1, base+2, base+3
#[cfg(target_arch = "aarch64")]
pub unsafe fn chacha_blocks_neon<const ROUNDS: usize>(
    state: &[u32; 16],
    keystream: &mut AlignedU8<256>,
) {
    let counter = (state[12] as u64) | ((state[13] as u64) << 32);

    let state_simd: [uint32x4_t; STATE_WORDS] = unsafe {
        [
            // constant
            vdupq_n_u32(state[0]),
            vdupq_n_u32(state[1]),
            vdupq_n_u32(state[2]),
            vdupq_n_u32(state[3]),
            // key
            vdupq_n_u32(state[4]),
            vdupq_n_u32(state[5]),
            vdupq_n_u32(state[6]),
            vdupq_n_u32(state[7]),
            vdupq_n_u32(state[8]),
            vdupq_n_u32(state[9]),
            vdupq_n_u32(state[10]),
            vdupq_n_u32(state[11]),
            // counter
            // Counter low: different for each block [0,1,2,3]
            vld1q_u32(
                [
                    (counter) as u32,
                    (counter + 1) as u32,
                    (counter + 2) as u32,
                    (counter + 3) as u32,
                ]
                .as_ptr(),
            ),
            // Counter high: different for each block if counter overflows 32-bit
            vld1q_u32(
                [
                    (counter >> 32) as u32,
                    ((counter + 1) >> 32) as u32,
                    ((counter + 2) >> 32) as u32,
                    ((counter + 3) >> 32) as u32,
                ]
                .as_ptr(),
            ),
            // nonce
            vdupq_n_u32(state[14]),
            vdupq_n_u32(state[15]),
        ]
    };

    chacha_neon_4blocks::<ROUNDS>(state_simd, keystream);
}

/// Compute 4 64-byte ChaCha blocks in parallel using NEON vectors.
#[inline(always)]
fn chacha_neon_4blocks<const ROUNDS: usize>(
    state: [uint32x4_t; STATE_WORDS],
    keystream: &mut AlignedU8<256>,
) {
    let keystream_ptr = keystream.0.as_mut_ptr();

    // the "working state" where we perform the ChaCha operations
    let mut working_state = state;

    for _ in 0..ROUNDS / 2 {
        // column rounds
        quarter_round(&mut working_state, 0, 4, 8, 12);
        quarter_round(&mut working_state, 1, 5, 9, 13);
        quarter_round(&mut working_state, 2, 6, 10, 14);
        quarter_round(&mut working_state, 3, 7, 11, 15);

        // diagonal rounds
        quarter_round(&mut working_state, 0, 5, 10, 15);
        quarter_round(&mut working_state, 1, 6, 11, 12);
        quarter_round(&mut working_state, 2, 7, 8, 13);
        quarter_round(&mut working_state, 3, 4, 9, 14);
    }

    // serialize the keystream as follow:
    // block1 || block2 || block3 || block4

    // Each iteration of the loop writes a 32-bit word for each block into keystream.
    // The first iteration writes block1[0], block2[0], block3[0], block4[0]
    // the second iterations writes block1[1], block2[1], block3[1], block4[1]
    // and so on, for the 16 32-bit words of the ChaCha state
    for word_index in 0..STATE_WORDS {
        // add working state to initial state to get the keystream
        let keystream_simd = unsafe { vaddq_u32(working_state[word_index], state[word_index]) };
        let mut lanes = [0u32; SIMD_LANES];
        unsafe { vst1q_u32(lanes.as_mut_ptr(), keystream_simd) };

        // TODO: there should be a faster way to directly XOR input with keystream SIMD here
        for block in 0..SIMD_LANES {
            // keystream[(block * STATE_WORDS) + word_index] = tmp[block].to_le();
            let byte_offset = (block * STATE_WORDS * 4) + (word_index * 4);
            unsafe {
                core::ptr::copy_nonoverlapping(
                    lanes[block].to_le_bytes().as_ptr(),
                    keystream_ptr.add(byte_offset),
                    4,
                );
            }
        }
    }
}

#[inline(always)]
fn quarter_round(state: &mut [uint32x4_t; STATE_WORDS], a: usize, b: usize, c: usize, d: usize) {
    // optimized rotate_left for NEON
    macro_rules! rotate_left {
        ($v:expr, 8) => {{
            let mask_bytes = [3u8, 0, 1, 2, 7, 4, 5, 6, 11, 8, 9, 10, 15, 12, 13, 14];
            let mask = vld1q_u8(mask_bytes.as_ptr());

            $v = vreinterpretq_u32_u8(vqtbl1q_u8(vreinterpretq_u8_u32($v), mask))
        }};
        ($v:expr, 16) => {
            $v = vreinterpretq_u32_u16(vrev32q_u16(vreinterpretq_u16_u32($v)))
        };
        ($v:expr, $r:literal) => {
            $v = vorrq_u32(vshlq_n_u32($v, $r), vshrq_n_u32($v, 32 - $r))
        };
    }

    unsafe {
        // a += b; d ^= a; d <<<= 16
        state[a] = vaddq_u32(state[a], state[b]);
        state[d] = veorq_u32(state[d], state[a]);
        // *d = vorrq_u32(vshlq_n_u32(*d, 16), vshrq_n_u32(*d, 16));
        rotate_left!(state[d], 16);

        // c += d; b ^= c; b <<<= 12
        state[c] = vaddq_u32(state[c], state[d]);
        state[b] = veorq_u32(state[b], state[c]);
        // *b = vorrq_u32(vshlq_n_u32(*b, 12), vshrq_n_u32(*b, 20));
        rotate_left!(state[b], 12);

        // a += b; d ^= a; d <<<= 8
        state[a] = vaddq_u32(state[a], state[b]);
        state[d] = veorq_u32(state[d], state[a]);
        // *d = vorrq_u32(vshlq_n_u32(*d, 8), vshrq_n_u32(*d, 24));
        rotate_left!(state[d], 8);

        // c += d; b ^= c; b <<<= 7
        state[c] = vaddq_u32(state[c], state[d]);
        state[b] = veorq_u32(state[b], state[c]);
        // *b = vorrq_u32(vshlq_n_u32(*b, 7), vshrq_n_u32(*b, 25));
        rotate_left!(state[b], 7);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Scalar helpers for testing
    #[inline(always)]
    fn scalar_quarter_round(state: &mut [u32; 16], a: usize, b: usize, c: usize, d: usize) {
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

    fn scalar_chacha_block<const ROUNDS: usize>(state: &[u32; 16]) -> [u32; 16] {
        let mut working_state = *state;
        for _ in 0..(ROUNDS / 2) {
            scalar_quarter_round(&mut working_state, 0, 4, 8, 12);
            scalar_quarter_round(&mut working_state, 1, 5, 9, 13);
            scalar_quarter_round(&mut working_state, 2, 6, 10, 14);
            scalar_quarter_round(&mut working_state, 3, 7, 11, 15);
            scalar_quarter_round(&mut working_state, 0, 5, 10, 15);
            scalar_quarter_round(&mut working_state, 1, 6, 11, 12);
            scalar_quarter_round(&mut working_state, 2, 7, 8, 13);
            scalar_quarter_round(&mut working_state, 3, 4, 9, 14);
        }
        working_state
            .iter_mut()
            .zip(state.iter())
            .for_each(|(ws, s)| *ws = ws.wrapping_add(*s));
        working_state
    }

    fn scalar_serialize_state(state: &[u32; 16], output: &mut [u8; 64]) {
        for (idx, word) in state.iter().enumerate() {
            output[idx * 4..idx * 4 + 4].copy_from_slice(&word.to_le_bytes());
        }
    }

    #[test]
    #[cfg(target_arch = "aarch64")]
    fn test_neon_produces_correct_output() {
        // Test that NEON implementation produces the same output as the scalar implementation
        let key = [0u8; 32];
        let nonce = [0u8; 8];

        // Create state
        let constants: [u32; 4] = [0x61707865, 0x3320646e, 0x79622d32, 0x6b206574];
        let mut state = [0u32; 16];
        state[0..4].copy_from_slice(&constants);
        for (idx, chunk) in key.chunks_exact(4).enumerate() {
            state[4 + idx] = u32::from_le_bytes(chunk.try_into().unwrap());
        }
        state[12] = 0;
        state[13] = 0;
        state[14] = u32::from_le_bytes(nonce[0..4].try_into().unwrap());
        state[15] = u32::from_le_bytes(nonce[4..8].try_into().unwrap());

        // Generate keystream using NEON
        let mut neon_output = AlignedU8([0u8; 256]);
        unsafe {
            chacha_blocks_neon::<20>(&state, &mut neon_output);
        }

        // Generate keystream using scalar implementation
        let mut scalar_output = [0u8; 64];
        let block = scalar_chacha_block::<20>(&state);
        scalar_serialize_state(&block, &mut scalar_output);

        // First block should match
        assert_eq!(
            &neon_output.0[0..64],
            &scalar_output[..],
            "First block mismatch:\nNEON:   {:02x?}\nScalar: {:02x?}",
            &neon_output.0[0..64],
            &scalar_output[..]
        );

        // Test second block (counter = 1)
        let mut state_c1 = state;
        state_c1[12] = 1;
        let block = scalar_chacha_block::<20>(&state_c1);
        scalar_serialize_state(&block, &mut scalar_output);
        assert_eq!(
            &neon_output.0[64..128],
            &scalar_output[..],
            "Second block mismatch:\nNEON:   {:02x?}\nScalar: {:02x?}",
            &neon_output.0[64..128],
            &scalar_output[..]
        );

        // Test third block (counter = 2)
        let mut state_c2 = state;
        state_c2[12] = 2;
        let block = scalar_chacha_block::<20>(&state_c2);
        scalar_serialize_state(&block, &mut scalar_output);
        assert_eq!(
            &neon_output.0[128..192],
            &scalar_output[..],
            "Third block mismatch"
        );

        // Test fourth block (counter = 3)
        let mut state_c3 = state;
        state_c3[12] = 3;
        let block = scalar_chacha_block::<20>(&state_c3);
        scalar_serialize_state(&block, &mut scalar_output);
        assert_eq!(
            &neon_output.0[192..256],
            &scalar_output[..],
            "Fourth block mismatch"
        );
    }

    #[test]
    #[cfg(target_arch = "aarch64")]
    fn test_neon_with_nonzero_key_and_nonce() {
        // Test with non-zero key and nonce
        let key: [u8; 32] = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b,
            0x1c, 0x1d, 0x1e, 0x1f,
        ];
        let nonce: [u8; 8] = [0x00, 0x00, 0x00, 0x4a, 0x00, 0x00, 0x00, 0x00];

        let constants: [u32; 4] = [0x61707865, 0x3320646e, 0x79622d32, 0x6b206574];
        let mut state = [0u32; 16];
        state[0..4].copy_from_slice(&constants);
        for (idx, chunk) in key.chunks_exact(4).enumerate() {
            state[4 + idx] = u32::from_le_bytes(chunk.try_into().unwrap());
        }
        state[12] = 1; // counter = 1
        state[13] = 0;
        state[14] = u32::from_le_bytes([nonce[0], nonce[1], nonce[2], nonce[3]]);
        state[15] = u32::from_le_bytes([nonce[4], nonce[5], nonce[6], nonce[7]]);

        let mut neon_output = AlignedU8([0u8; 256]);
        unsafe {
            chacha_blocks_neon::<20>(&state, &mut neon_output);
        }

        // Verify all 4 blocks
        for block_idx in 0..4u32 {
            let mut s = state;
            s[12] = 1 + block_idx;
            let block = scalar_chacha_block::<20>(&s);
            let mut scalar_output = [0u8; 64];
            scalar_serialize_state(&block, &mut scalar_output);
            assert_eq!(
                &neon_output.0[block_idx as usize * 64..(block_idx as usize + 1) * 64],
                &scalar_output[..],
                "Block {} mismatch",
                block_idx
            );
        }
    }

    #[test]
    #[cfg(target_arch = "aarch64")]
    fn test_neon_chacha8_and_chacha12() {
        let key = [1u8; 32];
        let nonce = [2u8; 8];

        let constants: [u32; 4] = [0x61707865, 0x3320646e, 0x79622d32, 0x6b206574];
        let mut state = [0u32; 16];
        state[0..4].copy_from_slice(&constants);
        for (idx, chunk) in key.chunks_exact(4).enumerate() {
            state[4 + idx] = u32::from_le_bytes(chunk.try_into().unwrap());
        }
        state[12] = 0;
        state[13] = 0;
        state[14] = u32::from_le_bytes([nonce[0], nonce[1], nonce[2], nonce[3]]);
        state[15] = u32::from_le_bytes([nonce[4], nonce[5], nonce[6], nonce[7]]);

        // Test ChaCha8
        let mut neon_output_8 = AlignedU8([0u8; 256]);
        unsafe {
            chacha_blocks_neon::<8>(&state, &mut neon_output_8);
        }
        let block_8 = scalar_chacha_block::<8>(&state);
        let mut scalar_output_8 = [0u8; 64];
        scalar_serialize_state(&block_8, &mut scalar_output_8);
        assert_eq!(
            &neon_output_8.0[0..64],
            &scalar_output_8[..],
            "ChaCha8 first block mismatch"
        );

        // Test ChaCha12
        let mut neon_output_12 = AlignedU8([0u8; 256]);
        unsafe {
            chacha_blocks_neon::<12>(&state, &mut neon_output_12);
        }
        let block_12 = scalar_chacha_block::<12>(&state);
        let mut scalar_output_12 = [0u8; 64];
        scalar_serialize_state(&block_12, &mut scalar_output_12);
        assert_eq!(
            &neon_output_12.0[0..64],
            &scalar_output_12[..],
            "ChaCha12 first block mismatch"
        );

        // Verify that different round counts produce different outputs
        assert_ne!(
            &neon_output_8.0[0..64],
            &neon_output_12.0[0..64],
            "ChaCha8 and ChaCha12 should produce different outputs"
        );
    }

    #[test]
    #[cfg(target_arch = "aarch64")]
    fn test_neon_counter_wrap() {
        // Test with counter that wraps around 32-bit boundary
        let key = [0u8; 32];
        let nonce = [0u8; 8];

        let constants: [u32; 4] = [0x61707865, 0x3320646e, 0x79622d32, 0x6b206574];
        let mut state = [0u32; 16];
        state[0..4].copy_from_slice(&constants);
        for (idx, chunk) in key.chunks_exact(4).enumerate() {
            state[4 + idx] = u32::from_le_bytes(chunk.try_into().unwrap());
        }
        // Set counter to just before 32-bit wrap
        state[12] = 0xFFFFFFFF;
        state[13] = 0;
        state[14] = u32::from_le_bytes(nonce[0..4].try_into().unwrap());
        state[15] = u32::from_le_bytes(nonce[4..8].try_into().unwrap());

        let mut neon_output = AlignedU8([0u8; 256]);
        unsafe {
            chacha_blocks_neon::<20>(&state, &mut neon_output);
        }

        // Verify all 4 blocks (counters 0xFFFFFFFF, 0x100000000, 0x100000001, 0x100000002)
        for block_idx in 0..4u64 {
            let counter = 0xFFFFFFFFu64 + block_idx;
            let mut s = state;
            s[12] = counter as u32;
            s[13] = (counter >> 32) as u32;
            let block = scalar_chacha_block::<20>(&s);
            let mut scalar_output = [0u8; 64];
            scalar_serialize_state(&block, &mut scalar_output);
            assert_eq!(
                &neon_output.0[block_idx as usize * 64..(block_idx as usize + 1) * 64],
                &scalar_output[..],
                "Block {} mismatch (counter = 0x{:x})",
                block_idx,
                counter
            );
        }
    }
}
