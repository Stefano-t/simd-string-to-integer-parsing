#![no_main]
use libfuzzer_sys::fuzz_target;
use simd_parsing::parse_integer_separator;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // TODO: change the hardcoded separators to something inside inputa data
        let _ = parse_integer_separator(s, b',', b'\n');
    }
});
