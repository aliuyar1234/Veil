//! Email address detection.

use once_cell::sync::Lazy;
use regex::Regex;

use crate::category::PiiCategory;
use crate::detector::{Detector, Match};
use crate::finding::ValidationStatus;

/// Regex pattern for email addresses (RFC 5322 simplified).
static EMAIL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap());

/// Detector for email addresses.
pub struct EmailDetector;

impl EmailDetector {
    /// Create a new email detector.
    pub fn new() -> Self {
        Self
    }
}

impl Default for EmailDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Detector for EmailDetector {
    fn name(&self) -> &str {
        "email"
    }

    fn category(&self) -> PiiCategory {
        PiiCategory::Email
    }

    fn detect(&self, text: &str) -> Vec<Match> {
        EMAIL_REGEX
            .find_iter(text)
            .map(|m| Match {
                start: m.start(),
                end: m.end(),
                text: m.as_str().to_string(),
            })
            .collect()
    }

    fn validate(&self, matched: &str) -> ValidationStatus {
        // Basic validation: must have @ and at least one dot after @
        if let Some(at_pos) = matched.find('@') {
            let domain = &matched[at_pos + 1..];
            if domain.contains('.') && !domain.starts_with('.') && !domain.ends_with('.') {
                return ValidationStatus::Valid;
            }
        }
        ValidationStatus::Invalid {
            reason: "Invalid email format".to_string(),
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
    fn test_detect_simple_email() {
        let detector = EmailDetector::new();
        let matches = detector.detect("Contact: john@example.com");

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].text, "john@example.com");
    }

    #[test]
    fn test_detect_multiple_emails() {
        let detector = EmailDetector::new();
        let matches = detector.detect("From: a@b.com, To: c@d.org");

        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn test_detect_complex_email() {
        let detector = EmailDetector::new();
        let matches = detector.detect("user.name+tag@sub.domain.co.uk");

        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_no_match_invalid() {
        let detector = EmailDetector::new();
        let matches = detector.detect("not an email @invalid");

        assert!(matches.is_empty());
    }

    #[test]
    fn test_validate_valid_email() {
        let detector = EmailDetector::new();
        assert!(matches!(
            detector.validate("test@example.com"),
            ValidationStatus::Valid
        ));
    }
}
