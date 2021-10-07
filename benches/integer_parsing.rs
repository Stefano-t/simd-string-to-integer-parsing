#![feature(test, asm)]
#![feature(option_result_unwrap_unchecked)]
#![allow(clippy::unreadable_literal)]
extern crate test;

use simd_string_to_integer_parsing::*;
use test::{black_box, Bencher};

// max integer with 32 bit is 4294967295

#[bench]
fn bench_check_chars_validity_valid(b: &mut Bencher) {
    let case = "0000001234567890";
    b.bytes = case.len() as u64;
    assert!(check_all_chars_are_valid(&case));
    b.iter(|| check_all_chars_are_valid(black_box(&case)))
}

#[bench]
fn bench_check_chars_validity_invalid(b: &mut Bencher) {
    let case = "00000,1234567890";
    b.bytes = case.len() as u64;
    assert!(false == check_all_chars_are_valid(&case));
    b.iter(|| check_all_chars_are_valid(black_box(&case)))
}

#[bench]
fn bench_first_byte_non_numeric_no_separator(b: &mut Bencher) {
    let case = "0000001234567890";
    b.bytes = case.len() as u64;
    assert_eq!(first_byte_non_numeric(&case), 0);
    b.iter(|| first_byte_non_numeric(black_box(&case)))
}

#[bench]
fn bench_first_byte_non_numeric_one_separator(b: &mut Bencher) {
    let case = "00000,1234567890";
    b.bytes = case.len() as u64;
    assert_eq!(first_byte_non_numeric(&case), 6);
    b.iter(|| first_byte_non_numeric(black_box(&case)))
}

#[bench]
fn bench_first_byte_non_numeric_multiple_separator(b: &mut Bencher) {
    let case = "1234,34567,67891";
    b.bytes = case.len() as u64;
    assert_eq!(first_byte_non_numeric(&case), 5);
    b.iter(|| first_byte_non_numeric(black_box(&case)))
}

#[bench]
fn bench_create_parsing_mask(b: &mut Bencher) {
    let case = "12345,234";
    let padded = format!("{:0>16}", case);
    b.bytes = padded.len() as u64;
    b.iter(|| create_parsing_mask(black_box(&padded)))
}

#[bench]
fn bench_parse_integer_no_simd_2_digits(b: &mut Bencher) {
    let case = "12";
    assert_eq!(parse_integer(&case), 12);
    b.bytes = case.len() as u64;
    b.iter(|| parse_integer(black_box(&case)))
}

#[bench]
fn bench_std_parse_2_digits(b: &mut Bencher) {
    let case = "12";
    assert_eq!(parse_integer(&case), 12);
    b.bytes = case.len() as u64;
    b.iter(|| black_box(&case).parse::<u32>().unwrap())
}

#[bench]
fn bench_parse_integer_no_simd_5_digits(b: &mut Bencher) {
    let case = "12345";
    assert_eq!(parse_integer(&case), 12345);
    b.bytes = case.len() as u64;
    b.iter(|| parse_integer(black_box(&case)))
}

#[bench]
fn bench_std_parse_5_digits(b: &mut Bencher) {
    let case = "12345";
    assert_eq!(parse_integer(&case), 12345);
    b.bytes = case.len() as u64;
    b.iter(|| black_box(&case).parse::<u32>().unwrap())
}

#[bench]
fn bench_parse_integer_no_simd_10_digits(b: &mut Bencher) {
    let case = "1234567890";
    assert_eq!(parse_integer(&case), 1234567890);
    b.bytes = case.len() as u64;
    b.iter(|| parse_integer(black_box(&case)))
}

#[bench]
fn bench_std_parse_10_digits(b: &mut Bencher) {
    let case = "1234567890";
    assert_eq!(parse_integer(&case), 1234567890);
    b.bytes = case.len() as u64;
    b.iter(|| black_box(&case).parse::<u32>().unwrap())
}

#[bench]
fn bench_parse_integer_no_simd_10_digits_separator(b: &mut Bencher) {
    let case = "1234567890,1";
    assert_eq!(parse_integer(&case), 1234567890);
    b.bytes = case.len() as u64;
    b.iter(|| parse_integer(black_box(&case)))
}

#[bench]
fn bench_parse_integer_simd_2_digits(b: &mut Bencher) {
    let case = "12,0123456789012";
    assert_eq!(parse_integer(&case), 12);
    b.bytes = case.len() as u64;
    b.iter(|| parse_integer(black_box(&case)))
}

#[bench]
fn bench_parse_integer_simd_5_digits(b: &mut Bencher) {
    let case = "12012,3456789012";
    assert_eq!(parse_integer(&case), 12012);
    b.bytes = case.len() as u64;
    b.iter(|| parse_integer(black_box(&case)))
}

#[bench]
fn bench_parse_integer_simd_8_digits(b: &mut Bencher) {
    let case = "12012345,6789012";
    assert_eq!(parse_integer(&case), 12012345);
    b.bytes = case.len() as u64;
    b.iter(|| parse_integer(black_box(&case)))
}

#[bench]
fn bench_parse_integer_simd_10_digits(b: &mut Bencher) {
    let case = "1201234567,89012";
    assert_eq!(parse_integer(&case), 1201234567);
    b.bytes = case.len() as u64;
    b.iter(|| parse_integer(black_box(&case)))
}

#[bench]
fn bench_parse_integer_simd_no_separator(b: &mut Bencher) {
    let case = "0000001234567890";
    assert_eq!(parse_integer(&case), 1234567890);
    b.bytes = case.len() as u64;
    b.iter(|| parse_integer(black_box(&case)))
}

// compile command:
// RUSTFLAGS='-C target-cpu=native' cargo bench
