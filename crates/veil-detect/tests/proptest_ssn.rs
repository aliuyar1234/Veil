//! Property-based tests for SSN validation.

use proptest::prelude::*;

// SSN validation (US format: XXX-XX-XXXX)
fn is_valid_ssn_format(s: &str) -> bool {
    // Extract only digits
    let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();

    // SSN must be exactly 9 digits
    if digits.len() != 9 {
        return false;
    }

    // Area number (first 3 digits) cannot be 000, 666, or 900-999
    let area: u32 = digits[0..3].parse().unwrap_or(0);
    if area == 0 || area == 666 || area >= 900 {
        return false;
    }

    // Group number (middle 2 digits) cannot be 00
    let group: u32 = digits[3..5].parse().unwrap_or(0);
    if group == 0 {
        return false;
    }

    // Serial number (last 4 digits) cannot be 0000
    let serial: u32 = digits[5..9].parse().unwrap_or(0);
    if serial == 0 {
        return false;
    }

    true
}

// Strategy to generate valid SSN formats
fn valid_ssn_strategy() -> impl Strategy<Value = String> {
    (
        001u32..666,    // area (excluding 666)
        01u32..100,     // group
        0001u32..10000, // serial
    )
        .prop_filter("Exclude 666 area code", |(area, _, _)| *area != 666)
        .prop_map(|(area, group, serial)| format!("{:03}-{:02}-{:04}", area, group, serial))
}

// Strategy to generate invalid SSNs
fn invalid_ssn_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        // Area 000
        Just("000-12-3456".to_string()),
        // Area 666
        Just("666-12-3456".to_string()),
        // Area 900+
        (900u32..1000, 01u32..100, 0001u32..10000)
            .prop_map(|(area, group, serial)| format!("{:03}-{:02}-{:04}", area, group, serial)),
        // Group 00
        (001u32..666, 0001u32..10000)
            .prop_map(|(area, serial)| format!("{:03}-00-{:04}", area, serial)),
        // Serial 0000
        (001u32..666, 01u32..100)
            .prop_map(|(area, group)| format!("{:03}-{:02}-0000", area, group)),
    ]
}

proptest! {
    #[test]
    fn valid_ssns_have_9_digits(ssn in valid_ssn_strategy()) {
        let digits: String = ssn.chars().filter(|c| c.is_ascii_digit()).collect();
        prop_assert_eq!(digits.len(), 9,
            "SSN should have 9 digits: {} -> {}", ssn, digits);
    }

    #[test]
    fn valid_ssns_pass_format_check(ssn in valid_ssn_strategy()) {
        prop_assert!(is_valid_ssn_format(&ssn),
            "Valid SSN should pass format check: {}", ssn);
    }

    #[test]
    fn invalid_ssns_fail_format_check(ssn in invalid_ssn_strategy()) {
        prop_assert!(!is_valid_ssn_format(&ssn),
            "Invalid SSN should fail format check: {}", ssn);
    }

    #[test]
    fn short_numbers_not_ssn(s in "[0-9]{1,8}") {
        prop_assert!(!is_valid_ssn_format(&s),
            "Short number should not be valid SSN: {}", s);
    }

    #[test]
    fn long_numbers_not_ssn(s in "[0-9]{10,20}") {
        prop_assert!(!is_valid_ssn_format(&s),
            "Long number should not be valid SSN: {}", s);
    }

    #[test]
    fn ssn_format_with_dashes(ssn in valid_ssn_strategy()) {
        // Valid SSNs should work with or without dashes
        let no_dashes: String = ssn.chars().filter(|c| c.is_ascii_digit()).collect();
        prop_assert!(is_valid_ssn_format(&no_dashes),
            "SSN without dashes should still be valid: {} -> {}", ssn, no_dashes);
    }

    #[test]
    fn all_zeros_not_ssn(_ in Just(())) {
        prop_assert!(!is_valid_ssn_format("000-00-0000"));
        prop_assert!(!is_valid_ssn_format("000000000"));
    }

    #[test]
    fn letters_not_ssn(s in "[a-z]{9}") {
        prop_assert!(!is_valid_ssn_format(&s),
            "Letters should not be valid SSN: {}", s);
    }
}
