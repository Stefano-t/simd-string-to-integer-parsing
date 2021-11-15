//! SSE4.1 implementations for parsing a u32 from a string.

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

/// Size of __m128i data type
pub(super) const VECTOR_SIZE: usize = std::mem::size_of::<__m128i>();

/// Checks that all the bytes are valid digits
#[allow(dead_code)]
#[inline]
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse4.1")]
pub(super) unsafe fn check_all_chars_are_valid(string: &str) -> bool {
    if string.len() < VECTOR_SIZE {
        return crate::fallback::check_all_chars_are_valid(string);
    }
    // since `last_digit_byte` counts the trailing zeros of the resulting mask,
    // if the mask is made of all 0s, meaning that the string is made of all
    // digits, the results will be 32, i.e. a u32 mask with all 0s
    last_digit_byte(string) == 32 
}

/// Returns the index of the last digit in the string
/// 
/// In case of a string made composed by all digits, the SSE4.1 implementation
/// without fallback call will return 32.
#[inline]
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse4.1")]
pub(super) unsafe fn last_digit_byte(s: &str) -> u32 {
    if s.len() < VECTOR_SIZE {
        return crate::fallback::last_digit_byte(s);
    }
    // initialize the constants
    let zeros = _mm_set1_epi8(b'0' as i8);
    let nines = _mm_set1_epi8(b'9' as i8); 

    // Load the data
    let value = _mm_loadu_si128(s.as_ptr() as _);
    // compare the values with the upper and lower bounds
    let bytes_bigger_or_equal_than_zero_mask = _mm_cmplt_epi8(value, zeros);
    let bytes_smaller_or_equal_than_nine_mask = _mm_cmpgt_epi8(value, nines);

    // OR the two masks to get the valid bytes
    let valid_bytes_mask = _mm_or_si128(
        bytes_bigger_or_equal_than_zero_mask,
        bytes_smaller_or_equal_than_nine_mask,
    );

    // load the most significant bit of each byte and count the trainling zeros
    _mm_movemask_epi8(valid_bytes_mask).trailing_zeros() 
}

/// Returns the index of the last char in the string different from `separator`
/// and `eol`
///
/// When the string is composed of all digits, then the returned index will be
/// 32, i.e a parsing mask made up of all zeros.
/// This method *assumes* that the string has exactly 16 chars and it's padded
/// with zeros if necessary.
#[inline]
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse2")]
pub(super) unsafe fn last_byte_without_separator(string: &str, separator: u8, eol: u8) -> u32 {
    if string.len() < VECTOR_SIZE {
        return crate::fallback::last_byte_without_separator(
            string,
            separator,
            eol);
    }
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
    movemask.trailing_zeros()
}

/// Parses 8 integers from input string using SIMD instructions.
///
/// The input string *must have* at least 16 chars, otherwise the internal
/// operations will load memory outside the string bound.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse4.1")]
pub(super) unsafe fn parse_8_chars_simd(s: &str) -> u32 {
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

/// Parses an u32 from the given string made of all numbers.
///
/// The input string *must have* at least 16 chars, otherwise the internal
/// operations will load memory outside the string bound.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse4.1")]
pub(super) unsafe fn parse_integer_simd_all_numbers(s: &str) -> u32 {
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

/// Parses 5 integers from input string using SIMD instructions.
///
/// The input string *must have* at least 16 chars, otherwise the internal
/// operations will load memory outside the string bound.#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse4.1")]
pub(super) unsafe fn parse_5_chars_simd(s: &str) -> u32 {
    let mut chunk = _mm_loadu_si128(s.as_ptr() as *const _);
    let zeros = _mm_set1_epi8(b'0' as i8);
    chunk = _mm_sub_epi16(chunk, zeros);

    let mult = _mm_set_epi8(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 10, 1, 10);
    chunk = _mm_maddubs_epi16(chunk, mult);
    // make first two 16 bit chunks a 4 digit number, while leaving the next 16
    // bits untouched
    let mult = _mm_set_epi16(0, 0, 0, 0, 0, 1, 1, 100);
    chunk = _mm_madd_epi16(chunk, mult);

    chunk = _mm_packus_epi32(chunk, chunk);
    // make room for the 1 digit number and sum them up
    let mult = _mm_set_epi16(0, 0, 0, 0, 0, 0, 1, 10);
    chunk = _mm_madd_epi16(chunk, mult);

    _mm_cvtsi128_si32(chunk) as u32
}

/// Parses 4 integers from input string using SIMD instructions.
///
/// The input string *must have* at least 16 chars, otherwise the internal
/// operations will load memory outside the string bound.#[cfg(target_arch = "x86_64")]
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse4.1")]
pub(super) unsafe fn parse_4_chars_simd(s: &str) -> u32 {
    let mut chunk = _mm_loadu_si128(s.as_ptr() as *const _);
    let zeros = _mm_set1_epi8(b'0' as i8);
    chunk = _mm_sub_epi16(chunk, zeros);

    let mult = _mm_set_epi8(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 10, 1, 10);
    chunk = _mm_maddubs_epi16(chunk, mult);
    // make first two 16 bit chunks a 4 digit number, while leaving the next 16
    // bits untouched
    let mult = _mm_set_epi16(0, 0, 0, 0, 0, 0, 1, 100);
    chunk = _mm_madd_epi16(chunk, mult);
    _mm_cvtsi128_si32(chunk) as u32
}

/// Parses 6 integers from input string using SIMD instructions.
///
/// The input string *must have* at least 16 chars, otherwise the internal
/// operations will load memory outside the string bound.#[cfg(target_arch = "x86_64")]
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse4.1")]
pub(super) unsafe fn parse_6_chars_simd(s: &str) -> u32 {
    let mut chunk = _mm_loadu_si128(s.as_ptr() as *const _);
    let zeros = _mm_set1_epi8(b'0' as i8);
    chunk = _mm_sub_epi16(chunk, zeros);

    let mult = _mm_set_epi8(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 10, 1, 10, 1, 10);
    chunk = _mm_maddubs_epi16(chunk, mult);
    // make first two 16 bit chunks a 4 digit number, while leaving the next 16
    // bits untouched
    let mult = _mm_set_epi16(0, 0, 0, 0, 0, 1, 1, 100);
    chunk = _mm_madd_epi16(chunk, mult);

    chunk = _mm_packus_epi32(chunk, chunk);
    // make room for the 2 digit number and sum them up
    let mult = _mm_set_epi16(0, 0, 0, 0, 0, 0, 1, 100);
    chunk = _mm_madd_epi16(chunk, mult);

    _mm_cvtsi128_si32(chunk) as u32
}

/// Parses 7 integers from input string using SIMD instructions.
///
/// The input string *must have* at least 16 chars, otherwise the internal
/// operations will load memory outside the string bound.#[cfg(target_arch = "x86_64")]
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse4.1")]
pub(super) unsafe fn parse_7_chars_simd(s: &str) -> u32 {
    let mut chunk = _mm_loadu_si128(s.as_ptr() as *const _);
    let zeros = _mm_set1_epi8(b'0' as i8);
    chunk = _mm_sub_epi16(chunk, zeros);

    let mult = _mm_set_epi8(0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 10, 1, 10, 1, 10);
    chunk = _mm_maddubs_epi16(chunk, mult);
    // make first two 16 bit chunks a 4 digit number, while leaving the next 16
    // bits untouched
    let mult = _mm_set_epi16(0, 0, 0, 0, 1, 10, 1, 100);
    chunk = _mm_madd_epi16(chunk, mult);

    chunk = _mm_packus_epi32(chunk, chunk);
    // make room for the 3 digit number and sum them up
    let mult = _mm_set_epi16(0, 0, 0, 0, 0, 0, 1, 1000);
    chunk = _mm_madd_epi16(chunk, mult);

    _mm_cvtsi128_si32(chunk) as u32
}

/// Parses 9 integers from input string using SIMD instructions.
///
/// The input string *must have* at least 16 chars, otherwise the internal
/// operations will load memory outside the string bound.#[cfg(target_arch = "x86_64")]
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse4.1")]
pub(super) unsafe fn parse_9_chars_simd(s: &str) -> u32 {
    let mut chunk = _mm_loadu_si128(s.as_ptr() as *const _);
    let zeros = _mm_set1_epi8(b'0' as i8);
    chunk = _mm_sub_epi16(chunk, zeros);

    // in all the below `_mm_set_epi8` operations, the first 1 starting from
    // left ensures that the last digit among the 9 is untouched
    let mult = _mm_set_epi8(0, 0, 0, 0, 0, 0, 0, 1, 1, 10, 1, 10, 1, 10, 1, 10);
    let chunk = _mm_maddubs_epi16(chunk, mult);

    let mult = _mm_set_epi16(0, 0, 0, 1, 1, 100, 1, 100);
    let chunk = _mm_madd_epi16(chunk, mult);

    let chunk = _mm_packus_epi32(chunk, chunk);
    let mult = _mm_set_epi16(0, 0, 0, 0, 0, 1, 1, 10000);
    let chunk = _mm_madd_epi16(chunk, mult);

    let chunk = _mm_cvtsi128_si64(chunk) as u64;
    // make room to place the remeaning digit
    (((chunk & 0x00000000ffffffff) * 10) + (chunk >> 32)) as u32
}

/// Parses 10 integers from input string using SIMD instructions.
///
/// The input string *must have* at least 16 chars, otherwise the internal
/// operations will load memory outside the string bound.#[cfg(target_arch = "x86_64")]
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse4.1")]
pub(super) unsafe fn parse_10_chars_simd(s: &str) -> u32 {
    let mut chunk = _mm_loadu_si128(s.as_ptr() as *const _);
    let zeros = _mm_set1_epi8(b'0' as i8);
    chunk = _mm_sub_epi16(chunk, zeros);

    let mult = _mm_set_epi8(0, 0, 0, 0, 0, 0, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10);
    let chunk = _mm_maddubs_epi16(chunk, mult);

    // in all the below `_mm_set_epi8` operations, the first 1 starting from
    // left ensures that the last digit among the 9 is untouched
    let mult = _mm_set_epi16(0, 0, 0, 1, 1, 100, 1, 100);
    let chunk = _mm_madd_epi16(chunk, mult);

    let chunk = _mm_packus_epi32(chunk, chunk);
    let mult = _mm_set_epi16(0, 0, 0, 0, 0, 1, 1, 10000);
    let chunk = _mm_madd_epi16(chunk, mult);

    let chunk = _mm_cvtsi128_si64(chunk) as u64;
    // make room to place the 2 remeaning digits
    (((chunk & 0x00000000ffffffff) * 100) + (chunk >> 32)) as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    static SEP: u8 = b',';
    static EOL: u8 = b'\n';

    #[test]
    fn check_all_chars_are_valid_simd_valid() {
        let s = "1234567890123456";
        unsafe {
            assert!(check_all_chars_are_valid(s));
        }
    }

    #[test]
    fn check_all_chars_are_valid_simd_invalid() {
        let s = "123456789,123456";
        unsafe {
            assert!(!check_all_chars_are_valid(s));
        }
    }

    #[test]
    fn last_byte_without_separator_first_digit() {
        let s = "1,23456789123456";
        unsafe {
            assert_eq!(last_byte_without_separator(s, SEP, EOL), 1);
        }
    }

    #[test]
    fn last_byte_without_separator_more_digit() {
        let s = "123456,789123456";
        unsafe {
            assert_eq!(last_byte_without_separator(s, SEP, EOL), 6);
        }
    }

    #[test]
    fn last_byte_without_separator_first_separator() {
        let s = ",123456789123456";
        unsafe {
            assert_eq!(last_byte_without_separator(s, SEP, EOL), 0);
        }
    }

    #[test]
    fn last_digit_byte_all_digits() {
        let s = "0123456789012345";
        unsafe {
            assert_eq!(last_digit_byte(s), 32);
        }
    }

    #[test]
    fn last_digit_byte_no_digit() {
        let s = "/!23456789012345";
        unsafe {
            assert_eq!(last_digit_byte(s), 0);
        }
    }

    #[test]
    fn last_digit_byte_some_digits() {
        let s = "1223!56789012345";
        unsafe {
            assert_eq!(last_digit_byte(s), 4);
        }
    }

    #[test]
    fn test_parse_10_chars_simd() {
        let s = "1234567890123456";
        unsafe {
            assert_eq!(parse_10_chars_simd(s), 1234567890);
        }
    }

    #[test]
    fn test_parse_9_chars_simd() {
        let s = "1234567890123456";
        unsafe {
            assert_eq!(parse_9_chars_simd(s), 123456789);
        }
    }
    #[test]
    fn test_parse_8_chars_simd() {
        let s = "1234567890123456";
        unsafe {
            assert_eq!(parse_8_chars_simd(s), 12345678);
        }
    }

    #[test]
    fn test_parse_7_chars_simd() {
        let s = "1234567890123456";
        unsafe {
            assert_eq!(parse_7_chars_simd(s), 1234567);
        }
    }

    #[test]
    fn test_parse_6_chars_simd() {
        let s = "1234567890123456";
        unsafe {
            assert_eq!(parse_6_chars_simd(s), 123456);
        }
    }

    #[test]
    fn test_parse_5_chars_simd() {
        let s = "1234567890123456";
        unsafe {
            assert_eq!(parse_5_chars_simd(s), 12345);
        }
    }

    #[test]
    fn test_parse_4_chars_simd() {
        let s = "1234567890123456";
        unsafe {
            assert_eq!(parse_4_chars_simd(s), 1234);
        }
    }

    #[test]
    fn parse_integer_simd_all_numbers_only_padding() {
        let s = "0000000000000000";
        unsafe {
            assert_eq!(parse_integer_simd_all_numbers(s), 0);
        }
    }

    #[test]
    fn parse_integer_simd_all_numbers_one_digit_padding() {
        let s = "0000000000000001";
        unsafe {
            assert_eq!(parse_integer_simd_all_numbers(s), 1);
        }
    }

    #[test]
    fn parse_integer_simd_all_numbers_mode_digits_padding() {
        let s = "0000000000012345";
        unsafe {
            assert_eq!(parse_integer_simd_all_numbers(s), 12345);
        }
    }
}
