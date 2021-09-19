#![feature(stdsimd)]

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
unsafe fn propagate(mut v: __m128i) -> __m128i {
    v = _mm_or_si128(v, _mm_slli_si128(v as _, 1) as _);
    v = _mm_or_si128(v, _mm_slli_si128(v as _, 2) as _);
    v = _mm_or_si128(v, _mm_slli_si128(v as _, 4) as _);
    v = _mm_or_si128(v, _mm_slli_si128(v as _, 8) as _);
    v
}

/// Finds the first occurences of comma or newline.
///
/// In case of error, it returns the u32::MAX
pub fn naive_find_char(string: &str) -> u32 {
    for (pos, c) in string.bytes().enumerate() {
        if c == b',' || c == b'\n' {
            return pos as u32;
        }
    }
    return u32::MAX;
}

/// Parses integer from string iterating through each char.
///
/// This function doesn't perform any kind of input validation: it assumes that the string
/// is composed by numeric values.
pub fn naive_parsing(s: &str) -> u32 {
    let mut result = 0;
    for digit in s.chars() {
        result *= 10;
        result += digit as u32 - '0' as u32;
    }
    result
}

/// Parses integer from string using folding function.
///
/// This function doesn't perform any cheking, neither in the chars in the string if they are
/// all numberic, nor on the lenght of the string.
pub fn naive_parsing_limit(s: &str) -> u32 {
    s.chars()
        .take(10)
        .fold(0, |a, c| a * 10 + c as u32 - '0' as u32)
}

pub fn naive_bytes(s: &str) -> u32 {
    let mut result = 0;
    for digit in s.bytes() {
        result *= 10;
        result += (digit - b'0') as u32;
    }
    result
}

pub fn naive_bytes_iter(s: &str) -> u32 {
    s.bytes().fold(0, |a, c| a * 10 + (c - b'0') as u32)
}

pub fn naive_bytes_and(s: &str) -> u32 {
    s.bytes().fold(0, |a, c| a * 10 + (c & 0x0f) as u32)
}

fn parse_8_chars(s: &str) -> u64 {
    // cast to a raw pointer
    let s = s.as_ptr() as *const _;
    let mut chunk = 0;
    unsafe {
        // copy the content of s into chuck
        std::ptr::copy_nonoverlapping(s, &mut chunk, std::mem::size_of_val(&chunk));
    }

    // the hex numbers must be read from right to left.
    // Masking with 0x0f will extract from the ASCII char only the required number,
    // because it is stored in 4 right most digits.
    // Example: '1' -> ascii 49 -> 00110001 -> 0011 0001 & 0x0f = 0001 = 1
    // Example: '9' -> ascii 57 -> 00111001 -> 0011 1001 & 0x0f = 1001 = 9

    // 1-byte mask trick (works on 4 pairs of single digits)
    let lower_digits = (chunk & 0x0f000f000f000f00) >> 8;
    let upper_digits = (chunk & 0x000f000f000f000f) * 10;
    let chunk = lower_digits + upper_digits;

    // 2-byte mask trick (works on 2 pairs of two digits)
    let lower_digits = (chunk & 0x00ff000000ff0000) >> 16;
    let upper_digits = (chunk & 0x000000ff000000ff) * 100;
    let chunk = lower_digits + upper_digits;

    // 4-byte mask trick (works on a pair of four digits)
    let lower_digits = (chunk & 0x0000ffff00000000) >> 32;
    let upper_digits = (chunk & 0x000000000000ffff) * 10000;
    let chunk = lower_digits + upper_digits;

    chunk
}

pub fn trick(s: &str) -> u64 {
    let (upper_digits, lower_digits) = s.split_at(8);
    // parse 16 digits from the string, the upper 8 digits and then the 8
    // lower digits
    parse_8_chars(upper_digits) * 100000000 + parse_8_chars(lower_digits)
}

/// Parses an integer from the string with SIMD.
///
/// This method *assumes* that the input string has exactly 16 chars, eventually
/// padded with zeros.
pub fn trick_simd(s: &str) -> u32 {
    let to_parse: [u8; 16] = [48; 16];
    let ptr_to_parse = to_parse.as_ptr() as *mut u8;
    unsafe {
        let index = first_byte_non_numeric(s) - 1;
        std::ptr::copy_nonoverlapping(
            s.as_ptr() as *const _,
            ptr_to_parse.add((16 - index) as usize),
            index as usize,
        );
        let mut chunk = _mm_lddqu_si128(ptr_to_parse as *const _);
        //let mut chunk = _mm_lddqu_si128(std::mem::transmute_copy(&s));
        let zeros = _mm_set1_epi8(b'0' as i8);
        chunk = _mm_sub_epi16(chunk, zeros);

        // if !check_all_chars_are_valid(s) {
        //     let mask = create_parsing_mask(s);
        //     chunk = _mm_and_si128(mask, chunk);
        // }

        // last 6 digits must be set to 0 becuase the max u32 integer has 10
        // digits, and we are processing a string with 16 chars. So we need to
        // remove the first 6 digits. Since the number is loaded in little
        // endian form, the first 6 digits become the last 6 digits.
        let mult = _mm_set_epi8(1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10);
        chunk = _mm_maddubs_epi16(chunk, mult);

        // remove the first 3 digits of the number
        let mult = _mm_set_epi16(1, 100, 1, 100, 1, 100, 1, 100);
        chunk = _mm_madd_epi16(chunk, mult);

        chunk = _mm_packus_epi32(chunk, chunk);
        // trim off the first digit and invalidate the first 4 couples of 2
        // bytes, since we are working with chucks of 4 bytes.
        let mult = _mm_set_epi16(0, 0, 0, 0, 1, 10000, 1, 10000);
        chunk = _mm_madd_epi16(chunk, mult);

        let chunk = _mm_cvtsi128_si64(chunk) as u64;
        (((chunk & 0x00000000ffffffff) * 100000000) + (chunk >> 32)) as u32
    }
}
