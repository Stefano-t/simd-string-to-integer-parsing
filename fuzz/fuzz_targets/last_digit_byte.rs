#![no_main]
use libfuzzer_sys::fuzz_target;
use simd_parsing::last_digit_byte;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = last_digit_byte(s);
    }
});
