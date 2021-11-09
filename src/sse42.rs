#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

pub(super) const VECTOR_SIZE: usize = std::mem::size_of::<__m128i>();

// byte arrays to determine if a string is made of all numbers
const NUMERIC_RANGE: &[u8; 16] = b"09\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
const NUMERIC_VALUES: &[u8; 16] = b"1234567890\0\0\0\0\0\0";

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

#[target_feature(enable = "sse4.2")]
pub unsafe fn last_byte_digit(s: &str, separator: u8, eol: u8) -> u32 {
    if s.len() < VECTOR_SIZE {
        return crate::fallback::last_byte_digit(s, separator, eol);
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
    fn last_byte_digit_no_digit() {
        let s = ",1234.4321\n    ";
        unsafe {
            assert_eq!(last_byte_digit(s, SEP, EOL), 0);
        }
    }

    #[test]
    fn last_byte_digit_one_digit() {
        let s = "1,2343211234432542";
        unsafe {
            assert_eq!(last_byte_digit(s, SEP, EOL), 1);
        }
    }

    #[test]
    fn last_byte_digit_more_digits() {
        let s = "123,44321\n12345";
        unsafe {
            assert_eq!(last_byte_digit(s, SEP, EOL), 3);
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
