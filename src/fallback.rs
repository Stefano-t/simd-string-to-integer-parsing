/// Parses an integer from the input string until a delimiter is encountered.
///
/// By default, it uses ',' and '\n' as delimiter. To parse the digits, it
/// exploits the fact that in ASCII encoding, digits are stored in the 4 least
/// significant bits of the ASCII code. As example, consider '1': in binary is
/// 0011-0001, and masking with 0x0f we get 0000-0001, which is 1.
pub fn parse_integer_byte_iterator(s: &str, separator: u8, eol: u8) -> Option<u32> {
    s.bytes()
        .take_while(|&byte| (byte != separator) && (byte != eol))
        .map(|x| (x & 0x0f) as u32)
        .reduce(|a, c| a * 10 + c)
}

/// Parses a limited amount of digits from the string
pub fn parse_byte_iterator_limited(s: &str, chars_to_parse: u32) -> u32 {
    s.bytes()
        .take(chars_to_parse as usize)
        .fold(0, |a, c| a * 10 + (c & 0x0f) as u32)
}

/// Checks if the string is composed of all numbers
pub fn check_all_chars_are_valid(s: &str) -> bool {
    s.bytes().all(|b| b >= b'0' && b <= b'9')
}
