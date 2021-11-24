//! SIMD implementations for parsing an u32 from a string

#![feature(stdsimd)]
#![deny(missing_docs)]
#![deny(clippy::missing_docs_in_private_items)]
#![deny(clippy::missing_safety_doc)]
#![warn(rustdoc::missing_doc_code_examples)]
#![warn(clippy::todo)]

pub mod avx;
pub mod fallback;
pub mod sse41;
pub mod sse42;

/// Holds the pointer to the function supporeted by the underlying CPU
static mut LAST_BYTE_DIGIT_SEP: unsafe fn(&str, u8, u8) -> u32 = last_byte_digit_dispatcher;

/// Implements a single dispatch method to assign the appropiate function to the
/// global variable LAST_BYTE_DIGIT
fn last_byte_digit_dispatcher(s: &str, separator: u8, eol: u8) -> u32 {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            // repelace the global variable with the pointer to the sse42
            // function
            unsafe {
                LAST_BYTE_DIGIT_SEP = avx::last_byte_without_separator;
                return avx::last_byte_without_separator(s, separator, eol);
            }
        }
        if is_x86_feature_detected!("sse4.2") {
            // repelace the global variable with the pointer to the sse42
            // function
            unsafe {
                LAST_BYTE_DIGIT_SEP = sse42::last_byte_without_separator;
                return sse42::last_byte_without_separator(s, separator, eol);
            }
        }
        if is_x86_feature_detected!("sse4.1") {
            // repelace the global variable with the pointer to the sse41
            // function
            unsafe {
                LAST_BYTE_DIGIT_SEP = sse41::last_byte_without_separator;
                return sse41::last_byte_without_separator(s, separator, eol);
            }
        }
    }

    unsafe {
        LAST_BYTE_DIGIT_SEP = fallback::last_byte_without_separator;
    }
    fallback::last_byte_without_separator(s, separator, eol)
}

/// Returns the index of the last char in the string different from `separator`
/// and `eol`
pub fn last_byte_without_separator(s: &str, separator: u8, eol: u8) -> u32 {
    unsafe { LAST_BYTE_DIGIT_SEP(s, separator, eol) }
}

/// Pointer to `last_digit_byte` supported by the underlying cpu
static mut LAST_DIGIT_BYTE: unsafe fn(&str) -> u32 = last_digit_byte_dispatcher;

/// Implements a single dispatch method to assign the appropiate function to the
/// global variable LAST_DIGIT_BYTE
fn last_digit_byte_dispatcher(s: &str) -> u32 {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            // repelace the global variable with the pointer to the sse42 function
            unsafe {
                LAST_DIGIT_BYTE = avx::last_digit_byte;
                return avx::last_digit_byte(s);
            }
        }
        if is_x86_feature_detected!("sse4.2") {
            // repelace the global variable with the pointer to the sse42 function
            unsafe {
                LAST_DIGIT_BYTE = sse42::last_digit_byte;
                return sse42::last_digit_byte(s);
            }
        }
        if is_x86_feature_detected!("sse4.1") {
            // repelace the global variable with the pointer to the sse41
            // function
            unsafe {
                LAST_DIGIT_BYTE = sse41::last_digit_byte;
                return sse41::last_digit_byte(s);
            }
        }
    }
    // fallback implementation
    unsafe {
        LAST_DIGIT_BYTE = fallback::last_digit_byte;
    }
    fallback::last_digit_byte(s)
}

/// Returns the index of the last digit in the string
pub fn last_digit_byte(s: &str) -> u32 {
    unsafe { LAST_DIGIT_BYTE(s) }
}

/// Pointer to `check_all_chars_are_valid` function supported by the underlying
/// cpu
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
    // fallback implementation
    unsafe {
        CHECK_CHARS = fallback::check_all_chars_are_valid;
    }
    fallback::check_all_chars_are_valid(s)
}

/// Deteremines if the string in made of all numbers
pub fn check_all_chars_are_valid(s: &str) -> bool {
    unsafe { CHECK_CHARS(s) }
}

/// Pointer to `parse_integer` supperted by the underlying CPU
static mut PARSE_INTEGER: unsafe fn(&str) -> Option<u32> = parse_integer_checked_dispatcher;

/// Assigns the correct implementation to `PARSE_INTEGER` according to the
/// underlying cpu
fn parse_integer_checked_dispatcher(s: &str) -> Option<u32> {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            unsafe {
                PARSE_INTEGER = parse_integer_checked_avx2;
                return parse_integer_checked_avx2(s);
            }
        }
        if is_x86_feature_detected!("sse4.1") {
            unsafe {
                PARSE_INTEGER = parse_integer_checked_sse41;
                return parse_integer_checked_sse41(s);
            }
        }
    }
    // fallback implementation
    unsafe {
        PARSE_INTEGER = fallback::parse_integer;
    }
    fallback::parse_integer(s)
}

/// Parses and `u32` from the input string when possibile using AVX2 instrinics.
///
/// If the input string is empty or the parsing function encounters an overflow,
/// then `None` will be returned.
unsafe fn parse_integer_checked_avx2(s: &str) -> Option<u32> {
    // Go back to fallback implementation if the string doesn't have the correct
    // size
    if s.len() < avx::VECTOR_SIZE {
        return fallback::parse_integer(s);
    }
    let index = avx::last_digit_byte(s);
    match index {
        8 => Some(avx::parse_8_chars_simd(s)),
        9 => Some(avx::parse_9_chars_simd(s)),
        7 => Some(avx::parse_7_chars_simd(s)),
        6 => Some(avx::parse_6_chars_simd(s)),
        5 => Some(avx::parse_5_chars_simd(s)),
        4 => Some(avx::parse_4_chars_simd(s)),
        1..=3 => Some(fallback::parse_byte_iterator_limited(s, index)),
        // Use the default implementation since we cannot guarantee overflow
        // check in the SIMD implementations
        _ => fallback::parse_integer(s),
    }
}

/// Parses and `u32` from the input string when possibile using AVX2 instrinics.
///
/// If the input string is empty or the parsing function encounters an overflow,
/// then `None` will be returned.
unsafe fn parse_integer_checked_sse41(s: &str) -> Option<u32> {
    // Go back to fallback implementation if the string doesn't have the correct
    // size
    if s.len() < sse41::VECTOR_SIZE {
        return fallback::parse_integer(s);
    }
    let index = sse41::last_digit_byte(s);
    match index {
        8 => Some(sse41::parse_8_chars_simd(s)),
        9 => Some(sse41::parse_9_chars_simd(s)),
        7 => Some(sse41::parse_7_chars_simd(s)),
        6 => Some(sse41::parse_6_chars_simd(s)),
        5 => Some(sse41::parse_5_chars_simd(s)),
        4 => Some(sse41::parse_4_chars_simd(s)),
        1..=3 => Some(fallback::parse_byte_iterator_limited(s, index)),
        // Use the default implementation since we cannot guarantee overflow
        // check in the SIMD implementations
        _ => fallback::parse_integer(s),
    }
}

/// Parses an `u32` from the input string.
///
/// In case of empty string or aritmethic overlfow, it will return None.
pub fn parse_integer(s: &str) -> Option<u32> {
    unsafe { PARSE_INTEGER(s) }
}

/// Pointer to `parse_integer` supperted by the underlying CPU
static mut PARSE_INTEGER_SEP: unsafe fn(&str, u8, u8) -> Option<u32> =
    parse_integer_sep_checked_dispatcher;

/// Assigns the correct implementation to `PARSE_INTEGER` according to the
/// underlying cpu
fn parse_integer_sep_checked_dispatcher(s: &str, sep: u8, eol: u8) -> Option<u32> {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            unsafe {
                PARSE_INTEGER_SEP = parse_integer_sep_checked_avx2;
                return parse_integer_sep_checked_avx2(s, sep, eol);
            }
        }
        if is_x86_feature_detected!("sse4.1") {
            unsafe {
                PARSE_INTEGER_SEP = parse_integer_sep_checked_sse41;
                return parse_integer_sep_checked_sse41(s, sep, eol);
            }
        }
    }
    // fallback implementation
    unsafe {
        PARSE_INTEGER_SEP = fallback::parse_integer_separator;
    }
    fallback::parse_integer_separator(s, sep, eol)
}

/// Parses and `u32` from the input string when possibile using AVX2 instrinics.
///
/// If the input string is empty or the parsing function encounters an overflow,
/// then `None` will be returned.
#[inline]
unsafe fn parse_integer_sep_checked_avx2(s: &str, sep: u8, eol: u8) -> Option<u32> {
    // Go back to fallback implementation if the string doesn't have the correct
    // size
    if s.len() < avx::VECTOR_SIZE {
        return fallback::parse_integer_separator(s, sep, eol);
    }
    let index = avx::last_byte_without_separator(s, sep, eol);
    match index {
        8 => Some(avx::parse_8_chars_simd(s)),
        9 => Some(avx::parse_9_chars_simd(s)),
        7 => Some(avx::parse_7_chars_simd(s)),
        6 => Some(avx::parse_6_chars_simd(s)),
        5 => Some(avx::parse_5_chars_simd(s)),
        4 => Some(avx::parse_4_chars_simd(s)),
        1..=3 => Some(fallback::parse_byte_iterator_limited(s, index)),
        // Use the default implementation since we cannot guarantee overflow
        // check in the SIMD implementations
        _ => fallback::parse_integer_separator(s, sep, eol),
    }
}

/// Parses and `u32` from the input string when possibile using AVX2 instrinics.
///
/// If the input string is empty or the parsing function encounters an overflow,
/// then `None` will be returned.
#[inline]
unsafe fn parse_integer_sep_checked_sse41(s: &str, sep: u8, eol: u8) -> Option<u32> {
    // Go back to fallback implementation if the string doesn't have the correct
    // size
    if s.len() < sse41::VECTOR_SIZE {
        return fallback::parse_integer_separator(s, sep, eol);
    }
    let index = sse41::last_byte_without_separator(s, sep, eol);
    match index {
        8 => Some(sse41::parse_8_chars_simd(s)),
        9 => Some(sse41::parse_9_chars_simd(s)),
        7 => Some(sse41::parse_7_chars_simd(s)),
        6 => Some(sse41::parse_6_chars_simd(s)),
        5 => Some(sse41::parse_5_chars_simd(s)),
        4 => Some(sse41::parse_4_chars_simd(s)),
        1..=3 => Some(fallback::parse_byte_iterator_limited(s, index)),
        // Use the default implementation since we cannot guarantee overflow
        // check in the SIMD implementations
        _ => fallback::parse_integer_separator(s, sep, eol),
    }
}

/// Parses an `u32` from the input string up to the first occurence of
/// `separator` or `eol`.
///
/// In case of empty string, aritmethic overlfow or absence of number to parse,
/// it will return None.
pub fn parse_integer_separator(s: &str, separator: u8, eol: u8) -> Option<u32> {
    unsafe { PARSE_INTEGER_SEP(s, separator, eol) }
}

/// Pointer to `parse_integer_separator` supperted by the underlying CPU
static mut PARSE_INTEGER_SEP_UN: unsafe fn(&str, u8, u8) -> u32 = parse_integer_sep_dispatcher;

/// Assigns the correct implementation to the global variable
/// PARSE_INTEGER_SEP_UN
unsafe fn parse_integer_sep_dispatcher(s: &str, separator: u8, eol: u8) -> u32 {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            PARSE_INTEGER_SEP_UN = parse_integer_separator_avx2;
            return parse_integer_separator_avx2(s, separator, eol);
        }
        if is_x86_feature_detected!("sse4.1") {
            PARSE_INTEGER_SEP_UN = parse_integer_separator_sse41;
            return parse_integer_separator_sse41(s, separator, eol);
        }
    }
    // fallback implementation
    PARSE_INTEGER_SEP_UN = fallback::parse_integer_separator_unchecked;
    fallback::parse_integer_separator_unchecked(s, separator, eol)
}

/// Parses an integer from the given string until a separator is found
///
/// If the string has length less than 16 chars (or 32 if AVX is used), then no
/// SIMD acceleration is used; in this case, the method resorts to an iterative
/// process to parse the integer. If the string has at least 16 chars (or 32
/// for AVX), then it can perform parsing exploiting the SIMD intrinsics.
///
/// # Safety
///
/// This method doens't check any kind of arithmetic overflow: if the input
/// string contains a number which doesn't fit into an `u32`, then a panic will
/// be thrown.
#[inline]
pub unsafe fn parse_integer_separator_unchecked(s: &str, separator: u8, eol: u8) -> u32 {
    PARSE_INTEGER_SEP_UN(s, separator, eol)
}

/// Pointer to `parse_integer_unchecked` function for the underlying CPU
static mut PARSE_INTEGER_UN: unsafe fn(&str) -> u32 = parse_integer_dispatcher;

/// Assigns the correct implementation to PARSE_INTEGER_UN variable based on the
/// the underlying CPU
unsafe fn parse_integer_dispatcher(s: &str) -> u32 {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            PARSE_INTEGER_UN = parse_integer_avx2;
            return parse_integer_avx2(s);
        }
        if is_x86_feature_detected!("sse4.1") {
            PARSE_INTEGER_UN = parse_integer_sse41;
            return parse_integer_sse41(s);
        }
    }
    PARSE_INTEGER_UN = fallback::parse_integer_unchecked;
    fallback::parse_integer_unchecked(s)
}

/// Parses an integer from the given string
///
/// If the string has length less than 16 chars (or 32 if AVX is used), then no
/// SIMD acceleration is used; in this case, the method resorts to an iterative
/// process to parse the integer. If the string has at least 16 chars (or 32
/// for AVX), then it can perform parsing exploiting the SIMD intrinsics.
///
/// # Safety
///
/// This method doens't check any kind of arithmetic overflow: if the input
/// string contains a number which doesn't fit into an `u32`, then a panic will
/// be thrown. Furthermore, if the string contains other chars than numbers,
/// they will be parsed as regular digit, invalidating the final number.
#[inline]
pub unsafe fn parse_integer_unchecked(s: &str) -> u32 {
    PARSE_INTEGER_UN(s)
}

/// Parses an u32 from the input string using AVX2 instrinics whenever is
/// possibile
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn parse_integer_avx2(s: &str) -> u32 {
    if s.len() < avx::VECTOR_SIZE {
        return fallback::parse_integer_unchecked(s);
    }
    // find the first occurence of a separator
    let index = avx::last_digit_byte(s);
    match index {
        8 => avx::parse_8_chars_simd(s),
        10 => avx::parse_10_chars_simd(s),
        9 => avx::parse_9_chars_simd(s),
        7 => avx::parse_7_chars_simd(s),
        6 => avx::parse_6_chars_simd(s),
        5 => avx::parse_5_chars_simd(s),
        4 => avx::parse_4_chars_simd(s),
        1..=3 => fallback::parse_byte_iterator_limited(s, index),
        // all the chars are numeric, and they should be padded with 0s to get a
        // correct result. If not, the parsed number will not be correct due to
        // internal processing techniques
        32 => avx::parse_padded_integer_simd_all_numbers(s),
        // there is no number to parse
        _ => panic!("No u32 to parse from input string!"),
    }
}

/// Parses an u32 from the input string using AVX2 instrinics whenever is
/// possibile up to the first occurence of `separator` or `eol`
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn parse_integer_separator_avx2(s: &str, separator: u8, eol: u8) -> u32 {
    if s.len() < avx::VECTOR_SIZE {
        return fallback::parse_integer_separator_unchecked(s, separator, eol);
    }
    // find the first occurence of a separator
    let index = avx::last_byte_without_separator(s, separator, eol);
    match index {
        8 => avx::parse_8_chars_simd(s),
        10 => avx::parse_10_chars_simd(s),
        9 => avx::parse_9_chars_simd(s),
        7 => avx::parse_7_chars_simd(s),
        6 => avx::parse_6_chars_simd(s),
        5 => avx::parse_5_chars_simd(s),
        4 => avx::parse_4_chars_simd(s),
        1..=3 => fallback::parse_byte_iterator_limited(s, index),
        // all the chars are numeric, and they should be padded with 0s to get a
        // correct result. If not, the parsed number will not be correct due to
        // internal processing techniques
        32 => avx::parse_padded_integer_simd_all_numbers(s),
        // there is no number to parse
        _ => panic!("No u32 to parse from input string!"),
    }
}

/// Parses an u32 from the input string using SSE4.1 instrinics whenever is
/// possibile
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse4.1")]
unsafe fn parse_integer_sse41(s: &str) -> u32 {
    if s.len() < sse41::VECTOR_SIZE {
        return fallback::parse_integer_unchecked(s);
    }
    // find the first occurence of a separator
    let index = sse41::last_digit_byte(s);
    match index {
        8 => sse41::parse_8_chars_simd(s),
        10 => sse41::parse_10_chars_simd(s),
        9 => sse41::parse_9_chars_simd(s),
        7 => sse41::parse_7_chars_simd(s),
        6 => sse41::parse_6_chars_simd(s),
        5 => sse41::parse_5_chars_simd(s),
        4 => sse41::parse_4_chars_simd(s),
        1..=3 => fallback::parse_byte_iterator_limited(s, index),
        // all the chars are numeric, maybe padded?
        32 => sse41::parse_integer_simd_all_numbers(s),
        // there is no u32 to parse
        _ => panic!("No u32 to parse from input string!"),
    }
}

/// Parses an u32 from the input string using SSE4.1 instrinics whenever is
/// possibile up to the first occurence of `separator` or `eol`
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse4.1")]
unsafe fn parse_integer_separator_sse41(s: &str, separator: u8, eol: u8) -> u32 {
    if s.len() < sse41::VECTOR_SIZE {
        return fallback::parse_integer_separator_unchecked(s, separator, eol);
    }
    // find the first occurence of a separator
    let index = sse41::last_byte_without_separator(s, separator, eol);
    match index {
        8 => sse41::parse_8_chars_simd(s),
        10 => sse41::parse_10_chars_simd(s),
        9 => sse41::parse_9_chars_simd(s),
        7 => sse41::parse_7_chars_simd(s),
        6 => sse41::parse_6_chars_simd(s),
        5 => sse41::parse_5_chars_simd(s),
        4 => sse41::parse_4_chars_simd(s),
        1..=3 => fallback::parse_byte_iterator_limited(s, index),
        // all the chars are numeric, maybe padded?
        32 => sse41::parse_integer_simd_all_numbers(s),
        // there is no u32 to parse
        _ => panic!("No u32 to parse from input string!"),
    }
}

// ====================
// benchmark only function
// ====================

/// SSE4.2 implementation for `parse_integer_separator` meant to be used only
/// during benchamarking
#[cfg(feature = "benchmark")]
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse4.2")]
unsafe fn parse_integer_separator_sse42(s: &str, separator: u8, eol: u8) -> u32 {
    if s.len() < sse41::VECTOR_SIZE {
        return fallback::parse_integer_separator_unchecked(s, separator, eol);
    }
    // find the first occurence of a separator
    let index = sse42::last_byte_without_separator(s, separator, eol);
    match index {
        8 => return sse41::parse_8_chars_simd(s),
        10 => return sse41::parse_10_chars_simd(s),
        9 => return sse41::parse_9_chars_simd(s),
        7 => return sse41::parse_7_chars_simd(s),
        6 => return sse41::parse_6_chars_simd(s),
        5 => return sse41::parse_5_chars_simd(s),
        4 => return sse41::parse_4_chars_simd(s),
        1..=3 => return fallback::parse_byte_iterator_limited(s, index),
        // all the chars are numeric, maybe padded?
        32 => return sse41::parse_integer_simd_all_numbers(s),
        // there is no u32 to parse
        _ => panic!("No u32 to parse from input string!"),
    }
}

/// SSE4.2 implementation for `parse_integer` meant to be used only during
/// benchamarking
#[cfg(feature = "benchmark")]
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse4.2")]
unsafe fn parse_integer_sse42(s: &str) -> u32 {
    if s.len() < sse41::VECTOR_SIZE {
        return fallback::parse_integer_unchecked(s);
    }
    // find the first occurence of a separator
    let index = sse42::last_digit_byte(s);
    match index {
        8 => return sse41::parse_8_chars_simd(s),
        10 => return sse41::parse_10_chars_simd(s),
        9 => return sse41::parse_9_chars_simd(s),
        7 => return sse41::parse_7_chars_simd(s),
        6 => return sse41::parse_6_chars_simd(s),
        5 => return sse41::parse_5_chars_simd(s),
        4 => return sse41::parse_4_chars_simd(s),
        1..=3 => return fallback::parse_byte_iterator_limited(s, index),
        // all the chars are numeric, maybe padded?
        32 => return sse41::parse_integer_simd_all_numbers(s),
        // there is no u32 to parse
        _ => panic!("No u32 to parse from input string!"),
    }
}

/// Safe wrapper around `parse_integer_separator_sse41` to use only during
/// benchmark
#[cfg(feature = "benchmark")]
#[inline]
pub fn safe_parse_integer_separator_sse41(s: &str, separator: u8, eol: u8) -> u32 {
    unsafe {
        return parse_integer_separator_sse41(s, separator, eol);
    }
}

/// Safe wrapper around `parse_integer_separator_sse42` to use only during
/// benchmark
#[cfg(feature = "benchmark")]
#[inline]
pub fn safe_parse_integer_separator_sse42(s: &str, separator: u8, eol: u8) -> u32 {
    unsafe {
        return parse_integer_separator_sse42(s, separator, eol);
    }
}

/// Safe wrapper around `parse_integer_avx2` to use only during benchmark
#[cfg(feature = "benchmark")]
#[inline]
pub fn safe_parse_integer_separator_avx2(s: &str, separator: u8, eol: u8) -> u32 {
    unsafe {
        return parse_integer_separator_avx2(s, separator, eol);
    }
}

/// Safe wrapper around `parse_integer_sse41` to use only during benchmark
#[cfg(feature = "benchmark")]
#[inline]
pub fn safe_parse_integer_sse41(s: &str) -> u32 {
    unsafe {
        return parse_integer_sse41(s);
    }
}

/// Safe wrapper around `parse_integer_sse42` to use only during benchmark
#[cfg(feature = "benchmark")]
#[inline]
pub fn safe_parse_integer_sse42(s: &str) -> u32 {
    unsafe {
        return parse_integer_sse42(s);
    }
}

/// Safe wrapper around `parse_integer_avx2` to use only during benchmark
#[cfg(feature = "benchmark")]
#[inline]
pub fn safe_parse_integer_avx2(s: &str) -> u32 {
    unsafe {
        return parse_integer_avx2(s);
    }
}

/// Safe wrapper around `sse41::check_all_chars_are_valid` to use only during
/// benchmark
#[cfg(feature = "benchmark")]
#[inline]
pub fn safe_check_all_chars_are_valid_sse41(s: &str) -> bool {
    unsafe {
        return sse41::check_all_chars_are_valid(s);
    }
}

/// Safe wrapper around `sse42::check_all_chars_are_valid` to use only during
/// benchmark
#[cfg(feature = "benchmark")]
#[inline]
pub fn safe_check_all_chars_are_valid_sse42(s: &str) -> bool {
    unsafe {
        return sse42::check_all_chars_are_valid(s);
    }
}

/// Safe wrapper around `avx::check_all_chars_are_valid` to use only during
/// benchmark
#[cfg(feature = "benchmark")]
#[inline]
pub fn safe_check_all_chars_are_valid_avx(s: &str) -> bool {
    unsafe {
        return avx::check_all_chars_are_valid(s);
    }
}

/// Safe wrapper around `sse41::last_byte_without_separator` to use only during
/// benchmark
#[cfg(feature = "benchmark")]
#[inline]
pub fn safe_last_byte_without_separator_sse41(s: &str, separator: u8, eol: u8) -> u32 {
    unsafe {
        return sse41::last_byte_without_separator(s, separator, eol);
    }
}

/// Safe wrapper around `sse42::last_byte_without_separator` to use only during
/// benchmark
#[cfg(feature = "benchmark")]
#[inline]
pub fn safe_last_byte_without_separator_sse42(s: &str, separator: u8, eol: u8) -> u32 {
    unsafe {
        return sse42::last_byte_without_separator(s, separator, eol);
    }
}

/// Safe wrapper around `avx::last_byte_without_separator` to use only during
/// benchmark
#[cfg(feature = "benchmark")]
#[inline]
pub fn safe_last_byte_without_separator_avx(s: &str, separator: u8, eol: u8) -> u32 {
    unsafe {
        return avx::last_byte_without_separator(s, separator, eol);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    static SEP: u8 = b',';
    static EOL: u8 = b'\n';

    // ===== fallback tests =====

    // ===== `parse_integer_separator` tests =====

    #[test]
    fn parse_integer_separator_empty() {
        let s = "";
        assert_eq!(parse_integer_separator(s, SEP, EOL), None);
    }

    #[test]
    fn parse_integer_separator_no_digit() {
        let s = ",,\n123";
        assert_eq!(parse_integer_separator(s, SEP, EOL), None);
    }

    #[test]
    fn parse_integer_separator_one_digit() {
        let s = "1,123,23\n0";
        assert_eq!(parse_integer_separator(s, SEP, EOL), Some(1));
    }

    #[test]
    fn parse_integer_separator_more_digits() {
        let s = "1123,23\n0";
        assert_eq!(parse_integer_separator(s, SEP, EOL), Some(1123));
    }

    #[test]
    fn parse_integer_separator_all_digits() {
        let s = "112323";
        assert_eq!(parse_integer_separator(s, SEP, EOL), Some(112323));
    }

    // ===== `parse_integer` tests =====

    #[test]
    fn parse_integer_empty() {
        let s = "";
        assert_eq!(parse_integer(s), None);
    }

    #[test]
    fn parse_integer_no_digit() {
        let s = ",,\n123";
        assert_eq!(parse_integer(s), None);
    }

    #[test]
    fn parse_integer_one_digit() {
        let s = "1,123,23\n0";
        assert_eq!(parse_integer(s), Some(1));
    }

    #[test]
    fn parse_integer_more_digits() {
        let s = "1123,23\n0";
        assert_eq!(parse_integer(s), Some(1123));
    }

    #[test]
    fn parse_integer_all_digits() {
        let s = "112323";
        assert_eq!(parse_integer(s), Some(112323));
    }

    // ===== AVX2 tests =====

    // ===== `parse_integer_separator` tests =====

    #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
    #[test]
    fn check_all_chars_are_valid_valid_avx2() {
        let s = "12345678901234567890123456789012";
        assert!(check_all_chars_are_valid(s));
    }

    #[cfg(all(target_arch = "x86_64", any(target_feature = "avx2")))]
    #[test]
    fn parse_integer_separator_avx2_fallback() {
        let s = "12345678";
        assert_eq!(parse_integer_separator(s, SEP, EOL), Some(12345678));
    }

    #[cfg(all(target_arch = "x86_64", any(target_feature = "avx2")))]
    #[test]
    fn parse_integer_separator_avx2_no_digit() {
        let s = ",,345678901234567890123456789012";
        assert_eq!(parse_integer_separator(s, SEP, EOL), None);
    }

    #[cfg(all(target_arch = "x86_64", any(target_feature = "avx2")))]
    #[test]
    fn parse_integer_separator_avx2_one_digit() {
        let s = "1,345678901234567890123456789012";
        assert_eq!(parse_integer_separator(s, SEP, EOL), Some(1));
    }

    #[cfg(all(target_arch = "x86_64", any(target_feature = "avx2")))]
    #[test]
    fn parse_integer_separator_avx2_more_digits() {
        let s = "12345678,01234567890123456789012";
        assert_eq!(parse_integer_separator(s, SEP, EOL), Some(12345678));
    }

    #[cfg(all(target_arch = "x86_64", any(target_feature = "avx2")))]
    #[test]
    fn parse_integer_separator_avx2_all_digits_padded() {
        let s = "00000000000000000000000012345678";
        assert_eq!(parse_integer_separator(s, SEP, EOL), Some(12345678));
    }

    // ===== `parse_integer` tests =====

    #[cfg(all(target_arch = "x86_64", any(target_feature = "avx2")))]
    #[test]
    fn parse_integer_avx2_fallback() {
        let s = "12345678";
        assert_eq!(parse_integer(s), Some(12345678));
    }

    #[cfg(all(target_arch = "x86_64", any(target_feature = "avx2")))]
    #[test]
    fn parse_integer_avx2_no_digit() {
        let s = ",,345678901234567890123456789012";
        assert_eq!(parse_integer(s), None);
    }

    #[cfg(all(target_arch = "x86_64", any(target_feature = "avx2")))]
    #[test]
    fn parse_integer_avx2_one_digit() {
        let s = "1,345678901234567890123456789012";
        assert_eq!(parse_integer(s), Some(1));
    }

    #[cfg(all(target_arch = "x86_64", any(target_feature = "avx2")))]
    #[test]
    fn parse_integer_avx2_more_digits() {
        let s = "12345678,01234567890123456789012";
        assert_eq!(parse_integer(s), Some(12345678));
    }

    #[cfg(all(target_arch = "x86_64", any(target_feature = "avx2")))]
    #[test]
    fn parse_integer_avx2_all_digits_padded() {
        let s = "00000000000000000000000012345678";
        assert_eq!(parse_integer(s), Some(12345678));
    }

    // ===== SSE4.1/2 tests =====

    // ===== `parse_integer_separator` tests =====

    #[cfg(all(
        target_arch = "x86_64",
        any(target_feature = "sse4.1", target_feature = "sse4.2")
    ))]
    #[test]
    fn parse_integer_separator_sse4_fallback() {
        let s = "1123,23";
        assert_eq!(parse_integer_separator(s, SEP, EOL), Some(1123));
    }

    #[cfg(all(
        target_arch = "x86_64",
        any(target_feature = "sse4.1", target_feature = "sse4.2")
    ))]
    #[test]
    fn parse_integer_separator_sse4_no_digit() {
        let s = ",\n12345678912345";
        assert_eq!(parse_integer_separator(s, SEP, EOL), None);
    }

    #[cfg(all(
        target_arch = "x86_64",
        any(target_feature = "sse4.1", target_feature = "sse4.2")
    ))]
    #[test]
    fn parse_integer_separator_sse4_one_digit() {
        let s = "1,12345678912345";
        assert_eq!(parse_integer_separator(s, SEP, EOL), Some(1));
    }

    #[cfg(all(
        target_arch = "x86_64",
        any(target_feature = "sse4.1", target_feature = "sse4.2")
    ))]
    #[test]
    fn parse_integer_separator_sse4_more_digits() {
        let s = "12345607,8912345";
        assert_eq!(parse_integer_separator(s, SEP, EOL), Some(12345607));
    }

    #[cfg(all(
        target_arch = "x86_64",
        any(target_feature = "sse4.1", target_feature = "sse4.2")
    ))]
    #[test]
    fn parse_integer_separator_sse4_all_digits_padded() {
        let s = "0000000012345678";
        assert_eq!(parse_integer_separator(s, SEP, EOL), Some(12345678));
    }

    // ===== `parse_integer` tests =====

    #[cfg(all(
        target_arch = "x86_64",
        any(target_feature = "sse4.1", target_feature = "sse4.2")
    ))]
    #[test]
    fn parse_integer_sse4_fallback() {
        let s = "1123,23";
        assert_eq!(parse_integer(s), Some(1123));
    }

    #[cfg(all(
        target_arch = "x86_64",
        any(target_feature = "sse4.1", target_feature = "sse4.2")
    ))]
    #[test]
    fn parse_integer_sse4_no_digit() {
        let s = ",\n12345678912345";
        assert_eq!(parse_integer(s), None);
    }

    #[cfg(all(
        target_arch = "x86_64",
        any(target_feature = "sse4.1", target_feature = "sse4.2")
    ))]
    #[test]
    fn parse_integer_sse4_one_digit() {
        let s = "1,12345678912345";
        assert_eq!(parse_integer(s), Some(1));
    }

    #[cfg(all(
        target_arch = "x86_64",
        any(target_feature = "sse4.1", target_feature = "sse4.2")
    ))]
    #[test]
    fn parse_integer_sse4_more_digits() {
        let s = "12345607,8912345";
        assert_eq!(parse_integer(s), Some(12345607));
    }

    #[cfg(all(
        target_arch = "x86_64",
        any(target_feature = "sse4.1", target_feature = "sse4.2")
    ))]
    #[test]
    fn parse_integer_sse4_all_digits_padded() {
        let s = "0000000012345678";
        assert_eq!(parse_integer(s), Some(12345678));
    }

    #[cfg(all(
        target_arch = "x86_64",
        any(target_feature = "sse4.1", target_feature = "sse4.2")
    ))]
    #[test]
    #[should_panic]
    fn parse_integer_sse4_unchecked_zero_digit() {
        let s = ",,00000012345678";
        unsafe {
            parse_integer_unchecked(s);
        }
    }

    #[cfg(all(
        target_arch = "x86_64",
        any(target_feature = "sse4.1", target_feature = "sse4.2")
    ))]
    #[test]
    #[should_panic]
    fn parse_integer_sse4_unchecked_more_than_10_digits() {
        let s = "123445123456,8456";
        unsafe {
            parse_integer_unchecked(s);
        }
    }
}
