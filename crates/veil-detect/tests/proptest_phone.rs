//! Property-based tests for phone number validation.

use proptest::prelude::*;

// Phone validation (simplified)
fn is_valid_phone_format(s: &str) -> bool {
    // Extract only digits
    let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();

    // Most phone numbers are 10-15 digits
    digits.len() >= 10 && digits.len() <= 15
}

// Strategy to generate valid US phone numbers
fn us_phone_strategy() -> impl Strategy<Value = String> {
    (
        "[2-9][0-9]{2}", // area code
        "[2-9][0-9]{2}", // exchange
        "[0-9]{4}",      // subscriber
    )
        .prop_map(|(area, exchange, subscriber)| format!("({}) {}-{}", area, exchange, subscriber))
}

// Strategy to generate international phone numbers
fn intl_phone_strategy() -> impl Strategy<Value = String> {
    (
        1u32..999,     // country code
        "[0-9]{9,12}", // number
    )
        .prop_map(|(cc, number)| format!("+{}{}", cc, number))
}

// Strategy to generate non-phone strings
fn non_phone_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        // Too short
        "[0-9]{1,5}",
        // Letters
        "[a-z]{10,15}",
        // Mixed
        "[a-z0-9]{15,20}",
    ]
}

proptest! {
    #[test]
    fn us_phones_have_10_digits(phone in us_phone_strategy()) {
        let digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
        prop_assert_eq!(digits.len(), 10,
            "US phone should have 10 digits: {} -> {}", phone, digits);
    }

    #[test]
    fn intl_phones_have_valid_length(phone in intl_phone_strategy()) {
        let digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
        prop_assert!(digits.len() >= 10 && digits.len() <= 15,
            "International phone should have 10-15 digits: {} -> {}", phone, digits);
    }

    #[test]
    fn valid_phones_pass_format_check(phone in us_phone_strategy()) {
        prop_assert!(is_valid_phone_format(&phone),
            "Valid phone should pass format check: {}", phone);
    }

    #[test]
    fn short_numbers_fail_phone_check(s in "[0-9]{1,9}") {
        prop_assert!(!is_valid_phone_format(&s),
            "Short number should fail: {}", s);
    }

    #[test]
    fn phone_format_ignores_non_digits(phone in us_phone_strategy()) {
        // Add random separators
        let with_dashes = phone.replace(' ', "-");
        prop_assert!(is_valid_phone_format(&with_dashes),
            "Phone with dashes should still be valid: {}", with_dashes);
    }

    #[test]
    fn all_letters_not_phone(s in "[a-z]{10,20}") {
        prop_assert!(!is_valid_phone_format(&s),
            "Letters should not be valid phone: {}", s);
    }
}
