#![no_main]
use libfuzzer_sys::fuzz_target;
use simd_parsing::last_byte_without_separator;

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }
    // Here we have at least one element in the buffer
    if let Ok(s) = std::str::from_utf8(data) {
        // Take the element in the middle as separator
        let sep = data[data.len() / 2];
        let _ = last_byte_without_separator(s, b',', b'\n');
    }
});
