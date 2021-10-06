#![feature(test, asm)]
#![feature(option_result_unwrap_unchecked)]
#![allow(clippy::unreadable_literal)]
extern crate test;

use simd_string_to_integer_parsing::*;
use test::{black_box, Bencher};

struct TestCase<'a> {
    input: &'a str,
    expected: u32,
}

const TEST_SUITE: [TestCase; 9] = [
    // test cases with less than 16 chars in input string
    TestCase {
        input: "0",
        expected: 0,
    },
    TestCase {
        input: "1",
        expected: 1,
    },
    TestCase {
        input: "1234",
        expected: 1234,
    },
    TestCase {
        input: "123456789",
        expected: 123456789,
    },
    TestCase {
        input: "12345,6789",
        expected: 12345,
    },
    TestCase {
        input: "123,4567",
        expected: 123,
    },
    // test cases with 16 chars or more in input string
    TestCase {
        input: "12345678,8900000",
        expected: 12345678,
    },
    TestCase {
        input: "123,111111111111",
        expected: 123,
    },
    TestCase {
        // padded input
        input: "0000000123456789",
        expected: 123456789,
    },
];

const TEST_SUITE_NO_SEPARATOR: [TestCase; 8] = [
    // test cases with less than 16 chars in input string
    TestCase {
        input: "0",
        expected: 0,
    },
    TestCase {
        input: "1",
        expected: 1,
    },
    TestCase {
        input: "1234",
        expected: 1234,
    },
    TestCase {
        input: "123456789",
        expected: 123456789,
    },
    // test cases with at lest 16 chars zero padded
    TestCase {
        input: "0000000123456789",
        expected: 123456789,
    },
    TestCase {
        input: "0000000000000789",
        expected: 789,
    },
    TestCase {
        input: "0000000000012345",
        expected: 12345,
    },
    TestCase {
        input: "0000000000000001",
        expected: 1,
    },
];

// max integer with 32 bit is 4294967295
const TEST_STR: &str = "1694206942";
const TEST_RES: u32 = 1694206942;

#[bench]
fn bench_std_parsing(b: &mut Bencher) {
    b.bytes = TEST_SUITE_NO_SEPARATOR
        .iter()
        .map(|test_case| test_case.input.len() as u64)
        .reduce(|a, b| a + b)
        .unwrap();
    b.iter(|| {
        for test_case in TEST_SUITE_NO_SEPARATOR.iter() {
            black_box(test_case.input).parse::<u32>().unwrap();
        }
    });
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
    for test_case in TEST_SUITE.iter() {
        assert_eq!(parse_integer(test_case.input), test_case.expected);
    }
    b.bytes = TEST_SUITE
        .iter()
        .map(|test_case| test_case.input.len() as u64)
        .reduce(|a, b| a + b)
        .unwrap();
    b.iter(|| {
        for test_case in TEST_SUITE.iter() {
            parse_integer(black_box(test_case.input));
        }
    });
}

#[bench]
fn bench_parse_integer_no_separator(b: &mut Bencher) {
    for test_case in TEST_SUITE_NO_SEPARATOR.iter() {
        assert_eq!(parse_integer(test_case.input), test_case.expected);
    }
    b.bytes = TEST_SUITE_NO_SEPARATOR
        .iter()
        .map(|test_case| test_case.input.len() as u64)
        .reduce(|a, b| a + b)
        .unwrap();
    b.iter(|| {
        for test_case in TEST_SUITE_NO_SEPARATOR.iter() {
            parse_integer(black_box(test_case.input));
        }
    });
}
// compile command:
// RUSTFLAGS='-C target-cpu=native' cargo bench
