#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

#[cfg(target_feature = "avx")]
pub const VECTOR_SIZE: usize = std::mem::size_of::<__m256i>(); // 32

#[inline]
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
/// Checks that all the bytes are valid digits
pub unsafe fn check_all_chars_are_valid(string: &str) -> bool {
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
pub unsafe fn last_byte_digit(string: &str, separator: u8, eol: u8) -> (u32, __m256i) {
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
    let index = movemask.trailing_zeros();

    let mask = propagate(or_comma_newline);
    (index, mask)
}

// TODO: correct this method, the shift is not correct
#[inline]
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
/// Propagates the input mask to the left.
unsafe fn propagate(mut v: __m256i) -> __m256i {
    v = _mm256_or_si256(v, _mm256_slli_si256(v as _, 1) as _);
    v = _mm256_or_si256(v, _mm256_slli_si256(v as _, 2) as _);
    v = _mm256_or_si256(v, _mm256_slli_si256(v as _, 4) as _);
    v = _mm256_or_si256(v, _mm256_slli_si256(v as _, 8) as _);
    v = _mm256_or_si256(v, _mm256_slli_si256(v as _, 16) as _);
    v
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
            assert!(check_all_chars_are_valid(s));
        }
    }
}
