#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

pub const VECTOR_SIZE: usize = std::mem::size_of::<__m128i>();

#[allow(dead_code)]
#[inline]
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse4.1")]
/// Checks that all the bytes are valid digits
pub unsafe fn check_all_chars_are_valid(string: &str) -> bool {
    if string.len() < VECTOR_SIZE {
        return crate::fallback::check_all_chars_are_valid(string);
    }
    // initialize the constants, saddly these cannot be actual constants,
    // because we have to initialize xmm registers, and the System V callin convention (https://wiki.osdev.org/System_V_ABI)
    // requires that the callee should set all the xmm register to zero before returning to the caller.
    let zeros = _mm_set1_epi8(b'0' as i8 - 1); // the -1 is because we don't have a >= instruction, we only have the >
    let nines = _mm_set1_epi8(b'9' as i8 + 1); // the +1 is because we don't have a <= instruction, we only have the <

    // Load the data
    let value = _mm_loadu_si128(string.as_ptr() as _);
    // compare the values with the upper and lower bounds
    let bytes_bigger_or_equal_than_zero_mask = _mm_cmpgt_epi8(value, zeros);
    let bytes_smaller_or_equal_than_nine_mask = _mm_cmplt_epi8(value, nines);

    // and the two masks to get the valid bytes
    let valid_bytes_mask = _mm_and_si128(
        bytes_bigger_or_equal_than_zero_mask,
        bytes_smaller_or_equal_than_nine_mask,
    );

    // return if all the bits are 1s
    _mm_test_all_ones(valid_bytes_mask) == 1
}

#[inline]
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse2")]
/// Finds the last digit value in the string, and compute the parsing mask.
///
/// When the string is composed of all digits, then the returned index will be
/// 32, i.e a parsing mask made up of all zeros.
/// This method *assumes* that the string has exactly 16 chars and it's padded
/// with zeros if necessary.
pub unsafe fn last_byte_digit(string: &str, separator: u8, eol: u8) -> (u32, __m128i) {
    // create costant registers
    let commas = _mm_set1_epi8(separator as i8);
    let newlines = _mm_set1_epi8(eol as i8);

    // load data into memory
    let value = _mm_loadu_si128(string.as_ptr() as _);

    // compare for equality and find the occurences of commas and newlines
    let comma_mask = _mm_cmpeq_epi8(value, commas);
    let newline_mask = _mm_cmpeq_epi8(value, newlines);

    // create the OR of the two regiters to place the first index correctly
    let or_comma_newline = _mm_or_si128(comma_mask, newline_mask);

    // creates a mask from the most significant bit of each 8-bit element,
    // and stores the result in an int
    let movemask = _mm_movemask_epi8(or_comma_newline);
    // the trailing zeros of the mask are the number of digits before the
    // separator in a little endian format
    let index = movemask.trailing_zeros();

    let mask = propagate(or_comma_newline);
    (index, mask)
}

#[inline]
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse2")]
/// Propagates the input mask to the left.
unsafe fn propagate(mut v: __m128i) -> __m128i {
    v = _mm_or_si128(v, _mm_slli_si128(v as _, 1) as _);
    v = _mm_or_si128(v, _mm_slli_si128(v as _, 2) as _);
    v = _mm_or_si128(v, _mm_slli_si128(v as _, 4) as _);
    v = _mm_or_si128(v, _mm_slli_si128(v as _, 8) as _);
    v
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse4.1")]
/// Parses 8 integers from input string using SIMD instructons.
///
/// The input string *must have* at least 16 chars, otherwise the internal
/// operations will load memory outside the string bound.
pub unsafe fn parse_8_chars_simd(s: &str) -> u32 {
    let mut chunk = _mm_lddqu_si128(s.as_ptr() as *const _);
    // do not touch last 8 chars, since we don't know what they contain, avoiding
    // any kind of underflow
    let zeros = _mm_set_epi8(
        0, 0, 0, 0, 0, 0, 0, 0, b'0' as i8, b'0' as i8, b'0' as i8, b'0' as i8, b'0' as i8,
        b'0' as i8, b'0' as i8, b'0' as i8,
    );
    chunk = _mm_sub_epi16(chunk, zeros);

    let mult = _mm_set_epi8(0, 0, 0, 0, 0, 0, 0, 0, 1, 10, 1, 10, 1, 10, 1, 10);
    chunk = _mm_maddubs_epi16(chunk, mult);

    let mult = _mm_set_epi16(1, 100, 1, 100, 1, 100, 1, 100);
    chunk = _mm_madd_epi16(chunk, mult);

    chunk = _mm_packus_epi32(chunk, chunk);
    let mult = _mm_set_epi16(1, 10000, 1, 10000, 1, 10000, 1, 10000);
    chunk = _mm_madd_epi16(chunk, mult);

    let chunk = _mm_cvtsi128_si32(chunk) as u32;
    chunk
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse4.1")]
pub unsafe fn parse_integer_simd_all_numbers(s: &str) -> u32 {
    let mut chunk = _mm_lddqu_si128(s.as_ptr() as *const _);
    let zeros = _mm_set1_epi8(b'0' as i8);
    chunk = _mm_sub_epi16(chunk, zeros);

    let mult = _mm_set_epi8(1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10);
    chunk = _mm_maddubs_epi16(chunk, mult);

    let mult = _mm_set_epi16(1, 100, 1, 100, 1, 100, 1, 100);
    chunk = _mm_madd_epi16(chunk, mult);

    chunk = _mm_packus_epi32(chunk, chunk);

    let mult = _mm_set_epi16(0, 0, 0, 0, 1, 10000, 1, 10000);
    chunk = _mm_madd_epi16(chunk, mult);

    let chunk = _mm_cvtsi128_si64(chunk) as u64;
    (((chunk & 0xffffffff) * 100000000) + (chunk >> 32)) as u32
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse4.1")]
pub unsafe fn parse_less_than_8_simd(s: &str, scaling_factor: u32, mask: __m128i) -> u32 {
    let mut chunk = _mm_lddqu_si128(s.as_ptr() as *const _);
    let zeros = _mm_set1_epi8(b'0' as i8);
    chunk = _mm_sub_epi16(chunk, zeros);

    // remove the unwanted part of the number to not parse
    chunk = _mm_andnot_si128(mask, chunk);

    let mult = _mm_set_epi8(0, 0, 0, 0, 0, 0, 0, 0, 1, 10, 1, 10, 1, 10, 1, 10);
    chunk = _mm_maddubs_epi16(chunk, mult);

    let mult = _mm_set_epi16(1, 100, 1, 100, 1, 100, 1, 100);
    chunk = _mm_madd_epi16(chunk, mult);

    chunk = _mm_packus_epi32(chunk, chunk);
    let mult = _mm_set_epi16(1, 10000, 1, 10000, 1, 10000, 1, 10000);
    chunk = _mm_madd_epi16(chunk, mult);

    let chunk = _mm_cvtsi128_si32(chunk) as u32;
    chunk / scaling_factor
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse4.1")]
/// Parses an integer of at least 8 digits from the string with SIMD.
///
/// This method *assumes* that the input string has exactly 16 chars, eventually
/// padded with zeros.
pub unsafe fn parse_more_than_8_simd(s: &str, scaling_factor: u64, mask: __m128i) -> u32 {
    let mut chunk = _mm_lddqu_si128(std::mem::transmute_copy(&s));
    let zeros = _mm_set1_epi8(b'0' as i8);
    chunk = _mm_sub_epi16(chunk, zeros);

    chunk = _mm_andnot_si128(mask, chunk);

    let mult = _mm_set_epi8(1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10);
    chunk = _mm_maddubs_epi16(chunk, mult);

    let mult = _mm_set_epi16(1, 100, 1, 100, 1, 100, 1, 100);
    chunk = _mm_madd_epi16(chunk, mult);

    chunk = _mm_packus_epi32(chunk, chunk);

    let mult = _mm_set_epi16(0, 0, 0, 0, 1, 10000, 1, 10000);
    chunk = _mm_madd_epi16(chunk, mult);

    let chunk = _mm_cvtsi128_si64(chunk) as u64;
    ((((chunk & 0xffffffff) * 100000000) + (chunk >> 32)) / scaling_factor) as u32
}
