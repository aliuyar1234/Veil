//! Phone number detection for global formats.
//!
//! Supports:
//! - DACH region (Germany, Austria, Switzerland)
//! - US/Canada (NANP)
//! - UK (landline and mobile)
//! - France
//! - Generic E.164 international format

use once_cell::sync::Lazy;
use regex::Regex;

use crate::category::PiiCategory;
use crate::detector::{Detector, Match};
use crate::finding::ValidationStatus;

/// Regex patterns for phone numbers in various formats.
/// Ordered from most specific to least specific (first match wins).
static PHONE_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // === DACH Region (existing - keep first for backward compatibility) ===
        // International format: +43 664 1234567 or +49 89 12345678
        Regex::new(r"\+(?:43|49|41)\s?[\d\s/-]{7,15}").unwrap(),
        // With country code prefix: 0043, 0049, 0041
        Regex::new(r"00(?:43|49|41)\s?[\d\s/-]{7,15}").unwrap(),
        // === US/Canada (NANP) ===
        // E.164: +1 555 123 4567
        Regex::new(r"\+1[\s.-]?\d{3}[\s.-]?\d{3}[\s.-]?\d{4}").unwrap(),
        // With 1 prefix: 1-555-123-4567
        Regex::new(r"1[\s.-]\d{3}[\s.-]\d{3}[\s.-]\d{4}").unwrap(),
        // Parentheses: (555) 123-4567
        Regex::new(r"\(\d{3}\)[\s.-]?\d{3}[\s.-]?\d{4}").unwrap(),
        // 10-digit: 555-123-4567 (requires separators to avoid matching other numbers)
        Regex::new(r"\d{3}[\s.-]\d{3}[\s.-]\d{4}").unwrap(),
        // === UK ===
        // E.164: +44 20 7946 0958 or +44 7911 123456
        Regex::new(r"\+44[\s.-]?\d{2,4}[\s.-]?\d{3,4}[\s.-]?\d{3,6}").unwrap(),
        // Local mobile: 07911 123456
        Regex::new(r"07\d{3}[\s.-]?\d{6}").unwrap(),
        // === France ===
        // E.164: +33 1 23 45 67 89
        Regex::new(r"\+33[\s.-]?\d[\s.-]?\d{2}[\s.-]?\d{2}[\s.-]?\d{2}[\s.-]?\d{2}").unwrap(),
        // === Generic E.164 (catch-all for any country) ===
        // +[country code][number] - 7 to 15 digits total
        Regex::new(r"\+[1-9]\d{6,14}").unwrap(),
        // === Local formats (existing DACH, lower priority) ===
        // Austrian/German local format: 01/234567 or 089/12345678
        Regex::new(r"0\d{1,4}[/\s-]?\d{4,10}").unwrap(),
        // Parentheses format: (01) 234 567
        Regex::new(r"\(0\d{1,4}\)\s?\d{3,10}").unwrap(),
    ]
});

/// Detector for phone numbers.
pub struct PhoneDetector;

impl PhoneDetector {
    /// Create a new phone detector.
    pub fn new() -> Self {
        Self
    }
}

impl Default for PhoneDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Detector for PhoneDetector {
    fn name(&self) -> &str {
        "phone"
    }

    fn category(&self) -> PiiCategory {
        PiiCategory::Phone
    }

    fn detect(&self, text: &str) -> Vec<Match> {
        let mut matches = Vec::new();
        let mut seen_ranges: Vec<(usize, usize)> = Vec::new();

        for pattern in PHONE_PATTERNS.iter() {
            for m in pattern.find_iter(text) {
                // Avoid overlapping matches
                let range = (m.start(), m.end());
                if !seen_ranges
                    .iter()
                    .any(|&(s, e)| (range.0 >= s && range.0 < e) || (range.1 > s && range.1 <= e))
                {
                    let matched_text = m.as_str().to_string();
                    // Filter out too-short matches
                    let digit_count = matched_text.chars().filter(|c| c.is_ascii_digit()).count();
                    if digit_count >= 7 {
                        matches.push(Match {
                            start: m.start(),
                            end: m.end(),
                            text: matched_text,
                        });
                        seen_ranges.push(range);
                    }
                }
            }
        }

        matches.sort_by_key(|m| m.start);
        matches
    }

    fn validate(&self, matched: &str) -> ValidationStatus {
        // Count digits
        let digit_count = matched.chars().filter(|c| c.is_ascii_digit()).count();

        if (7..=15).contains(&digit_count) {
            ValidationStatus::Unvalidated
        } else {
            ValidationStatus::Invalid {
                reason: format!("Invalid phone number length: {} digits", digit_count),
            }
        }
    }

    fn base_confidence(&self) -> f32 {
        0.9
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_international_format() {
        let detector = PhoneDetector::new();
        let matches = detector.detect("Call: +43 664 1234567");

        assert_eq!(matches.len(), 1);
        assert!(matches[0].text.contains("+43"));
    }

    #[test]
    fn test_detect_german_format() {
        let detector = PhoneDetector::new();
        let matches = detector.detect("Tel: +49 89 12345678");

        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_detect_local_format() {
        let detector = PhoneDetector::new();
        let matches = detector.detect("Anruf: 01/2345678");

        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_detect_with_country_prefix() {
        let detector = PhoneDetector::new();
        let matches = detector.detect("Phone: 0043 664 1234567");

        assert_eq!(matches.len(), 1);
    }

    // === US Phone Number Tests (User Story 1) ===

    #[test]
    fn test_detect_us_e164() {
        let detector = PhoneDetector::new();
        let matches = detector.detect("Call +1 555 123 4567 for info");
        assert_eq!(matches.len(), 1);
        assert!(matches[0].text.contains("+1"));
    }

    #[test]
    fn test_detect_us_e164_hyphen() {
        let detector = PhoneDetector::new();
        let matches = detector.detect("Call +1-555-123-4567 for info");
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_detect_us_parentheses() {
        let detector = PhoneDetector::new();
        let matches = detector.detect("Phone: (555) 123-4567");
        assert_eq!(matches.len(), 1);
        assert!(matches[0].text.contains("(555)"));
    }

    #[test]
    fn test_detect_us_10_digit() {
        let detector = PhoneDetector::new();
        let matches = detector.detect("Call 555-123-4567 today");
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_detect_us_toll_free() {
        let detector = PhoneDetector::new();
        let matches = detector.detect("Toll free: 1-800-555-1234");
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_detect_us_with_1_prefix() {
        let detector = PhoneDetector::new();
        let matches = detector.detect("Call 1 555 123 4567 now");
        assert_eq!(matches.len(), 1);
    }

    // === UK Phone Number Tests (User Story 2) ===

    #[test]
    fn test_detect_uk_e164_landline() {
        let detector = PhoneDetector::new();
        let matches = detector.detect("London: +44 20 7946 0958");
        assert_eq!(matches.len(), 1);
        assert!(matches[0].text.contains("+44"));
    }

    #[test]
    fn test_detect_uk_e164_mobile() {
        let detector = PhoneDetector::new();
        let matches = detector.detect("Mobile: +44 7911 123456");
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_detect_uk_local_mobile() {
        let detector = PhoneDetector::new();
        let matches = detector.detect("Call 07911 123456");
        assert_eq!(matches.len(), 1);
        assert!(matches[0].text.starts_with("07"));
    }

    #[test]
    fn test_detect_uk_local_mobile_hyphen() {
        let detector = PhoneDetector::new();
        let matches = detector.detect("Call 07911-123456");
        assert_eq!(matches.len(), 1);
    }

    // === France Phone Number Tests (User Story 3) ===

    #[test]
    fn test_detect_france_e164() {
        let detector = PhoneDetector::new();
        let matches = detector.detect("Tel: +33 1 23 45 67 89");
        assert_eq!(matches.len(), 1);
        assert!(matches[0].text.contains("+33"));
    }

    #[test]
    fn test_detect_france_mobile() {
        let detector = PhoneDetector::new();
        let matches = detector.detect("Mobile: +33 6 12 34 56 78");
        assert_eq!(matches.len(), 1);
    }

    // === Generic E.164 International Tests (User Story 3) ===

    #[test]
    fn test_detect_japan_e164() {
        let detector = PhoneDetector::new();
        let matches = detector.detect("Japan: +81312345678");
        assert_eq!(matches.len(), 1);
        assert!(matches[0].text.contains("+81"));
    }

    #[test]
    fn test_detect_australia_e164() {
        let detector = PhoneDetector::new();
        let matches = detector.detect("Australia: +61212345678");
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_detect_india_e164() {
        let detector = PhoneDetector::new();
        let matches = detector.detect("India: +919876543210");
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_detect_brazil_e164() {
        let detector = PhoneDetector::new();
        let matches = detector.detect("Brazil: +551112345678");
        assert_eq!(matches.len(), 1);
    }

    // === Mixed Formats Test ===

    #[test]
    fn test_detect_mixed_formats() {
        let detector = PhoneDetector::new();
        let text = "US: (555) 123-4567, UK: +44 7911 123456, DE: +49 89 12345678";
        let matches = detector.detect(text);
        assert_eq!(matches.len(), 3);
    }

    // === Edge Cases ===

    #[test]
    fn test_no_overlapping_matches() {
        let detector = PhoneDetector::new();
        let matches = detector.detect("+1 555 123 4567");
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_validation_valid_length() {
        let detector = PhoneDetector::new();
        let status = detector.validate("+1 555 123 4567");
        assert!(matches!(status, ValidationStatus::Unvalidated));
    }

    #[test]
    fn test_validation_too_short() {
        let detector = PhoneDetector::new();
        let status = detector.validate("123456");
        assert!(matches!(status, ValidationStatus::Invalid { .. }));
    }
}
