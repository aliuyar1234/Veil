//! IBAN validation using MOD-97 algorithm (ISO 7064).

/// Validate an IBAN using the MOD-97 checksum algorithm.
///
/// # Arguments
/// * `iban` - The IBAN string to validate (with or without spaces)
///
/// # Returns
/// `true` if the IBAN is valid, `false` otherwise
pub fn validate_iban(iban: &str) -> bool {
    // Remove spaces and convert to uppercase
    let clean: String = iban
        .chars()
        .filter(|c| c.is_alphanumeric())
        .map(|c| c.to_ascii_uppercase())
        .collect();

    // Check length (15-34 characters)
    if clean.len() < 15 || clean.len() > 34 {
        return false;
    }

    // Check country code (first 2 chars must be letters)
    let country_code = &clean[..2];
    if !country_code.chars().all(|c| c.is_ascii_alphabetic()) {
        return false;
    }

    // Check check digits (chars 3-4 must be digits)
    let check_digits = &clean[2..4];
    if !check_digits.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }

    // Move first 4 chars to end
    let rearranged = format!("{}{}", &clean[4..], &clean[..4]);

    // Convert letters to numbers (A=10, B=11, ..., Z=35)
    let numeric: String = rearranged
        .chars()
        .map(|c| {
            if c.is_ascii_alphabetic() {
                format!("{}", c as u32 - 55)
            } else {
                c.to_string()
            }
        })
        .collect();

    // MOD 97 check
    mod97(&numeric) == 1
}

/// Calculate MOD 97 of a numeric string.
fn mod97(digits: &str) -> u32 {
    digits.chars().fold(0u32, |acc, c| {
        let digit = c.to_digit(10).unwrap_or(0);
        (acc * 10 + digit) % 97
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_german_iban() {
        assert!(validate_iban("DE89370400440532013000"));
    }

    #[test]
    fn test_valid_austrian_iban() {
        assert!(validate_iban("AT611904300234573201"));
    }

    #[test]
    fn test_valid_swiss_iban() {
        assert!(validate_iban("CH9300762011623852957"));
    }

    #[test]
    fn test_valid_iban_with_spaces() {
        assert!(validate_iban("DE89 3704 0044 0532 0130 00"));
    }

    #[test]
    fn test_invalid_checksum() {
        assert!(!validate_iban("DE89370400440532013001"));
    }

    #[test]
    fn test_too_short() {
        assert!(!validate_iban("DE8937040044"));
    }

    #[test]
    fn test_too_long() {
        let long_iban = "DE89".to_string() + &"0".repeat(35);
        assert!(!validate_iban(&long_iban));
    }
}
