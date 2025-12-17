//! Passport number detection.
//!
//! Detects passport numbers from US, UK, and EU countries.
//! Supports numeric (9 digits) and alphanumeric formats.

use once_cell::sync::Lazy;
use regex::Regex;

use crate::category::PiiCategory;
use crate::detector::{Detector, Match};
use crate::finding::ValidationStatus;

/// Regex patterns for passport numbers.
/// Supports US, UK, and EU formats.
static PASSPORT_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // US passport with letter prefix: A12345678
        Regex::new(r"\b[A-Z]\d{8}\b").unwrap(),
        // 9-digit numeric (US, UK): 123456789
        Regex::new(r"\b\d{9}\b").unwrap(),
        // Generic alphanumeric 9 chars (German, French, etc.): C1234567D, 12AB34567
        Regex::new(r"\b[A-Z0-9]{9}\b").unwrap(),
        // Generic alphanumeric 6-8 chars (some EU countries)
        Regex::new(r"\b[A-Z]{1,2}\d{6,7}\b").unwrap(),
    ]
});

/// Minimum valid passport length.
const MIN_PASSPORT_LENGTH: usize = 6;

/// Maximum valid passport length.
const MAX_PASSPORT_LENGTH: usize = 9;

/// Detector for passport numbers.
pub struct PassportDetector;

impl PassportDetector {
    /// Create a new passport detector.
    pub fn new() -> Self {
        Self
    }
}

impl Default for PassportDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Detector for PassportDetector {
    fn name(&self) -> &str {
        "passport"
    }

    fn category(&self) -> PiiCategory {
        PiiCategory::Passport
    }

    fn detect(&self, text: &str) -> Vec<Match> {
        let mut matches = Vec::new();
        let mut seen_ranges: Vec<(usize, usize)> = Vec::new();

        for pattern in PASSPORT_PATTERNS.iter() {
            for m in pattern.find_iter(text) {
                let start = m.start();
                let end = m.end();
                let matched_text = m.as_str();

                // Skip if too short or too long
                let len = matched_text.len();
                if !(MIN_PASSPORT_LENGTH..=MAX_PASSPORT_LENGTH).contains(&len) {
                    continue;
                }

                // Skip pure alphabetic matches (not passports)
                if matched_text.chars().all(|c| c.is_ascii_alphabetic()) {
                    continue;
                }

                // Prevent overlapping matches
                let overlaps = seen_ranges.iter().any(|(s, e)| start < *e && end > *s);

                if !overlaps {
                    seen_ranges.push((start, end));
                    matches.push(Match {
                        start,
                        end,
                        text: matched_text.to_string(),
                    });
                }
            }
        }

        matches
    }

    fn validate(&self, matched: &str) -> ValidationStatus {
        let len = matched.len();

        // Check length constraints
        if len < MIN_PASSPORT_LENGTH {
            return ValidationStatus::Invalid {
                reason: format!(
                    "Passport number too short: {} chars (min {})",
                    len, MIN_PASSPORT_LENGTH
                ),
            };
        }

        if len > MAX_PASSPORT_LENGTH {
            return ValidationStatus::Invalid {
                reason: format!(
                    "Passport number too long: {} chars (max {})",
                    len, MAX_PASSPORT_LENGTH
                ),
            };
        }

        // Must contain at least one digit
        if !matched.chars().any(|c| c.is_ascii_digit()) {
            return ValidationStatus::Invalid {
                reason: "Passport number must contain digits".to_string(),
            };
        }

        // Must be alphanumeric only
        if !matched.chars().all(|c| c.is_ascii_alphanumeric()) {
            return ValidationStatus::Invalid {
                reason: "Passport number must be alphanumeric".to_string(),
            };
        }

        ValidationStatus::Valid
    }

    fn base_confidence(&self) -> f32 {
        0.85
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // T024: Test detect US passport 9-digit format
    #[test]
    fn test_detect_us_passport_9digit() {
        let detector = PassportDetector::new();
        let matches = detector.detect("Passport: 123456789");

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].text, "123456789");
    }

    // T025: Test detect US passport alphanumeric A12345678
    #[test]
    fn test_detect_us_passport_alphanumeric() {
        let detector = PassportDetector::new();
        let matches = detector.detect("Passport: A12345678");

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].text, "A12345678");
    }

    // T026: Test validate passport length 6-9 chars
    #[test]
    fn test_validate_passport_length() {
        let detector = PassportDetector::new();

        // Too short
        let result = detector.validate("12345");
        assert!(matches!(result, ValidationStatus::Invalid { .. }));

        // Valid lengths
        assert!(matches!(detector.validate("123456"), ValidationStatus::Valid));
        assert!(matches!(
            detector.validate("1234567"),
            ValidationStatus::Valid
        ));
        assert!(matches!(
            detector.validate("12345678"),
            ValidationStatus::Valid
        ));
        assert!(matches!(
            detector.validate("123456789"),
            ValidationStatus::Valid
        ));

        // Too long
        let result = detector.validate("1234567890");
        assert!(matches!(result, ValidationStatus::Invalid { .. }));
    }

    // T035: Test detect UK passport 9-digit
    #[test]
    fn test_detect_uk_passport() {
        let detector = PassportDetector::new();
        let matches = detector.detect("UK Passport: 123456789");

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].text, "123456789");
    }

    // T036: Test detect German passport alphanumeric
    #[test]
    fn test_detect_german_passport() {
        let detector = PassportDetector::new();
        let matches = detector.detect("Reisepass: C1234567D");

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].text, "C1234567D");
    }

    // T037: Test detect French passport alphanumeric
    #[test]
    fn test_detect_french_passport() {
        let detector = PassportDetector::new();
        let matches = detector.detect("Passeport: 12AB34567");

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].text, "12AB34567");
    }

    #[test]
    fn test_validate_valid_passport() {
        let detector = PassportDetector::new();
        assert!(matches!(
            detector.validate("A12345678"),
            ValidationStatus::Valid
        ));
    }

    #[test]
    fn test_validate_rejects_non_alphanumeric() {
        let detector = PassportDetector::new();
        let result = detector.validate("123-456-789");
        assert!(matches!(result, ValidationStatus::Invalid { .. }));
    }

    #[test]
    fn test_detect_multiple_passports() {
        let detector = PassportDetector::new();
        let matches = detector.detect("US: 123456789, UK: 987654321");

        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn test_base_confidence() {
        let detector = PassportDetector::new();
        assert!((detector.base_confidence() - 0.85).abs() < f32::EPSILON);
    }

    #[test]
    fn test_category() {
        let detector = PassportDetector::new();
        assert_eq!(detector.category(), PiiCategory::Passport);
    }

    #[test]
    fn test_name() {
        let detector = PassportDetector::new();
        assert_eq!(detector.name(), "passport");
    }

    #[test]
    fn test_no_match_too_short() {
        let detector = PassportDetector::new();
        let matches = detector.detect("Code: 12345");

        // 5 digits should not match
        assert!(matches.is_empty());
    }

    #[test]
    fn test_no_match_pure_letters() {
        let detector = PassportDetector::new();
        let matches = detector.detect("ABCDEFGHI");

        // Pure letters should not match (not a passport)
        assert!(matches.is_empty());
    }
}
