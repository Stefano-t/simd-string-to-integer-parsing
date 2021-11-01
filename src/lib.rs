#![feature(stdsimd)]

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

pub mod avx;
pub mod fallback;
pub mod sse41;
pub mod sse42;

#[inline]
pub fn last_byte_digit(s: &str, separator: u8, eof: u8) -> (u32, __m128i) {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("sse4.2") {
            return unsafe { sse42::last_byte_digit(s, separator, eof) };
        }
        if is_x86_feature_detected!("sse4.1") {
            return unsafe { sse41::last_byte_digit(s, separator, eof) };
        }
    }

    panic!(
        "Function `last_byte_digit` is only supported for sse41 or sse4.2.
For now, there is no fallback function for this implementation"
    );
}

#[inline]
pub fn check_all_chars_are_valid(s: &str) -> bool {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            return unsafe { avx::check_all_chars_are_valid(s) };
        }
        if is_x86_feature_detected!("sse4.2") {
            return unsafe { sse42::check_all_chars_are_valid(s) };
        }
        if is_x86_feature_detected!("sse4.1") {
            return unsafe { sse41::check_all_chars_are_valid(s) };
        }
    }

    fallback::check_all_chars_are_valid(s)
}

/// Parses an integer from the given string
///
/// If the string has length less than 16 chars, then no SIMD acceleration is
/// used; in this case, the method resorts to an iterative process to parse the
/// integer.  If the string has at least 16 chars, then it can perform parsing
/// exploiting the SIMD intrinsics.
pub fn parse_integer(s: &str, separator: u8, eol: u8) -> Option<u32> {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            return unsafe { parse_integer_avx2(s, separator, eol) };
        }
        if is_x86_feature_detected!("sse4.1") {
            return unsafe { parse_integer_sse41(s, separator, eol) };
        }
    }

    fallback::parse_integer_byte_iterator(s, separator, eol)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn parse_integer_avx2(s: &str, separator: u8, eol: u8) -> Option<u32> {
    if s.len() < avx::VECTOR_SIZE {
        return fallback::parse_integer_byte_iterator(s, separator, eol);
    }
    // find the first occurence of a separator
    let (index, mask) = avx::last_byte_digit(s, separator, eol);
    match index {
        8 => return Some(avx::parse_8_chars_simd(s)),
        10 => return Some(avx::parse_more_than_8_simd(s, 1000000, mask)),
        9 => return Some(avx::parse_more_than_8_simd(s, 10000000, mask)),
        7 => return Some(avx::parse_less_than_8_simd(s, 10, mask)),
        6 => return Some(avx::parse_less_than_8_simd(s, 100, mask)),
        5 => return Some(avx::parse_less_than_8_simd(s, 1000, mask)),
        4 => return Some(avx::parse_less_than_8_simd(s, 10000, mask)),
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
    let (index, mask) = last_byte_digit(s, separator, eol);
    match index {
        8 => return Some(sse41::parse_8_chars_simd(s)),
        10 => return Some(sse41::parse_more_than_8_simd(s, 1000000, mask)),
        9 => return Some(sse41::parse_more_than_8_simd(s, 10000000, mask)),
        7 => return Some(sse41::parse_less_than_8_simd(s, 10, mask)),
        6 => return Some(sse41::parse_less_than_8_simd(s, 100, mask)),
        5 => return Some(sse41::parse_less_than_8_simd(s, 1000, mask)),
        4 => return Some(sse41::parse_less_than_8_simd(s, 10000, mask)),
        1..=3 => return Some(fallback::parse_byte_iterator_limited(s, index)),
        // all the chars are numeric, maybe padded?
        32 => return Some(sse41::parse_integer_simd_all_numbers(s)),
        // there is no u32 to parse
        _ => return None,
    }
}
