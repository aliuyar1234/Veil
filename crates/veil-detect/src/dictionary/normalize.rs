//! Text normalization for dictionary matching.

#![allow(dead_code)]

use unicode_normalization::UnicodeNormalization;

/// Normalize text for dictionary matching.
///
/// Applies NFD normalization and case folding to enable matching
/// regardless of case or Unicode representation (e.g., "ü" vs "u\u{0308}").
///
/// # Examples
///
/// ```
/// use veil_detect::dictionary::normalize_for_matching;
///
/// assert_eq!(normalize_for_matching("MÜLLER"), normalize_for_matching("müller"));
/// assert_eq!(normalize_for_matching("Café"), normalize_for_matching("cafe\u{0301}"));
/// ```
pub fn normalize_for_matching(s: &str) -> String {
    // NFD decomposition followed by lowercase
    s.nfd().collect::<String>().to_lowercase()
}

/// Check if two strings match after normalization.
pub fn normalized_eq(a: &str, b: &str) -> bool {
    normalize_for_matching(a) == normalize_for_matching(b)
}

/// Normalize for storage in FST (must be ASCII-safe for FST).
///
/// FST requires byte-ordered keys, so we normalize to NFD lowercase
/// and keep the full Unicode representation.
pub fn normalize_for_storage(s: &str) -> String {
    normalize_for_matching(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_case() {
        assert_eq!(normalize_for_matching("HELLO"), "hello");
        assert_eq!(normalize_for_matching("Hello"), "hello");
        assert_eq!(normalize_for_matching("hello"), "hello");
    }

    #[test]
    fn test_normalize_umlauts() {
        // German umlauts
        let normalized = normalize_for_matching("Müller");
        assert!(normalized.contains("u"));
        // NFD decomposes ü to u + combining diaeresis

        // Case insensitive
        assert_eq!(
            normalize_for_matching("MÜLLER"),
            normalize_for_matching("müller")
        );
    }

    #[test]
    fn test_normalize_accents() {
        // French accents
        assert_eq!(
            normalize_for_matching("Café"),
            normalize_for_matching("café")
        );
    }

    #[test]
    fn test_normalized_eq() {
        assert!(normalized_eq("Müller", "MÜLLER"));
        assert!(normalized_eq("Maria", "maria"));
        assert!(!normalized_eq("Max", "Moritz"));
    }

    #[test]
    fn test_eszett() {
        // German ß
        let normalized = normalize_for_matching("Straße");
        assert!(normalized.contains("ß") || normalized.contains("ss"));
    }
}
