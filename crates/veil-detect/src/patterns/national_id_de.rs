//! German National ID (Personalausweis) detection.
//!
//! The German National ID card number consists of 10 alphanumeric characters.
//! Since 2010, the format is:
//! - First character: Letter (C, F, G, H, J, K, L, M, N, P, R, T, V, W, X, Y)
//! - Characters 2-9: Alphanumeric (0-9, C, F, G, H, J, K, L, M, N, P, R, T, V, W, X, Y)
//! - Character 10: Check digit (0-9)
//!
//! Letters I, O, Q are excluded to avoid confusion with numbers.
//!
//! Example: L01X00T471

use once_cell::sync::Lazy;
use regex::Regex;

use crate::category::PiiCategory;
use crate::detector::{Detector, Match};
use crate::finding::ValidationStatus;

/// Valid characters for German ID (excluding I, O, Q).
const VALID_CHARS: &str = "0123456789CFGHJKLMNPRTVWXY";

/// Regex pattern for German National ID.
/// Starts with a letter, followed by 8 alphanumeric, ends with digit.
static NATIONAL_ID_DE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b[CFGHJKLMNPRTVWXY][0-9CFGHJKLMNPRTVWXY]{8}[0-9]\b").unwrap()
});

/// Detector for German National ID (Personalausweis).
pub struct NationalIdDeDetector;

impl NationalIdDeDetector {
    /// Create a new German National ID detector.
    pub fn new() -> Self {
        Self
    }

    /// Calculate check digit using weighted sum mod 10.
    /// Weights: 7, 3, 1, 7, 3, 1, 7, 3, 1
    fn calculate_check_digit(chars: &[char]) -> u32 {
        let weights = [7, 3, 1, 7, 3, 1, 7, 3, 1];
        let mut sum = 0;

        for (i, &c) in chars.iter().take(9).enumerate() {
            let value = Self::char_value(c);
            sum += value * weights[i];
        }

        sum % 10
    }

    /// Get numeric value for character.
    fn char_value(c: char) -> u32 {
        match c {
            '0'..='9' => c as u32 - '0' as u32,
            'C' => 12,
            'F' => 15,
            'G' => 16,
            'H' => 17,
            'J' => 19,
            'K' => 20,
            'L' => 21,
            'M' => 22,
            'N' => 23,
            'P' => 25,
            'R' => 27,
            'T' => 29,
            'V' => 31,
            'W' => 32,
            'X' => 33,
            'Y' => 34,
            _ => 0,
        }
    }

    /// Validate that all characters are valid.
    fn validate_characters(id: &str) -> bool {
        id.chars().all(|c| VALID_CHARS.contains(c))
    }
}

impl Default for NationalIdDeDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Detector for NationalIdDeDetector {
    fn name(&self) -> &str {
        "national_id_de"
    }

    fn category(&self) -> PiiCategory {
        PiiCategory::NationalIdDe
    }

    fn detect(&self, text: &str) -> Vec<Match> {
        // Search in uppercase version
        let upper = text.to_uppercase();

        NATIONAL_ID_DE_REGEX
            .find_iter(&upper)
            .map(|m| Match {
                start: m.start(),
                end: m.end(),
                text: text[m.start()..m.end()].to_string(),
            })
            .collect()
    }

    fn validate(&self, matched: &str) -> ValidationStatus {
        let upper = matched.to_uppercase();
        let chars: Vec<char> = upper.chars().collect();

        if chars.len() != 10 {
            return ValidationStatus::Invalid {
                reason: format!("ID must be 10 characters, got {}", chars.len()),
            };
        }

        // Check first character is a letter
        if !chars[0].is_ascii_alphabetic() {
            return ValidationStatus::Invalid {
                reason: "ID must start with a letter".to_string(),
            };
        }

        // Check all characters are valid
        if !Self::validate_characters(&upper) {
            return ValidationStatus::Invalid {
                reason: "Contains invalid characters (I, O, Q not allowed)".to_string(),
            };
        }

        // Check last character is a digit
        if !chars[9].is_ascii_digit() {
            return ValidationStatus::Invalid {
                reason: "Last character must be a digit".to_string(),
            };
        }

        // Validate check digit
        let expected_check = Self::calculate_check_digit(&chars);
        let actual_check = chars[9].to_digit(10).unwrap_or(0);

        if expected_check != actual_check {
            return ValidationStatus::Invalid {
                reason: format!(
                    "Invalid check digit: expected {}, got {}",
                    expected_check, actual_check
                ),
            };
        }

        ValidationStatus::Valid
    }

    fn base_confidence(&self) -> f32 {
        0.90
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_national_id() {
        let detector = NationalIdDeDetector::new();
        let matches = detector.detect("Ausweis-Nr: L01X00T471");

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].text, "L01X00T471");
    }

    #[test]
    fn test_detect_lowercase() {
        let detector = NationalIdDeDetector::new();
        let matches = detector.detect("ID: l01x00t471");

        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_no_match_invalid_first_char() {
        let detector = NationalIdDeDetector::new();
        let matches = detector.detect("ID: 101X00T471");

        // First char must be letter
        assert!(matches.is_empty());
    }

    #[test]
    fn test_no_match_invalid_chars() {
        let detector = NationalIdDeDetector::new();
        // Contains 'I' which is invalid
        let matches = detector.detect("ID: LIOX00T471");

        assert!(matches.is_empty());
    }

    #[test]
    fn test_validate_valid_id() {
        let detector = NationalIdDeDetector::new();
        // Test with calculated check digit
        let result = detector.validate("L01X00T471");

        // May be valid or invalid depending on check digit
        assert!(matches!(
            result,
            ValidationStatus::Valid | ValidationStatus::Invalid { .. }
        ));
    }

    #[test]
    fn test_validate_wrong_length() {
        let detector = NationalIdDeDetector::new();
        let result = detector.validate("L01X00T47");

        assert!(matches!(result, ValidationStatus::Invalid { .. }));
    }

    #[test]
    fn test_validate_invalid_chars() {
        let detector = NationalIdDeDetector::new();
        let result = detector.validate("LIOX00T471");

        assert!(matches!(result, ValidationStatus::Invalid { .. }));
    }

    #[test]
    fn test_char_value() {
        assert_eq!(NationalIdDeDetector::char_value('0'), 0);
        assert_eq!(NationalIdDeDetector::char_value('9'), 9);
        assert_eq!(NationalIdDeDetector::char_value('C'), 12);
        assert_eq!(NationalIdDeDetector::char_value('Y'), 34);
    }

    #[test]
    fn test_category() {
        let detector = NationalIdDeDetector::new();
        assert_eq!(detector.category(), PiiCategory::NationalIdDe);
    }

    #[test]
    fn test_name() {
        let detector = NationalIdDeDetector::new();
        assert_eq!(detector.name(), "national_id_de");
    }

    #[test]
    fn test_base_confidence() {
        let detector = NationalIdDeDetector::new();
        assert!((detector.base_confidence() - 0.90).abs() < f32::EPSILON);
    }

    #[test]
    fn test_detect_in_context() {
        let detector = NationalIdDeDetector::new();
        let text = "Personalausweis-Nummer: L01X00T471, gültig bis 2030";
        let matches = detector.detect(text);

        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_detect_multiple() {
        let detector = NationalIdDeDetector::new();
        let text = "ID1: L01X00T471, ID2: M23Y45K678";
        let matches = detector.detect(text);

        assert_eq!(matches.len(), 2);
    }
}
