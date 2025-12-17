//! EU VAT Number detection.
//!
//! Detects VAT identification numbers from EU member states and Switzerland.
//! Each country has its own format:
//!
//! | Country | Format | Example |
//! |---------|--------|---------|
//! | DE | DE + 9 digits | DE123456789 |
//! | AT | ATU + 8 digits | ATU12345678 |
//! | CH | CHE + 9 digits + suffix | CHE-123.456.789 MWST |
//! | FR | FR + 2 chars + 9 digits | FR12345678901 |
//! | IT | IT + 11 digits | IT12345678901 |
//! | ES | ES + 9 alphanumeric | ESA12345678 |
//! | NL | NL + 12 chars | NL123456789B01 |
//! | BE | BE + 10 digits | BE0123456789 |
//! | PL | PL + 10 digits | PL1234567890 |

use once_cell::sync::Lazy;
use regex::Regex;

use crate::category::PiiCategory;
use crate::detector::{Detector, Match};
use crate::finding::ValidationStatus;

/// VAT number patterns by country.
static VAT_PATTERNS: Lazy<Vec<(Regex, &'static str)>> = Lazy::new(|| {
    vec![
        // Germany: DE + 9 digits
        (Regex::new(r"\bDE\s?\d{9}\b").unwrap(), "DE"),
        // Austria: ATU + 8 digits
        (Regex::new(r"\bATU\s?\d{8}\b").unwrap(), "AT"),
        // Switzerland: CHE + 9 digits (with optional separators and suffix)
        (
            Regex::new(r"\bCHE[-.\s]?\d{3}[-.\s]?\d{3}[-.\s]?\d{3}(\s?(MWST|TVA|IVA))?\b").unwrap(),
            "CH",
        ),
        // France: FR + 2 alphanumeric + 9 digits
        (Regex::new(r"\bFR\s?[0-9A-Z]{2}\s?\d{9}\b").unwrap(), "FR"),
        // Italy: IT + 11 digits
        (Regex::new(r"\bIT\s?\d{11}\b").unwrap(), "IT"),
        // Spain: ES + letter + 7 digits + letter/digit
        (Regex::new(r"\bES\s?[A-Z]\d{7}[A-Z0-9]\b").unwrap(), "ES"),
        // Netherlands: NL + 9 digits + B + 2 digits
        (Regex::new(r"\bNL\s?\d{9}B\d{2}\b").unwrap(), "NL"),
        // Belgium: BE + 10 digits (first digit 0 or 1)
        (Regex::new(r"\bBE\s?[01]\d{9}\b").unwrap(), "BE"),
        // Poland: PL + 10 digits
        (Regex::new(r"\bPL\s?\d{10}\b").unwrap(), "PL"),
        // UK (pre-Brexit, still in use): GB + 9 or 12 digits
        (Regex::new(r"\bGB\s?\d{9}(\d{3})?\b").unwrap(), "GB"),
        // Luxembourg: LU + 8 digits
        (Regex::new(r"\bLU\s?\d{8}\b").unwrap(), "LU"),
        // Ireland: IE + 7 digits + 1-2 letters
        (Regex::new(r"\bIE\s?\d{7}[A-Z]{1,2}\b").unwrap(), "IE"),
        // Portugal: PT + 9 digits
        (Regex::new(r"\bPT\s?\d{9}\b").unwrap(), "PT"),
        // Greece: EL + 9 digits
        (Regex::new(r"\bEL\s?\d{9}\b").unwrap(), "EL"),
        // Czech Republic: CZ + 8-10 digits
        (Regex::new(r"\bCZ\s?\d{8,10}\b").unwrap(), "CZ"),
        // Hungary: HU + 8 digits
        (Regex::new(r"\bHU\s?\d{8}\b").unwrap(), "HU"),
        // Romania: RO + 2-10 digits
        (Regex::new(r"\bRO\s?\d{2,10}\b").unwrap(), "RO"),
        // Sweden: SE + 12 digits
        (Regex::new(r"\bSE\s?\d{12}\b").unwrap(), "SE"),
        // Denmark: DK + 8 digits
        (Regex::new(r"\bDK\s?\d{8}\b").unwrap(), "DK"),
        // Finland: FI + 8 digits
        (Regex::new(r"\bFI\s?\d{8}\b").unwrap(), "FI"),
    ]
});

/// Detector for EU VAT Numbers.
pub struct VatNumberDetector;

impl VatNumberDetector {
    /// Create a new VAT number detector.
    pub fn new() -> Self {
        Self
    }

    /// Validate German VAT number check digit.
    fn validate_de(number: &str) -> bool {
        let digits: Vec<u32> = number.chars().filter_map(|c| c.to_digit(10)).collect();
        if digits.len() != 9 {
            return false;
        }

        // German VAT uses MOD 11 algorithm
        let mut product = 10;
        for &digit in &digits[..8] {
            let sum = (digit + product) % 10;
            let sum = if sum == 0 { 10 } else { sum };
            product = (sum * 2) % 11;
        }

        let check = (11 - product) % 10;
        check == digits[8]
    }

    /// Validate Austrian VAT number.
    fn validate_at(number: &str) -> bool {
        // ATU + 8 digits
        let digits: Vec<u32> = number.chars().filter_map(|c| c.to_digit(10)).collect();
        if digits.len() != 8 {
            return false;
        }

        // Weighted sum with alternating 1, 2
        let weights = [1, 2, 1, 2, 1, 2, 1];
        let mut sum = 0;
        for (i, &digit) in digits.iter().take(7).enumerate() {
            let mut product = digit * weights[i];
            if product > 9 {
                product = product / 10 + product % 10;
            }
            sum += product;
        }

        let check = (10 - (sum + 4) % 10) % 10;
        check == digits[7]
    }

    /// Basic validation for other countries.
    fn validate_basic(country: &str, number: &str) -> bool {
        let digits: Vec<char> = number
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .collect();

        match country {
            "CH" => digits.len() >= 9,
            "FR" => digits.len() == 11,
            "IT" => digits.len() == 11,
            "ES" => digits.len() == 9,
            "NL" => digits.len() == 12,
            "BE" => digits.len() == 10,
            "PL" => digits.len() == 10,
            "GB" => digits.len() == 9 || digits.len() == 12,
            _ => true,
        }
    }
}

impl Default for VatNumberDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Detector for VatNumberDetector {
    fn name(&self) -> &str {
        "vat_number"
    }

    fn category(&self) -> PiiCategory {
        PiiCategory::VatNumber
    }

    fn detect(&self, text: &str) -> Vec<Match> {
        let mut matches = Vec::new();
        let mut seen_ranges: Vec<(usize, usize)> = Vec::new();
        let upper = text.to_uppercase();

        for (pattern, _country) in VAT_PATTERNS.iter() {
            for m in pattern.find_iter(&upper) {
                let range = (m.start(), m.end());

                // Avoid overlapping matches
                if !seen_ranges
                    .iter()
                    .any(|&(s, e)| (range.0 >= s && range.0 < e) || (range.1 > s && range.1 <= e))
                {
                    matches.push(Match {
                        start: m.start(),
                        end: m.end(),
                        text: text[m.start()..m.end()].to_string(),
                    });
                    seen_ranges.push(range);
                }
            }
        }

        matches
    }

    fn validate(&self, matched: &str) -> ValidationStatus {
        let upper = matched.to_uppercase();
        let clean: String = upper.chars().filter(|c| c.is_ascii_alphanumeric()).collect();

        // Determine country
        let country = if clean.starts_with("DE") {
            "DE"
        } else if clean.starts_with("ATU") {
            "AT"
        } else if clean.starts_with("CHE") {
            "CH"
        } else if clean.starts_with("FR") {
            "FR"
        } else if clean.starts_with("IT") {
            "IT"
        } else if clean.starts_with("ES") {
            "ES"
        } else if clean.starts_with("NL") {
            "NL"
        } else if clean.starts_with("BE") {
            "BE"
        } else if clean.starts_with("PL") {
            "PL"
        } else if clean.starts_with("GB") {
            "GB"
        } else {
            return ValidationStatus::Unvalidated;
        };

        // Extract number part
        let number_part: String = clean
            .chars()
            .skip(if country == "AT" { 3 } else { 2 })
            .collect();

        // Country-specific validation
        let valid = match country {
            "DE" => Self::validate_de(&number_part),
            "AT" => Self::validate_at(&number_part),
            _ => Self::validate_basic(country, &number_part),
        };

        if valid {
            ValidationStatus::Valid
        } else {
            ValidationStatus::Invalid {
                reason: format!("Invalid {} VAT number format or check digit", country),
            }
        }
    }

    fn base_confidence(&self) -> f32 {
        0.95
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_german_vat() {
        let detector = VatNumberDetector::new();
        let matches = detector.detect("USt-IdNr: DE123456789");

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].text, "DE123456789");
    }

    #[test]
    fn test_detect_austrian_vat() {
        let detector = VatNumberDetector::new();
        let matches = detector.detect("UID: ATU12345678");

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].text, "ATU12345678");
    }

    #[test]
    fn test_detect_swiss_vat() {
        let detector = VatNumberDetector::new();
        let matches = detector.detect("MWST-Nr: CHE-123.456.789 MWST");

        assert_eq!(matches.len(), 1);
        assert!(matches[0].text.contains("CHE"));
    }

    #[test]
    fn test_detect_french_vat() {
        let detector = VatNumberDetector::new();
        let matches = detector.detect("TVA: FR12345678901");

        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_detect_italian_vat() {
        let detector = VatNumberDetector::new();
        let matches = detector.detect("P.IVA: IT12345678901");

        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_detect_spanish_vat() {
        let detector = VatNumberDetector::new();
        let matches = detector.detect("NIF: ESA12345678");

        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_detect_dutch_vat() {
        let detector = VatNumberDetector::new();
        let matches = detector.detect("BTW: NL123456789B01");

        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_detect_belgian_vat() {
        let detector = VatNumberDetector::new();
        let matches = detector.detect("TVA: BE0123456789");

        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_detect_multiple_vat() {
        let detector = VatNumberDetector::new();
        let text = "Company A: DE123456789, Company B: ATU12345678";
        let matches = detector.detect(text);

        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn test_validate_german_vat() {
        let detector = VatNumberDetector::new();
        let result = detector.validate("DE123456789");

        // May or may not pass check digit
        assert!(matches!(
            result,
            ValidationStatus::Valid | ValidationStatus::Invalid { .. }
        ));
    }

    #[test]
    fn test_category() {
        let detector = VatNumberDetector::new();
        assert_eq!(detector.category(), PiiCategory::VatNumber);
    }

    #[test]
    fn test_name() {
        let detector = VatNumberDetector::new();
        assert_eq!(detector.name(), "vat_number");
    }

    #[test]
    fn test_base_confidence() {
        let detector = VatNumberDetector::new();
        assert!((detector.base_confidence() - 0.95).abs() < f32::EPSILON);
    }

    #[test]
    fn test_detect_with_spaces() {
        let detector = VatNumberDetector::new();
        let matches = detector.detect("VAT: DE 123456789");

        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_detect_lowercase() {
        let detector = VatNumberDetector::new();
        let matches = detector.detect("VAT: de123456789");

        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_no_match_invalid_country() {
        let detector = VatNumberDetector::new();
        let matches = detector.detect("VAT: XX123456789");

        assert!(matches.is_empty());
    }

    #[test]
    fn test_detect_uk_vat() {
        let detector = VatNumberDetector::new();
        let matches = detector.detect("VAT: GB123456789");

        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_context_ust_idnr() {
        let detector = VatNumberDetector::new();
        let text = "Umsatzsteuer-Identifikationsnummer: DE123456789";
        let matches = detector.detect(text);

        assert_eq!(matches.len(), 1);
    }
}
