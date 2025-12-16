//! Credit card number detection with Luhn validation.

use once_cell::sync::Lazy;
use regex::Regex;

use crate::category::PiiCategory;
use crate::detector::{Detector, Match};
use crate::finding::ValidationStatus;
use crate::validators::validate_luhn;

/// Regex pattern for credit card numbers.
/// Matches 13-19 digits with optional spaces or dashes.
static CREDIT_CARD_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\b(?:\d[\s-]?){13,19}\b").unwrap());

/// Detector for credit card numbers.
pub struct CreditCardDetector;

impl CreditCardDetector {
    /// Create a new credit card detector.
    pub fn new() -> Self {
        Self
    }
}

impl Default for CreditCardDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Detector for CreditCardDetector {
    fn name(&self) -> &str {
        "credit_card"
    }

    fn category(&self) -> PiiCategory {
        PiiCategory::CreditCard
    }

    fn detect(&self, text: &str) -> Vec<Match> {
        CREDIT_CARD_REGEX
            .find_iter(text)
            .filter_map(|m| {
                let matched = m.as_str();
                // Count actual digits
                let digit_count = matched.chars().filter(|c| c.is_ascii_digit()).count();

                // Must have 13-19 digits
                if (13..=19).contains(&digit_count) {
                    Some(Match {
                        start: m.start(),
                        end: m.end(),
                        text: matched.to_string(),
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    fn validate(&self, matched: &str) -> ValidationStatus {
        if validate_luhn(matched) {
            ValidationStatus::Valid
        } else {
            ValidationStatus::Invalid {
                reason: "Invalid Luhn checksum".to_string(),
            }
        }
    }

    fn base_confidence(&self) -> f32 {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_visa() {
        let detector = CreditCardDetector::new();
        let matches = detector.detect("Card: 4111111111111111");

        assert_eq!(matches.len(), 1);
        assert!(matches[0].text.contains("4111"));
    }

    #[test]
    fn test_detect_with_spaces() {
        let detector = CreditCardDetector::new();
        let matches = detector.detect("Card: 4111 1111 1111 1111");

        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_detect_with_dashes() {
        let detector = CreditCardDetector::new();
        let matches = detector.detect("Card: 4111-1111-1111-1111");

        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_validate_valid_card() {
        let detector = CreditCardDetector::new();
        assert!(matches!(
            detector.validate("4111111111111111"),
            ValidationStatus::Valid
        ));
    }

    #[test]
    fn test_validate_invalid_card() {
        let detector = CreditCardDetector::new();
        assert!(matches!(
            detector.validate("4111111111111112"),
            ValidationStatus::Invalid { .. }
        ));
    }
}
