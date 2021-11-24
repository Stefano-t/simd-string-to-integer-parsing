use simd_parsing::*;

fn main() {
    let mut s = "12345";
    // `parse_integer` returns an Option, it is a safe version
    if let Some(n) = parse_integer(s) {
        println!("basic parsing: {}", n);
    }

    // My CPU supports AVX2, so the parsing methods will call that
    // intrinsics for parsing the number, both for checked and
    // unchecked methods. If your CPU supports only SSE4.1 or SSE4.2,
    // then your input string should have at least length 16 to get
    // the acceleration benefit.
    s = "12345678,11111111111111111111111";
    if let Some(n) = parse_integer(s) {
        println!("parsing with SIMD acceleration: {}", n);
    }

    // You can also specify a separator to check for
    if let Some(n) = parse_integer_separator(s, b',', b'\n') {
        println!("parsing with separator: {}", n);
    }

    // If the input string is empty, then `None` is returned
    s = "";
    if let None = parse_integer(s) {
        println!("None to parse");
    }

    // Returns `None` also if the input string has a number that
    // exceeds u32::MAX
    let s = format!("{}", u32::MAX as u64 + 1);
    if let None = parse_integer(&s) {
        println!("None to parse");
    }

    // You can also use the unsafe version of the two previous
    // parsing; they are faster but if the input string doesn't have a
    // digit to parse or the number of digits exceed u32::MAX, then
    // they will panic.
    let s = "12345678,11111111111111111111111";
    unsafe {
        println!("unchecked parsing: {}", parse_integer_unchecked(s));
        println!(
            "unchecked parsing with specified separator: {}",
            parse_integer_separator_unchecked(s, b',', b'\n')
        );
    }
}
