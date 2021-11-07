#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

pub const VECTOR_SIZE: usize = std::mem::size_of::<__m128i>();

// byte array to determine if a byte array is made of all numbers
const NUMERIC_RANGE: &[u8; 16] = b"09\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
const NUMERIC_VALUES: &[u8; 16] = b"1234567890\0\0\0\0\0\0";

#[allow(dead_code)]
#[target_feature(enable = "sse2")]
unsafe fn dump_m128i(v: __m128i) {
    let mut vdup = v;
    let lower = _mm_cvtsi128_si64(vdup);
    vdup = _mm_bsrli_si128(vdup, 8);
    let upper = _mm_cvtsi128_si64(vdup);
    println!("{:064b}\n{:064b}", upper, lower);
}

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
pub unsafe fn last_byte_digit(s: &str, _separator: u8, _eol: u8) -> u32 {
    // ignore `separator` and `eol`, since function `_mm_cmpistrm` can
    // compare automatically all numeric values and decect when they are not
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
