//! US Social Security Number (SSN) detection.
//!
//! Detects SSNs in hyphenated (123-45-6789) and space-separated (123 45 6789) formats.
//! Validates area, group, and serial numbers per SSA rules.

use once_cell::sync::Lazy;
use regex::Regex;

use crate::category::PiiCategory;
use crate::detector::{Detector, Match};
use crate::finding::ValidationStatus;

/// Regex patterns for US Social Security Numbers.
/// Supports hyphenated and space-separated formats.
static SSN_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // Hyphenated: 123-45-6789
        Regex::new(r"\b(\d{3})-(\d{2})-(\d{4})\b").unwrap(),
        // Space-separated: 123 45 6789
        Regex::new(r"\b(\d{3})\s(\d{2})\s(\d{4})\b").unwrap(),
    ]
});

/// Invalid SSN area numbers (first 3 digits).
const INVALID_AREAS: &[&str] = &["000", "666"];

/// Detector for US Social Security Numbers.
pub struct SsnDetector;

impl SsnDetector {
    /// Create a new SSN detector.
    pub fn new() -> Self {
        Self
    }

    /// Check if area number is in reserved range (900-999 for ITINs).
    fn is_reserved_area(area: &str) -> bool {
        area.starts_with('9')
    }
}

impl Default for SsnDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Detector for SsnDetector {
    fn name(&self) -> &str {
        "ssn"
    }

    fn category(&self) -> PiiCategory {
        PiiCategory::Ssn
    }

    fn detect(&self, text: &str) -> Vec<Match> {
        let mut matches = Vec::new();
        let mut seen_ranges: Vec<(usize, usize)> = Vec::new();

        for pattern in SSN_PATTERNS.iter() {
            for m in pattern.find_iter(text) {
                let start = m.start();
                let end = m.end();

                // Prevent overlapping matches
                let overlaps = seen_ranges
                    .iter()
                    .any(|(s, e)| start < *e && end > *s);

                if !overlaps {
                    seen_ranges.push((start, end));
                    matches.push(Match {
                        start,
                        end,
                        text: m.as_str().to_string(),
                    });
                }
            }
        }

        matches
    }

    fn validate(&self, matched: &str) -> ValidationStatus {
        // Extract digits only
        let digits: String = matched.chars().filter(|c| c.is_ascii_digit()).collect();

        if digits.len() != 9 {
            return ValidationStatus::Invalid {
                reason: format!("SSN must have 9 digits, got {}", digits.len()),
            };
        }

        let area = &digits[0..3];
        let group = &digits[3..5];
        let serial = &digits[5..9];

        // Check invalid area numbers
        if INVALID_AREAS.contains(&area) {
            return ValidationStatus::Invalid {
                reason: format!("Invalid SSN area number: {}", area),
            };
        }

        // Check reserved area numbers (9XX for ITINs)
        if Self::is_reserved_area(area) {
            return ValidationStatus::Invalid {
                reason: format!("Reserved SSN area number (ITIN): {}", area),
            };
        }

        // Check invalid group
        if group == "00" {
            return ValidationStatus::Invalid {
                reason: "Invalid SSN group number: 00".to_string(),
            };
        }

        // Check invalid serial
        if serial == "0000" {
            return ValidationStatus::Invalid {
                reason: "Invalid SSN serial number: 0000".to_string(),
            };
        }

        ValidationStatus::Valid
    }

    fn base_confidence(&self) -> f32 {
        0.95
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // T009: Test detect SSN hyphenated format 123-45-6789
    #[test]
    fn test_detect_ssn_hyphenated() {
        let detector = SsnDetector::new();
        let matches = detector.detect("SSN: 123-45-6789");

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].text, "123-45-6789");
    }

    // T010: Test detect SSN space format 123 45 6789
    #[test]
    fn test_detect_ssn_space_separated() {
        let detector = SsnDetector::new();
        let matches = detector.detect("SSN: 123 45 6789");

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].text, "123 45 6789");
    }

    // T011: Test validate SSN rejects area 000
    #[test]
    fn test_validate_rejects_area_000() {
        let detector = SsnDetector::new();
        let result = detector.validate("000-12-3456");

        assert!(matches!(result, ValidationStatus::Invalid { .. }));
        if let ValidationStatus::Invalid { reason } = result {
            assert!(reason.contains("000"));
        }
    }

    // T012: Test validate SSN rejects area 666
    #[test]
    fn test_validate_rejects_area_666() {
        let detector = SsnDetector::new();
        let result = detector.validate("666-12-3456");

        assert!(matches!(result, ValidationStatus::Invalid { .. }));
        if let ValidationStatus::Invalid { reason } = result {
            assert!(reason.contains("666"));
        }
    }

    // T013: Test validate SSN rejects group 00
    #[test]
    fn test_validate_rejects_group_00() {
        let detector = SsnDetector::new();
        let result = detector.validate("123-00-3456");

        assert!(matches!(result, ValidationStatus::Invalid { .. }));
        if let ValidationStatus::Invalid { reason } = result {
            assert!(reason.contains("00"));
        }
    }

    // T014: Test validate SSN rejects serial 0000
    #[test]
    fn test_validate_rejects_serial_0000() {
        let detector = SsnDetector::new();
        let result = detector.validate("123-45-0000");

        assert!(matches!(result, ValidationStatus::Invalid { .. }));
        if let ValidationStatus::Invalid { reason } = result {
            assert!(reason.contains("0000"));
        }
    }

    // Additional tests for valid SSNs
    #[test]
    fn test_validate_valid_ssn() {
        let detector = SsnDetector::new();
        let result = detector.validate("123-45-6789");

        assert!(matches!(result, ValidationStatus::Valid));
    }

    #[test]
    fn test_validate_rejects_reserved_area_9xx() {
        let detector = SsnDetector::new();
        let result = detector.validate("900-12-3456");

        assert!(matches!(result, ValidationStatus::Invalid { .. }));
        if let ValidationStatus::Invalid { reason } = result {
            assert!(reason.contains("900") || reason.contains("ITIN"));
        }
    }

    #[test]
    fn test_detect_multiple_ssns() {
        let detector = SsnDetector::new();
        let matches = detector.detect("SSN1: 123-45-6789, SSN2: 987-65-4321");

        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn test_no_overlap_between_formats() {
        let detector = SsnDetector::new();
        // Ensure that the same SSN isn't matched twice
        let matches = detector.detect("123-45-6789");

        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_base_confidence() {
        let detector = SsnDetector::new();
        assert!((detector.base_confidence() - 0.95).abs() < f32::EPSILON);
    }

    #[test]
    fn test_category() {
        let detector = SsnDetector::new();
        assert_eq!(detector.category(), PiiCategory::Ssn);
    }

    #[test]
    fn test_name() {
        let detector = SsnDetector::new();
        assert_eq!(detector.name(), "ssn");
    }
}
