#![feature(test, asm)]
#![feature(option_result_unwrap_unchecked)]
#![allow(clippy::unreadable_literal)]
extern crate test;

use simd_string_to_integer_parsing::*;
use test::{black_box, Bencher};


// max integer with 32 bit is 4294967295
const TEST_STR: &str = "694206942";
const TEST_RES: u32 = 694206942;

#[bench]
fn bench_std_parse(b: &mut Bencher) {
    b.bytes = TEST_STR.len() as u64;
    b.iter(|| black_box(TEST_STR).parse::<u32>().unwrap())
}

#[bench]
fn bench_naive_parsing(b: &mut Bencher) {
    assert_eq!(naive_parsing(TEST_STR), TEST_RES);
    b.bytes = TEST_STR.len() as u64;
    b.iter(|| naive_parsing(black_box(TEST_STR)))
}

#[bench]
fn bench_naive_parsing_limit(b: &mut Bencher) {
    assert_eq!(naive_parsing_limit(TEST_STR), TEST_RES);
    b.bytes = TEST_STR.len() as u64;
    b.iter(|| naive_parsing_limit(black_box(TEST_STR)))
}

#[bench]
fn bench_naive_bytes(b: &mut Bencher) {
    assert_eq!(naive_bytes(TEST_STR), TEST_RES);
    b.bytes = TEST_STR.len() as u64;
    b.iter(|| naive_bytes(black_box(TEST_STR)))
}

#[bench]
fn bench_naive_bytes_iter(b: &mut Bencher) {
    assert_eq!(naive_bytes_iter(TEST_STR), TEST_RES);
    b.bytes = TEST_STR.len() as u64;
    b.iter(|| naive_bytes_iter(black_box(TEST_STR)))
}

#[bench]
fn bench_naive_bytes_and(b: &mut Bencher) {
    assert_eq!(naive_bytes_and(TEST_STR), TEST_RES);
    b.bytes = TEST_STR.len() as u64;
    b.iter(|| naive_bytes_and(black_box(TEST_STR)))
}

#[bench]
fn bench_check_chars_validity(b: &mut Bencher) {
    b.bytes = TEST_STR.len() as u64;

    let padded_str = format!("{:0>16}", TEST_STR);
    assert!(check_all_chars_are_valid(&padded_str));
    b.iter(|| check_all_chars_are_valid(black_box(&padded_str)))
}

#[bench]
fn bench_trick(b: &mut Bencher) {
    // we need padding because the `trick` algorithm only works with 16 chars string.
    // Since the largest number we can represent with 32 bit has 10 digits, we need
    // to use 64 bit numbers in order to use the algotihms, because it works only with
    // sizes of the power of 2.
    let padded_str = format!("{:0>16}", TEST_STR);
    assert_eq!(trick(&padded_str), TEST_RES as u64);

    b.bytes = padded_str.len() as u64;

    b.iter(|| trick(black_box(&padded_str)))
}

#[bench]
fn bench_trick_simd(b: &mut Bencher) {
    let padded_str = format!("{:0>16}", TEST_STR);
    assert_eq!(trick_simd(&padded_str), TEST_RES as u64);

    b.bytes = TEST_STR.len() as u64;

    b.iter(|| trick_simd(black_box(&padded_str)))
}

// compile command:
// RUSTFLAGS='-C target-cpu=native' cargo bench
