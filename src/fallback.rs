//! Fallback implementations for parsing an u32 from a string

/// Parses an integer from the input string until a delimiter is encountered.
///
/// To parse the digits, it exploits the fact that in ASCII encoding, digits are
/// stored in the 4 least significant bits of the ASCII code. As example,
/// consider '1': in binary is 0011-0001, and masking with 0x0F we get
/// 0000-0001, which is 1.
#[inline]
pub fn parse_integer_separator(s: &str, separator: u8, eol: u8) -> Option<u32> {
    // Extract the iter
    let mut iter = s.bytes()
        .take_while(|&byte| (byte != separator) && (byte != eol));

    // Control if there is at least one element
    let first = iter.next()?;

    // Fold the result and check for overflow
    iter.try_fold((first & 0x0F) as u32, |a, c| {
        if let Some(m) = a.checked_mul(10u32) {
            Some(m + (c & 0x0F) as u32)
        } else {
            None
        }
    })
}

/// Parses an u32 from the input string up to the first occurence of `separator`
/// or `eol`.
///
/// If the input string is empty, then the result will be 0.
///
/// # Safety
///
/// No kind of overflow check is performed inside this method: if the input
/// string contains a number which doens't fit in a `u32`, a panic will be
/// thrown.
#[inline]
pub unsafe fn parse_integer_separator_unchecked(
    s: &str,
    separator: u8,
    eol: u8
) -> u32 {
    s.bytes()
        .take_while(|&byte| (byte != separator) && (byte != eol))
        .fold(0u32, |a, c| (a * 10) + (c & 0x0F) as u32)
}

/// Parses an integer from the input string
///
/// To parse the digits, it exploits the fact that in ASCII encoding, digits are
/// stored in the 4 least significant bits of the ASCII code. As example,
/// consider '1': in binary is 0011-0001, and masking with 0x0f we get
/// 0000-0001, which is 1.
#[inline]
pub fn parse_integer(s: &str) -> Option<u32> {
    // Extract the iter
    let mut iter = s.bytes()
        .take_while(|&byte| (byte >= b'0') && (byte <= b'9'));

    // Control if there is at least one element
    let first = iter.next()?;

    // Fold the result and check for overflow
    iter.try_fold((first & 0x0F) as u32, |a, c| {
        if let Some(m) = a.checked_mul(10u32) {
            Some(m + (c & 0x0F) as u32)
        } else {
            None
        }
    })
}

/// Parses an u32 from the given string
///
/// This method doesn't check if the input string contains only digits, so make
/// sure to supply a valid string. If the input string is empty, the result will
/// be 0.
///
/// # Safety
///
/// No kind of overflow check is performed inside this method. So, if the input
/// string contains a number which doesn't fit in a `u32`, a panic will be
/// thrown. Furthermore, if the string contains other chars besides digits, they
/// will parsed as valid digits, corrupting the result.
#[inline]
pub unsafe fn parse_integer_unchecked(s: &str) -> u32 {
    s.bytes()
        .fold(0u32, |a, c| (a * 10) + (c & 0x0F) as u32)
}

/// Parses a limited amount of digits from the string
#[inline]
pub fn parse_byte_iterator_limited(s: &str, chars_to_parse: u32) -> u32 {
    s.bytes()
        .take(chars_to_parse as usize)
        .fold(0, |a, c| a * 10 + (c & 0x0f) as u32)
}

/// Checks if the string is composed of all numbers
#[inline]
pub fn check_all_chars_are_valid(s: &str) -> bool {
    s.bytes().all(|b| b >= b'0' && b <= b'9')
}

/// Returns the index of the last digit not equals to separator or eol
#[inline]
pub fn last_byte_without_separator(s: &str, separator: u8, eol: u8) -> u32 {
    s.bytes()
        .take_while(|&byte| (byte != separator) && (byte != eol))
        .count() as u32
}

/// Returns the index of the last digit in the string
#[inline]
pub fn last_digit_byte(s: &str) -> u32 {
    s.bytes()
        .take_while(|&b| (b >= b'0') && (b <= b'9'))
        .count() as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    static SEP: u8 = b',';
    static EOL: u8 = b'\n';

    #[test]
    fn last_byte_without_separator_no_digit() {
        let s = ",1234.4321\n";
        assert_eq!(last_byte_without_separator(s, SEP, EOL), 0);
    }

    #[test]
    fn last_byte_without_separator_one_digit() {
        let s = "1,2344321";
        assert_eq!(last_byte_without_separator(s, SEP, EOL), 1);
    }

    #[test]
    fn last_byte_without_separator_more_digits() {
        let s = "123,44321\n";
        assert_eq!(last_byte_without_separator(s, SEP, EOL), 3);
    }

    #[test]
    fn last_byte_without_separator_empty_str() {
        let s = "";
        assert_eq!(last_byte_without_separator(s, SEP, EOL), 0);
    }

    #[test]
    fn last_digit_byte_empty_str() {
        let s = "";
        assert_eq!(last_digit_byte(s), 0);
    }

    #[test]
    fn last_digit_byte_one_digit() {
        let s = "1";
        assert_eq!(last_digit_byte(s), 1);
    }

    #[test]
    fn last_digit_byte_more_digits() {
        let s = "012349";
        assert_eq!(last_digit_byte(s), 6);
    }

    #[test]
    fn last_digit_byte_more_digits_and_separators() {
        let s = "0123,!49";
        assert_eq!(last_digit_byte(s), 4);
    }
    
    #[test]
    fn check_all_chars_are_valid_one_digit() {
        let s = "1";
        assert!(check_all_chars_are_valid(s));
    }

    #[test]
    fn check_all_chars_are_valid_more_digits() {
        let s = "12345";
        assert!(check_all_chars_are_valid(s));
    }

    #[test]
    fn check_all_chars_are_valid_invalid() {
        let s = "1234,412";
        assert!(!check_all_chars_are_valid(s));
    }

    #[test]
    fn check_all_chars_are_valid_empty() {
        let s = "";
        assert!(check_all_chars_are_valid(s));
    }

    #[test]
    fn parse_byte_iterator_limited_one_digit() {
        let s = "12345678";
        assert_eq!(parse_byte_iterator_limited(s, 1), 1);
    }

    #[test]
    fn parse_byte_iterator_limited_more_digits() {
        let s = "12345678";
        assert_eq!(parse_byte_iterator_limited(s, 4), 1234);
    }

    #[test]
    fn parse_byte_iterator_limited_zero_index() {
        let s = "12345678";
        assert_eq!(parse_byte_iterator_limited(s, 0), 0);
    }

    #[test]
    fn parse_byte_iterator_limited_empty_but_index() {
        let s = "";
        assert_eq!(parse_byte_iterator_limited(s, 1), 0);
    }

    #[test]
    fn parse_integer_separator_no_separator() {
        let s = "12345678";
        assert_eq!(parse_integer_separator(s, SEP, EOL), Some(12345678));
    }

    #[test]
    fn parse_integer_separator_separator() {
        let s = "123,45678";
        assert_eq!(parse_integer_separator(s, SEP, EOL), Some(123));
    }

    #[test]
    fn parse_integer_separator_empty() {
        let s = "";
        assert_eq!(parse_integer_separator(s, SEP, EOL), None);
    }

    #[test]
    fn parse_integer_separator_only_separators() {
        let s = "\n\n,,";
        assert_eq!(parse_integer_separator(s, SEP, EOL), None);
    }

    #[test]
    fn parse_integer_empty() {
        let s = "";
        assert_eq!(parse_integer(s), None);
    }

    #[test]
    fn parse_integer_max_u32() {
        let s = format!("{}", u32::MAX);
        assert_eq!(parse_integer(&s), Some(u32::MAX));
    }
}
