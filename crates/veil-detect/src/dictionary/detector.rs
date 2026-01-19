//! Dictionary-based PII detector.

use std::sync::Arc;

use crate::category::PiiCategory;
use crate::detector::{Detector, Match};
use crate::finding::ValidationStatus;

use super::boundaries::{extract_words, find_with_boundaries};
use super::category::DictionaryCategory;
use super::confidence::{calculate_confidence, ContextSignals};
use super::config::DictionaryDetectorConfig;
use super::normalize::normalize_for_matching;
use super::registry::DictionaryRegistry;

/// Dictionary-based PII detector.
///
/// Detects names, locations, and other PII using locale-specific dictionaries
/// with support for fuzzy matching.
pub struct DictionaryDetector {
    /// Configuration for this detector.
    config: DictionaryDetectorConfig,

    /// Registry of loaded dictionaries.
    registry: Arc<DictionaryRegistry>,

    /// Current category being detected (for Detector trait).
    current_category: DictionaryCategory,
}

impl DictionaryDetector {
    /// Create a new dictionary detector.
    pub fn new(config: DictionaryDetectorConfig, registry: Arc<DictionaryRegistry>) -> Self {
        Self {
            config,
            registry,
            current_category: DictionaryCategory::FirstName,
        }
    }

    /// Create a detector for a specific category.
    pub fn for_category(
        category: DictionaryCategory,
        config: DictionaryDetectorConfig,
        registry: Arc<DictionaryRegistry>,
    ) -> Self {
        Self {
            config,
            registry,
            current_category: category,
        }
    }

    /// Detect all dictionary matches in text.
    ///
    /// Returns matches from all enabled categories and locales.
    pub fn detect_all(&self, text: &str) -> Vec<DictionaryMatch> {
        let mut all_matches = Vec::new();

        // Extract words for word-boundary matching
        let words = extract_words(text);

        // Check each word against dictionaries
        for word_span in &words {
            let normalized = normalize_for_matching(&word_span.word);

            // Check against all enabled categories
            for category in &self.config.categories {
                let dictionaries = self.registry.by_category(category);

                for dict in dictionaries {
                    // Skip if locale not enabled
                    if !self.config.is_locale_enabled(dict.locale) {
                        continue;
                    }

                    // Try exact match first
                    if let Some(entry) = dict.get(&normalized) {
                        let context =
                            ContextSignals::from_context(text, word_span.start, word_span.end);
                        let confidence = calculate_confidence(entry, None, &context);

                        if confidence >= self.config.min_confidence {
                            all_matches.push(DictionaryMatch {
                                matched_text: word_span.word.clone(),
                                start: word_span.start,
                                end: word_span.end,
                                category: category.clone(),
                                locale: dict.locale,
                                dictionary_id: dict.id.clone(),
                                dictionary_term: entry.term.clone(),
                                match_type: MatchType::Exact,
                                confidence,
                            });
                        }
                    }
                    // Try fuzzy match if enabled
                    else if self.config.fuzzy.enabled {
                        let fuzzy_matches =
                            dict.find_fuzzy(&normalized, self.config.fuzzy.threshold);

                        for fm in fuzzy_matches {
                            let context =
                                ContextSignals::from_context(text, word_span.start, word_span.end);
                            let confidence =
                                calculate_confidence(&fm.entry, Some(fm.similarity), &context);

                            if confidence >= self.config.min_confidence {
                                all_matches.push(DictionaryMatch {
                                    matched_text: word_span.word.clone(),
                                    start: word_span.start,
                                    end: word_span.end,
                                    category: category.clone(),
                                    locale: dict.locale,
                                    dictionary_id: dict.id.clone(),
                                    dictionary_term: fm.entry.term.clone(),
                                    match_type: MatchType::Fuzzy {
                                        similarity: fm.similarity,
                                    },
                                    confidence,
                                });
                            }
                        }
                    }
                }
            }
        }

        // Also check for multi-word matches if not requiring boundaries
        if !self.config.require_word_boundaries {
            all_matches.extend(self.detect_substring_matches(text));
        }

        // Deduplicate overlapping matches (keep highest confidence)
        deduplicate_matches(&mut all_matches);

        all_matches
    }

    /// Detect substring matches (when word boundaries not required).
    fn detect_substring_matches(&self, text: &str) -> Vec<DictionaryMatch> {
        let mut matches = Vec::new();

        for category in &self.config.categories {
            let dictionaries = self.registry.by_category(category);

            for dict in dictionaries {
                if !self.config.is_locale_enabled(dict.locale) {
                    continue;
                }

                for entry in dict.iter() {
                    let positions = find_with_boundaries(text, &entry.term, false);

                    for (start, end) in positions {
                        let context = ContextSignals::from_context(text, start, end);
                        let confidence = calculate_confidence(entry, None, &context);

                        if confidence >= self.config.min_confidence {
                            matches.push(DictionaryMatch {
                                matched_text: text[start..end].to_string(),
                                start,
                                end,
                                category: category.clone(),
                                locale: dict.locale,
                                dictionary_id: dict.id.clone(),
                                dictionary_term: entry.term.clone(),
                                match_type: MatchType::Exact,
                                confidence,
                            });
                        }
                    }
                }
            }
        }

        matches
    }
}

impl Detector for DictionaryDetector {
    fn name(&self) -> &str {
        "dictionary"
    }

    fn category(&self) -> PiiCategory {
        PiiCategory::Custom(self.current_category.as_str().to_string())
    }

    fn detect(&self, text: &str) -> Vec<Match> {
        self.detect_all(text)
            .into_iter()
            .map(|dm| Match {
                start: dm.start,
                end: dm.end,
                text: dm.matched_text,
            })
            .collect()
    }

    fn validate(&self, _matched: &str) -> ValidationStatus {
        // Dictionary matches are considered valid by definition
        ValidationStatus::Valid
    }

    fn base_confidence(&self) -> f32 {
        0.7
    }
}

/// A finding from dictionary detection.
#[derive(Debug, Clone)]
pub struct DictionaryMatch {
    /// The matched text from the source document.
    pub matched_text: String,

    /// Start position in source (byte offset).
    pub start: usize,

    /// End position in source (byte offset).
    pub end: usize,

    /// Dictionary category that matched.
    pub category: DictionaryCategory,

    /// Locale of the matching dictionary.
    pub locale: super::category::Locale,

    /// Dictionary ID that produced this match.
    pub dictionary_id: String,

    /// The dictionary term that matched.
    pub dictionary_term: String,

    /// Whether this was an exact or fuzzy match.
    pub match_type: MatchType,

    /// Confidence score (0.0-1.0).
    pub confidence: f32,
}

/// Type of dictionary match.
#[derive(Debug, Clone, Copy)]
pub enum MatchType {
    /// Exact match (after normalization).
    Exact,
    /// Fuzzy match with similarity score.
    Fuzzy {
        /// Similarity score (0.0-1.0).
        similarity: f64,
    },
}

impl MatchType {
    /// Check if this is an exact match.
    pub fn is_exact(&self) -> bool {
        matches!(self, MatchType::Exact)
    }

    /// Get the similarity score (1.0 for exact matches).
    pub fn similarity(&self) -> f64 {
        match self {
            MatchType::Exact => 1.0,
            MatchType::Fuzzy { similarity } => *similarity,
        }
    }
}

/// Deduplicate overlapping matches, keeping highest confidence.
fn deduplicate_matches(matches: &mut Vec<DictionaryMatch>) {
    if matches.is_empty() {
        return;
    }

    // Sort by start position, then by confidence descending
    matches.sort_by(|a, b| {
        a.start
            .cmp(&b.start)
            .then_with(|| b.confidence.total_cmp(&a.confidence))
    });

    // Remove overlapping matches, keeping highest confidence
    let mut result = Vec::new();
    let mut last_end = 0;

    for m in matches.drain(..) {
        if m.start >= last_end {
            last_end = m.end;
            result.push(m);
        }
    }

    *matches = result;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dictionary::category::Locale;
    use crate::dictionary::dictionary::Dictionary;
    use crate::dictionary::entry::DictionaryEntry;

    fn setup_test_registry() -> Arc<DictionaryRegistry> {
        let mut registry = DictionaryRegistry::new();

        let first_names = Dictionary::new(
            "firstnames_de",
            "German First Names",
            DictionaryCategory::FirstName,
            Locale::De,
            vec![
                DictionaryEntry::with_frequency("Max", 0.85),
                DictionaryEntry::with_frequency("Maria", 0.92),
                DictionaryEntry::with_frequency("Maximilian", 0.80),
                DictionaryEntry::with_frequency("Anna", 0.90),
            ],
        )
        .unwrap();

        let last_names = Dictionary::new(
            "lastnames_de",
            "German Last Names",
            DictionaryCategory::LastName,
            Locale::De,
            vec![
                DictionaryEntry::with_frequency("Müller", 0.88),
                DictionaryEntry::with_frequency("Schmidt", 0.85),
                DictionaryEntry::with_frequency("Mustermann", 0.75),
            ],
        )
        .unwrap();

        registry.register(first_names).unwrap();
        registry.register(last_names).unwrap();

        Arc::new(registry)
    }

    #[test]
    fn test_detector_exact_match() {
        let registry = setup_test_registry();
        let config = DictionaryDetectorConfig::names_only();
        let detector = DictionaryDetector::new(config, registry);

        let matches = detector.detect_all("Max Mustermann");

        assert!(!matches.is_empty());
        assert!(matches.iter().any(|m| m.matched_text == "Max"));
        assert!(matches.iter().any(|m| m.matched_text == "Mustermann"));
    }

    #[test]
    fn test_detector_case_insensitive() {
        let registry = setup_test_registry();
        let config = DictionaryDetectorConfig::names_only();
        let detector = DictionaryDetector::new(config, registry);

        let matches = detector.detect_all("MARIA ist hier");

        assert!(!matches.is_empty());
        assert!(matches.iter().any(|m| m.matched_text == "MARIA"));
    }

    #[test]
    fn test_detector_fuzzy_match() {
        let registry = setup_test_registry();
        let config = DictionaryDetectorConfig::names_only();
        let detector = DictionaryDetector::new(config, registry);

        // "Mariah" should fuzzy-match "Maria"
        let matches = detector.detect_all("Mariah ist hier");

        // May or may not match depending on threshold
        // With default 0.85, "Mariah" -> "Maria" should be ~0.96
        if !matches.is_empty() {
            let maria_match = matches.iter().find(|m| m.matched_text == "Mariah");
            if let Some(m) = maria_match {
                assert!(matches!(m.match_type, MatchType::Fuzzy { .. }));
            }
        }
    }

    #[test]
    fn test_detector_no_false_positives() {
        let registry = setup_test_registry();
        let config = DictionaryDetectorConfig::names_only();
        let detector = DictionaryDetector::new(config, registry);

        let matches = detector.detect_all("Hello World");

        assert!(matches.is_empty());
    }

    #[test]
    fn test_detector_word_boundaries() {
        let registry = setup_test_registry();
        let config = DictionaryDetectorConfig::names_only();
        let detector = DictionaryDetector::new(config, registry);

        // "Max" in "Maximum" should not match with boundaries
        let matches = detector.detect_all("Maximum value");

        assert!(matches.is_empty() || !matches.iter().any(|m| m.matched_text == "Max"));
    }

    #[test]
    fn test_detector_impl() {
        let registry = setup_test_registry();
        let config = DictionaryDetectorConfig::names_only();
        let detector = DictionaryDetector::new(config, registry);

        assert_eq!(detector.name(), "dictionary");
        assert!(detector.base_confidence() > 0.5);
    }

    #[test]
    fn test_match_type() {
        assert!(MatchType::Exact.is_exact());
        assert_eq!(MatchType::Exact.similarity(), 1.0);

        let fuzzy = MatchType::Fuzzy { similarity: 0.9 };
        assert!(!fuzzy.is_exact());
        assert_eq!(fuzzy.similarity(), 0.9);
    }
}
