//! AVX2 SIMD implementation for base32 encoding/decoding.

use super::*;
use std::arch::x86_64::*;

/// Check if AVX2 is available at runtime.
#[inline]
pub fn is_available() -> bool {
    is_x86_feature_detected!("avx2")
}

/// Encode using AVX2 SIMD instructions.
///
/// Processes 10 input bytes at a time, producing 16 output bytes.
/// Then processes a second group to get 20 bytes -> 32 chars.
///
/// # Safety
/// Caller must ensure AVX2 is available (check with `is_available()`).
#[target_feature(enable = "avx2")]
pub unsafe fn encode_avx2(output: &mut [u8], data: &[u8], alphabet: &[u8; 32], padding: bool) {
    // Only use SIMD for standard alphabet
    let use_simd = alphabet == ALPHABET_STANDARD_BYTES || alphabet == ALPHABET_HEX_BYTES;

    if !use_simd || data.len() < 10 {
        super::encode_into_unchecked(output, data, alphabet, padding);
        return;
    }

    let is_hex = alphabet == ALPHABET_HEX_BYTES;

    // Process 10 bytes at a time (2 groups of 5 bytes = 16 output chars)
    // This allows us to use 128-bit operations for better efficiency
    let full_chunks = data.len() / 10;

    let mut in_idx = 0;
    let mut out_idx = 0;

    // Process pairs of 10-byte chunks when possible for AVX2 efficiency
    let chunk_pairs = full_chunks / 2;
    for _ in 0..chunk_pairs {
        // Load 20 bytes (will be processed as two 10-byte groups)
        let mut input_buf = [0u8; 32];
        input_buf[..20].copy_from_slice(&data[in_idx..in_idx + 20]);

        // Process 20 bytes -> 32 characters using optimized bit extraction
        let result = encode_20_bytes_avx2(&input_buf, is_hex);

        // Store 32 bytes of output
        _mm256_storeu_si256(output.as_mut_ptr().add(out_idx) as *mut __m256i, result);

        in_idx += 20;
        out_idx += 32;
    }

    // Handle remaining 10-byte chunk if odd number
    if full_chunks % 2 == 1 && in_idx + 10 <= data.len() {
        let mut input_buf = [0u8; 16];
        input_buf[..10].copy_from_slice(&data[in_idx..in_idx + 10]);

        let result = encode_10_bytes_sse(&input_buf, is_hex);
        _mm_storeu_si128(output.as_mut_ptr().add(out_idx) as *mut __m128i, result);

        in_idx += 10;
        out_idx += 16;
    }

    // Handle remaining bytes with scalar code
    if in_idx < data.len() {
        let remaining_data = &data[in_idx..];
        let remaining_output = &mut output[out_idx..];
        super::encode_into_unchecked(remaining_output, remaining_data, alphabet, padding);
    }
}

/// Encode 20 bytes to 32 base32 characters using AVX2.
/// Uses optimized bit manipulation with SIMD.
#[target_feature(enable = "avx2")]
#[inline]
unsafe fn encode_20_bytes_avx2(input: &[u8; 32], is_hex: bool) -> __m256i {
    // For base32, each 5 bytes becomes 8 characters
    // We process 4 groups of 5 bytes -> 32 characters

    // The bit layout for 5 bytes (40 bits) -> 8 x 5-bit values:
    // Byte 0: bits 7-3 -> char 0, bits 2-0 -> part of char 1
    // Byte 1: bits 7-6 -> part of char 1, bits 5-1 -> char 2, bit 0 -> part of char 3
    // Byte 2: bits 7-4 -> part of char 3, bits 3-0 -> part of char 4
    // Byte 3: bit 7 -> part of char 4, bits 6-2 -> char 5, bits 1-0 -> part of char 6
    // Byte 4: bits 7-5 -> part of char 6, bits 4-0 -> char 7

    // Use scalar extraction (optimized with loop unrolling) then SIMD for ASCII conversion
    let mut values = [0u8; 32];

    // Process 4 groups of 5 bytes
    for g in 0..4 {
        let i = g * 5;
        let b0 = input[i];
        let b1 = input[i + 1];
        let b2 = input[i + 2];
        let b3 = input[i + 3];
        let b4 = input[i + 4];

        let o = g * 8;
        values[o] = b0 >> 3;
        values[o + 1] = ((b0 & 0x07) << 2) | (b1 >> 6);
        values[o + 2] = (b1 >> 1) & 0x1F;
        values[o + 3] = ((b1 & 0x01) << 4) | (b2 >> 4);
        values[o + 4] = ((b2 & 0x0F) << 1) | (b3 >> 7);
        values[o + 5] = (b3 >> 2) & 0x1F;
        values[o + 6] = ((b3 & 0x03) << 3) | (b4 >> 5);
        values[o + 7] = b4 & 0x1F;
    }

    // Now use SIMD to convert 5-bit values to ASCII
    let vals = _mm256_loadu_si256(values.as_ptr() as *const __m256i);
    values_to_ascii_avx2(vals, is_hex)
}

/// Encode 10 bytes to 16 base32 characters using SSE.
#[target_feature(enable = "avx2")]
#[inline]
unsafe fn encode_10_bytes_sse(input: &[u8; 16], is_hex: bool) -> __m128i {
    // Process 2 groups of 5 bytes -> 16 characters
    let mut values = [0u8; 16];

    for g in 0..2 {
        let i = g * 5;
        let b0 = input[i];
        let b1 = input[i + 1];
        let b2 = input[i + 2];
        let b3 = input[i + 3];
        let b4 = input[i + 4];

        let o = g * 8;
        values[o] = b0 >> 3;
        values[o + 1] = ((b0 & 0x07) << 2) | (b1 >> 6);
        values[o + 2] = (b1 >> 1) & 0x1F;
        values[o + 3] = ((b1 & 0x01) << 4) | (b2 >> 4);
        values[o + 4] = ((b2 & 0x0F) << 1) | (b3 >> 7);
        values[o + 5] = (b3 >> 2) & 0x1F;
        values[o + 6] = ((b3 & 0x03) << 3) | (b4 >> 5);
        values[o + 7] = b4 & 0x1F;
    }

    let vals = _mm_loadu_si128(values.as_ptr() as *const __m128i);
    values_to_ascii_sse(vals, is_hex)
}

/// Convert 5-bit values to ASCII using AVX2.
#[target_feature(enable = "avx2")]
#[inline]
unsafe fn values_to_ascii_avx2(values: __m256i, is_hex: bool) -> __m256i {
    if is_hex {
        // Hex alphabet: 0-9 -> '0'-'9', 10-31 -> 'A'-'V'
        let ten = _mm256_set1_epi8(10);
        let is_digit = _mm256_cmpgt_epi8(ten, values);

        let digit_offset = _mm256_set1_epi8(b'0' as i8);
        let letter_offset = _mm256_set1_epi8((b'A' as i8) - 10);

        let offsets = _mm256_blendv_epi8(letter_offset, digit_offset, is_digit);
        _mm256_add_epi8(values, offsets)
    } else {
        // Standard alphabet: 0-25 -> 'A'-'Z', 26-31 -> '2'-'7'
        let twenty_six = _mm256_set1_epi8(26);
        let is_letter = _mm256_cmpgt_epi8(twenty_six, values);

        let letter_offset = _mm256_set1_epi8(b'A' as i8);
        let digit_offset = _mm256_set1_epi8((b'2' as i8) - 26);

        let offsets = _mm256_blendv_epi8(digit_offset, letter_offset, is_letter);
        _mm256_add_epi8(values, offsets)
    }
}

/// Convert 5-bit values to ASCII using SSE.
#[target_feature(enable = "avx2")]
#[inline]
unsafe fn values_to_ascii_sse(values: __m128i, is_hex: bool) -> __m128i {
    if is_hex {
        let ten = _mm_set1_epi8(10);
        let is_digit = _mm_cmplt_epi8(values, ten);

        let digit_offset = _mm_set1_epi8(b'0' as i8);
        let letter_offset = _mm_set1_epi8((b'A' as i8) - 10);

        let offsets = _mm_blendv_epi8(letter_offset, digit_offset, is_digit);
        _mm_add_epi8(values, offsets)
    } else {
        let twenty_six = _mm_set1_epi8(26);
        let is_letter = _mm_cmplt_epi8(values, twenty_six);

        let letter_offset = _mm_set1_epi8(b'A' as i8);
        let digit_offset = _mm_set1_epi8((b'2' as i8) - 26);

        let offsets = _mm_blendv_epi8(digit_offset, letter_offset, is_letter);
        _mm_add_epi8(values, offsets)
    }
}

/// Decode using AVX2 SIMD instructions.
///
/// Processes 32 input characters at a time, producing 20 output bytes.
///
/// # Safety
/// Caller must ensure AVX2 is available (check with `is_available()`).
#[target_feature(enable = "avx2")]
pub unsafe fn decode_avx2(
    output: &mut [u8],
    input: &[u8],
    alphabet: &[u8; 32],
) -> Result<usize, Error> {
    let input_len = input.len();

    // Only use SIMD for standard alphabet and sufficient length
    let use_simd = (alphabet == ALPHABET_STANDARD_BYTES || alphabet == ALPHABET_HEX_BYTES) && input_len >= 16;

    if !use_simd {
        let input_str = std::str::from_utf8(input).map_err(|_| Error::InvalidCharacter('\0'))?;
        let decoded = super::decode(input_str, alphabet_to_enum(alphabet))?;
        output[..decoded.len()].copy_from_slice(&decoded);
        return Ok(decoded.len());
    }

    let is_hex = alphabet == ALPHABET_HEX_BYTES;

    // Count padding
    let padding_len = input.iter().rev().take_while(|&&b| b == b'=').count();
    let effective_len = input_len - padding_len;

    // Process 16-character chunks (-> 10 bytes)
    let full_chunks = effective_len / 16;

    let mut in_idx = 0;
    let mut out_idx = 0;

    // Process pairs of 16-char chunks for AVX2 efficiency (32 chars -> 20 bytes)
    let chunk_pairs = full_chunks / 2;
    for _ in 0..chunk_pairs {
        let input_vec = _mm256_loadu_si256(input.as_ptr().add(in_idx) as *const __m256i);

        // Decode ASCII to 5-bit values
        let (values, valid) = ascii_to_values_avx2(input_vec, is_hex);

        if !valid {
            // Find the invalid character
            for i in 0..32 {
                let c = input[in_idx + i];
                if !is_valid_base32_char(c, is_hex) {
                    return Err(Error::InvalidCharacter(c as char));
                }
            }
        }

        // Convert 32 5-bit values to 20 bytes
        let decoded = decode_32_values_to_20_bytes(values);

        // Store first 20 bytes (copy from array to avoid overwriting)
        std::ptr::copy_nonoverlapping(decoded.as_ptr(), output.as_mut_ptr().add(out_idx), 20);

        in_idx += 32;
        out_idx += 20;
    }

    // Handle remaining 16-char chunk if odd number
    if full_chunks % 2 == 1 && in_idx + 16 <= effective_len {
        let input_vec = _mm_loadu_si128(input.as_ptr().add(in_idx) as *const __m128i);

        let (values, valid) = ascii_to_values_sse(input_vec, is_hex);

        if !valid {
            for i in 0..16 {
                let c = input[in_idx + i];
                if !is_valid_base32_char(c, is_hex) {
                    return Err(Error::InvalidCharacter(c as char));
                }
            }
        }

        let decoded = decode_16_values_to_10_bytes(values);
        std::ptr::copy_nonoverlapping(decoded.as_ptr(), output.as_mut_ptr().add(out_idx), 10);

        in_idx += 16;
        out_idx += 10;
    }

    // Handle remaining characters with scalar code
    if in_idx < effective_len {
        let remaining_input =
            std::str::from_utf8(&input[in_idx..]).map_err(|_| Error::InvalidCharacter('\0'))?;
        let decoded = super::decode(remaining_input, alphabet_to_enum(alphabet))?;
        output[out_idx..out_idx + decoded.len()].copy_from_slice(&decoded);
        out_idx += decoded.len();
    }

    Ok(out_idx)
}

/// Convert alphabet bytes to Alphabet enum.
#[inline]
fn alphabet_to_enum(alphabet: &[u8; 32]) -> super::Alphabet<'_> {
    if alphabet == ALPHABET_STANDARD_BYTES {
        super::Alphabet::Standard
    } else if alphabet == ALPHABET_HEX_BYTES {
        super::Alphabet::Hex
    } else {
        super::Alphabet::Custom(alphabet)
    }
}

/// Check if a character is valid for base32.
#[inline]
fn is_valid_base32_char(c: u8, is_hex: bool) -> bool {
    if is_hex {
        c.is_ascii_digit() || (b'A'..=b'V').contains(&c) || (b'a'..=b'v').contains(&c)
    } else {
        c.is_ascii_uppercase() || c.is_ascii_lowercase() || (b'2'..=b'7').contains(&c)
    }
}

/// Convert ASCII to 5-bit values using AVX2.
#[target_feature(enable = "avx2")]
#[inline]
unsafe fn ascii_to_values_avx2(input: __m256i, is_hex: bool) -> (__m256i, bool) {
    if is_hex {
        // Hex alphabet: '0'-'9' -> 0-9, 'A'-'V' -> 10-31, 'a'-'v' -> 10-31
        let ascii_zero = _mm256_set1_epi8(b'0' as i8);
        let ascii_a = _mm256_set1_epi8(b'A' as i8);
        let ascii_a_lower = _mm256_set1_epi8(b'a' as i8);
        let ten = _mm256_set1_epi8(10);
        let twenty_two = _mm256_set1_epi8(22);

        // Check if digit (0-9)
        let offset_digit = _mm256_sub_epi8(input, ascii_zero);
        let is_digit = _mm256_and_si256(
            _mm256_cmpgt_epi8(offset_digit, _mm256_set1_epi8(-1)),
            _mm256_cmpgt_epi8(ten, offset_digit),
        );

        // Check if uppercase letter (A-V)
        let offset_upper = _mm256_sub_epi8(input, ascii_a);
        let is_upper = _mm256_and_si256(
            _mm256_cmpgt_epi8(offset_upper, _mm256_set1_epi8(-1)),
            _mm256_cmpgt_epi8(twenty_two, offset_upper),
        );

        // Check if lowercase letter (a-v)
        let offset_lower = _mm256_sub_epi8(input, ascii_a_lower);
        let is_lower = _mm256_and_si256(
            _mm256_cmpgt_epi8(offset_lower, _mm256_set1_epi8(-1)),
            _mm256_cmpgt_epi8(twenty_two, offset_lower),
        );

        // Check validity
        let is_valid = _mm256_or_si256(_mm256_or_si256(is_digit, is_upper), is_lower);
        let all_valid = _mm256_movemask_epi8(is_valid) == -1i32;

        // Calculate values
        let digit_val = offset_digit;
        let upper_val = _mm256_add_epi8(offset_upper, ten);
        let lower_val = _mm256_add_epi8(offset_lower, ten);

        let result = _mm256_blendv_epi8(
            _mm256_blendv_epi8(lower_val, upper_val, is_upper),
            digit_val,
            is_digit,
        );

        (result, all_valid)
    } else {
        // Standard alphabet: 'A'-'Z' -> 0-25, '2'-'7' -> 26-31, also lowercase
        let ascii_a = _mm256_set1_epi8(b'A' as i8);
        let ascii_a_lower = _mm256_set1_epi8(b'a' as i8);
        let ascii_two = _mm256_set1_epi8(b'2' as i8);
        let twenty_six = _mm256_set1_epi8(26);
        let six = _mm256_set1_epi8(6);

        // Check if uppercase letter (A-Z)
        let offset_upper = _mm256_sub_epi8(input, ascii_a);
        let is_upper = _mm256_and_si256(
            _mm256_cmpgt_epi8(offset_upper, _mm256_set1_epi8(-1)),
            _mm256_cmpgt_epi8(twenty_six, offset_upper),
        );

        // Check if lowercase letter (a-z)
        let offset_lower = _mm256_sub_epi8(input, ascii_a_lower);
        let is_lower = _mm256_and_si256(
            _mm256_cmpgt_epi8(offset_lower, _mm256_set1_epi8(-1)),
            _mm256_cmpgt_epi8(twenty_six, offset_lower),
        );

        // Check if digit (2-7)
        let offset_digit = _mm256_sub_epi8(input, ascii_two);
        let is_digit = _mm256_and_si256(
            _mm256_cmpgt_epi8(offset_digit, _mm256_set1_epi8(-1)),
            _mm256_cmpgt_epi8(six, offset_digit),
        );

        // Check validity
        let is_valid = _mm256_or_si256(_mm256_or_si256(is_upper, is_lower), is_digit);
        let all_valid = _mm256_movemask_epi8(is_valid) == -1i32;

        // Calculate values
        let upper_val = offset_upper;
        let lower_val = offset_lower;
        let digit_val = _mm256_add_epi8(offset_digit, twenty_six);

        let result = _mm256_blendv_epi8(
            _mm256_blendv_epi8(lower_val, digit_val, is_digit),
            upper_val,
            is_upper,
        );

        (result, all_valid)
    }
}

/// Convert ASCII to 5-bit values using SSE.
#[target_feature(enable = "avx2")]
#[inline]
unsafe fn ascii_to_values_sse(input: __m128i, is_hex: bool) -> (__m128i, bool) {
    if is_hex {
        let ascii_zero = _mm_set1_epi8(b'0' as i8);
        let ascii_a = _mm_set1_epi8(b'A' as i8);
        let ascii_a_lower = _mm_set1_epi8(b'a' as i8);
        let ten = _mm_set1_epi8(10);
        let twenty_two = _mm_set1_epi8(22);

        let offset_digit = _mm_sub_epi8(input, ascii_zero);
        let is_digit = _mm_and_si128(
            _mm_cmpgt_epi8(offset_digit, _mm_set1_epi8(-1)),
            _mm_cmpgt_epi8(ten, offset_digit),
        );

        let offset_upper = _mm_sub_epi8(input, ascii_a);
        let is_upper = _mm_and_si128(
            _mm_cmpgt_epi8(offset_upper, _mm_set1_epi8(-1)),
            _mm_cmpgt_epi8(twenty_two, offset_upper),
        );

        let offset_lower = _mm_sub_epi8(input, ascii_a_lower);
        let is_lower = _mm_and_si128(
            _mm_cmpgt_epi8(offset_lower, _mm_set1_epi8(-1)),
            _mm_cmpgt_epi8(twenty_two, offset_lower),
        );

        let is_valid = _mm_or_si128(_mm_or_si128(is_digit, is_upper), is_lower);
        let all_valid = _mm_movemask_epi8(is_valid) == 0xFFFF;

        let digit_val = offset_digit;
        let upper_val = _mm_add_epi8(offset_upper, ten);
        let lower_val = _mm_add_epi8(offset_lower, ten);

        let result = _mm_blendv_epi8(
            _mm_blendv_epi8(lower_val, upper_val, is_upper),
            digit_val,
            is_digit,
        );

        (result, all_valid)
    } else {
        let ascii_a = _mm_set1_epi8(b'A' as i8);
        let ascii_a_lower = _mm_set1_epi8(b'a' as i8);
        let ascii_two = _mm_set1_epi8(b'2' as i8);
        let twenty_six = _mm_set1_epi8(26);
        let six = _mm_set1_epi8(6);

        let offset_upper = _mm_sub_epi8(input, ascii_a);
        let is_upper = _mm_and_si128(
            _mm_cmpgt_epi8(offset_upper, _mm_set1_epi8(-1)),
            _mm_cmpgt_epi8(twenty_six, offset_upper),
        );

        let offset_lower = _mm_sub_epi8(input, ascii_a_lower);
        let is_lower = _mm_and_si128(
            _mm_cmpgt_epi8(offset_lower, _mm_set1_epi8(-1)),
            _mm_cmpgt_epi8(twenty_six, offset_lower),
        );

        let offset_digit = _mm_sub_epi8(input, ascii_two);
        let is_digit = _mm_and_si128(
            _mm_cmpgt_epi8(offset_digit, _mm_set1_epi8(-1)),
            _mm_cmpgt_epi8(six, offset_digit),
        );

        let is_valid = _mm_or_si128(_mm_or_si128(is_upper, is_lower), is_digit);
        let all_valid = _mm_movemask_epi8(is_valid) == 0xFFFF;

        let upper_val = offset_upper;
        let lower_val = offset_lower;
        let digit_val = _mm_add_epi8(offset_digit, twenty_six);

        let result = _mm_blendv_epi8(
            _mm_blendv_epi8(lower_val, digit_val, is_digit),
            upper_val,
            is_upper,
        );

        (result, all_valid)
    }
}

/// Convert 32 5-bit values to 20 bytes.
#[target_feature(enable = "avx2")]
#[inline]
unsafe fn decode_32_values_to_20_bytes(values: __m256i) -> [u8; 20] {
    let val_bytes: [u8; 32] = std::mem::transmute(values);
    let mut out_bytes = [0u8; 20];

    // Process 4 groups of 8 values -> 5 bytes each
    for g in 0..4 {
        let i = g * 8;
        let v0 = val_bytes[i];
        let v1 = val_bytes[i + 1];
        let v2 = val_bytes[i + 2];
        let v3 = val_bytes[i + 3];
        let v4 = val_bytes[i + 4];
        let v5 = val_bytes[i + 5];
        let v6 = val_bytes[i + 6];
        let v7 = val_bytes[i + 7];

        let o = g * 5;
        out_bytes[o] = (v0 << 3) | (v1 >> 2);
        out_bytes[o + 1] = (v1 << 6) | (v2 << 1) | (v3 >> 4);
        out_bytes[o + 2] = (v3 << 4) | (v4 >> 1);
        out_bytes[o + 3] = (v4 << 7) | (v5 << 2) | (v6 >> 3);
        out_bytes[o + 4] = (v6 << 5) | v7;
    }

    out_bytes
}

/// Convert 16 5-bit values to 10 bytes.
#[target_feature(enable = "avx2")]
#[inline]
unsafe fn decode_16_values_to_10_bytes(values: __m128i) -> [u8; 10] {
    let val_bytes: [u8; 16] = std::mem::transmute(values);
    let mut out_bytes = [0u8; 10];

    // Process 2 groups of 8 values -> 5 bytes each
    for g in 0..2 {
        let i = g * 8;
        let v0 = val_bytes[i];
        let v1 = val_bytes[i + 1];
        let v2 = val_bytes[i + 2];
        let v3 = val_bytes[i + 3];
        let v4 = val_bytes[i + 4];
        let v5 = val_bytes[i + 5];
        let v6 = val_bytes[i + 6];
        let v7 = val_bytes[i + 7];

        let o = g * 5;
        out_bytes[o] = (v0 << 3) | (v1 >> 2);
        out_bytes[o + 1] = (v1 << 6) | (v2 << 1) | (v3 >> 4);
        out_bytes[o + 2] = (v3 << 4) | (v4 >> 1);
        out_bytes[o + 3] = (v4 << 7) | (v5 << 2) | (v6 >> 3);
        out_bytes[o + 4] = (v6 << 5) | v7;
    }

    out_bytes
}
