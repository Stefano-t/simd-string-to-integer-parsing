#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

pub(super) const VECTOR_SIZE: usize = std::mem::size_of::<__m256i>(); // 32

#[inline]
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
/// Checks that all the bytes are valid digits
pub(super) unsafe fn check_all_chars_are_valid(string: &str) -> bool {
    if string.len() < VECTOR_SIZE {
        return crate::fallback::check_all_chars_are_valid(string);
    }
    let value = _mm256_loadu_si256(string.as_ptr() as *const _);
    // the -1 is because we don't have a >= instruction, we only have the >
    let zeros = _mm256_set1_epi8(b'0' as i8 - 1);
    // the +1 is because we don't have a <= instruction, we only have the <
    let nines = _mm256_set1_epi8(b'9' as i8 + 1);

    // compare the values with the upper and lower bounds
    let bytes_bigger_or_equal_than_zero_mask = _mm256_cmpgt_epi8(value, zeros);
    // we need to swap the operands since AVX2 hasn't got `less than` operation
    let bytes_smaller_or_equal_than_nine_mask = _mm256_cmpgt_epi8(nines, value);

    // and the two masks to get the valid bytes
    let valid_bytes_mask = _mm256_and_si256(
        bytes_bigger_or_equal_than_zero_mask,
        bytes_smaller_or_equal_than_nine_mask,
    );

    // when the mask is composed by all 1s, the `movemask` loads into in a i32
    // all most significant bit of 8-bit element, and crates an i32 made of all
    // 1s, i.e. -1 in two's complement
    _mm256_movemask_epi8(valid_bytes_mask) == -1
}

#[inline]
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
pub(super) unsafe fn last_byte_digit(string: &str, separator: u8, eol: u8) -> u32 {
    // create costant registers
    let commas = _mm256_set1_epi8(separator as i8);
    let newlines = _mm256_set1_epi8(eol as i8);

    // load data into memory
    let value = _mm256_loadu_si256(string.as_ptr() as *const _);

    // compare for equality and find the occurences of commas and newlines
    let comma_mask = _mm256_cmpeq_epi8(value, commas);
    let newline_mask = _mm256_cmpeq_epi8(value, newlines);

    // create the OR of the two regiters to place the first index correctly
    let or_comma_newline = _mm256_or_si256(comma_mask, newline_mask);

    // creates a mask from the most significant bit of each 8-bit element,
    // and stores the result in an int
    let movemask = _mm256_movemask_epi8(or_comma_newline);
    // the trailing zeros of the mask are the number of digits before the
    // separator in a little endian format
    movemask.trailing_zeros()
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
pub(super) unsafe fn parse_10_chars_simd(s: &str) -> u32 {
    let mut chunk = _mm256_loadu_si256(s.as_ptr() as *const _);
    let zeros = _mm256_set1_epi8(b'0' as i8);
    chunk = _mm256_sub_epi8(chunk, zeros);

    let mult = _mm256_set_epi8(
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 10, 1, 10, 1, 10, 1,
        10, 1, 10,
    );
    chunk = _mm256_maddubs_epi16(chunk, mult);

    let mult = _mm256_set_epi16(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 100, 1, 100);
    chunk = _mm256_madd_epi16(chunk, mult);

    chunk = _mm256_packus_epi32(chunk, chunk);
    let mult = _mm256_set_epi16(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 10000);
    chunk = _mm256_madd_epi16(chunk, mult);
    let chunk = _mm256_extract_epi64(chunk, 0) as u64;
    (((chunk & 0x00000000ffffffff) * 100) + (chunk >> 32)) as u32
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
pub(super) unsafe fn parse_9_chars_simd(s: &str) -> u32 {
    let mut chunk = _mm256_loadu_si256(s.as_ptr() as *const _);
    let zeros = _mm256_set1_epi8(b'0' as i8);
    chunk = _mm256_sub_epi8(chunk, zeros);

    let mult = _mm256_set_epi8(
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 10, 1, 10, 1,
        10, 1, 10,
    );
    chunk = _mm256_maddubs_epi16(chunk, mult);

    let mult = _mm256_set_epi16(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 100, 1, 100);
    chunk = _mm256_madd_epi16(chunk, mult);

    chunk = _mm256_packus_epi32(chunk, chunk);
    let mult = _mm256_set_epi16(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 10000);
    chunk = _mm256_madd_epi16(chunk, mult);
    let chunk = _mm256_extract_epi64(chunk, 0) as u64;
    (((chunk & 0x00000000ffffffff) * 10) + (chunk >> 32)) as u32
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
pub(super) unsafe fn parse_7_chars_simd(s: &str) -> u32 {
    let mut chunk = _mm256_loadu_si256(s.as_ptr() as *const _);
    let zeros = _mm256_set1_epi8(b'0' as i8);
    chunk = _mm256_sub_epi8(chunk, zeros);

    let mult = _mm256_set_epi8(
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 10, 1, 10,
        1, 10,
    );
    chunk = _mm256_maddubs_epi16(chunk, mult);

    let mult = _mm256_set_epi16(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 10, 1, 100);
    chunk = _mm256_madd_epi16(chunk, mult);

    chunk = _mm256_packus_epi32(chunk, chunk);
    let mult = _mm256_set_epi16(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1000);
    chunk = _mm256_madd_epi16(chunk, mult);

    _mm256_cvtsi256_si32(chunk) as u32
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
pub(super) unsafe fn parse_6_chars_simd(s: &str) -> u32 {
    let mut chunk = _mm256_loadu_si256(s.as_ptr() as *const _);
    let zeros = _mm256_set1_epi8(b'0' as i8);
    chunk = _mm256_sub_epi8(chunk, zeros);

    let mult = _mm256_set_epi8(
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 10, 1, 10,
        1, 10,
    );
    chunk = _mm256_maddubs_epi16(chunk, mult);

    let mult = _mm256_set_epi16(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 100);
    chunk = _mm256_madd_epi16(chunk, mult);

    chunk = _mm256_packus_epi32(chunk, chunk);
    let mult = _mm256_set_epi16(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 100);
    chunk = _mm256_madd_epi16(chunk, mult);

    _mm256_cvtsi256_si32(chunk) as u32
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
pub(super) unsafe fn parse_5_chars_simd(s: &str) -> u32 {
    let mut chunk = _mm256_loadu_si256(s.as_ptr() as *const _);
    let zeros = _mm256_set1_epi8(b'0' as i8);
    chunk = _mm256_sub_epi8(chunk, zeros);

    let mult = _mm256_set_epi8(
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 10,
        1, 10,
    );
    chunk = _mm256_maddubs_epi16(chunk, mult);

    let mult = _mm256_set_epi16(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 100);
    chunk = _mm256_madd_epi16(chunk, mult);

    chunk = _mm256_packus_epi32(chunk, chunk);
    let mult = _mm256_set_epi16(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 10);
    chunk = _mm256_madd_epi16(chunk, mult);

    _mm256_cvtsi256_si32(chunk) as u32
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
pub(super) unsafe fn parse_4_chars_simd(s: &str) -> u32 {
    let mut chunk = _mm256_loadu_si256(s.as_ptr() as *const _);
    let zeros = _mm256_set1_epi8(b'0' as i8);
    chunk = _mm256_sub_epi8(chunk, zeros);

    let mult = _mm256_set_epi8(
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 10,
        1, 10,
    );
    chunk = _mm256_maddubs_epi16(chunk, mult);

    let mult = _mm256_set_epi16(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 100);
    chunk = _mm256_madd_epi16(chunk, mult);
    _mm256_cvtsi256_si32(chunk) as u32
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
/// Parses 8 integers from input string using SIMD instructons.
///
/// The input string *must have* at least 32 chars, otherwise the internal
/// operations will load memory outside the string bound.
pub(super) unsafe fn parse_8_chars_simd(s: &str) -> u32 {
    let mut chunk = _mm256_loadu_si256(s.as_ptr() as *const _);
    let zeros = _mm256_set1_epi8(b'0' as i8);
    chunk = _mm256_sub_epi64(chunk, zeros);

    let mult = _mm256_set_epi8(
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 10, 1, 10, 1,
        10, 1, 10,
    );
    chunk = _mm256_maddubs_epi16(chunk, mult);

    let mult = _mm256_set_epi16(
        1, 100, 1, 100, 1, 100, 1, 100, 1, 100, 1, 100, 1, 100, 1, 100,
    );
    chunk = _mm256_madd_epi16(chunk, mult);

    chunk = _mm256_packus_epi32(chunk, chunk);
    let mult = _mm256_set_epi16(
        1, 10000, 1, 10000, 1, 10000, 1, 10000, 1, 10000, 1, 10000, 1, 10000, 1, 10000,
    );
    chunk = _mm256_madd_epi16(chunk, mult);

    let chunk = _mm256_cvtsi256_si32(chunk) as u32;
    chunk
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
/// Parses an u32 from a string padded with zeros.
pub(super) unsafe fn parse_padded_integer_simd_all_numbers(s: &str) -> u32 {
    let mut chunk = _mm256_loadu_si256(s.as_ptr() as *const _);
    let zeros = _mm256_set1_epi8(b'0' as i8);
    chunk = _mm256_sub_epi16(chunk, zeros);

    let mult = _mm256_set_epi8(
        1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10,
        1, 10, 1, 10, 1, 10,
    );
    chunk = _mm256_maddubs_epi16(chunk, mult);

    let mult = _mm256_set_epi16(
        1, 100, 1, 100, 1, 100, 1, 100, 1, 100, 1, 100, 1, 100, 1, 100,
    );
    chunk = _mm256_madd_epi16(chunk, mult);

    chunk = _mm256_packus_epi32(chunk, chunk);

    let mult = _mm256_set_epi16(
        1, 10000, 1, 10000, 1, 10000, 1, 10000, 1, 10000, 1, 10000, 1, 10000, 1, 10000,
    );
    chunk = _mm256_madd_epi16(chunk, mult);

    // the index 3 is used because, since the string is padded with
    // 0s, the number is located in the 64 rightmost bits
    let chunk = _mm256_extract_epi64(chunk, 3) as u64;
    (((chunk & 0xffffffff) * 100000000) + (chunk >> 32)) as u32
}

#[cfg(test)]
mod tests {
    use crate::avx::*;

    #[test]
    fn test_check_numbers_all_valid_when_true() {
        let s = "11111111111111111111111111111111";
        unsafe {
            assert!(check_all_chars_are_valid(s));
        }
    }

    #[test]
    fn test_check_numbers_all_valid_when_false() {
        let s = "1111111=111111111111111111111111";
        unsafe {
            assert!(!check_all_chars_are_valid(s));
        }
    }

    #[test]
    fn test_last_byte_digit_below_10() {
        let s = "1111111,111111111111111111111111";
        unsafe {
            let last = last_byte_digit(s, b',', b'\n');
            assert_eq!(last, 7);
        }
    }

    #[test]
    fn test_last_byte_digit_no_sep() {
        let s = "11111111111111111111111111111111";
        unsafe {
            let last = last_byte_digit(s, b',', b'\n');
            assert_eq!(last, 32);
        }
    }

    #[test]
    fn test_last_byte_digit_multiple_sep() {
        let s = "11111,1111\n111111111111111111111";
        unsafe {
            let last = last_byte_digit(s, b',', b'\n');
            assert_eq!(last, 5);
        }
    }

    #[test]
    fn test_parse_10_chars_simd() {
        let s = "12345678911111111111111111111111";
        unsafe {
            assert_eq!(parse_10_chars_simd(s), 1234567891);
        }
    }

    #[test]
    fn test_parse_9_chars_simd() {
        let s = "12345678911111111111111111111111";
        unsafe {
            assert_eq!(parse_9_chars_simd(s), 123456789);
        }
    }

    #[test]
    fn test_parse_8_chars_simd() {
        let s = "12345678111111111111111111111111";
        unsafe {
            assert_eq!(parse_8_chars_simd(s), 12345678);
        }
    }

    #[test]
    fn test_parse_7_chars_simd() {
        let s = "12345678111111111111111111111111";
        unsafe {
            assert_eq!(parse_7_chars_simd(s), 1234567);
        }
    }

    #[test]
    fn test_parse_6_chars_simd() {
        let s = "12345678111111111111111111111111";
        unsafe {
            assert_eq!(parse_6_chars_simd(s), 123456);
        }
    }

    #[test]
    fn test_parse_5_chars_simd() {
        let s = "12345678111111111111111111111111";
        unsafe {
            assert_eq!(parse_5_chars_simd(s), 12345);
        }
    }

    #[test]
    fn test_parse_4_chars_simd() {
        let s = "12345678111111111111111111111111";
        unsafe {
            assert_eq!(parse_4_chars_simd(s), 1234);
        }
    }
}
