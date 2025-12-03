//! NEON-optimized ChaCha block function implementation for ARM64.
//!
//! This module provides a NEON-accelerated implementation of the ChaCha stream cipher
//! that processes 4 blocks in parallel using 128-bit SIMD registers.

#[cfg(target_arch = "aarch64")]
use core::arch::aarch64::*;

/// Process 4 ChaCha blocks in parallel using NEON.
/// The state array represents the initial state with base counter.
/// Produces 4 keystream blocks (256 bytes total) for counter values:
/// base, base+1, base+2, base+3
#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
pub unsafe fn chacha_blocks_neon<const ROUNDS: usize>(state: &[u32; 16], output: &mut [u8; 256]) {
    let counter_base = (state[12] as u64) | ((state[13] as u64) << 32);

    // Load state into 16 uint32x4_t registers for 4-way parallel processing
    // Each register v[i] holds the same state word from 4 parallel blocks
    // Layout: [block0, block1, block2, block3]
    let mut v0 = vdupq_n_u32(state[0]);
    let mut v1 = vdupq_n_u32(state[1]);
    let mut v2 = vdupq_n_u32(state[2]);
    let mut v3 = vdupq_n_u32(state[3]);
    let mut v4 = vdupq_n_u32(state[4]);
    let mut v5 = vdupq_n_u32(state[5]);
    let mut v6 = vdupq_n_u32(state[6]);
    let mut v7 = vdupq_n_u32(state[7]);
    let mut v8 = vdupq_n_u32(state[8]);
    let mut v9 = vdupq_n_u32(state[9]);
    let mut v10 = vdupq_n_u32(state[10]);
    let mut v11 = vdupq_n_u32(state[11]);

    // Counter low: different for each block [0,1,2,3]
    let counter_vals: [u32; 4] = [
        (counter_base) as u32,
        (counter_base + 1) as u32,
        (counter_base + 2) as u32,
        (counter_base + 3) as u32,
    ];
    let mut v12 = vld1q_u32(counter_vals.as_ptr());

    // Counter high: different for each block if counter overflows 32-bit
    let counter_hi_vals: [u32; 4] = [
        (counter_base >> 32) as u32,
        ((counter_base + 1) >> 32) as u32,
        ((counter_base + 2) >> 32) as u32,
        ((counter_base + 3) >> 32) as u32,
    ];
    let mut v13 = vld1q_u32(counter_hi_vals.as_ptr());

    let mut v14 = vdupq_n_u32(state[14]);
    let mut v15 = vdupq_n_u32(state[15]);

    // Save original state
    let orig0 = v0;
    let orig1 = v1;
    let orig2 = v2;
    let orig3 = v3;
    let orig4 = v4;
    let orig5 = v5;
    let orig6 = v6;
    let orig7 = v7;
    let orig8 = v8;
    let orig9 = v9;
    let orig10 = v10;
    let orig11 = v11;
    let orig12 = v12;
    let orig13 = v13;
    let orig14 = v14;
    let orig15 = v15;

    // Perform ROUNDS/2 double rounds
    for _ in 0..(ROUNDS / 2) {
        // Column rounds: (0,4,8,12), (1,5,9,13), (2,6,10,14), (3,7,11,15)

        // Quarter round on (v0, v4, v8, v12)
        v0 = vaddq_u32(v0, v4);
        v12 = veorq_u32(v12, v0);
        v12 = vorrq_u32(vshlq_n_u32(v12, 16), vshrq_n_u32(v12, 16));
        v8 = vaddq_u32(v8, v12);
        v4 = veorq_u32(v4, v8);
        v4 = vorrq_u32(vshlq_n_u32(v4, 12), vshrq_n_u32(v4, 20));
        v0 = vaddq_u32(v0, v4);
        v12 = veorq_u32(v12, v0);
        v12 = vorrq_u32(vshlq_n_u32(v12, 8), vshrq_n_u32(v12, 24));
        v8 = vaddq_u32(v8, v12);
        v4 = veorq_u32(v4, v8);
        v4 = vorrq_u32(vshlq_n_u32(v4, 7), vshrq_n_u32(v4, 25));

        // Quarter round on (v1, v5, v9, v13)
        v1 = vaddq_u32(v1, v5);
        v13 = veorq_u32(v13, v1);
        v13 = vorrq_u32(vshlq_n_u32(v13, 16), vshrq_n_u32(v13, 16));
        v9 = vaddq_u32(v9, v13);
        v5 = veorq_u32(v5, v9);
        v5 = vorrq_u32(vshlq_n_u32(v5, 12), vshrq_n_u32(v5, 20));
        v1 = vaddq_u32(v1, v5);
        v13 = veorq_u32(v13, v1);
        v13 = vorrq_u32(vshlq_n_u32(v13, 8), vshrq_n_u32(v13, 24));
        v9 = vaddq_u32(v9, v13);
        v5 = veorq_u32(v5, v9);
        v5 = vorrq_u32(vshlq_n_u32(v5, 7), vshrq_n_u32(v5, 25));

        // Quarter round on (v2, v6, v10, v14)
        v2 = vaddq_u32(v2, v6);
        v14 = veorq_u32(v14, v2);
        v14 = vorrq_u32(vshlq_n_u32(v14, 16), vshrq_n_u32(v14, 16));
        v10 = vaddq_u32(v10, v14);
        v6 = veorq_u32(v6, v10);
        v6 = vorrq_u32(vshlq_n_u32(v6, 12), vshrq_n_u32(v6, 20));
        v2 = vaddq_u32(v2, v6);
        v14 = veorq_u32(v14, v2);
        v14 = vorrq_u32(vshlq_n_u32(v14, 8), vshrq_n_u32(v14, 24));
        v10 = vaddq_u32(v10, v14);
        v6 = veorq_u32(v6, v10);
        v6 = vorrq_u32(vshlq_n_u32(v6, 7), vshrq_n_u32(v6, 25));

        // Quarter round on (v3, v7, v11, v15)
        v3 = vaddq_u32(v3, v7);
        v15 = veorq_u32(v15, v3);
        v15 = vorrq_u32(vshlq_n_u32(v15, 16), vshrq_n_u32(v15, 16));
        v11 = vaddq_u32(v11, v15);
        v7 = veorq_u32(v7, v11);
        v7 = vorrq_u32(vshlq_n_u32(v7, 12), vshrq_n_u32(v7, 20));
        v3 = vaddq_u32(v3, v7);
        v15 = veorq_u32(v15, v3);
        v15 = vorrq_u32(vshlq_n_u32(v15, 8), vshrq_n_u32(v15, 24));
        v11 = vaddq_u32(v11, v15);
        v7 = veorq_u32(v7, v11);
        v7 = vorrq_u32(vshlq_n_u32(v7, 7), vshrq_n_u32(v7, 25));

        // Diagonal rounds: (0,5,10,15), (1,6,11,12), (2,7,8,13), (3,4,9,14)

        // Quarter round on (v0, v5, v10, v15)
        v0 = vaddq_u32(v0, v5);
        v15 = veorq_u32(v15, v0);
        v15 = vorrq_u32(vshlq_n_u32(v15, 16), vshrq_n_u32(v15, 16));
        v10 = vaddq_u32(v10, v15);
        v5 = veorq_u32(v5, v10);
        v5 = vorrq_u32(vshlq_n_u32(v5, 12), vshrq_n_u32(v5, 20));
        v0 = vaddq_u32(v0, v5);
        v15 = veorq_u32(v15, v0);
        v15 = vorrq_u32(vshlq_n_u32(v15, 8), vshrq_n_u32(v15, 24));
        v10 = vaddq_u32(v10, v15);
        v5 = veorq_u32(v5, v10);
        v5 = vorrq_u32(vshlq_n_u32(v5, 7), vshrq_n_u32(v5, 25));

        // Quarter round on (v1, v6, v11, v12)
        v1 = vaddq_u32(v1, v6);
        v12 = veorq_u32(v12, v1);
        v12 = vorrq_u32(vshlq_n_u32(v12, 16), vshrq_n_u32(v12, 16));
        v11 = vaddq_u32(v11, v12);
        v6 = veorq_u32(v6, v11);
        v6 = vorrq_u32(vshlq_n_u32(v6, 12), vshrq_n_u32(v6, 20));
        v1 = vaddq_u32(v1, v6);
        v12 = veorq_u32(v12, v1);
        v12 = vorrq_u32(vshlq_n_u32(v12, 8), vshrq_n_u32(v12, 24));
        v11 = vaddq_u32(v11, v12);
        v6 = veorq_u32(v6, v11);
        v6 = vorrq_u32(vshlq_n_u32(v6, 7), vshrq_n_u32(v6, 25));

        // Quarter round on (v2, v7, v8, v13)
        v2 = vaddq_u32(v2, v7);
        v13 = veorq_u32(v13, v2);
        v13 = vorrq_u32(vshlq_n_u32(v13, 16), vshrq_n_u32(v13, 16));
        v8 = vaddq_u32(v8, v13);
        v7 = veorq_u32(v7, v8);
        v7 = vorrq_u32(vshlq_n_u32(v7, 12), vshrq_n_u32(v7, 20));
        v2 = vaddq_u32(v2, v7);
        v13 = veorq_u32(v13, v2);
        v13 = vorrq_u32(vshlq_n_u32(v13, 8), vshrq_n_u32(v13, 24));
        v8 = vaddq_u32(v8, v13);
        v7 = veorq_u32(v7, v8);
        v7 = vorrq_u32(vshlq_n_u32(v7, 7), vshrq_n_u32(v7, 25));

        // Quarter round on (v3, v4, v9, v14)
        v3 = vaddq_u32(v3, v4);
        v14 = veorq_u32(v14, v3);
        v14 = vorrq_u32(vshlq_n_u32(v14, 16), vshrq_n_u32(v14, 16));
        v9 = vaddq_u32(v9, v14);
        v4 = veorq_u32(v4, v9);
        v4 = vorrq_u32(vshlq_n_u32(v4, 12), vshrq_n_u32(v4, 20));
        v3 = vaddq_u32(v3, v4);
        v14 = veorq_u32(v14, v3);
        v14 = vorrq_u32(vshlq_n_u32(v14, 8), vshrq_n_u32(v14, 24));
        v9 = vaddq_u32(v9, v14);
        v4 = veorq_u32(v4, v9);
        v4 = vorrq_u32(vshlq_n_u32(v4, 7), vshrq_n_u32(v4, 25));
    }

    // Add original state
    v0 = vaddq_u32(v0, orig0);
    v1 = vaddq_u32(v1, orig1);
    v2 = vaddq_u32(v2, orig2);
    v3 = vaddq_u32(v3, orig3);
    v4 = vaddq_u32(v4, orig4);
    v5 = vaddq_u32(v5, orig5);
    v6 = vaddq_u32(v6, orig6);
    v7 = vaddq_u32(v7, orig7);
    v8 = vaddq_u32(v8, orig8);
    v9 = vaddq_u32(v9, orig9);
    v10 = vaddq_u32(v10, orig10);
    v11 = vaddq_u32(v11, orig11);
    v12 = vaddq_u32(v12, orig12);
    v13 = vaddq_u32(v13, orig13);
    v14 = vaddq_u32(v14, orig14);
    v15 = vaddq_u32(v15, orig15);

    // Transpose from row-major (state words) to column-major (blocks)
    // v0 = [s0b0, s0b1, s0b2, s0b3]
    // We need: block0 = [s0b0, s1b0, s2b0, ..., s15b0]

    // Use NEON 4x4 matrix transpose
    // Transpose groups of 4 rows at a time

    // Group 1: v0, v1, v2, v3 -> rows 0-3 of each block
    let t0 = vzip1q_u32(v0, v1); // [s0b0, s1b0, s0b1, s1b1]
    let t1 = vzip2q_u32(v0, v1); // [s0b2, s1b2, s0b3, s1b3]
    let t2 = vzip1q_u32(v2, v3); // [s2b0, s3b0, s2b1, s3b1]
    let t3 = vzip2q_u32(v2, v3); // [s2b2, s3b2, s2b3, s3b3]

    // Reinterpret as u64 for final interleave
    let t0_64 = vreinterpretq_u64_u32(t0);
    let t1_64 = vreinterpretq_u64_u32(t1);
    let t2_64 = vreinterpretq_u64_u32(t2);
    let t3_64 = vreinterpretq_u64_u32(t3);

    let r0_03 = vreinterpretq_u32_u64(vzip1q_u64(t0_64, t2_64)); // block0 words 0-3
    let r1_03 = vreinterpretq_u32_u64(vzip2q_u64(t0_64, t2_64)); // block1 words 0-3
    let r2_03 = vreinterpretq_u32_u64(vzip1q_u64(t1_64, t3_64)); // block2 words 0-3
    let r3_03 = vreinterpretq_u32_u64(vzip2q_u64(t1_64, t3_64)); // block3 words 0-3

    // Group 2: v4, v5, v6, v7 -> rows 4-7 of each block
    let t4 = vzip1q_u32(v4, v5);
    let t5 = vzip2q_u32(v4, v5);
    let t6 = vzip1q_u32(v6, v7);
    let t7 = vzip2q_u32(v6, v7);

    let t4_64 = vreinterpretq_u64_u32(t4);
    let t5_64 = vreinterpretq_u64_u32(t5);
    let t6_64 = vreinterpretq_u64_u32(t6);
    let t7_64 = vreinterpretq_u64_u32(t7);

    let r0_47 = vreinterpretq_u32_u64(vzip1q_u64(t4_64, t6_64));
    let r1_47 = vreinterpretq_u32_u64(vzip2q_u64(t4_64, t6_64));
    let r2_47 = vreinterpretq_u32_u64(vzip1q_u64(t5_64, t7_64));
    let r3_47 = vreinterpretq_u32_u64(vzip2q_u64(t5_64, t7_64));

    // Group 3: v8, v9, v10, v11 -> rows 8-11 of each block
    let t8 = vzip1q_u32(v8, v9);
    let t9 = vzip2q_u32(v8, v9);
    let t10 = vzip1q_u32(v10, v11);
    let t11 = vzip2q_u32(v10, v11);

    let t8_64 = vreinterpretq_u64_u32(t8);
    let t9_64 = vreinterpretq_u64_u32(t9);
    let t10_64 = vreinterpretq_u64_u32(t10);
    let t11_64 = vreinterpretq_u64_u32(t11);

    let r0_811 = vreinterpretq_u32_u64(vzip1q_u64(t8_64, t10_64));
    let r1_811 = vreinterpretq_u32_u64(vzip2q_u64(t8_64, t10_64));
    let r2_811 = vreinterpretq_u32_u64(vzip1q_u64(t9_64, t11_64));
    let r3_811 = vreinterpretq_u32_u64(vzip2q_u64(t9_64, t11_64));

    // Group 4: v12, v13, v14, v15 -> rows 12-15 of each block
    let t12 = vzip1q_u32(v12, v13);
    let t13 = vzip2q_u32(v12, v13);
    let t14 = vzip1q_u32(v14, v15);
    let t15 = vzip2q_u32(v14, v15);

    let t12_64 = vreinterpretq_u64_u32(t12);
    let t13_64 = vreinterpretq_u64_u32(t13);
    let t14_64 = vreinterpretq_u64_u32(t14);
    let t15_64 = vreinterpretq_u64_u32(t15);

    let r0_1215 = vreinterpretq_u32_u64(vzip1q_u64(t12_64, t14_64));
    let r1_1215 = vreinterpretq_u32_u64(vzip2q_u64(t12_64, t14_64));
    let r2_1215 = vreinterpretq_u32_u64(vzip1q_u64(t13_64, t15_64));
    let r3_1215 = vreinterpretq_u32_u64(vzip2q_u64(t13_64, t15_64));

    // Store block 0
    vst1q_u32(output.as_mut_ptr().add(0) as *mut u32, r0_03);
    vst1q_u32(output.as_mut_ptr().add(16) as *mut u32, r0_47);
    vst1q_u32(output.as_mut_ptr().add(32) as *mut u32, r0_811);
    vst1q_u32(output.as_mut_ptr().add(48) as *mut u32, r0_1215);

    // Store block 1
    vst1q_u32(output.as_mut_ptr().add(64) as *mut u32, r1_03);
    vst1q_u32(output.as_mut_ptr().add(80) as *mut u32, r1_47);
    vst1q_u32(output.as_mut_ptr().add(96) as *mut u32, r1_811);
    vst1q_u32(output.as_mut_ptr().add(112) as *mut u32, r1_1215);

    // Store block 2
    vst1q_u32(output.as_mut_ptr().add(128) as *mut u32, r2_03);
    vst1q_u32(output.as_mut_ptr().add(144) as *mut u32, r2_47);
    vst1q_u32(output.as_mut_ptr().add(160) as *mut u32, r2_811);
    vst1q_u32(output.as_mut_ptr().add(176) as *mut u32, r2_1215);

    // Store block 3
    vst1q_u32(output.as_mut_ptr().add(192) as *mut u32, r3_03);
    vst1q_u32(output.as_mut_ptr().add(208) as *mut u32, r3_47);
    vst1q_u32(output.as_mut_ptr().add(224) as *mut u32, r3_811);
    vst1q_u32(output.as_mut_ptr().add(240) as *mut u32, r3_1215);
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
        let mut neon_output = [0u8; 256];
        unsafe {
            chacha_blocks_neon::<20>(&state, &mut neon_output);
        }

        // Generate keystream using scalar implementation
        let mut scalar_output = [0u8; 64];
        let block = scalar_chacha_block::<20>(&state);
        scalar_serialize_state(&block, &mut scalar_output);

        // First block should match
        assert_eq!(
            &neon_output[0..64],
            &scalar_output[..],
            "First block mismatch:\nNEON:   {:02x?}\nScalar: {:02x?}",
            &neon_output[0..64],
            &scalar_output[..]
        );

        // Test second block (counter = 1)
        let mut state_c1 = state;
        state_c1[12] = 1;
        let block = scalar_chacha_block::<20>(&state_c1);
        scalar_serialize_state(&block, &mut scalar_output);
        assert_eq!(
            &neon_output[64..128],
            &scalar_output[..],
            "Second block mismatch:\nNEON:   {:02x?}\nScalar: {:02x?}",
            &neon_output[64..128],
            &scalar_output[..]
        );

        // Test third block (counter = 2)
        let mut state_c2 = state;
        state_c2[12] = 2;
        let block = scalar_chacha_block::<20>(&state_c2);
        scalar_serialize_state(&block, &mut scalar_output);
        assert_eq!(
            &neon_output[128..192],
            &scalar_output[..],
            "Third block mismatch"
        );

        // Test fourth block (counter = 3)
        let mut state_c3 = state;
        state_c3[12] = 3;
        let block = scalar_chacha_block::<20>(&state_c3);
        scalar_serialize_state(&block, &mut scalar_output);
        assert_eq!(
            &neon_output[192..256],
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

        let mut neon_output = [0u8; 256];
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
                &neon_output[block_idx as usize * 64..(block_idx as usize + 1) * 64],
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
        let mut neon_output_8 = [0u8; 256];
        unsafe {
            chacha_blocks_neon::<8>(&state, &mut neon_output_8);
        }
        let block_8 = scalar_chacha_block::<8>(&state);
        let mut scalar_output_8 = [0u8; 64];
        scalar_serialize_state(&block_8, &mut scalar_output_8);
        assert_eq!(
            &neon_output_8[0..64],
            &scalar_output_8[..],
            "ChaCha8 first block mismatch"
        );

        // Test ChaCha12
        let mut neon_output_12 = [0u8; 256];
        unsafe {
            chacha_blocks_neon::<12>(&state, &mut neon_output_12);
        }
        let block_12 = scalar_chacha_block::<12>(&state);
        let mut scalar_output_12 = [0u8; 64];
        scalar_serialize_state(&block_12, &mut scalar_output_12);
        assert_eq!(
            &neon_output_12[0..64],
            &scalar_output_12[..],
            "ChaCha12 first block mismatch"
        );

        // Verify that different round counts produce different outputs
        assert_ne!(
            &neon_output_8[0..64],
            &neon_output_12[0..64],
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

        let mut neon_output = [0u8; 256];
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
                &neon_output[block_idx as usize * 64..(block_idx as usize + 1) * 64],
                &scalar_output[..],
                "Block {} mismatch (counter = 0x{:x})",
                block_idx,
                counter
            );
        }
    }
}
