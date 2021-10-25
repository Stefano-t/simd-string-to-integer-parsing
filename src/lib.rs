#![feature(stdsimd)]

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

pub mod fallback;
#[cfg(target_feature = "sse2")]
pub mod sse2;
#[cfg(target_feature = "sse4.2")]
pub mod sse42;

#[cfg(target_feature = "sse2")]
pub const VECTOR_SIZE: usize = std::mem::size_of::<__m128i>();

#[inline]
pub fn last_byte_digit(s: &str, separator: u8, eof: u8) -> (u32, __m128i) {
    #[inline]
    #[cfg(target_feature = "sse4.2")]
    unsafe fn imp_fn(s: &str, separator: u8, eof: u8) -> (u32, __m128i) {
        sse42::last_byte_digit(s, separator, eof)
    }

    #[inline]
    #[cfg(all(not(target_feature = "sse4.2"), target_feature = "sse2"))]
    unsafe fn imp_fn(s: &str, separator: u8, eof: u8) -> (u32, __m128i) {
        sse2::last_byte_digit(s, separator, eof)
    }

    unsafe { imp_fn(s, separator, eof) }
}

#[inline]
pub fn check_all_chars_are_valid(s: &str) -> bool {
    #[inline]
    #[cfg(target_feature = "sse4.2")]
    unsafe fn imp_fn(s: &str) -> bool {
        sse42::check_all_chars_are_valid(s)
    }

    #[inline]
    #[cfg(all(not(target_feature = "sse4.2"), target_feature = "sse2"))]
    unsafe fn imp_fn(s: &str) -> bool {
        sse2::check_all_chars_are_valid(s)
    }
    unsafe { imp_fn(s) }
}

/// Parses an integer from the given string
///
/// If the string has length less than 16 chars, then no SIMD acceleration is
/// used; in this case, the method resorts to an iterative process to parse the
/// integer.  If the string has at least 16 chars, then it can perform parsing
/// exploiting the SIMD intrinsics.
pub fn parse_integer(s: &str, separator: u8, eol: u8) -> Option<u32> {
    if s.len() < VECTOR_SIZE {
        return fallback::parse_integer_byte_iterator(s, separator, eol);
    }
    // find the first occurence of a separator
    let (index, mask) = last_byte_digit(s, separator, eol);
    match index {
        8 => return Some(sse2::parse_8_chars_simd(s)),
        10 => return Some(sse2::parse_more_than_8_simd(s, 1000000, mask)),
        9 => return Some(sse2::parse_more_than_8_simd(s, 10000000, mask)),
        7 => return Some(sse2::parse_less_than_8_simd(s, 10, mask)),
        6 => return Some(sse2::parse_less_than_8_simd(s, 100, mask)),
        5 => return Some(sse2::parse_less_than_8_simd(s, 1000, mask)),
        4 => return Some(sse2::parse_less_than_8_simd(s, 10000, mask)),
        1..=3 => return Some(fallback::parse_byte_iterator_limited(s, index)),
        // all the chars are numeric, maybe padded?
        32 => return Some(sse2::parse_integer_simd_all_numbers(s)),
        // there is no u32 to parse
        _ => return None,
    }
}
