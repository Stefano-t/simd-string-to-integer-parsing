#![feature(stdsimd)]

pub mod avx;
pub mod fallback;
pub mod sse41;
pub mod sse42;

/// Holds the pointer to the function supporeted by the underlying CPU
static mut LAST_BYTE_DIGIT: unsafe fn(&str, u8, u8) -> u32 = last_byte_digit_dispatcher;

/// Implements a single dispatch method to assign the appropiate function to the
/// global variable LAST_BYTE_DIGIT
fn last_byte_digit_dispatcher(s: &str, separator: u8, eol: u8) -> u32 {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("sse4.2") {
            // repelace the global variable with the pointer to the sse42 function
            unsafe {
                LAST_BYTE_DIGIT = sse42::last_byte_without_separator;
                return sse42::last_byte_without_separator(s, separator, eol);
            }
        }
        if is_x86_feature_detected!("sse4.1") {
            // repelace the global variable with the pointer to the sse41 function
            unsafe {
                LAST_BYTE_DIGIT = sse41::last_byte_without_separator;
                return sse41::last_byte_without_separator(s, separator, eol);
            }
        }
    }

    unsafe {
        LAST_BYTE_DIGIT = fallback::last_byte_without_separator;
    }
    return fallback::last_byte_without_separator(s, separator, eol);
}

/// Returns the index of the last char in the string different from `separator` and `eol`
pub fn last_byte_without_separator(s: &str, separator: u8, eol: u8) -> u32 {
    unsafe { LAST_BYTE_DIGIT(s, separator, eol) }
}

static mut CHECK_CHARS: unsafe fn(&str) -> bool = check_chars_dispatcher;

/// Implements a single dispatch method to assign the appropiate function to the
/// global variable CHECK_CHARS
fn check_chars_dispatcher(s: &str) -> bool {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            unsafe {
                CHECK_CHARS = avx::check_all_chars_are_valid;
                return avx::check_all_chars_are_valid(s);
            }
        }
        if is_x86_feature_detected!("sse4.2") {
            unsafe {
                CHECK_CHARS = sse42::check_all_chars_are_valid;
                return sse42::check_all_chars_are_valid(s);
            }
        }
        if is_x86_feature_detected!("sse4.1") {
            unsafe {
                CHECK_CHARS = sse41::check_all_chars_are_valid;
                return sse41::check_all_chars_are_valid(s);
            }
        }
    }

    unsafe {
        CHECK_CHARS = fallback::check_all_chars_are_valid;
    }
    return fallback::check_all_chars_are_valid(s);
}

/// Deteremines if the string in made of all numbers
pub fn check_all_chars_are_valid(s: &str) -> bool {
    unsafe { CHECK_CHARS(s) }
}

static mut PARSE_INTEGER: unsafe fn(&str, u8, u8) -> Option<u32> = parse_integer_dispatcher;

/// Assigns the correct implementation to the global variable PARSE_INTEGER
fn parse_integer_dispatcher(s: &str, separator: u8, eol: u8) -> Option<u32> {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            unsafe {
                PARSE_INTEGER = parse_integer_avx2;
                return parse_integer_avx2(s, separator, eol);
            }
        }
        if is_x86_feature_detected!("sse4.1") {
            unsafe {
                PARSE_INTEGER = parse_integer_sse41;
                return parse_integer_sse41(s, separator, eol);
            }
        }
    }
    unsafe {
        PARSE_INTEGER = fallback::parse_integer_byte_iterator;
    }
    fallback::parse_integer_byte_iterator(s, separator, eol)
}

/// Parses an integer from the given string
///
/// If the string has length less than 16 chars (or 32 if AVX is used), then no
/// SIMD acceleration is used; in this case, the method resorts to an iterative
/// process to parse the integer.  If the string has at least 16 chars (or 32
/// for AVX), then it can perform parsing exploiting the SIMD intrinsics.
pub fn parse_integer(s: &str, separator: u8, eol: u8) -> Option<u32> {
    unsafe { PARSE_INTEGER(s, separator, eol) }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn parse_integer_avx2(s: &str, separator: u8, eol: u8) -> Option<u32> {
    if s.len() < avx::VECTOR_SIZE {
        return fallback::parse_integer_byte_iterator(s, separator, eol);
    }
    // find the first occurence of a separator
    let index = avx::last_byte_without_separator(s, separator, eol);
    match index {
        8 => return Some(avx::parse_8_chars_simd(s)),
        10 => return Some(avx::parse_10_chars_simd(s)),
        9 => return Some(avx::parse_9_chars_simd(s)),
        7 => return Some(avx::parse_7_chars_simd(s)),
        6 => return Some(avx::parse_6_chars_simd(s)),
        5 => return Some(avx::parse_5_chars_simd(s)),
        4 => return Some(avx::parse_4_chars_simd(s)),
        1..=3 => return Some(fallback::parse_byte_iterator_limited(s, index)),
        // all the chars are numeric, and they should be padded with 0s to get a
        // correct result. If not, the parsed number will not be correct due to
        // internal processing techniques
        32 => return Some(avx::parse_padded_integer_simd_all_numbers(s)),
        // there is no number to parse
        _ => return None,
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse4.1")]
unsafe fn parse_integer_sse41(s: &str, separator: u8, eol: u8) -> Option<u32> {
    if s.len() < sse41::VECTOR_SIZE {
        return fallback::parse_integer_byte_iterator(s, separator, eol);
    }
    // find the first occurence of a separator
    let index = last_byte_without_separator(s, separator, eol);
    match index {
        8 => return Some(sse41::parse_8_chars_simd(s)),
        10 => return Some(sse41::parse_10_chars_simd(s)),
        9 => return Some(sse41::parse_9_chars_simd(s)),
        7 => return Some(sse41::parse_7_chars_simd(s)),
        6 => return Some(sse41::parse_6_chars_simd(s)),
        5 => return Some(sse41::parse_5_chars_simd(s)),
        4 => return Some(sse41::parse_4_chars_simd(s)),
        1..=3 => return Some(fallback::parse_byte_iterator_limited(s, index)),
        // all the chars are numeric, maybe padded?
        32 => return Some(sse41::parse_integer_simd_all_numbers(s)),
        // there is no u32 to parse
        _ => return None,
    }
}

#[cfg(feature = "benchmark")]
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse4.2")]
unsafe fn parse_integer_sse42(s: &str, separator: u8, eol: u8) -> Option<u32> {
    if s.len() < sse41::VECTOR_SIZE {
        return fallback::parse_integer_byte_iterator(s, separator, eol);
    }
    // find the first occurence of a separator
    let index = sse42::last_byte_without_separator(s, separator, eol);
    match index {
        8 => return Some(sse41::parse_8_chars_simd(s)),
        10 => return Some(sse41::parse_10_chars_simd(s)),
        9 => return Some(sse41::parse_9_chars_simd(s)),
        7 => return Some(sse41::parse_7_chars_simd(s)),
        6 => return Some(sse41::parse_6_chars_simd(s)),
        5 => return Some(sse41::parse_5_chars_simd(s)),
        4 => return Some(sse41::parse_4_chars_simd(s)),
        1..=3 => return Some(fallback::parse_byte_iterator_limited(s, index)),
        // all the chars are numeric, maybe padded?
        32 => return Some(sse41::parse_integer_simd_all_numbers(s)),
        // there is no u32 to parse
        _ => return None,
    }
}

#[cfg(feature = "benchmark")]
pub fn safe_parse_integer_sse41(s: &str, separator: u8, eol: u8) -> Option<u32> {
    unsafe {
        return parse_integer_sse41(s, separator, eol);
    }
}

#[cfg(feature = "benchmark")]
pub fn safe_parse_integer_sse42(s: &str, separator: u8, eol: u8) -> Option<u32> {
    unsafe {
        return parse_integer_sse42(s, separator, eol);
    }
}

#[cfg(feature = "benchmark")]
pub fn safe_parse_integer_avx2(s: &str, separator: u8, eol: u8) -> Option<u32> {
    unsafe {
        return parse_integer_avx2(s, separator, eol);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    static SEP: u8 = b',';
    static EOL: u8 = b'\n';

    // ===== fallback tests =====

    #[test]
    fn parse_integer_empty() {
        let s = "";
        assert_eq!(parse_integer(s, SEP, EOL), None);
    }

    #[test]
    fn parse_integer_no_digit() {
        let s = ",,\n123";
        assert_eq!(parse_integer(s, SEP, EOL), None);
    }

    #[test]
    fn parse_integer_one_digit() {
        let s = "1,123,23\n0";
        assert_eq!(parse_integer(s, SEP, EOL), Some(1));
    }

    #[test]
    fn parse_integer_more_digits() {
        let s = "1123,23\n0";
        assert_eq!(parse_integer(s, SEP, EOL), Some(1123));
    }

    #[test]
    fn parse_integer_all_digits() {
        let s = "112323";
        assert_eq!(parse_integer(s, SEP, EOL), Some(112323));
    }

    // ===== AVX2 tests =====

    #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
    #[test]
    fn check_all_chars_are_valid_valid_avx2() {
        let s = "12345678901234567890123456789012";
        assert!(check_all_chars_are_valid(s));
    }

    #[cfg(all(target_arch = "x86_64", any(target_feature = "avx2")))]
    #[test]
    fn parse_integer_avx2_fallback() {
        let s = "12345678";
        assert_eq!(parse_integer(s, SEP, EOL), Some(12345678));
    }

    #[cfg(all(target_arch = "x86_64", any(target_feature = "avx2")))]
    #[test]
    fn parse_integer_avx2_no_digit() {
        let s = ",,345678901234567890123456789012";
        assert_eq!(parse_integer(s, SEP, EOL), None);
    }

    #[cfg(all(target_arch = "x86_64", any(target_feature = "avx2")))]
    #[test]
    fn parse_integer_avx2_one_digit() {
        let s = "1,345678901234567890123456789012";
        assert_eq!(parse_integer(s, SEP, EOL), Some(1));
    }

    #[cfg(all(target_arch = "x86_64", any(target_feature = "avx2")))]
    #[test]
    fn parse_integer_avx2_more_digits() {
        let s = "12345678,01234567890123456789012";
        assert_eq!(parse_integer(s, SEP, EOL), Some(12345678));
    }

    #[cfg(all(target_arch = "x86_64", any(target_feature = "avx2")))]
    #[test]
    fn parse_integer_avx2_all_digits_padded() {
        let s = "00000000000000000000000012345678";
        assert_eq!(parse_integer(s, SEP, EOL), Some(12345678));
    }

    // ===== SSE4.1/2 tests =====

    #[cfg(all(
        target_arch = "x86_64",
        any(target_feature = "sse4.1", target_feature = "sse4.2")
    ))]
    #[test]
    fn parse_integer_sse4_fallback() {
        let s = "1123,23";
        assert_eq!(parse_integer(s, SEP, EOL), Some(1123));
    }

    #[cfg(all(
        target_arch = "x86_64",
        any(target_feature = "sse4.1", target_feature = "sse4.2")
    ))]
    #[test]
    fn parse_integer_sse4_no_digit() {
        let s = ",\n12345678912345";
        assert_eq!(parse_integer(s, SEP, EOL), None);
    }

    #[cfg(all(
        target_arch = "x86_64",
        any(target_feature = "sse4.1", target_feature = "sse4.2")
    ))]
    #[test]
    fn parse_integer_sse4_one_digit() {
        let s = "1,12345678912345";
        assert_eq!(parse_integer(s, SEP, EOL), Some(1));
    }

    #[cfg(all(
        target_arch = "x86_64",
        any(target_feature = "sse4.1", target_feature = "sse4.2")
    ))]
    #[test]
    fn parse_integer_sse4_more_digits() {
        let s = "12345607,8912345";
        assert_eq!(parse_integer(s, SEP, EOL), Some(12345607));
    }

    #[cfg(all(
        target_arch = "x86_64",
        any(target_feature = "sse4.1", target_feature = "sse4.2")
    ))]
    #[test]
    fn parse_integer_sse4_all_digits_padded() {
        let s = "0000000012345678";
        assert_eq!(parse_integer(s, SEP, EOL), Some(12345678));
    }
}
