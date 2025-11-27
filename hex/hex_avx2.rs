//! AVX2 SIMD implementation for hex encoding/decoding.

use super::*;
use std::arch::x86_64::*;

/// Extra bytes allocated for SIMD writes that may overshoot.
pub const AVX2_EXTRA_BYTES: usize = 32;

/// Check if AVX2 is available at runtime.
#[inline]
pub fn is_available() -> bool {
    is_x86_feature_detected!("avx2")
}

/// Encode using AVX2 SIMD instructions.
///
/// Processes 32 input bytes at a time, producing 64 output bytes.
///
/// # Safety
/// Caller must ensure AVX2 is available (check with `is_available()`).
#[target_feature(enable = "avx2")]
pub unsafe fn encode_avx2(output: &mut [u8], data: &[u8], alphabet: &[u8; 16]) {
    // Only use SIMD for lowercase alphabet (we can optimize for specific cases)
    let use_simd = alphabet == ALPHABET_LOWER || alphabet == ALPHABET_UPPER;

    if !use_simd {
        super::encode_into_unchecked(output, data, alphabet);
        return;
    }

    let is_upper = alphabet == ALPHABET_UPPER;

    // Process 32 bytes at a time
    let full_chunks = data.len() / 32;
    let simd_input_len = full_chunks * 32;

    let mut in_idx = 0;
    let mut out_idx = 0;

    // ASCII offset for a-f (lowercase) or A-F (uppercase)
    let letter_offset = if is_upper { b'A' - 10 } else { b'a' - 10 };

    for _ in 0..full_chunks {
        // Load 32 bytes of input
        let input = _mm256_loadu_si256(data.as_ptr().add(in_idx) as *const __m256i);

        // Split into high and low nibbles
        let mask_0f = _mm256_set1_epi8(0x0F);
        let lo_nibbles = _mm256_and_si256(input, mask_0f);
        let hi_nibbles = _mm256_and_si256(_mm256_srli_epi16(input, 4), mask_0f);

        // Convert nibbles to ASCII hex characters
        // If nibble < 10: add '0' (0x30)
        // If nibble >= 10: add 'a' - 10 (0x57) or 'A' - 10 (0x37)
        let nine = _mm256_set1_epi8(9);
        let ascii_zero = _mm256_set1_epi8(b'0' as i8);
        let letter_off = _mm256_set1_epi8(letter_offset as i8);

        // High nibbles
        let hi_is_letter = _mm256_cmpgt_epi8(hi_nibbles, nine);
        let hi_offset = _mm256_blendv_epi8(ascii_zero, letter_off, hi_is_letter);
        let hi_ascii = _mm256_add_epi8(hi_nibbles, hi_offset);

        // Low nibbles
        let lo_is_letter = _mm256_cmpgt_epi8(lo_nibbles, nine);
        let lo_offset = _mm256_blendv_epi8(ascii_zero, letter_off, lo_is_letter);
        let lo_ascii = _mm256_add_epi8(lo_nibbles, lo_offset);

        // Interleave high and low nibbles
        // We need to produce: h0 l0 h1 l1 h2 l2 ...
        let lo_interleaved = _mm256_unpacklo_epi8(hi_ascii, lo_ascii);
        let hi_interleaved = _mm256_unpackhi_epi8(hi_ascii, lo_ascii);

        // The unpack operation works on 128-bit lanes, so we need to permute
        // to get the correct order across lanes
        let result_lo = _mm256_permute2x128_si256(lo_interleaved, hi_interleaved, 0x20);
        let result_hi = _mm256_permute2x128_si256(lo_interleaved, hi_interleaved, 0x31);

        // Store 64 bytes of output
        _mm256_storeu_si256(output.as_mut_ptr().add(out_idx) as *mut __m256i, result_lo);
        _mm256_storeu_si256(
            output.as_mut_ptr().add(out_idx + 32) as *mut __m256i,
            result_hi,
        );

        in_idx += 32;
        out_idx += 64;
    }

    // Handle remaining bytes with scalar code
    if simd_input_len < data.len() {
        let remaining_data = &data[simd_input_len..];
        let remaining_output = &mut output[out_idx..];
        super::encode_into_unchecked(remaining_output, remaining_data, alphabet);
    }
}

/// Decode using AVX2 SIMD instructions.
///
/// Processes 64 input bytes (hex chars) at a time, producing 32 output bytes.
///
/// # Safety
/// Caller must ensure AVX2 is available (check with `is_available()`).
#[target_feature(enable = "avx2")]
pub unsafe fn decode_avx2(output: &mut [u8], input: &[u8]) -> Result<usize, Error> {
    let input_len = input.len();

    if !input_len.is_multiple_of(2) {
        return Err(Error::InvalidLength);
    }

    // Process 64 hex chars at a time (32 output bytes)
    let full_chunks = input_len / 64;

    let mut in_idx = 0;
    let mut out_idx = 0;

    for _ in 0..full_chunks {
        // Load 64 bytes of hex input (two 256-bit loads)
        let input_lo = _mm256_loadu_si256(input.as_ptr().add(in_idx) as *const __m256i);
        let input_hi = _mm256_loadu_si256(input.as_ptr().add(in_idx + 32) as *const __m256i);

        // Decode each 256-bit vector to nibbles
        let (nibbles_lo, valid_lo) = decode_hex_chars(input_lo);
        let (nibbles_hi, valid_hi) = decode_hex_chars(input_hi);

        // Check for invalid characters
        if !valid_lo || !valid_hi {
            // Fall back to scalar to find the invalid character
            for i in 0..64 {
                let c = input[in_idx + i];
                if !c.is_ascii_hexdigit() {
                    return Err(Error::InvalidCharacter(c as char));
                }
            }
        }

        // Pack nibbles into bytes: each pair of nibbles becomes one byte
        // nibbles_lo: [n0, n1, n2, n3, ..., n31] - 32 nibbles from first 32 hex chars
        // nibbles_hi: [n32, n33, ..., n63] - 32 nibbles from next 32 hex chars
        // We need: [(n0<<4)|n1, (n2<<4)|n3, ...] -> 16 bytes from each vector

        // Use maddubs to combine pairs: multiply first by 16 and add second
        // maddubs: (a[0]*b[0] + a[1]*b[1], a[2]*b[2] + a[3]*b[3], ...)
        // We want: n0*16 + n1, n2*16 + n3, ...
        let mult = _mm256_set1_epi16(0x0110); // [16, 1, 16, 1, ...]

        let packed_lo = _mm256_maddubs_epi16(nibbles_lo, mult);
        let packed_hi = _mm256_maddubs_epi16(nibbles_hi, mult);

        // packed_lo now has 16 values in 16-bit slots: [b0, b1, ..., b15]
        // packed_hi now has 16 values in 16-bit slots: [b16, b17, ..., b31]

        // Pack 16-bit to 8-bit using packus
        // packus takes two vectors and packs them, but operates on 128-bit lanes separately
        // Result: [lo_lane0, hi_lane0, lo_lane1, hi_lane1]
        let packed = _mm256_packus_epi16(packed_lo, packed_hi);

        // Permute to get correct order across lanes
        // After packus: [b0-b7, b16-b23, b8-b15, b24-b31]
        // We want: [b0-b7, b8-b15, b16-b23, b24-b31]
        let result = _mm256_permute4x64_epi64(packed, 0b11011000);

        // Store 32 bytes of output
        _mm256_storeu_si256(output.as_mut_ptr().add(out_idx) as *mut __m256i, result);

        in_idx += 64;
        out_idx += 32;
    }

    // Handle remaining bytes with scalar code
    if in_idx < input_len {
        let remaining_input =
            std::str::from_utf8(&input[in_idx..]).map_err(|_| Error::InvalidUtf8)?;
        let decoded = super::decode_checked(remaining_input, ALPHABET_LOWER)?;
        output[out_idx..out_idx + decoded.len()].copy_from_slice(&decoded);
        out_idx += decoded.len();
    }

    Ok(out_idx)
}

/// Decode a vector of 32 hex characters to 32 nibbles (4-bit values).
/// Returns (decoded_values, all_valid).
#[target_feature(enable = "avx2")]
#[inline]
unsafe fn decode_hex_chars(input: __m256i) -> (__m256i, bool) {
    // Character ranges:
    // '0'-'9' = 0x30-0x39 -> values 0-9
    // 'A'-'F' = 0x41-0x46 -> values 10-15
    // 'a'-'f' = 0x61-0x66 -> values 10-15

    let ascii_zero = _mm256_set1_epi8(b'0' as i8);
    let ascii_a = _mm256_set1_epi8(b'a' as i8);
    let ascii_upper_a = _mm256_set1_epi8(b'A' as i8);
    let six = _mm256_set1_epi8(6);
    let ten = _mm256_set1_epi8(10);

    // Check if in '0'-'9' range
    let offset_digit = _mm256_sub_epi8(input, ascii_zero);
    let is_digit = _mm256_and_si256(
        _mm256_cmpgt_epi8(offset_digit, _mm256_set1_epi8(-1)),
        _mm256_cmpgt_epi8(_mm256_set1_epi8(10), offset_digit),
    );

    // Check if in 'a'-'f' range
    let offset_lower = _mm256_sub_epi8(input, ascii_a);
    let is_lower = _mm256_and_si256(
        _mm256_cmpgt_epi8(offset_lower, _mm256_set1_epi8(-1)),
        _mm256_cmpgt_epi8(six, offset_lower),
    );

    // Check if in 'A'-'F' range
    let offset_upper = _mm256_sub_epi8(input, ascii_upper_a);
    let is_upper = _mm256_and_si256(
        _mm256_cmpgt_epi8(offset_upper, _mm256_set1_epi8(-1)),
        _mm256_cmpgt_epi8(six, offset_upper),
    );

    // Combine validity check
    let is_valid = _mm256_or_si256(_mm256_or_si256(is_digit, is_lower), is_upper);
    let all_valid = _mm256_movemask_epi8(is_valid) == -1i32;

    // Calculate nibble values
    // For digits: value = input - '0'
    // For lowercase: value = input - 'a' + 10
    // For uppercase: value = input - 'A' + 10
    let digit_val = offset_digit;
    let lower_val = _mm256_add_epi8(offset_lower, ten);
    let upper_val = _mm256_add_epi8(offset_upper, ten);

    // Select the correct value based on character type
    let result = _mm256_blendv_epi8(
        _mm256_blendv_epi8(upper_val, lower_val, is_lower),
        digit_val,
        is_digit,
    );

    (result, all_valid)
}
