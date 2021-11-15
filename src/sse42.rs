//! SSE4.2 implementation for parsing an u32 from a string

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

/// Size of _m128i data type
pub(super) const VECTOR_SIZE: usize = std::mem::size_of::<__m128i>();

/// Byte array to determine if the chars in a string are in range '0'..'9'
const NUMERIC_RANGE: &[u8; 16] = b"09\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
/// Byte array to determine if a string is made of all numbers
const NUMERIC_VALUES: &[u8; 16] = b"1234567890\0\0\0\0\0\0";

/// Returns true if the string is composed by only digits
/// 
/// # Safety
/// 
/// Since this function is enabled only when SSE4.2 cpu flag is detected, it
/// will be called only in this circumstance. The intrinics work with a string
/// of at least length 16: in case of less chars, an iterative process will be
/// called.
#[target_feature(enable = "sse4.2")]
pub unsafe fn check_all_chars_are_valid(s: &str) -> bool {
    if s.len() < VECTOR_SIZE {
        return crate::fallback::check_all_chars_are_valid(s);
    }
    let to_cmp = _mm_loadu_si128(s.as_ptr() as *const _);
    let range = _mm_loadu_si128(NUMERIC_RANGE.as_ptr() as *const _);
    let idx = _mm_cmpistri(
        range,
        to_cmp,
        _SIDD_CMP_RANGES | _SIDD_MASKED_NEGATIVE_POLARITY,
    );
    idx == 16
}

/// Returns the index of the last digit in the string
/// 
/// In case of a string made of all numbers, the call to the SSE4.2 will return
/// 32, since the mask has value 0. This happens only when the string has length
/// at least 16 and the intrinisic is called
/// 
/// # Safety
/// 
/// Since this function is enabled only when SSE4.2 cpu flag is detected, it
/// will be called only in this circumstance. The intrinics work with a string
/// of at least length 16: in case of less chars, an iterative process will be
/// called.
#[target_feature(enable = "sse4.2")]
pub unsafe fn last_digit_byte(s: &str) -> u32 {
    if s.len() < VECTOR_SIZE {
        return crate::fallback::last_digit_byte(s);
    }
    let to_cmp = _mm_loadu_si128(s.as_ptr() as *const _);
    let valid_nums = _mm_loadu_si128(NUMERIC_VALUES.as_ptr() as *const _);
    let mask = _mm_cmpistrm(
        valid_nums,
        to_cmp,
        // cmp for any match | negate the result | create a byte mask
        _SIDD_CMP_EQUAL_ANY | _SIDD_NEGATIVE_POLARITY | _SIDD_UNIT_MASK,
    );
    // translate the mask into an integer
    let idx = _mm_movemask_epi8(mask);
    idx.trailing_zeros()
}

/// Returns the index of the last char in the string different from `separator`
/// and `eol`
/// 
/// In case of a string without the given separators, the call to the SSE4.2
/// will return 32, since the mask has value 0. This happens only when the
/// string has length at least 16 and the intrinisic is called
/// 
/// # Safety
/// 
/// Since this function is enabled only when SSE4.2 cpu flag is detected, it
/// will be called only in this circumstance. The intrinics work with a string
/// of at least length 16: in case of less chars, an iterative process will be
/// called.
#[target_feature(enable = "sse4.2")]
pub unsafe fn last_byte_without_separator(s: &str, separator: u8, eol: u8) -> u32 {
    if s.len() < VECTOR_SIZE {
        return crate::fallback::last_byte_without_separator(s, separator, eol);
    }
    let to_cmp = _mm_loadu_si128(s.as_ptr() as *const _);
    let valid_nums = _mm_loadu_si128(NUMERIC_VALUES.as_ptr() as *const _);
    let mask = _mm_cmpistrm(
        valid_nums,
        to_cmp,
        // cmp for any match | negate the result | create a byte mask
        _SIDD_CMP_EQUAL_ANY | _SIDD_NEGATIVE_POLARITY | _SIDD_UNIT_MASK,
    );
    // translate the mask into an integer
    let idx = _mm_movemask_epi8(mask);
    idx.trailing_zeros()
}

#[cfg(test)]
mod tests {
    use super::*;
    static SEP: u8 = b',';
    static EOL: u8 = b'\n';

    #[test]
    fn last_byte_without_separator_no_digit() {
        let s = ",1234.4321\n    ";
        unsafe {
            assert_eq!(last_byte_without_separator(s, SEP, EOL), 0);
        }
    }

    #[test]
    fn last_byte_without_separator_one_digit() {
        let s = "1,2343211234432542";
        unsafe {
            assert_eq!(last_byte_without_separator(s, SEP, EOL), 1);
        }
    }

    #[test]
    fn last_byte_without_separator_more_digits() {
        let s = "123,44321\n12345";
        unsafe {
            assert_eq!(last_byte_without_separator(s, SEP, EOL), 3);
        }
    }
    
    #[test]
    fn last_digit_byte_all_numbers() {
        let s = "1239443218123459";
        unsafe {
            assert_eq!(last_digit_byte(s), 32);
        }
    }

    #[test]
    fn last_digit_byte_no_number() {
        let s = "/2.944321812345";
        unsafe {
            assert_eq!(last_digit_byte(s), 0);
        }
    }

    #[test]
    fn last_digit_byte_some_digits() {
        let s = "129,44321812345";
        unsafe {
            assert_eq!(last_digit_byte(s), 3);
        }
    }

    #[test]
    fn check_all_chars_are_valid_valid() {
        let s = "1234567890123456";
        unsafe {
            assert!(check_all_chars_are_valid(s));
        }
    }

    #[test]
    fn check_all_chars_are_valid_invalid() {
        let s = "123456789,123456";
        unsafe {
            assert!(!check_all_chars_are_valid(s));
        }
    }
}
