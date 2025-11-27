//! AVX2 SIMD implementation for base64 encoding/decoding.

use super::*;
use std::arch::x86_64::*;

/// Check if AVX2 is available at runtime.
#[inline]
pub fn is_available() -> bool {
    is_x86_feature_detected!("avx2")
}

/// Reshuffle bytes for encoding: takes 24 bytes and produces 32 6-bit values.
#[target_feature(enable = "avx2")]
#[inline]
unsafe fn enc_reshuffle(input: __m256i) -> __m256i {
    // translation from SSE into AVX2 of procedure
    // https://github.com/WojciechMula/base64simd/blob/master/encode/unpack_bigendian.cpp
    let input: __m256i = _mm256_shuffle_epi8(
        input,
        _mm256_set_epi8(
            10, 11, 9, 10, 7, 8, 6, 7, 4, 5, 3, 4, 1, 2, 0, 1, 14, 15, 13, 14, 11, 12, 10, 11, 8,
            9, 7, 8, 5, 6, 4, 5,
        ),
    );

    let t0: __m256i = _mm256_and_si256(input, _mm256_set1_epi32(0x0fc0fc00u32 as i32));
    let t1: __m256i = _mm256_mulhi_epu16(t0, _mm256_set1_epi32(0x04000040));

    let t2 = _mm256_and_si256(input, _mm256_set1_epi32(0x003f03f0));
    let t3 = _mm256_mullo_epi16(t2, _mm256_set1_epi32(0x01000010));

    _mm256_or_si256(t1, t3)
}

/// Translate 6-bit indices to ASCII characters for the standard alphabet.
#[target_feature(enable = "avx2")]
#[inline]
unsafe fn enc_translate(input: __m256i) -> __m256i {
    let lut: __m256i = _mm256_setr_epi8(
        65, 71, -4, -4, -4, -4, -4, -4, -4, -4, -4, -4, -19, -16, 0, 0, 65, 71, -4, -4, -4, -4, -4,
        -4, -4, -4, -4, -4, -19, -16, 0, 0,
    );
    let mut indices = _mm256_subs_epu8(input, _mm256_set1_epi8(51));
    let mask = _mm256_cmpgt_epi8(input, _mm256_set1_epi8(25));
    indices = _mm256_sub_epi8(indices, mask);

    _mm256_add_epi8(input, _mm256_shuffle_epi8(lut, indices))
}

/// Reshuffle decoded 6-bit values into bytes.
#[target_feature(enable = "avx2")]
#[inline]
unsafe fn dec_reshuffle(input: __m256i) -> __m256i {
    let merge_ab_and_bc: __m256i = _mm256_maddubs_epi16(input, _mm256_set1_epi32(0x01400140));
    let out: __m256i = _mm256_madd_epi16(merge_ab_and_bc, _mm256_set1_epi32(0x00011000));

    let out = _mm256_shuffle_epi8(
        out,
        _mm256_setr_epi8(
            2, 1, 0, 6, 5, 4, 10, 9, 8, 14, 13, 12, -1, -1, -1, -1, 2, 1, 0, 6, 5, 4, 10, 9, 8, 14,
            13, 12, -1, -1, -1, -1,
        ),
    );
    _mm256_permutevar8x32_epi32(out, _mm256_setr_epi32(0, 1, 2, 4, 5, 6, -1, -1))
}

/// Encode using AVX2 SIMD instructions.
///
/// # Safety
/// Caller must ensure AVX2 is available (check with `is_available()`).
#[target_feature(enable = "avx2")]
pub unsafe fn encode_avx2(output: &mut [u8], data: &[u8], padding: bool) {
    // Process 24-byte chunks using iterator, producing 32 bytes of output each
    let full_chunks = data.len() / 24;
    let simd_input_len = full_chunks * 24;

    // Use chunks_exact for the SIMD path
    data.chunks_exact(24)
        .zip(output.chunks_exact_mut(32))
        .for_each(|(input_chunk, output_chunk)| {
            // Create a properly aligned buffer with 4 padding bytes at the start
            let mut aligned_buf = [0u8; 32];
            aligned_buf[4..28].copy_from_slice(input_chunk);

            // Load from offset 0 of our aligned buffer (which has the data at offset 4)
            let inputvector = _mm256_loadu_si256(aligned_buf.as_ptr() as *const __m256i);
            let reshuffled = enc_reshuffle(inputvector);
            let translated = enc_translate(reshuffled);

            _mm256_storeu_si256(output_chunk.as_mut_ptr() as *mut __m256i, translated);
        });

    // Handle remaining bytes with scalar code
    let remaining_data = &data[simd_input_len..];
    if !remaining_data.is_empty() {
        let out_offset = full_chunks * 32;
        let remaining_output = &mut output[out_offset..];
        super::encode_to_slice(remaining_output, remaining_data, ALPHABET_STANDARD, padding);
    }
}

/// Decode using AVX2 SIMD instructions.
///
/// Returns the number of bytes written to output, or an error.
///
/// # Safety
/// Caller must ensure AVX2 is available (check with `is_available()`).
#[target_feature(enable = "avx2")]
pub unsafe fn decode_avx2(output: &mut [u8], input: &[u8]) -> Result<usize, Error> {
    let input_len = input.len();

    // We need at least 45 bytes for the AVX2 path (32 for SIMD + 13 for safety margin)
    // Calculate how many full 32-byte chunks we can safely process
    let safe_len = if input_len >= 45 { input_len - 13 } else { 0 };
    let full_chunks = safe_len / 32;

    // Lookup tables for decoding (moved outside loop for clarity)
    let lut_lo: __m256i = _mm256_setr_epi8(
        0x15, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x13, 0x1A, 0x1B, 0x1B, 0x1B,
        0x1A, 0x15, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x13, 0x1A, 0x1B, 0x1B,
        0x1B, 0x1A,
    );
    let lut_hi: __m256i = _mm256_setr_epi8(
        0x10, 0x10, 0x01, 0x02, 0x04, 0x08, 0x04, 0x08, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
        0x10, 0x10, 0x10, 0x01, 0x02, 0x04, 0x08, 0x04, 0x08, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
        0x10, 0x10,
    );
    let lut_roll: __m256i = _mm256_setr_epi8(
        0, 16, 19, 4, -65, -65, -71, -71, 0, 0, 0, 0, 0, 0, 0, 0, 0, 16, 19, 4, -65, -65, -71, -71,
        0, 0, 0, 0, 0, 0, 0, 0,
    );
    let mask_2f: __m256i = _mm256_set1_epi8(0x2f);

    // Process chunks using iterator with try_fold to handle early exit on invalid chars
    let simd_input = &input[..full_chunks * 32];
    let simd_output = &mut output[..full_chunks * 24];

    let processed = simd_input
        .chunks_exact(32)
        .zip(simd_output.chunks_exact_mut(24))
        .try_fold(0usize, |chunks_done, (input_chunk, output_chunk)| {
            let str_vec = _mm256_loadu_si256(input_chunk.as_ptr() as *const __m256i);

            // Lookup
            let hi_nibbles: __m256i = _mm256_srli_epi32(str_vec, 4);
            let lo_nibbles: __m256i = _mm256_and_si256(str_vec, mask_2f);

            let lo: __m256i = _mm256_shuffle_epi8(lut_lo, lo_nibbles);
            let eq_2f: __m256i = _mm256_cmpeq_epi8(str_vec, mask_2f);

            let hi_nibbles = _mm256_and_si256(hi_nibbles, mask_2f);
            let hi: __m256i = _mm256_shuffle_epi8(lut_hi, hi_nibbles);
            let roll: __m256i = _mm256_shuffle_epi8(lut_roll, _mm256_add_epi8(eq_2f, hi_nibbles));

            // Check for invalid characters - return None to break iteration
            if _mm256_testz_si256(lo, hi) == 0 {
                return None;
            }

            let str_vec = _mm256_add_epi8(str_vec, roll);

            // Reshuffle to packed output
            let result = dec_reshuffle(str_vec);
            _mm256_storeu_si256(output_chunk.as_mut_ptr() as *mut __m256i, result);

            Some(chunks_done + 1)
        })
        .unwrap_or(0);

    let in_offset = processed * 32;
    let out_offset = processed * 24;

    // Handle remaining bytes with scalar decoder
    if in_offset < input_len {
        let remaining_input = &input[in_offset..];
        let remaining_output = &mut output[out_offset..];

        // Decode remaining bytes using scalar implementation
        let remaining_str =
            std::str::from_utf8(remaining_input).map_err(|_| Error::InvalidCharacter('\0'))?;
        let decoded = super::decode_with(remaining_str, ALPHABET_STANDARD)?;
        remaining_output[..decoded.len()].copy_from_slice(&decoded);
        return Ok(out_offset + decoded.len());
    }

    Ok(out_offset)
}
