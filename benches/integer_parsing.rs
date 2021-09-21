#![feature(test, asm)]
#![feature(option_result_unwrap_unchecked)]
#![allow(clippy::unreadable_literal)]
extern crate test;

use simd_string_to_integer_parsing::*;
use test::{black_box, Bencher};


// max integer with 32 bit is 4294967295
//const TEST_STR: &str = "1694206942";
//const TEST_RES: u32 = 1694206942;

#[bench]
fn bench_std_parse(b: &mut Bencher) {
    let s = "1211120134";
    b.bytes = s.len() as u64;
    b.iter(|| black_box(s).parse::<u32>().unwrap())
}

#[bench]
fn bench_check_chars_validity(b: &mut Bencher) {
    b.bytes = TEST_STR.len() as u64;

    let padded_str = format!("{:0>16}", TEST_STR);
    assert!(check_all_chars_are_valid(&padded_str));
    b.iter(|| check_all_chars_are_valid(black_box(&padded_str)))
}

#[bench]
fn bench_first_byte_non_numeric(b: &mut Bencher) {
    let s = "11211,012301,010";
    let padded = format!("{:0>16}", s);
    b.bytes = padded.len() as u64;
    assert_eq!(first_byte_non_numeric(&padded), 6);
    b.iter(|| first_byte_non_numeric(black_box(&padded)))
}

#[bench]
fn bench_create_parsing_mask(b: &mut Bencher) {
    let s = "12345,234";
    let padded = format!("{:0>16}", s);
    b.bytes = padded.len() as u64;
    b.iter(|| create_parsing_mask(black_box(&padded)))
}

#[bench]
fn bench_parse_integer(b: &mut Bencher) {
    let strings = vec!("1234,234", "123456789", "1234567890,01234567");
    let results = vec!(1234, 123456789, 1234567890);
    for i in 0..strings.len() {
        let s = strings[i];
        let result = results[i];

        assert_eq!(parse_integer(&s), result);

        b.bytes = s.len() as u64;

        b.iter(|| parse_integer(black_box(&s)))
    }
}
// compile command:
// RUSTFLAGS='-C target-cpu=native' cargo bench
