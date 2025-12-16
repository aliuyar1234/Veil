//! Phone number detection for DACH region (DE, AT, CH).

use once_cell::sync::Lazy;
use regex::Regex;

use crate::category::PiiCategory;
use crate::detector::{Detector, Match};
use crate::finding::ValidationStatus;

/// Regex patterns for phone numbers in various formats.
static PHONE_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // International format: +43 664 1234567 or +49 89 12345678
        Regex::new(r"\+(?:43|49|41)\s?[\d\s/-]{7,15}").unwrap(),
        // With country code prefix: 0043, 0049, 0041
        Regex::new(r"00(?:43|49|41)\s?[\d\s/-]{7,15}").unwrap(),
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
}
