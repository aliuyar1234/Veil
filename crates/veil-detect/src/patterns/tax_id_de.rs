//! German Tax ID (Steueridentifikationsnummer) detection.
//!
//! The German Tax ID is an 11-digit number assigned to every person
//! registered in Germany for tax purposes. It uses the ISO 7064 MOD 11,10
//! check digit algorithm.
//!
//! Format: 11 digits (e.g., 12345678903)
//! - First digit cannot be 0
//! - Exactly one digit appears twice or three times (with one digit missing)
//! - Last digit is a check digit

use once_cell::sync::Lazy;
use regex::Regex;

use crate::category::PiiCategory;
use crate::detector::{Detector, Match};
use crate::finding::ValidationStatus;

/// Regex pattern for German Tax ID.
/// 11 consecutive digits, first digit not 0.
static TAX_ID_DE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\b[1-9]\d{10}\b").unwrap());

/// Detector for German Tax ID (Steueridentifikationsnummer).
pub struct TaxIdDeDetector;

impl TaxIdDeDetector {
    /// Create a new German Tax ID detector.
    pub fn new() -> Self {
        Self
    }

    /// Validate using ISO 7064 MOD 11,10 algorithm.
    fn validate_check_digit(number: &str) -> bool {
        let digits: Vec<u32> = number.chars().filter_map(|c| c.to_digit(10)).collect();

        if digits.len() != 11 {
            return false;
        }

        // ISO 7064 MOD 11,10 algorithm
        let mut product = 10;
        for &digit in &digits[..10] {
            let sum = (digit + product) % 10;
            let sum = if sum == 0 { 10 } else { sum };
            product = (sum * 2) % 11;
        }

        let check_digit = (11 - product) % 10;
        check_digit == digits[10]
    }

    /// Validate digit distribution (one digit appears 2-3 times).
    fn validate_distribution(number: &str) -> bool {
        let digits: Vec<char> = number.chars().take(10).collect();
        let mut counts = [0u8; 10];

        for c in digits {
            if let Some(d) = c.to_digit(10) {
                counts[d as usize] += 1;
            }
        }

        // Check: exactly one digit appears 2 or 3 times
        let doubles = counts.iter().filter(|&&c| c == 2).count();
        let triples = counts.iter().filter(|&&c| c == 3).count();

        // Valid: one double OR one triple
        (doubles == 1 && triples == 0) || (doubles == 0 && triples == 1)
    }
}

impl Default for TaxIdDeDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Detector for TaxIdDeDetector {
    fn name(&self) -> &str {
        "tax_id_de"
    }

    fn category(&self) -> PiiCategory {
        PiiCategory::TaxIdDe
    }

    fn detect(&self, text: &str) -> Vec<Match> {
        TAX_ID_DE_REGEX
            .find_iter(text)
            .map(|m| Match {
                start: m.start(),
                end: m.end(),
                text: m.as_str().to_string(),
            })
            .collect()
    }

    fn validate(&self, matched: &str) -> ValidationStatus {
        // Remove any whitespace
        let clean: String = matched.chars().filter(|c| c.is_ascii_digit()).collect();

        if clean.len() != 11 {
            return ValidationStatus::Invalid {
                reason: format!("Tax ID must be 11 digits, got {}", clean.len()),
            };
        }

        if clean.starts_with('0') {
            return ValidationStatus::Invalid {
                reason: "Tax ID cannot start with 0".to_string(),
            };
        }

        if !Self::validate_distribution(&clean) {
            return ValidationStatus::Invalid {
                reason: "Invalid digit distribution".to_string(),
            };
        }

        if !Self::validate_check_digit(&clean) {
            return ValidationStatus::Invalid {
                reason: "Invalid check digit".to_string(),
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

    #[test]
    fn test_detect_tax_id() {
        let detector = TaxIdDeDetector::new();
        let matches = detector.detect("Steuer-ID: 12345678903");

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].text, "12345678903");
    }

    #[test]
    fn test_detect_with_context() {
        let detector = TaxIdDeDetector::new();
        let matches = detector.detect("Ihre Steueridentifikationsnummer: 86095742719");

        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_no_match_starts_with_zero() {
        let detector = TaxIdDeDetector::new();
        let matches = detector.detect("Number: 01234567890");

        // Pattern doesn't match numbers starting with 0
        assert!(matches.is_empty());
    }

    #[test]
    fn test_no_match_wrong_length() {
        let detector = TaxIdDeDetector::new();

        // Too short (10 digits)
        let matches = detector.detect("Number: 1234567890");
        assert!(matches.is_empty());

        // 12 digits - regex requires word boundary, so no partial match
        let matches = detector.detect("Number: 123456789012");
        assert!(matches.is_empty());
    }

    #[test]
    fn test_validate_check_digit() {
        // Valid test number with correct check digit
        assert!(TaxIdDeDetector::validate_check_digit("86095742719"));
    }

    #[test]
    fn test_validate_invalid_check_digit() {
        // Same number with wrong check digit
        assert!(!TaxIdDeDetector::validate_check_digit("86095742710"));
    }

    #[test]
    fn test_validate_distribution_valid() {
        // One digit appears twice
        assert!(TaxIdDeDetector::validate_distribution("8609574271"));
    }

    #[test]
    fn test_category() {
        let detector = TaxIdDeDetector::new();
        assert_eq!(detector.category(), PiiCategory::TaxIdDe);
    }

    #[test]
    fn test_name() {
        let detector = TaxIdDeDetector::new();
        assert_eq!(detector.name(), "tax_id_de");
    }

    #[test]
    fn test_base_confidence() {
        let detector = TaxIdDeDetector::new();
        assert!((detector.base_confidence() - 0.95).abs() < f32::EPSILON);
    }

    #[test]
    fn test_detect_multiple() {
        let detector = TaxIdDeDetector::new();
        let matches = detector.detect("ID1: 12345678903, ID2: 98765432109");

        assert_eq!(matches.len(), 2);
    }
}
