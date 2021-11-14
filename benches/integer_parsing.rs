#![feature(test, asm)]
#![feature(option_result_unwrap_unchecked)]
#![allow(clippy::unreadable_literal)]
extern crate test;

use simd_parsing::*;
use test::{black_box, Bencher};

// max integer with 32 bit is 4294967295

#[bench]
fn bench_check_chars_validity_valid_fallback(b: &mut Bencher) {
    let case = "000000123456789";
    b.bytes = case.len() as u64;
    assert!(check_all_chars_are_valid(&case));
    b.iter(|| fallback::check_all_chars_are_valid(black_box(&case)))
}

#[bench]
fn bench_check_chars_validity_valid_sse41(b: &mut Bencher) {
    let case = "1234567890123456";
    b.bytes = case.len() as u64;
    unsafe {
        b.iter(|| sse41::check_all_chars_are_valid(black_box(&case)));
    }
}

#[bench]
fn bench_check_chars_validity_valid_sse42(b: &mut Bencher) {
    let case = "1234567890123456";
    b.bytes = case.len() as u64;
    unsafe {
        b.iter(|| sse42::check_all_chars_are_valid(black_box(&case)));
    }
}

#[bench]
fn bench_check_chars_validity_valid_avx(b: &mut Bencher) {
    let case = "12345678901234567890123456789012";
    b.bytes = case.len() as u64;
    unsafe {
        b.iter(|| avx::check_all_chars_are_valid(black_box(&case)));
    }
}

#[bench]
fn bench_last_byte_without_separator_fallback(b: &mut Bencher) {
    let case = "0000001,2345678";
    b.bytes = case.len() as u64;
    b.iter(|| last_byte_without_separator(black_box(&case), b',', b'\n'))
}

#[bench]
fn bench_last_byte_without_separator_sse41(b: &mut Bencher) {
    let case = "1234567,23456781";
    b.bytes = case.len() as u64;
    unsafe { b.iter(|| sse41::last_byte_without_separator(black_box(&case), b',', b'\n')) }
}

#[bench]
fn bench_last_byte_without_separator_sse42(b: &mut Bencher) {
    let case = "0000001,23456789";
    b.bytes = case.len() as u64;
    unsafe { b.iter(|| sse42::last_byte_without_separator(black_box(&case), b',', b'\n')) }
}

#[bench]
fn bench_last_byte_without_separator_avx(b: &mut Bencher) {
    let case = "1234567,234567810123456789012345";
    b.bytes = case.len() as u64;
    unsafe { b.iter(|| avx::last_byte_without_separator(black_box(&case), b',', b'\n')) }
}

#[bench]
fn bench_parse_integer_2_digits_fallback(b: &mut Bencher) {
    let case = "12";
    b.bytes = case.len() as u64;
    b.iter(|| fallback::parse_integer_byte_iterator(black_box(&case), b',', b'\n'))
}

#[bench]
#[cfg(feature = "benchmark")]
fn bench_parse_integer_2_digits_sse41_separator(b: &mut Bencher) {
    let case = "12,1111111111111";
    b.bytes = case.len() as u64;
    b.iter(|| safe_parse_integer_sse41(black_box(&case), b',', b'\n'))
}

#[bench]
#[cfg(feature = "benchmark")]
fn bench_parse_integer_2_digits_avx_separator(b: &mut Bencher) {
    let case = "12,11111111111112222222222222222";
    b.bytes = case.len() as u64;
    b.iter(|| safe_parse_integer_avx2(black_box(&case), b',', b'\n'))
}

#[bench]
fn bench_standard_parse_2_digits(b: &mut Bencher) {
    let case = "12";
    b.bytes = case.len() as u64;
    b.iter(|| black_box(&case).parse::<u32>())
}

#[bench]
fn bench_parse_integer_5_digits_fallback(b: &mut Bencher) {
    let case = "12345";
    b.bytes = case.len() as u64;
    b.iter(|| fallback::parse_integer_byte_iterator(black_box(&case), b',', b'\n'))
}

#[bench]
#[cfg(feature = "benchmark")]
fn bench_parse_integer_5_digits_sse41_separator(b: &mut Bencher) {
    let case = "12345,1234567890";
    b.bytes = case.len() as u64;
    b.iter(|| safe_parse_integer_sse41(black_box(&case), b',', b'\n'))
}

#[bench]
#[cfg(feature = "benchmark")]
fn bench_parse_integer_5_digits_avx_separator(b: &mut Bencher) {
    let case = "12345,12345678901234567890123456";
    b.bytes = case.len() as u64;
    b.iter(|| safe_parse_integer_avx2(black_box(&case), b',', b'\n'))
}

#[bench]
fn bench_standard_parse_5_digits(b: &mut Bencher) {
    let case = "12345";
    b.bytes = case.len() as u64;
    b.iter(|| black_box(&case).parse::<u32>())
}

#[bench]
fn bench_parse_integer_10_digits_fallback(b: &mut Bencher) {
    let case = "1234567890";
    assert_eq!(parse_integer(&case, b',', b'\n'), Some(1234567890));
    b.bytes = case.len() as u64;
    b.iter(|| fallback::parse_integer_byte_iterator(black_box(&case), b',', b'\n'))
}

#[bench]
#[cfg(feature = "benchmark")]
fn bench_parse_integer_10_digits_sse41_separator(b: &mut Bencher) {
    let case = "1234512345,67890";
    b.bytes = case.len() as u64;
    b.iter(|| safe_parse_integer_sse41(black_box(&case), b',', b'\n'))
}

#[bench]
#[cfg(feature = "benchmark")]
fn bench_parse_integer_10_digits_avx_separator(b: &mut Bencher) {
    let case = "1234512345,678901234567890123456";
    b.bytes = case.len() as u64;
    b.iter(|| safe_parse_integer_avx2(black_box(&case), b',', b'\n'))
}

#[bench]
fn bench_standard_parse_10_digits(b: &mut Bencher) {
    let case = "1234567890";
    b.bytes = case.len() as u64;
    b.iter(|| black_box(&case).parse::<u32>().unwrap())
}

// compile command:
// RUSTFLAGS='-C target-cpu=native' cargo bench
