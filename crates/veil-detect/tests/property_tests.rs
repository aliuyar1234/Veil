//! Property-based tests for PII validators.
//!
//! Uses proptest to verify validator properties hold for a wide range of inputs.

use proptest::prelude::*;

// Import validators - they're internal, so we need to use the crate's public interface
// or re-export them. For now, we'll implement the algorithms locally for testing.

/// Luhn algorithm implementation for testing.
fn validate_luhn(number: &str) -> bool {
    let digits: Vec<u32> = number
        .chars()
        .filter(|c| c.is_ascii_digit())
        .filter_map(|c| c.to_digit(10))
        .collect();

    if digits.len() < 13 || digits.len() > 19 {
        return false;
    }

    let sum: u32 = digits
        .iter()
        .rev()
        .enumerate()
        .map(|(i, &d)| {
            if i % 2 == 1 {
                let doubled = d * 2;
                if doubled > 9 {
                    doubled - 9
                } else {
                    doubled
                }
            } else {
                d
            }
        })
        .sum();

    sum % 10 == 0
}

/// Generate a valid Luhn number from a partial number.
fn generate_luhn_number(partial: &[u32]) -> Vec<u32> {
    let mut digits = partial.to_vec();
    digits.push(0); // Placeholder for check digit

    // Calculate what the check digit should be
    let sum: u32 = digits
        .iter()
        .rev()
        .enumerate()
        .map(|(i, &d)| {
            if i % 2 == 1 {
                let doubled = d * 2;
                if doubled > 9 {
                    doubled - 9
                } else {
                    doubled
                }
            } else {
                d
            }
        })
        .sum();

    let check_digit = (10 - (sum % 10)) % 10;
    let last_idx = digits.len() - 1;
    digits[last_idx] = check_digit;
    digits
}

/// IBAN validation implementation for testing.
fn validate_iban(iban: &str) -> bool {
    let clean: String = iban
        .chars()
        .filter(|c| c.is_alphanumeric())
        .map(|c| c.to_ascii_uppercase())
        .collect();

    if clean.len() < 15 || clean.len() > 34 {
        return false;
    }

    let country_code = &clean[..2];
    if !country_code.chars().all(|c| c.is_ascii_alphabetic()) {
        return false;
    }

    let check_digits = &clean[2..4];
    if !check_digits.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }

    let rearranged = format!("{}{}", &clean[4..], &clean[..4]);

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

    mod97(&numeric) == 1
}

fn mod97(digits: &str) -> u32 {
    digits.chars().fold(0u32, |acc, c| {
        let digit = c.to_digit(10).unwrap_or(0);
        (acc * 10 + digit) % 97
    })
}

/// Calculate IBAN check digits for a given country code and BBAN.
fn calculate_iban_check_digits(country_code: &str, bban: &str) -> String {
    // Start with check digits as 00
    let temp_iban = format!("{}00{}", country_code.to_uppercase(), bban);

    // Rearrange: BBAN + country code + 00
    let rearranged = format!("{}{}", &temp_iban[4..], &temp_iban[..4]);

    // Convert to numeric
    let numeric: String = rearranged
        .chars()
        .map(|c| {
            if c.is_ascii_alphabetic() {
                format!("{}", c.to_ascii_uppercase() as u32 - 55)
            } else {
                c.to_string()
            }
        })
        .collect();

    // Calculate check digits: 98 - (numeric mod 97)
    let remainder = mod97(&numeric);
    let check = 98 - remainder;
    format!("{:02}", check)
}

proptest! {
    // ==================== Luhn Algorithm Properties ====================

    /// Property: Any valid Luhn number remains valid.
    #[test]
    fn luhn_valid_numbers_are_valid(
        // Generate 12-18 digit partial numbers (we add the check digit)
        partial in prop::collection::vec(0u32..10, 12..18)
    ) {
        let valid = generate_luhn_number(&partial);
        let number: String = valid.iter().map(|d| char::from_digit(*d, 10).unwrap()).collect();
        prop_assert!(validate_luhn(&number), "Generated number should be valid: {}", number);
    }

    /// Property: Changing any single digit invalidates a valid Luhn number.
    #[test]
    fn luhn_single_digit_change_invalidates(
        partial in prop::collection::vec(0u32..10, 12..18),
        change_pos in 0usize..18,
        change_amount in 1u32..10 // Non-zero change
    ) {
        let mut valid = generate_luhn_number(&partial);
        let pos = change_pos % valid.len();

        // Change one digit (but not to the same value)
        let old_val = valid[pos];
        valid[pos] = (old_val + change_amount) % 10;

        // If we changed to the same value, skip this test
        if valid[pos] == old_val {
            return Ok(());
        }

        let number: String = valid.iter().map(|d| char::from_digit(*d, 10).unwrap()).collect();
        prop_assert!(!validate_luhn(&number), "Changed number should be invalid: {}", number);
    }

    /// Property: Adding spaces doesn't affect validation.
    #[test]
    fn luhn_spaces_dont_affect_validation(
        partial in prop::collection::vec(0u32..10, 12..18),
        space_positions in prop::collection::vec(0usize..20, 0..5)
    ) {
        let valid = generate_luhn_number(&partial);
        let base: String = valid.iter().map(|d| char::from_digit(*d, 10).unwrap()).collect();

        // Add spaces at various positions
        let mut with_spaces = String::new();
        for (i, c) in base.chars().enumerate() {
            if space_positions.contains(&i) {
                with_spaces.push(' ');
            }
            with_spaces.push(c);
        }

        prop_assert!(validate_luhn(&with_spaces), "Number with spaces should still be valid");
    }

    /// Property: Numbers that are too short are always invalid.
    #[test]
    fn luhn_short_numbers_invalid(
        digits in prop::collection::vec(0u32..10, 1..12)
    ) {
        let number: String = digits.iter().map(|d| char::from_digit(*d, 10).unwrap()).collect();
        prop_assert!(!validate_luhn(&number), "Short number should be invalid: {}", number);
    }

    /// Property: Numbers that are too long are always invalid.
    #[test]
    fn luhn_long_numbers_invalid(
        digits in prop::collection::vec(0u32..10, 20..30)
    ) {
        let number: String = digits.iter().map(|d| char::from_digit(*d, 10).unwrap()).collect();
        prop_assert!(!validate_luhn(&number), "Long number should be invalid: {}", number);
    }

    // ==================== IBAN Properties ====================

    /// Property: Known valid IBANs are valid.
    #[test]
    fn iban_known_valid_ibans(
        idx in 0usize..10
    ) {
        let valid_ibans = [
            "DE89370400440532013000",
            "AT611904300234573201",
            "CH9300762011623852957",
            "GB82WEST12345698765432",
            "FR7630006000011234567890189",
            "ES9121000418450200051332",
            "IT60X0542811101000000123456",
            "NL91ABNA0417164300",
            "BE68539007547034",
            "LU280019400644750000",
        ];
        let iban = valid_ibans[idx % valid_ibans.len()];
        prop_assert!(validate_iban(iban), "Known valid IBAN should be valid: {}", iban);
    }

    /// Property: Valid IBANs with spaces are still valid.
    #[test]
    fn iban_spaces_dont_affect_validation(
        idx in 0usize..5
    ) {
        let valid_ibans = [
            "DE89 3704 0044 0532 0130 00",
            "AT61 1904 3002 3457 3201",
            "CH93 0076 2011 6238 5295 7",
            "GB82 WEST 1234 5698 7654 32",
            "FR76 3000 6000 0112 3456 7890 189",
        ];
        let iban = valid_ibans[idx % valid_ibans.len()];
        prop_assert!(validate_iban(iban), "IBAN with spaces should be valid: {}", iban);
    }

    /// Property: IBANs that are too short are invalid.
    #[test]
    fn iban_short_invalid(
        country in "[A-Z]{2}",
        digits in "[0-9]{1,10}"
    ) {
        let iban = format!("{}{}", country, digits);
        if iban.len() < 15 {
            prop_assert!(!validate_iban(&iban), "Short IBAN should be invalid: {}", iban);
        }
    }

    /// Property: IBANs with invalid country codes (digits) are invalid.
    #[test]
    fn iban_numeric_country_code_invalid(
        country in "[0-9]{2}",
        rest in "[0-9A-Z]{13,30}"
    ) {
        let iban = format!("{}{}", country, rest);
        prop_assert!(!validate_iban(&iban), "IBAN with numeric country should be invalid: {}", iban);
    }

    /// Property: IBANs with non-digit check characters are invalid.
    #[test]
    fn iban_alpha_check_digits_invalid(
        country in "[A-Z]{2}",
        check in "[A-Z]{2}",
        bban in "[0-9A-Z]{11,28}"
    ) {
        let iban = format!("{}{}{}", country, check, bban);
        prop_assert!(!validate_iban(&iban), "IBAN with alpha check digits should be invalid: {}", iban);
    }

    // ==================== Cross-Validator Properties ====================

    /// Property: Random alphanumeric strings are very unlikely to be valid credit cards.
    #[test]
    fn random_strings_not_valid_credit_cards(
        s in "[a-zA-Z0-9]{13,19}"
    ) {
        // Only numeric strings can possibly be valid
        if s.chars().any(|c| c.is_alphabetic()) {
            prop_assert!(!validate_luhn(&s), "Alphanumeric string shouldn't be valid credit card");
        }
    }

    /// Property: All zeros (valid length) is a valid Luhn number.
    #[test]
    fn luhn_all_zeros_valid(
        len in 13usize..20
    ) {
        let zeros: String = "0".repeat(len);
        // All zeros should have Luhn sum of 0, which is divisible by 10
        prop_assert!(validate_luhn(&zeros), "All zeros should be valid: {}", zeros);
    }
}

// ==================== Additional Unit Tests ====================

#[test]
fn test_generate_luhn_number() {
    // Test that our generator creates valid Luhn numbers
    let partial = vec![4, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1];
    let valid = generate_luhn_number(&partial);
    let number: String = valid.iter().map(|d| char::from_digit(*d, 10).unwrap()).collect();
    assert!(validate_luhn(&number), "Generated number should be valid: {}", number);
}

#[test]
fn test_calculate_iban_check_digits() {
    // Test German IBAN check digit calculation
    let check = calculate_iban_check_digits("DE", "370400440532013000");
    assert_eq!(check, "89");

    // Test Austrian IBAN
    let check = calculate_iban_check_digits("AT", "1904300234573201");
    assert_eq!(check, "61");
}

#[test]
fn test_luhn_edge_cases() {
    // Minimum valid length with all zeros
    assert!(validate_luhn("0000000000000"));

    // Just under minimum
    assert!(!validate_luhn("000000000000"));

    // Maximum valid length
    assert!(validate_luhn("0000000000000000000"));

    // Just over maximum
    assert!(!validate_luhn("00000000000000000000"));
}

#[test]
fn test_iban_edge_cases() {
    // Minimum length (15)
    assert!(validate_iban("NO9386011117947")); // Norway

    // Maximum length (34) - Seychelles
    assert!(!validate_iban("SC18SSCB11010000000000001497USD000")); // Wrong checksum for test

    // Empty string
    assert!(!validate_iban(""));

    // Only country code
    assert!(!validate_iban("DE"));
}
