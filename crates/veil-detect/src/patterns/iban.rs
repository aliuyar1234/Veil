//! IBAN detection with MOD-97 validation.

use once_cell::sync::Lazy;
use regex::Regex;

use crate::category::PiiCategory;
use crate::detector::{Detector, Match};
use crate::finding::ValidationStatus;
use crate::validators::validate_iban;

/// Regex pattern for IBAN (International Bank Account Number).
/// Matches country code (2 letters) + check digits (2 digits) + BBAN (up to 30 alphanumeric).
static IBAN_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\b[A-Z]{2}\s?[0-9]{2}[\s]?([A-Z0-9]\s?){11,30}\b").unwrap());

/// Detector for IBAN numbers.
pub struct IbanDetector;

impl IbanDetector {
    /// Create a new IBAN detector.
    pub fn new() -> Self {
        Self
    }
}

impl Default for IbanDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Detector for IbanDetector {
    fn name(&self) -> &str {
        "iban"
    }

    fn category(&self) -> PiiCategory {
        PiiCategory::Iban
    }

    fn detect(&self, text: &str) -> Vec<Match> {
        // Convert to uppercase for matching
        let upper = text.to_uppercase();

        IBAN_REGEX
            .find_iter(&upper)
            .map(|m| {
                // Find corresponding position in original text
                Match {
                    start: m.start(),
                    end: m.end(),
                    text: text[m.start()..m.end()].to_string(),
                }
            })
            .collect()
    }

    fn validate(&self, matched: &str) -> ValidationStatus {
        if validate_iban(matched) {
            ValidationStatus::Valid
        } else {
            ValidationStatus::Invalid {
                reason: "Invalid IBAN checksum".to_string(),
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
    fn test_detect_german_iban() {
        let detector = IbanDetector::new();
        let matches = detector.detect("IBAN: DE89370400440532013000");

        assert_eq!(matches.len(), 1);
        assert!(matches[0].text.contains("DE89"));
    }

    #[test]
    fn test_detect_iban_with_spaces() {
        let detector = IbanDetector::new();
        let matches = detector.detect("IBAN: DE89 3704 0044 0532 0130 00");

        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_validate_valid_iban() {
        let detector = IbanDetector::new();
        assert!(matches!(
            detector.validate("DE89370400440532013000"),
            ValidationStatus::Valid
        ));
    }

    #[test]
    fn test_validate_invalid_iban() {
        let detector = IbanDetector::new();
        assert!(matches!(
            detector.validate("DE89370400440532013001"),
            ValidationStatus::Invalid { .. }
        ));
    }
}
