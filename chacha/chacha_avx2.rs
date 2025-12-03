//! AVX2-optimized ChaCha block function implementation.
//!
//! This module provides an AVX2-accelerated implementation of the ChaCha stream cipher
//! that processes 4 blocks in parallel using 256-bit SIMD registers.

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

/// Process 4 ChaCha blocks in parallel using AVX2.
/// The state array represents the initial state with base counter.
/// Produces 4 keystream blocks (256 bytes total) for counter values:
/// base, base+1, base+2, base+3
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
pub unsafe fn chacha_blocks_avx2<const ROUNDS: usize>(state: &[u32; 16], output: &mut [u8; 256]) {
    let counter_base = (state[12] as u64) | ((state[13] as u64) << 32);

    // Load state into 16 __m128i registers for 4-way parallel processing
    // Each register v[i] holds the same state word from 4 parallel blocks
    // Layout: [block0, block1, block2, block3]
    let mut v0 = _mm_set1_epi32(state[0] as i32);
    let mut v1 = _mm_set1_epi32(state[1] as i32);
    let mut v2 = _mm_set1_epi32(state[2] as i32);
    let mut v3 = _mm_set1_epi32(state[3] as i32);
    let mut v4 = _mm_set1_epi32(state[4] as i32);
    let mut v5 = _mm_set1_epi32(state[5] as i32);
    let mut v6 = _mm_set1_epi32(state[6] as i32);
    let mut v7 = _mm_set1_epi32(state[7] as i32);
    let mut v8 = _mm_set1_epi32(state[8] as i32);
    let mut v9 = _mm_set1_epi32(state[9] as i32);
    let mut v10 = _mm_set1_epi32(state[10] as i32);
    let mut v11 = _mm_set1_epi32(state[11] as i32);

    // Counter low: different for each block [0,1,2,3]
    let mut v12 = _mm_setr_epi32(
        (counter_base) as i32,
        (counter_base + 1) as i32,
        (counter_base + 2) as i32,
        (counter_base + 3) as i32,
    );

    // Counter high
    let mut v13 = _mm_setr_epi32(
        (counter_base >> 32) as i32,
        ((counter_base + 1) >> 32) as i32,
        ((counter_base + 2) >> 32) as i32,
        ((counter_base + 3) >> 32) as i32,
    );

    let mut v14 = _mm_set1_epi32(state[14] as i32);
    let mut v15 = _mm_set1_epi32(state[15] as i32);

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
        v0 = _mm_add_epi32(v0, v4);
        v12 = _mm_xor_si128(v12, v0);
        v12 = _mm_or_si128(_mm_slli_epi32(v12, 16), _mm_srli_epi32(v12, 16));
        v8 = _mm_add_epi32(v8, v12);
        v4 = _mm_xor_si128(v4, v8);
        v4 = _mm_or_si128(_mm_slli_epi32(v4, 12), _mm_srli_epi32(v4, 20));
        v0 = _mm_add_epi32(v0, v4);
        v12 = _mm_xor_si128(v12, v0);
        v12 = _mm_or_si128(_mm_slli_epi32(v12, 8), _mm_srli_epi32(v12, 24));
        v8 = _mm_add_epi32(v8, v12);
        v4 = _mm_xor_si128(v4, v8);
        v4 = _mm_or_si128(_mm_slli_epi32(v4, 7), _mm_srli_epi32(v4, 25));

        // Quarter round on (v1, v5, v9, v13)
        v1 = _mm_add_epi32(v1, v5);
        v13 = _mm_xor_si128(v13, v1);
        v13 = _mm_or_si128(_mm_slli_epi32(v13, 16), _mm_srli_epi32(v13, 16));
        v9 = _mm_add_epi32(v9, v13);
        v5 = _mm_xor_si128(v5, v9);
        v5 = _mm_or_si128(_mm_slli_epi32(v5, 12), _mm_srli_epi32(v5, 20));
        v1 = _mm_add_epi32(v1, v5);
        v13 = _mm_xor_si128(v13, v1);
        v13 = _mm_or_si128(_mm_slli_epi32(v13, 8), _mm_srli_epi32(v13, 24));
        v9 = _mm_add_epi32(v9, v13);
        v5 = _mm_xor_si128(v5, v9);
        v5 = _mm_or_si128(_mm_slli_epi32(v5, 7), _mm_srli_epi32(v5, 25));

        // Quarter round on (v2, v6, v10, v14)
        v2 = _mm_add_epi32(v2, v6);
        v14 = _mm_xor_si128(v14, v2);
        v14 = _mm_or_si128(_mm_slli_epi32(v14, 16), _mm_srli_epi32(v14, 16));
        v10 = _mm_add_epi32(v10, v14);
        v6 = _mm_xor_si128(v6, v10);
        v6 = _mm_or_si128(_mm_slli_epi32(v6, 12), _mm_srli_epi32(v6, 20));
        v2 = _mm_add_epi32(v2, v6);
        v14 = _mm_xor_si128(v14, v2);
        v14 = _mm_or_si128(_mm_slli_epi32(v14, 8), _mm_srli_epi32(v14, 24));
        v10 = _mm_add_epi32(v10, v14);
        v6 = _mm_xor_si128(v6, v10);
        v6 = _mm_or_si128(_mm_slli_epi32(v6, 7), _mm_srli_epi32(v6, 25));

        // Quarter round on (v3, v7, v11, v15)
        v3 = _mm_add_epi32(v3, v7);
        v15 = _mm_xor_si128(v15, v3);
        v15 = _mm_or_si128(_mm_slli_epi32(v15, 16), _mm_srli_epi32(v15, 16));
        v11 = _mm_add_epi32(v11, v15);
        v7 = _mm_xor_si128(v7, v11);
        v7 = _mm_or_si128(_mm_slli_epi32(v7, 12), _mm_srli_epi32(v7, 20));
        v3 = _mm_add_epi32(v3, v7);
        v15 = _mm_xor_si128(v15, v3);
        v15 = _mm_or_si128(_mm_slli_epi32(v15, 8), _mm_srli_epi32(v15, 24));
        v11 = _mm_add_epi32(v11, v15);
        v7 = _mm_xor_si128(v7, v11);
        v7 = _mm_or_si128(_mm_slli_epi32(v7, 7), _mm_srli_epi32(v7, 25));

        // Diagonal rounds: (0,5,10,15), (1,6,11,12), (2,7,8,13), (3,4,9,14)

        // Quarter round on (v0, v5, v10, v15)
        v0 = _mm_add_epi32(v0, v5);
        v15 = _mm_xor_si128(v15, v0);
        v15 = _mm_or_si128(_mm_slli_epi32(v15, 16), _mm_srli_epi32(v15, 16));
        v10 = _mm_add_epi32(v10, v15);
        v5 = _mm_xor_si128(v5, v10);
        v5 = _mm_or_si128(_mm_slli_epi32(v5, 12), _mm_srli_epi32(v5, 20));
        v0 = _mm_add_epi32(v0, v5);
        v15 = _mm_xor_si128(v15, v0);
        v15 = _mm_or_si128(_mm_slli_epi32(v15, 8), _mm_srli_epi32(v15, 24));
        v10 = _mm_add_epi32(v10, v15);
        v5 = _mm_xor_si128(v5, v10);
        v5 = _mm_or_si128(_mm_slli_epi32(v5, 7), _mm_srli_epi32(v5, 25));

        // Quarter round on (v1, v6, v11, v12)
        v1 = _mm_add_epi32(v1, v6);
        v12 = _mm_xor_si128(v12, v1);
        v12 = _mm_or_si128(_mm_slli_epi32(v12, 16), _mm_srli_epi32(v12, 16));
        v11 = _mm_add_epi32(v11, v12);
        v6 = _mm_xor_si128(v6, v11);
        v6 = _mm_or_si128(_mm_slli_epi32(v6, 12), _mm_srli_epi32(v6, 20));
        v1 = _mm_add_epi32(v1, v6);
        v12 = _mm_xor_si128(v12, v1);
        v12 = _mm_or_si128(_mm_slli_epi32(v12, 8), _mm_srli_epi32(v12, 24));
        v11 = _mm_add_epi32(v11, v12);
        v6 = _mm_xor_si128(v6, v11);
        v6 = _mm_or_si128(_mm_slli_epi32(v6, 7), _mm_srli_epi32(v6, 25));

        // Quarter round on (v2, v7, v8, v13)
        v2 = _mm_add_epi32(v2, v7);
        v13 = _mm_xor_si128(v13, v2);
        v13 = _mm_or_si128(_mm_slli_epi32(v13, 16), _mm_srli_epi32(v13, 16));
        v8 = _mm_add_epi32(v8, v13);
        v7 = _mm_xor_si128(v7, v8);
        v7 = _mm_or_si128(_mm_slli_epi32(v7, 12), _mm_srli_epi32(v7, 20));
        v2 = _mm_add_epi32(v2, v7);
        v13 = _mm_xor_si128(v13, v2);
        v13 = _mm_or_si128(_mm_slli_epi32(v13, 8), _mm_srli_epi32(v13, 24));
        v8 = _mm_add_epi32(v8, v13);
        v7 = _mm_xor_si128(v7, v8);
        v7 = _mm_or_si128(_mm_slli_epi32(v7, 7), _mm_srli_epi32(v7, 25));

        // Quarter round on (v3, v4, v9, v14)
        v3 = _mm_add_epi32(v3, v4);
        v14 = _mm_xor_si128(v14, v3);
        v14 = _mm_or_si128(_mm_slli_epi32(v14, 16), _mm_srli_epi32(v14, 16));
        v9 = _mm_add_epi32(v9, v14);
        v4 = _mm_xor_si128(v4, v9);
        v4 = _mm_or_si128(_mm_slli_epi32(v4, 12), _mm_srli_epi32(v4, 20));
        v3 = _mm_add_epi32(v3, v4);
        v14 = _mm_xor_si128(v14, v3);
        v14 = _mm_or_si128(_mm_slli_epi32(v14, 8), _mm_srli_epi32(v14, 24));
        v9 = _mm_add_epi32(v9, v14);
        v4 = _mm_xor_si128(v4, v9);
        v4 = _mm_or_si128(_mm_slli_epi32(v4, 7), _mm_srli_epi32(v4, 25));
    }

    // Add original state
    v0 = _mm_add_epi32(v0, orig0);
    v1 = _mm_add_epi32(v1, orig1);
    v2 = _mm_add_epi32(v2, orig2);
    v3 = _mm_add_epi32(v3, orig3);
    v4 = _mm_add_epi32(v4, orig4);
    v5 = _mm_add_epi32(v5, orig5);
    v6 = _mm_add_epi32(v6, orig6);
    v7 = _mm_add_epi32(v7, orig7);
    v8 = _mm_add_epi32(v8, orig8);
    v9 = _mm_add_epi32(v9, orig9);
    v10 = _mm_add_epi32(v10, orig10);
    v11 = _mm_add_epi32(v11, orig11);
    v12 = _mm_add_epi32(v12, orig12);
    v13 = _mm_add_epi32(v13, orig13);
    v14 = _mm_add_epi32(v14, orig14);
    v15 = _mm_add_epi32(v15, orig15);

    // Transpose from row-major (state words) to column-major (blocks)
    // v0 = [s0b0, s0b1, s0b2, s0b3]
    // We need: block0 = [s0b0, s1b0, s2b0, ..., s15b0]

    // Use SSE 4x4 matrix transpose
    // Transpose groups of 4 rows at a time

    // Group 1: v0, v1, v2, v3 -> rows 0-3 of each block
    let t0 = _mm_unpacklo_epi32(v0, v1); // [s0b0, s1b0, s0b1, s1b1]
    let t1 = _mm_unpackhi_epi32(v0, v1); // [s0b2, s1b2, s0b3, s1b3]
    let t2 = _mm_unpacklo_epi32(v2, v3); // [s2b0, s3b0, s2b1, s3b1]
    let t3 = _mm_unpackhi_epi32(v2, v3); // [s2b2, s3b2, s2b3, s3b3]

    let r0_03 = _mm_unpacklo_epi64(t0, t2); // [s0b0, s1b0, s2b0, s3b0] = block0 words 0-3
    let r1_03 = _mm_unpackhi_epi64(t0, t2); // [s0b1, s1b1, s2b1, s3b1] = block1 words 0-3
    let r2_03 = _mm_unpacklo_epi64(t1, t3); // [s0b2, s1b2, s2b2, s3b2] = block2 words 0-3
    let r3_03 = _mm_unpackhi_epi64(t1, t3); // [s0b3, s1b3, s2b3, s3b3] = block3 words 0-3

    // Group 2: v4, v5, v6, v7 -> rows 4-7 of each block
    let t4 = _mm_unpacklo_epi32(v4, v5);
    let t5 = _mm_unpackhi_epi32(v4, v5);
    let t6 = _mm_unpacklo_epi32(v6, v7);
    let t7 = _mm_unpackhi_epi32(v6, v7);

    let r0_47 = _mm_unpacklo_epi64(t4, t6);
    let r1_47 = _mm_unpackhi_epi64(t4, t6);
    let r2_47 = _mm_unpacklo_epi64(t5, t7);
    let r3_47 = _mm_unpackhi_epi64(t5, t7);

    // Group 3: v8, v9, v10, v11 -> rows 8-11 of each block
    let t8 = _mm_unpacklo_epi32(v8, v9);
    let t9 = _mm_unpackhi_epi32(v8, v9);
    let t10 = _mm_unpacklo_epi32(v10, v11);
    let t11 = _mm_unpackhi_epi32(v10, v11);

    let r0_811 = _mm_unpacklo_epi64(t8, t10);
    let r1_811 = _mm_unpackhi_epi64(t8, t10);
    let r2_811 = _mm_unpacklo_epi64(t9, t11);
    let r3_811 = _mm_unpackhi_epi64(t9, t11);

    // Group 4: v12, v13, v14, v15 -> rows 12-15 of each block
    let t12 = _mm_unpacklo_epi32(v12, v13);
    let t13 = _mm_unpackhi_epi32(v12, v13);
    let t14 = _mm_unpacklo_epi32(v14, v15);
    let t15 = _mm_unpackhi_epi32(v14, v15);

    let r0_1215 = _mm_unpacklo_epi64(t12, t14);
    let r1_1215 = _mm_unpackhi_epi64(t12, t14);
    let r2_1215 = _mm_unpacklo_epi64(t13, t15);
    let r3_1215 = _mm_unpackhi_epi64(t13, t15);

    // Store block 0
    _mm_store_si128(output.as_mut_ptr().add(0) as *mut __m128i, r0_03);
    _mm_store_si128(output.as_mut_ptr().add(16) as *mut __m128i, r0_47);
    _mm_store_si128(output.as_mut_ptr().add(32) as *mut __m128i, r0_811);
    _mm_store_si128(output.as_mut_ptr().add(48) as *mut __m128i, r0_1215);

    // Store block 1
    _mm_store_si128(output.as_mut_ptr().add(64) as *mut __m128i, r1_03);
    _mm_store_si128(output.as_mut_ptr().add(80) as *mut __m128i, r1_47);
    _mm_store_si128(output.as_mut_ptr().add(96) as *mut __m128i, r1_811);
    _mm_store_si128(output.as_mut_ptr().add(112) as *mut __m128i, r1_1215);

    // Store block 2
    _mm_store_si128(output.as_mut_ptr().add(128) as *mut __m128i, r2_03);
    _mm_store_si128(output.as_mut_ptr().add(144) as *mut __m128i, r2_47);
    _mm_store_si128(output.as_mut_ptr().add(160) as *mut __m128i, r2_811);
    _mm_store_si128(output.as_mut_ptr().add(176) as *mut __m128i, r2_1215);

    // Store block 3
    _mm_store_si128(output.as_mut_ptr().add(192) as *mut __m128i, r3_03);
    _mm_store_si128(output.as_mut_ptr().add(208) as *mut __m128i, r3_47);
    _mm_store_si128(output.as_mut_ptr().add(224) as *mut __m128i, r3_811);
    _mm_store_si128(output.as_mut_ptr().add(240) as *mut __m128i, r3_1215);
}


#[repr(align(8))]
pub struct AlignedU8x512(pub [u8; 512]);

/// Process 8 ChaCha blocks in parallel using full AVX2 256-bit registers.
/// The state array represents the initial state with base counter.
/// Produces 8 keystream blocks (512 bytes total) for counter values:
/// base, base+1, base+2, base+3, base+4, base+5, base+6, base+7
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
pub unsafe fn chacha_blocks_avx2_x8<const ROUNDS: usize>(
    state: &[u32; 16],
    output: &mut AlignedU8x512,
) {
    let counter_base = (state[12] as u64) | ((state[13] as u64) << 32);

    // Shuffle masks for byte rotations (more efficient than shift+or)
    // ROL16: rotate each 32-bit word left by 16 bits = swap pairs of bytes
    let rot16 = _mm256_setr_epi8(
        2, 3, 0, 1, 6, 7, 4, 5, 10, 11, 8, 9, 14, 15, 12, 13, 2, 3, 0, 1, 6, 7, 4, 5, 10, 11, 8, 9,
        14, 15, 12, 13,
    );
    // ROL8: rotate each 32-bit word left by 8 bits
    let rot8 = _mm256_setr_epi8(
        3, 0, 1, 2, 7, 4, 5, 6, 11, 8, 9, 10, 15, 12, 13, 14, 3, 0, 1, 2, 7, 4, 5, 6, 11, 8, 9, 10,
        15, 12, 13, 14,
    );

    // Load state into __m256i for 8-way parallel processing
    let mut v0 = _mm256_set1_epi32(state[0] as i32);
    let mut v1 = _mm256_set1_epi32(state[1] as i32);
    let mut v2 = _mm256_set1_epi32(state[2] as i32);
    let mut v3 = _mm256_set1_epi32(state[3] as i32);
    let mut v4 = _mm256_set1_epi32(state[4] as i32);
    let mut v5 = _mm256_set1_epi32(state[5] as i32);
    let mut v6 = _mm256_set1_epi32(state[6] as i32);
    let mut v7 = _mm256_set1_epi32(state[7] as i32);
    let mut v8 = _mm256_set1_epi32(state[8] as i32);
    let mut v9 = _mm256_set1_epi32(state[9] as i32);
    let mut v10 = _mm256_set1_epi32(state[10] as i32);
    let mut v11 = _mm256_set1_epi32(state[11] as i32);

    // Counters for 8 blocks [0..7]
    let mut v12 = _mm256_setr_epi32(
        (counter_base) as i32,
        (counter_base + 1) as i32,
        (counter_base + 2) as i32,
        (counter_base + 3) as i32,
        (counter_base + 4) as i32,
        (counter_base + 5) as i32,
        (counter_base + 6) as i32,
        (counter_base + 7) as i32,
    );
    let mut v13 = _mm256_setr_epi32(
        (counter_base >> 32) as i32,
        ((counter_base + 1) >> 32) as i32,
        ((counter_base + 2) >> 32) as i32,
        ((counter_base + 3) >> 32) as i32,
        ((counter_base + 4) >> 32) as i32,
        ((counter_base + 5) >> 32) as i32,
        ((counter_base + 6) >> 32) as i32,
        ((counter_base + 7) >> 32) as i32,
    );
    let mut v14 = _mm256_set1_epi32(state[14] as i32);
    let mut v15 = _mm256_set1_epi32(state[15] as i32);

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

    for _ in 0..(ROUNDS / 2) {
        // Column rounds - use shuffle for 8/16-bit rotations, shift+or for 7/12-bit
        v0 = _mm256_add_epi32(v0, v4);
        v12 = _mm256_xor_si256(v12, v0);
        v12 = _mm256_shuffle_epi8(v12, rot16);
        v8 = _mm256_add_epi32(v8, v12);
        v4 = _mm256_xor_si256(v4, v8);
        v4 = _mm256_or_si256(_mm256_slli_epi32(v4, 12), _mm256_srli_epi32(v4, 20));
        v0 = _mm256_add_epi32(v0, v4);
        v12 = _mm256_xor_si256(v12, v0);
        v12 = _mm256_shuffle_epi8(v12, rot8);
        v8 = _mm256_add_epi32(v8, v12);
        v4 = _mm256_xor_si256(v4, v8);
        v4 = _mm256_or_si256(_mm256_slli_epi32(v4, 7), _mm256_srli_epi32(v4, 25));

        v1 = _mm256_add_epi32(v1, v5);
        v13 = _mm256_xor_si256(v13, v1);
        v13 = _mm256_shuffle_epi8(v13, rot16);
        v9 = _mm256_add_epi32(v9, v13);
        v5 = _mm256_xor_si256(v5, v9);
        v5 = _mm256_or_si256(_mm256_slli_epi32(v5, 12), _mm256_srli_epi32(v5, 20));
        v1 = _mm256_add_epi32(v1, v5);
        v13 = _mm256_xor_si256(v13, v1);
        v13 = _mm256_shuffle_epi8(v13, rot8);
        v9 = _mm256_add_epi32(v9, v13);
        v5 = _mm256_xor_si256(v5, v9);
        v5 = _mm256_or_si256(_mm256_slli_epi32(v5, 7), _mm256_srli_epi32(v5, 25));

        v2 = _mm256_add_epi32(v2, v6);
        v14 = _mm256_xor_si256(v14, v2);
        v14 = _mm256_shuffle_epi8(v14, rot16);
        v10 = _mm256_add_epi32(v10, v14);
        v6 = _mm256_xor_si256(v6, v10);
        v6 = _mm256_or_si256(_mm256_slli_epi32(v6, 12), _mm256_srli_epi32(v6, 20));
        v2 = _mm256_add_epi32(v2, v6);
        v14 = _mm256_xor_si256(v14, v2);
        v14 = _mm256_shuffle_epi8(v14, rot8);
        v10 = _mm256_add_epi32(v10, v14);
        v6 = _mm256_xor_si256(v6, v10);
        v6 = _mm256_or_si256(_mm256_slli_epi32(v6, 7), _mm256_srli_epi32(v6, 25));

        v3 = _mm256_add_epi32(v3, v7);
        v15 = _mm256_xor_si256(v15, v3);
        v15 = _mm256_shuffle_epi8(v15, rot16);
        v11 = _mm256_add_epi32(v11, v15);
        v7 = _mm256_xor_si256(v7, v11);
        v7 = _mm256_or_si256(_mm256_slli_epi32(v7, 12), _mm256_srli_epi32(v7, 20));
        v3 = _mm256_add_epi32(v3, v7);
        v15 = _mm256_xor_si256(v15, v3);
        v15 = _mm256_shuffle_epi8(v15, rot8);
        v11 = _mm256_add_epi32(v11, v15);
        v7 = _mm256_xor_si256(v7, v11);
        v7 = _mm256_or_si256(_mm256_slli_epi32(v7, 7), _mm256_srli_epi32(v7, 25));

        // Diagonal rounds
        v0 = _mm256_add_epi32(v0, v5);
        v15 = _mm256_xor_si256(v15, v0);
        v15 = _mm256_shuffle_epi8(v15, rot16);
        v10 = _mm256_add_epi32(v10, v15);
        v5 = _mm256_xor_si256(v5, v10);
        v5 = _mm256_or_si256(_mm256_slli_epi32(v5, 12), _mm256_srli_epi32(v5, 20));
        v0 = _mm256_add_epi32(v0, v5);
        v15 = _mm256_xor_si256(v15, v0);
        v15 = _mm256_shuffle_epi8(v15, rot8);
        v10 = _mm256_add_epi32(v10, v15);
        v5 = _mm256_xor_si256(v5, v10);
        v5 = _mm256_or_si256(_mm256_slli_epi32(v5, 7), _mm256_srli_epi32(v5, 25));

        v1 = _mm256_add_epi32(v1, v6);
        v12 = _mm256_xor_si256(v12, v1);
        v12 = _mm256_shuffle_epi8(v12, rot16);
        v11 = _mm256_add_epi32(v11, v12);
        v6 = _mm256_xor_si256(v6, v11);
        v6 = _mm256_or_si256(_mm256_slli_epi32(v6, 12), _mm256_srli_epi32(v6, 20));
        v1 = _mm256_add_epi32(v1, v6);
        v12 = _mm256_xor_si256(v12, v1);
        v12 = _mm256_shuffle_epi8(v12, rot8);
        v11 = _mm256_add_epi32(v11, v12);
        v6 = _mm256_xor_si256(v6, v11);
        v6 = _mm256_or_si256(_mm256_slli_epi32(v6, 7), _mm256_srli_epi32(v6, 25));

        v2 = _mm256_add_epi32(v2, v7);
        v13 = _mm256_xor_si256(v13, v2);
        v13 = _mm256_shuffle_epi8(v13, rot16);
        v8 = _mm256_add_epi32(v8, v13);
        v7 = _mm256_xor_si256(v7, v8);
        v7 = _mm256_or_si256(_mm256_slli_epi32(v7, 12), _mm256_srli_epi32(v7, 20));
        v2 = _mm256_add_epi32(v2, v7);
        v13 = _mm256_xor_si256(v13, v2);
        v13 = _mm256_shuffle_epi8(v13, rot8);
        v8 = _mm256_add_epi32(v8, v13);
        v7 = _mm256_xor_si256(v7, v8);
        v7 = _mm256_or_si256(_mm256_slli_epi32(v7, 7), _mm256_srli_epi32(v7, 25));

        v3 = _mm256_add_epi32(v3, v4);
        v14 = _mm256_xor_si256(v14, v3);
        v14 = _mm256_shuffle_epi8(v14, rot16);
        v9 = _mm256_add_epi32(v9, v14);
        v4 = _mm256_xor_si256(v4, v9);
        v4 = _mm256_or_si256(_mm256_slli_epi32(v4, 12), _mm256_srli_epi32(v4, 20));
        v3 = _mm256_add_epi32(v3, v4);
        v14 = _mm256_xor_si256(v14, v3);
        v14 = _mm256_shuffle_epi8(v14, rot8);
        v9 = _mm256_add_epi32(v9, v14);
        v4 = _mm256_xor_si256(v4, v9);
        v4 = _mm256_or_si256(_mm256_slli_epi32(v4, 7), _mm256_srli_epi32(v4, 25));
    }

    // Add original state
    v0 = _mm256_add_epi32(v0, orig0);
    v1 = _mm256_add_epi32(v1, orig1);
    v2 = _mm256_add_epi32(v2, orig2);
    v3 = _mm256_add_epi32(v3, orig3);
    v4 = _mm256_add_epi32(v4, orig4);
    v5 = _mm256_add_epi32(v5, orig5);
    v6 = _mm256_add_epi32(v6, orig6);
    v7 = _mm256_add_epi32(v7, orig7);
    v8 = _mm256_add_epi32(v8, orig8);
    v9 = _mm256_add_epi32(v9, orig9);
    v10 = _mm256_add_epi32(v10, orig10);
    v11 = _mm256_add_epi32(v11, orig11);
    v12 = _mm256_add_epi32(v12, orig12);
    v13 = _mm256_add_epi32(v13, orig13);
    v14 = _mm256_add_epi32(v14, orig14);
    v15 = _mm256_add_epi32(v15, orig15);

    // Extract and transpose low 128-bit lanes (blocks 0-3)
    let v0_lo = _mm256_castsi256_si128(v0);
    let v0_hi = _mm256_extracti128_si256::<1>(v0);
    let v1_lo = _mm256_castsi256_si128(v1);
    let v1_hi = _mm256_extracti128_si256::<1>(v1);
    let v2_lo = _mm256_castsi256_si128(v2);
    let v2_hi = _mm256_extracti128_si256::<1>(v2);
    let v3_lo = _mm256_castsi256_si128(v3);
    let v3_hi = _mm256_extracti128_si256::<1>(v3);
    let v4_lo = _mm256_castsi256_si128(v4);
    let v4_hi = _mm256_extracti128_si256::<1>(v4);
    let v5_lo = _mm256_castsi256_si128(v5);
    let v5_hi = _mm256_extracti128_si256::<1>(v5);
    let v6_lo = _mm256_castsi256_si128(v6);
    let v6_hi = _mm256_extracti128_si256::<1>(v6);
    let v7_lo = _mm256_castsi256_si128(v7);
    let v7_hi = _mm256_extracti128_si256::<1>(v7);
    let v8_lo = _mm256_castsi256_si128(v8);
    let v8_hi = _mm256_extracti128_si256::<1>(v8);
    let v9_lo = _mm256_castsi256_si128(v9);
    let v9_hi = _mm256_extracti128_si256::<1>(v9);
    let v10_lo = _mm256_castsi256_si128(v10);
    let v10_hi = _mm256_extracti128_si256::<1>(v10);
    let v11_lo = _mm256_castsi256_si128(v11);
    let v11_hi = _mm256_extracti128_si256::<1>(v11);
    let v12_lo = _mm256_castsi256_si128(v12);
    let v12_hi = _mm256_extracti128_si256::<1>(v12);
    let v13_lo = _mm256_castsi256_si128(v13);
    let v13_hi = _mm256_extracti128_si256::<1>(v13);
    let v14_lo = _mm256_castsi256_si128(v14);
    let v14_hi = _mm256_extracti128_si256::<1>(v14);
    let v15_lo = _mm256_castsi256_si128(v15);
    let v15_hi = _mm256_extracti128_si256::<1>(v15);

    // Transpose blocks 0-3 from low halves
    let t0 = _mm_unpacklo_epi32(v0_lo, v1_lo);
    let t1 = _mm_unpackhi_epi32(v0_lo, v1_lo);
    let t2 = _mm_unpacklo_epi32(v2_lo, v3_lo);
    let t3 = _mm_unpackhi_epi32(v2_lo, v3_lo);
    let r0 = _mm_unpacklo_epi64(t0, t2);
    let r1 = _mm_unpackhi_epi64(t0, t2);
    let r2 = _mm_unpacklo_epi64(t1, t3);
    let r3 = _mm_unpackhi_epi64(t1, t3);
    let t4 = _mm_unpacklo_epi32(v4_lo, v5_lo);
    let t5 = _mm_unpackhi_epi32(v4_lo, v5_lo);
    let t6 = _mm_unpacklo_epi32(v6_lo, v7_lo);
    let t7 = _mm_unpackhi_epi32(v6_lo, v7_lo);
    let r4 = _mm_unpacklo_epi64(t4, t6);
    let r5 = _mm_unpackhi_epi64(t4, t6);
    let r6 = _mm_unpacklo_epi64(t5, t7);
    let r7 = _mm_unpackhi_epi64(t5, t7);
    let t8 = _mm_unpacklo_epi32(v8_lo, v9_lo);
    let t9 = _mm_unpackhi_epi32(v8_lo, v9_lo);
    let t10 = _mm_unpacklo_epi32(v10_lo, v11_lo);
    let t11 = _mm_unpackhi_epi32(v10_lo, v11_lo);
    let r8 = _mm_unpacklo_epi64(t8, t10);
    let r9 = _mm_unpackhi_epi64(t8, t10);
    let r10 = _mm_unpacklo_epi64(t9, t11);
    let r11 = _mm_unpackhi_epi64(t9, t11);
    let t12 = _mm_unpacklo_epi32(v12_lo, v13_lo);
    let t13 = _mm_unpackhi_epi32(v12_lo, v13_lo);
    let t14 = _mm_unpacklo_epi32(v14_lo, v15_lo);
    let t15 = _mm_unpackhi_epi32(v14_lo, v15_lo);
    let r12 = _mm_unpacklo_epi64(t12, t14);
    let r13 = _mm_unpackhi_epi64(t12, t14);
    let r14 = _mm_unpacklo_epi64(t13, t15);
    let r15 = _mm_unpackhi_epi64(t13, t15);

    // Store blocks 0-3
    _mm_store_si128(output.0.as_mut_ptr().add(0) as *mut __m128i, r0);
    _mm_store_si128(output.0.as_mut_ptr().add(16) as *mut __m128i, r4);
    _mm_store_si128(output.0.as_mut_ptr().add(32) as *mut __m128i, r8);
    _mm_store_si128(output.0.as_mut_ptr().add(48) as *mut __m128i, r12);
    _mm_store_si128(output.0.as_mut_ptr().add(64) as *mut __m128i, r1);
    _mm_store_si128(output.0.as_mut_ptr().add(80) as *mut __m128i, r5);
    _mm_store_si128(output.0.as_mut_ptr().add(96) as *mut __m128i, r9);
    _mm_store_si128(output.0.as_mut_ptr().add(112) as *mut __m128i, r13);
    _mm_store_si128(output.0.as_mut_ptr().add(128) as *mut __m128i, r2);
    _mm_store_si128(output.0.as_mut_ptr().add(144) as *mut __m128i, r6);
    _mm_store_si128(output.0.as_mut_ptr().add(160) as *mut __m128i, r10);
    _mm_store_si128(output.0.as_mut_ptr().add(176) as *mut __m128i, r14);
    _mm_store_si128(output.0.as_mut_ptr().add(192) as *mut __m128i, r3);
    _mm_store_si128(output.0.as_mut_ptr().add(208) as *mut __m128i, r7);
    _mm_store_si128(output.0.as_mut_ptr().add(224) as *mut __m128i, r11);
    _mm_store_si128(output.0.as_mut_ptr().add(240) as *mut __m128i, r15);

    // Transpose blocks 4-7 from high halves
    let t0 = _mm_unpacklo_epi32(v0_hi, v1_hi);
    let t1 = _mm_unpackhi_epi32(v0_hi, v1_hi);
    let t2 = _mm_unpacklo_epi32(v2_hi, v3_hi);
    let t3 = _mm_unpackhi_epi32(v2_hi, v3_hi);
    let r0 = _mm_unpacklo_epi64(t0, t2);
    let r1 = _mm_unpackhi_epi64(t0, t2);
    let r2 = _mm_unpacklo_epi64(t1, t3);
    let r3 = _mm_unpackhi_epi64(t1, t3);
    let t4 = _mm_unpacklo_epi32(v4_hi, v5_hi);
    let t5 = _mm_unpackhi_epi32(v4_hi, v5_hi);
    let t6 = _mm_unpacklo_epi32(v6_hi, v7_hi);
    let t7 = _mm_unpackhi_epi32(v6_hi, v7_hi);
    let r4 = _mm_unpacklo_epi64(t4, t6);
    let r5 = _mm_unpackhi_epi64(t4, t6);
    let r6 = _mm_unpacklo_epi64(t5, t7);
    let r7 = _mm_unpackhi_epi64(t5, t7);
    let t8 = _mm_unpacklo_epi32(v8_hi, v9_hi);
    let t9 = _mm_unpackhi_epi32(v8_hi, v9_hi);
    let t10 = _mm_unpacklo_epi32(v10_hi, v11_hi);
    let t11 = _mm_unpackhi_epi32(v10_hi, v11_hi);
    let r8 = _mm_unpacklo_epi64(t8, t10);
    let r9 = _mm_unpackhi_epi64(t8, t10);
    let r10 = _mm_unpacklo_epi64(t9, t11);
    let r11 = _mm_unpackhi_epi64(t9, t11);
    let t12 = _mm_unpacklo_epi32(v12_hi, v13_hi);
    let t13 = _mm_unpackhi_epi32(v12_hi, v13_hi);
    let t14 = _mm_unpacklo_epi32(v14_hi, v15_hi);
    let t15 = _mm_unpackhi_epi32(v14_hi, v15_hi);
    let r12 = _mm_unpacklo_epi64(t12, t14);
    let r13 = _mm_unpackhi_epi64(t12, t14);
    let r14 = _mm_unpacklo_epi64(t13, t15);
    let r15 = _mm_unpackhi_epi64(t13, t15);

    // Store blocks 4-7
    _mm_store_si128(output.0.as_mut_ptr().add(256) as *mut __m128i, r0);
    _mm_store_si128(output.0.as_mut_ptr().add(272) as *mut __m128i, r4);
    _mm_store_si128(output.0.as_mut_ptr().add(288) as *mut __m128i, r8);
    _mm_store_si128(output.0.as_mut_ptr().add(304) as *mut __m128i, r12);
    _mm_store_si128(output.0.as_mut_ptr().add(320) as *mut __m128i, r1);
    _mm_store_si128(output.0.as_mut_ptr().add(336) as *mut __m128i, r5);
    _mm_store_si128(output.0.as_mut_ptr().add(352) as *mut __m128i, r9);
    _mm_store_si128(output.0.as_mut_ptr().add(368) as *mut __m128i, r13);
    _mm_store_si128(output.0.as_mut_ptr().add(384) as *mut __m128i, r2);
    _mm_store_si128(output.0.as_mut_ptr().add(400) as *mut __m128i, r6);
    _mm_store_si128(output.0.as_mut_ptr().add(416) as *mut __m128i, r10);
    _mm_store_si128(output.0.as_mut_ptr().add(432) as *mut __m128i, r14);
    _mm_store_si128(output.0.as_mut_ptr().add(448) as *mut __m128i, r3);
    _mm_store_si128(output.0.as_mut_ptr().add(464) as *mut __m128i, r7);
    _mm_store_si128(output.0.as_mut_ptr().add(480) as *mut __m128i, r11);
    _mm_store_si128(output.0.as_mut_ptr().add(496) as *mut __m128i, r15);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn test_avx2_produces_correct_output() {
        if !std::arch::is_x86_feature_detected!("avx2") {
            println!("AVX2 not available, skipping test");
            return;
        }

        // Test that AVX2 implementation produces the same output as the scalar implementation
        let key = [0u8; 32];
        let nonce = [0u8; 8];

        // Create state
        let constants: [u32; 4] = [0x61707865, 0x3320646e, 0x79622d32, 0x6b206574];
        let mut state = [0u32; 16];
        state[0..4].copy_from_slice(&constants);
        for (i, chunk) in key.chunks_exact(4).enumerate() {
            state[4 + i] = u32::from_le_bytes(chunk.try_into().unwrap());
        }
        state[12] = 0;
        state[13] = 0;
        state[14] = u32::from_le_bytes(nonce[0..4].try_into().unwrap());
        state[15] = u32::from_le_bytes(nonce[4..8].try_into().unwrap());

        // Generate keystream using AVX2
        let mut avx2_output = [0u8; 256];
        unsafe {
            chacha_blocks_avx2::<20>(&state, &mut avx2_output);
        }

        // Generate keystream using scalar implementation
        let mut scalar_output = [0u8; 64];
        let block = scalar_chacha_block::<20>(&state);
        scalar_serialize_state(&block, &mut scalar_output);

        // First block should match
        assert_eq!(
            &avx2_output[0..64],
            &scalar_output[..],
            "First block mismatch:\nAVX2:   {:02x?}\nScalar: {:02x?}",
            &avx2_output[0..64],
            &scalar_output[..]
        );

        // Test second block (counter = 1)
        let mut state_c1 = state;
        state_c1[12] = 1;
        let block = scalar_chacha_block::<20>(&state_c1);
        scalar_serialize_state(&block, &mut scalar_output);
        assert_eq!(
            &avx2_output[64..128],
            &scalar_output[..],
            "Second block mismatch:\nAVX2:   {:02x?}\nScalar: {:02x?}",
            &avx2_output[64..128],
            &scalar_output[..]
        );

        // Test third block (counter = 2)
        let mut state_c2 = state;
        state_c2[12] = 2;
        let block = scalar_chacha_block::<20>(&state_c2);
        scalar_serialize_state(&block, &mut scalar_output);
        assert_eq!(
            &avx2_output[128..192],
            &scalar_output[..],
            "Third block mismatch"
        );

        // Test fourth block (counter = 3)
        let mut state_c3 = state;
        state_c3[12] = 3;
        let block = scalar_chacha_block::<20>(&state_c3);
        scalar_serialize_state(&block, &mut scalar_output);
        assert_eq!(
            &avx2_output[192..256],
            &scalar_output[..],
            "Fourth block mismatch"
        );
    }

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn test_avx2_x8_produces_correct_output() {
        if !std::arch::is_x86_feature_detected!("avx2") {
            println!("AVX2 not available, skipping test");
            return;
        }

        let key = [0u8; 32];
        let nonce = [0u8; 8];

        let constants: [u32; 4] = [0x61707865, 0x3320646e, 0x79622d32, 0x6b206574];
        let mut state = [0u32; 16];
        state[0..4].copy_from_slice(&constants);
        for (i, chunk) in key.chunks_exact(4).enumerate() {
            state[4 + i] = u32::from_le_bytes(chunk.try_into().unwrap());
        }
        state[12] = 0;
        state[13] = 0;
        state[14] = u32::from_le_bytes([nonce[0], nonce[1], nonce[2], nonce[3]]);
        state[15] = u32::from_le_bytes([nonce[4], nonce[5], nonce[6], nonce[7]]);

        let mut avx2_output = AlignedU8x512([0u8; 512]);
        unsafe {
            chacha_blocks_avx2_x8::<20>(&state, &mut avx2_output);
        }

        // Verify all 8 blocks
        for block_idx in 0..8u32 {
            let mut s = state;
            s[12] = block_idx;
            let block = scalar_chacha_block::<20>(&s);
            let mut scalar_output = [0u8; 64];
            scalar_serialize_state(&block, &mut scalar_output);
            assert_eq!(
                &avx2_output[block_idx as usize * 64..(block_idx as usize + 1) * 64],
                &scalar_output[..],
                "Block {} mismatch",
                block_idx
            );
        }
    }

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
        for (i, word) in state.iter().enumerate() {
            output[i * 4..i * 4 + 4].copy_from_slice(&word.to_le_bytes());
        }
    }
}
