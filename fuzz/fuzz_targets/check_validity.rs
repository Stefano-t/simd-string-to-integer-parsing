#![no_main]
use libfuzzer_sys::fuzz_target;
use simd_parsing::check_all_chars_are_valid;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = check_all_chars_are_valid(s);
    }
});
