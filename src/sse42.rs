#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

// minimum size required by an input string to use SIMD algorithms
const VECTOR_SIZE: usize = std::mem::size_of::<__m128i>();

// byte array to determine if a byte array is made of all numbers
const NUMERIC_RANGE: &[u8; 16] = b"90\0\0\0\0\0\0\0\0\0\0\0\0\0\0"
const NUMERIC_VALUES: &[u8; 16] = b"1234567890\0\0\0\0\0\0";


#[allow(dead_code)]
#[target_feature(enable = "sse2")]
unsafe fn dump_m128i(const v: __m128i) {
    let mut vdup = v;
    let lower = _mm_cvtsi128_si64(vdup);
    vdup = _mm_bsrli_si128(vdup, 8);
    let upper = _mm_cvtsi128_si64(vdup);
    println!("{:064b}\n{:064b}", upper, lower);
}

#[target_feature(enable = "sse4.2")]
pub unsafe fn check_all_chars_are_valid(s: &str) -> bool {
    let to_cmp = _mm_loadu_si128(s.as_ptr() as *const _);
    let range = _mm_loadu_si128(NUMERIC_RANGE.as_ptr() as *const _);
    const mode: i32 = _SIDD_CMP_RANGES;
    let idx = _mm_cmpistri(to_cmp, range, mode);
    idx == 16
}

#[target_feature(enable = "sse4.2")]
pub unsafe fn parsing_mask(s: &str) -> __m128i {
    let to_cmp = _mm_loadu_si128(s.as_ptr() as *const _);
    let valid_nums = _mm_loadu_si128(NUMERIC_VALUES.as_ptr() as *const _);
    let mask = _mm_cmpistrm(
        valid_nums,
        to_cmp,
        // cmp for any match | negate the result | create a byte mask
        _SIDD_CMP_EQUAL_ANY | _SIDD_NEGATIVE_POLARITY | _SIDD_UNIT_MASK
    );
    propagate(mask)
}

#[inline]
/// Propagates the input mask to the left.
unsafe fn propagate(mut v: __m128i) -> __m128i {
    v = _mm_or_si128(v, _mm_slli_si128(v as _, 1) as _);
    v = _mm_or_si128(v, _mm_slli_si128(v as _, 2) as _);
    v = _mm_or_si128(v, _mm_slli_si128(v as _, 4) as _);
    v = _mm_or_si128(v, _mm_slli_si128(v as _, 8) as _);
    v
}
