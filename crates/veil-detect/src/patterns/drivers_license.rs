//! Driver's license number detection.
//!
//! Detects driver's license numbers from major US states:
//! - California: 1 letter + 7 digits (A1234567)
//! - New York: 9 digits
//! - Texas: 8 digits
//! - Florida: 1 letter + 12 digits (A123456789012)
//! - Illinois: 1 letter + 11 digits (A12345678901)

use once_cell::sync::Lazy;
use regex::Regex;

use crate::category::PiiCategory;
use crate::detector::{Detector, Match};
use crate::finding::ValidationStatus;

/// Regex patterns for US driver's license numbers by state.
static DL_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // Florida: 1 letter + 12 digits
        Regex::new(r"\b[A-Z]\d{12}\b").unwrap(),
        // Illinois: 1 letter + 11 digits
        Regex::new(r"\b[A-Z]\d{11}\b").unwrap(),
        // New York: 9 digits (must be careful - overlaps with SSN/passport)
        Regex::new(r"\b\d{9}\b").unwrap(),
        // Texas: 8 digits
        Regex::new(r"\b\d{8}\b").unwrap(),
        // California: 1 letter + 7 digits
        Regex::new(r"\b[A-Z]\d{7}\b").unwrap(),
    ]
});

/// Minimum valid driver's license length.
const MIN_DL_LENGTH: usize = 7;

/// Maximum valid driver's license length.
const MAX_DL_LENGTH: usize = 13;

/// Detector for driver's license numbers.
pub struct DriversLicenseDetector;

impl DriversLicenseDetector {
    /// Create a new driver's license detector.
    pub fn new() -> Self {
        Self
    }
}

impl Default for DriversLicenseDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Detector for DriversLicenseDetector {
    fn name(&self) -> &str {
        "drivers_license"
    }

    fn category(&self) -> PiiCategory {
        PiiCategory::DriversLicense
    }

    fn detect(&self, text: &str) -> Vec<Match> {
        let mut matches = Vec::new();
        let mut seen_ranges: Vec<(usize, usize)> = Vec::new();

        for pattern in DL_PATTERNS.iter() {
            for m in pattern.find_iter(text) {
                let start = m.start();
                let end = m.end();
                let matched_text = m.as_str();

                // Check length constraints
                let len = matched_text.len();
                if !(MIN_DL_LENGTH..=MAX_DL_LENGTH).contains(&len) {
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
        if len < MIN_DL_LENGTH {
            return ValidationStatus::Invalid {
                reason: format!(
                    "Driver's license too short: {} chars (min {})",
                    len, MIN_DL_LENGTH
                ),
            };
        }

        if len > MAX_DL_LENGTH {
            return ValidationStatus::Invalid {
                reason: format!(
                    "Driver's license too long: {} chars (max {})",
                    len, MAX_DL_LENGTH
                ),
            };
        }

        // Must contain at least one digit
        if !matched.chars().any(|c| c.is_ascii_digit()) {
            return ValidationStatus::Invalid {
                reason: "Driver's license must contain digits".to_string(),
            };
        }

        // Must be alphanumeric only
        if !matched.chars().all(|c| c.is_ascii_alphanumeric()) {
            return ValidationStatus::Invalid {
                reason: "Driver's license must be alphanumeric".to_string(),
            };
        }

        ValidationStatus::Valid
    }

    fn base_confidence(&self) -> f32 {
        0.80
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // T040: Test detect California DL (1 letter + 7 digits)
    #[test]
    fn test_detect_california_dl() {
        let detector = DriversLicenseDetector::new();
        let matches = detector.detect("CA DL: A1234567");

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].text, "A1234567");
    }

    // T041: Test detect Texas DL (8 digits)
    #[test]
    fn test_detect_texas_dl() {
        let detector = DriversLicenseDetector::new();
        let matches = detector.detect("TX DL: 12345678");

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].text, "12345678");
    }

    // T042: Test detect Florida DL (1 letter + 12 digits)
    #[test]
    fn test_detect_florida_dl() {
        let detector = DriversLicenseDetector::new();
        let matches = detector.detect("FL DL: A123456789012");

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].text, "A123456789012");
    }

    // T043: Test detect Illinois DL (1 letter + 11 digits)
    #[test]
    fn test_detect_illinois_dl() {
        let detector = DriversLicenseDetector::new();
        let matches = detector.detect("IL DL: A12345678901");

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].text, "A12345678901");
    }

    // T044: Test validate DL length 7-13 chars
    #[test]
    fn test_validate_dl_length() {
        let detector = DriversLicenseDetector::new();

        // Too short
        let result = detector.validate("123456");
        assert!(matches!(result, ValidationStatus::Invalid { .. }));

        // Valid lengths
        assert!(matches!(
            detector.validate("A123456"),
            ValidationStatus::Valid
        )); // 7 chars
        assert!(matches!(
            detector.validate("12345678"),
            ValidationStatus::Valid
        )); // 8 chars
        assert!(matches!(
            detector.validate("123456789"),
            ValidationStatus::Valid
        )); // 9 chars
        assert!(matches!(
            detector.validate("A12345678901"),
            ValidationStatus::Valid
        )); // 12 chars
        assert!(matches!(
            detector.validate("A123456789012"),
            ValidationStatus::Valid
        )); // 13 chars

        // Too long
        let result = detector.validate("A1234567890123");
        assert!(matches!(result, ValidationStatus::Invalid { .. }));
    }

    // Test detect New York DL (9 digits)
    #[test]
    fn test_detect_newyork_dl() {
        let detector = DriversLicenseDetector::new();
        let matches = detector.detect("NY DL: 123456789");

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].text, "123456789");
    }

    #[test]
    fn test_validate_valid_dl() {
        let detector = DriversLicenseDetector::new();
        assert!(matches!(
            detector.validate("A1234567"),
            ValidationStatus::Valid
        ));
    }

    #[test]
    fn test_validate_rejects_non_alphanumeric() {
        let detector = DriversLicenseDetector::new();
        let result = detector.validate("A123-456");
        assert!(matches!(result, ValidationStatus::Invalid { .. }));
    }

    #[test]
    fn test_detect_multiple_dls() {
        let detector = DriversLicenseDetector::new();
        let matches = detector.detect("CA: A1234567, TX: 87654321");

        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn test_base_confidence() {
        let detector = DriversLicenseDetector::new();
        assert!((detector.base_confidence() - 0.80).abs() < f32::EPSILON);
    }

    #[test]
    fn test_category() {
        let detector = DriversLicenseDetector::new();
        assert_eq!(detector.category(), PiiCategory::DriversLicense);
    }

    #[test]
    fn test_name() {
        let detector = DriversLicenseDetector::new();
        assert_eq!(detector.name(), "drivers_license");
    }

    #[test]
    fn test_no_match_too_short() {
        let detector = DriversLicenseDetector::new();
        let matches = detector.detect("Code: 12345");

        // 5 digits should not match
        assert!(matches.is_empty());
    }

    #[test]
    fn test_no_overlapping_matches() {
        let detector = DriversLicenseDetector::new();
        // Florida DL with 13 chars should be detected once, not multiple times
        let matches = detector.detect("A123456789012");

        assert_eq!(matches.len(), 1);
    }
}
