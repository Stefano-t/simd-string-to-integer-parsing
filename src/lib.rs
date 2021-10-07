#![feature(stdsimd)]

// TODO do not replicate masks

use std::arch::x86_64::{
    __m128i,
    // Set register to all zeros.
    // _mm_setzero_si128,

    // Compute the binary and between two registers
    // Reference: https://software.intel.com/sites/landingpage/IntrinsicsGuide/#text=_mm_and&expand=1046,6130,6130,4335,4335,1046,1180,349
    _mm_and_si128,

    // negate the first argument, and then compute an AND with second argument
    _mm_andnot_si128,

    // Shift left by byte
    _mm_bsrli_si128,

    // Compare packed 8-bit integers for equality, and create a mask with 0xFF
    // if the byte of the first regitest is equal to the byte of the second
    // register, otherwise 0x00.
    _mm_cmpeq_epi8,

    // Compare every byte of two registers and
    // create a mask with 0xFF if the byte of the first register
    // is strictly greater than the byte of the second register,
    // and 0x00 if they are different.
    // Reference: https://software.intel.com/sites/landingpage/IntrinsicsGuide/#text=_mm_cmpgt_epi8&expand=1046,6130,6130,4335,4335,1046
    _mm_cmpgt_epi8,

    // Compare every byte of two registers and
    // create a mask with 0xFF if the byte of the first register
    // is strictly less than the byte of the second register,
    // and 0x00 if they are different.
    // Reference: https://software.intel.com/sites/landingpage/IntrinsicsGuide/#text=_mm_cmplt_epi8&expand=1046,6130,6130,4335,4335,1046,1180
    _mm_cmplt_epi8,

    _mm_cvtsi128_si32,
    // Copy the lower 64-bit integer.
    _mm_cvtsi128_si64,

    // Shift left the first register by bytes while shifting in zeros.
    // _mm_bslli_si128,

    // Load 128-bits of integer data from unaligned memory. This
    // intrinsic may perform better than _mm_loadu_si128 when the data crosses a
    // cache line boundary.
    _mm_lddqu_si128,

    // load unaliged bits from memory
    // there also exist _mm_load_si128 which
    // is faster but panics if the data address
    // is not a multiple of 8 (bytes).
    // Reference: https://software.intel.com/sites/landingpage/IntrinsicsGuide/#text=_mm_loadu_si128&expand=1046,6130,6130,4335,4335
    _mm_loadu_si128,

    // Multiply packed signed 16-bit integers, producing intermediate
    // signed 32-bit integers. Horizontally add adjacent pairs of intermediate
    // 32-bit integers, and pack the results.
    _mm_madd_epi16,

    // Vertically multiply each unsigned 8-bit integer from a with the
    // corresponding signed 8-bit integer from b, producing intermediate signed
    // 16-bit integers. Horizontally add adjacent pairs of intermediate signed
    // 16-bit integers, and pack the saturated results in dst.
    _mm_maddubs_epi16,

    // Compute the bitwise OR of the two regiters.
    _mm_or_si128,

    // Convert packed signed 32-bit integers from a and b to packed 16-bit
    // integers using unsigned saturation, and store the results in dst.
    _mm_packus_epi32,

    // Set all the values in a register to a
    // constant byte
    // Reference: https://software.intel.com/sites/landingpage/IntrinsicsGuide/#text=_mm_set1_epi8&expand=1046,6130,6130
    _mm_set1_epi8,

    // Set packed 16-bit integers in dst with the supplied values.
    _mm_set_epi16,

    // Set packed 8-bit integers in dst with the supplied values.
    _mm_set_epi8,

    // Logic shidft to the left by bytes
    _mm_slli_si128,

    // Subtract packed 16-bit integers in b from packed 16-bit integers in a,
    // and store the results in dst.
    _mm_sub_epi16,

    // Returns 1 if the register is all ones, 0 otherwise
    // Reference: https://software.intel.com/sites/landingpage/IntrinsicsGuide/#text=zero&expand=1046,6130,6130,4335,4335,1046,1180,349,4836,7128&techs=SSE,SSE2,SSE3,SSSE3,SSE4_1,SSE4_2
    _mm_test_all_ones,
};

// minimum size required by an input string to use SIMD algorithms
const VECTOR_SIZE: usize = std::mem::size_of::<__m128i>();

/// Parses an integer from the given string
///
/// If the string has length less than 16 chars, then no SIMD acceleration is
/// used; in this case, the method resorts to an iterative process to parse the
/// integer.  If the string has at least 16 chars, then it can perform parsing
/// exploiting the SIMD intrincs.
pub fn parse_integer(s: &str) -> u32 {
    // TODO: handle error in string, i.e. no number to parse
    // cannot use SIMD acceleration, at least 16 chars are required
    if s.len() < VECTOR_SIZE {
        return parse_integer_byte_iterator(s);
    }
    // find the first occurence of a separator
    let index = first_byte_non_numeric(s);
    match index {
        9 => return parse_8_chars_simd(s),
        11 => return parse_more_than_8_simd(s, 1000000),
        10 => return parse_more_than_8_simd(s, 10000000),
        8 => return parse_less_than_8_simd(s, 10),
        7 => return parse_less_than_8_simd(s, 100),
        6 => return parse_less_than_8_simd(s, 1000),
        5 => return parse_less_than_8_simd(s, 10000),
        2..=4 => return parse_4_or_less_chars(s, index - 1),
        // all the chars is numeric, maybe padded?
        0 => return parse_integer_simd_all_numbers(s),
        // TODO: throw an error here
        // it not an u32 number
        _ => return 0_u32,
    }
}

#[inline]
/// Checks that all the bytes are valid digits
///
/// This *assumes* that the string has exactly length 16
/// and it's padded with zeros if needed.
pub fn check_all_chars_are_valid(string: &str) -> bool {
    // We rust cannot guarantee safety of AVX and pointers in general
    // therefore we need to work inside an unsafe block
    unsafe {
        // initialize the constants, saddly these cannot be actual constants,
        // because we have to initialize xmm registers, and the System V callin convention (https://wiki.osdev.org/System_V_ABI)
        // requires that the callee should set all the xmm register to zero before returning to the caller.
        let zeros = _mm_set1_epi8(b'0' as i8 - 1); // the -1 is because we don't have a >= instruction, we only have the >
        let nines = _mm_set1_epi8(b'9' as i8 + 1); // the +1 is because we don't have a <= instruction, we only have the <

        // Load the data
        let value = _mm_loadu_si128(string.as_ptr() as _);
        // compare the values with the upper and lower bounds
        let bytes_bigger_or_equal_than_zero_mask = _mm_cmpgt_epi8(value, zeros);
        let bytes_smaller_or_equal_than_nine_mask = _mm_cmplt_epi8(value, nines);

        // and the two masks to get the valid bytes
        let valid_bytes_mask = _mm_and_si128(
            bytes_bigger_or_equal_than_zero_mask,
            bytes_smaller_or_equal_than_nine_mask,
        );

        // return if all the bits are 1s
        _mm_test_all_ones(valid_bytes_mask) == 1
    }
}

#[inline]
/// Finds the first nonnumeric value in the string.
///
/// This *assumes* that the string has exactly 16 chars
/// and it's padded with zeros if necessary.
pub fn first_byte_non_numeric(string: &str) -> u32 {
    unsafe {
        // create costant registers
        let commas = _mm_set1_epi8(b',' as i8);
        let newlines = _mm_set1_epi8(b'\n' as i8);

        // load data into memory
        let value = _mm_loadu_si128(string.as_ptr() as _);

        // compare for equality and find the occurences of commas and newlines
        let comma_mask = _mm_cmpeq_epi8(value, commas);
        let newline_mask = _mm_cmpeq_epi8(value, newlines);

        // create the OR of the two regiters to place the first index correctly
        let or_comma_newline = _mm_or_si128(comma_mask, newline_mask);

        // Since there is no instruction to convert 128 integer into a u128,
        // first load to upper 64 bits and then the lower 64 bits, shifting
        // by 8 bytes.
        let lower = _mm_cvtsi128_si64(or_comma_newline) as u64;
        let or_comma_newline = _mm_bsrli_si128(or_comma_newline, 8);
        let upper = _mm_cvtsi128_si64(or_comma_newline) as u64;

        // if the first 8 bytes are all set to zero, check the upper part
        if lower == 0 {
            // if also upper is 0, then no separator is present
            if upper == 0 {
                return 0;
            }
            // divide by 8 because we want to count in byte, and add 9 because
            // lower is all zeros, so 8 byte are skipped, plus 1 because the comma
            // or newline is not included in the trailing count.
            upper.trailing_zeros() / 8 + 9
        } else {
            // the comma or newline is in the first 8 bytes of the number
            lower.trailing_zeros() / 8 + 1
        }
    }
}

/// Creates a mask to remove all the elements after the separator.
///
/// By default, this method matches only ',' and '\n'
pub fn create_parsing_mask(s: &str) -> __m128i {
    unsafe {
        // create costant registers
        let commas = _mm_set1_epi8(b',' as i8);
        let newlines = _mm_set1_epi8(b'\n' as i8);

        // load data into memory
        let value = _mm_loadu_si128(s.as_ptr() as _);

        // compare for equality and find the occurences of commas and newlines
        let comma_mask = _mm_cmpeq_epi8(value, commas);
        let newline_mask = _mm_cmpeq_epi8(value, newlines);

        // create the OR of the two regiters to place the first index correctly
        let or_comma_newline = _mm_or_si128(comma_mask, newline_mask);

        propagate(or_comma_newline)
    }
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

/// Parses an integer from the input string until a delimiter is encountered.
///
/// By default, it uses ',' and '\n' as delimiter. To parse the digits, it
/// exploits the fact that in ASCII encoding, digits are stored in the 4 least
/// significant bits of the ASCII code. As example, consider '1': in binary is
/// 0011-0001, and masking with 0x0f we get 0000-0001, which is 1.
pub fn parse_integer_byte_iterator(s: &str) -> u32 {
    s.bytes()
        .take_while(|&byte| (byte != b',') && (byte != b'\n'))
        .fold(0, |a, c| a * 10 + (c & 0x0f) as u32)
}

/// Parses 8 integers from input string using SIMD instructons.
///
/// The input string *must have* at least 16 chars, otherwise the internal
/// operations will load memory outside the string bound.
pub fn parse_8_chars_simd(s: &str) -> u32 {
    unsafe {
        let mut chunk = _mm_lddqu_si128(s.as_ptr() as *const _);
        // do not touch last 8 chars, since we don't know what they contain, avoiding
        // any kind of underflow
        let zeros = _mm_set_epi8(
            0, 0, 0, 0, 0, 0, 0, 0, b'0' as i8, b'0' as i8, b'0' as i8, b'0' as i8, b'0' as i8,
            b'0' as i8, b'0' as i8, b'0' as i8,
        );
        chunk = _mm_sub_epi16(chunk, zeros);

        let mult = _mm_set_epi8(0, 0, 0, 0, 0, 0, 0, 0, 1, 10, 1, 10, 1, 10, 1, 10);
        chunk = _mm_maddubs_epi16(chunk, mult);

        let mult = _mm_set_epi16(1, 100, 1, 100, 1, 100, 1, 100);
        chunk = _mm_madd_epi16(chunk, mult);

        chunk = _mm_packus_epi32(chunk, chunk);
        let mult = _mm_set_epi16(1, 10000, 1, 10000, 1, 10000, 1, 10000);
        chunk = _mm_madd_epi16(chunk, mult);

        let chunk = _mm_cvtsi128_si32(chunk) as u32;
        chunk
    }
}

pub fn parse_integer_simd_all_numbers(s: &str) -> u32 {
    unsafe {
        let mut chunk = _mm_lddqu_si128(s.as_ptr() as *const _);
        let zeros = _mm_set1_epi8(b'0' as i8);
        chunk = _mm_sub_epi16(chunk, zeros);

        let mult = _mm_set_epi8(1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10);
        chunk = _mm_maddubs_epi16(chunk, mult);

        let mult = _mm_set_epi16(1, 100, 1, 100, 1, 100, 1, 100);
        chunk = _mm_madd_epi16(chunk, mult);

        chunk = _mm_packus_epi32(chunk, chunk);

        let mult = _mm_set_epi16(0, 0, 0, 0, 1, 10000, 1, 10000);
        chunk = _mm_madd_epi16(chunk, mult);

        let chunk = _mm_cvtsi128_si64(chunk) as u64;
        (((chunk & 0xffffffff) * 100000000) + (chunk >> 32)) as u32
    }
}

pub fn parse_less_than_8_simd(s: &str, scaling_factor: u32) -> u32 {
    unsafe {
        let mut chunk = _mm_lddqu_si128(s.as_ptr() as *const _);
        let zeros = _mm_set1_epi8(b'0' as i8);
        chunk = _mm_sub_epi16(chunk, zeros);

        // create the mask and remove the unwanted part of the number to not parse
        let mask = create_parsing_mask(s);
        chunk = _mm_andnot_si128(mask, chunk);

        let mult = _mm_set_epi8(0, 0, 0, 0, 0, 0, 0, 0, 1, 10, 1, 10, 1, 10, 1, 10);
        chunk = _mm_maddubs_epi16(chunk, mult);

        let mult = _mm_set_epi16(1, 100, 1, 100, 1, 100, 1, 100);
        chunk = _mm_madd_epi16(chunk, mult);

        chunk = _mm_packus_epi32(chunk, chunk);
        let mult = _mm_set_epi16(1, 10000, 1, 10000, 1, 10000, 1, 10000);
        chunk = _mm_madd_epi16(chunk, mult);

        let chunk = _mm_cvtsi128_si32(chunk) as u32;
        chunk / scaling_factor
    }
}

pub fn parse_4_or_less_chars(s: &str, chars_to_parse: u32) -> u32 {
    s.bytes()
        .take(chars_to_parse as usize)
        .fold(0, |a, c| a * 10 + (c & 0x0f) as u32)
}

/// Parses an integer of at least 8 digits from the string with SIMD.
///
/// This method *assumes* that the input string has exactly 16 chars, eventually
/// padded with zeros.
pub fn parse_more_than_8_simd(s: &str, scaling_factor: u64) -> u32 {
    unsafe {
        let mut chunk = _mm_lddqu_si128(std::mem::transmute_copy(&s));
        let zeros = _mm_set1_epi8(b'0' as i8);
        chunk = _mm_sub_epi16(chunk, zeros);

        // create the mask and remove the unwanted part of the number to not parse
        let mask = create_parsing_mask(s);
        chunk = _mm_andnot_si128(mask, chunk);

        let mult = _mm_set_epi8(1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10);
        chunk = _mm_maddubs_epi16(chunk, mult);

        let mult = _mm_set_epi16(1, 100, 1, 100, 1, 100, 1, 100);
        chunk = _mm_madd_epi16(chunk, mult);

        chunk = _mm_packus_epi32(chunk, chunk);

        let mult = _mm_set_epi16(0, 0, 0, 0, 1, 10000, 1, 10000);
        chunk = _mm_madd_epi16(chunk, mult);

        let chunk = _mm_cvtsi128_si64(chunk) as u64;
        ((((chunk & 0xffffffff) * 100000000) + (chunk >> 32)) / scaling_factor) as u32
    }
}
