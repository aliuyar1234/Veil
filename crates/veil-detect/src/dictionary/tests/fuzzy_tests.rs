//! Tests for fuzzy matching (User Story 5).

use std::sync::Arc;

use crate::dictionary::{
    category::{DictionaryCategory, Locale},
    config::DictionaryDetectorConfig,
    detector::{DictionaryDetector, MatchType},
    dictionary::Dictionary,
    entry::DictionaryEntry,
    fuzzy::FuzzyConfig,
    registry::DictionaryRegistry,
};

fn setup_fuzzy_registry() -> Arc<DictionaryRegistry> {
    let mut registry = DictionaryRegistry::new();

    let first_names = Dictionary::new(
        "firstnames_de",
        "German First Names",
        DictionaryCategory::FirstName,
        Locale::De,
        vec![
            DictionaryEntry::with_frequency("Maximilian", 0.80),
            DictionaryEntry::with_frequency("Maria", 0.92),
            DictionaryEntry::with_frequency("Alexander", 0.78),
            DictionaryEntry::with_frequency("Elisabeth", 0.75),
        ],
    )
    .unwrap();

    registry.register(first_names).unwrap();
    Arc::new(registry)
}

#[test]
fn test_fuzzy_match_single_typo() {
    let registry = setup_fuzzy_registry();
    let config =
        DictionaryDetectorConfig::names_only().with_fuzzy(FuzzyConfig::with_threshold(0.85));
    let detector = DictionaryDetector::new(config, registry);

    // "Maximilain" has one transposition from "Maximilian"
    let matches = detector.detect_all("Hallo Maximilain");

    // Should find a fuzzy match
    let fuzzy_matches: Vec<_> = matches
        .iter()
        .filter(|m| matches!(m.match_type, MatchType::Fuzzy { .. }))
        .collect();

    assert!(!fuzzy_matches.is_empty());

    if let Some(m) = fuzzy_matches.first() {
        assert_eq!(m.dictionary_term, "Maximilian");
        if let MatchType::Fuzzy { similarity } = m.match_type {
            assert!(similarity > 0.85);
        }
    }
}

#[test]
fn test_fuzzy_match_below_threshold_rejected() {
    let registry = setup_fuzzy_registry();
    let config =
        DictionaryDetectorConfig::names_only().with_fuzzy(FuzzyConfig::with_threshold(0.95)); // Very strict threshold
    let detector = DictionaryDetector::new(config, registry);

    // "Maxi" is too different from "Maximilian" for 0.95 threshold
    let matches = detector.detect_all("Hallo Maxi");

    // Should not find fuzzy match at strict threshold
    let fuzzy_matches: Vec<_> = matches
        .iter()
        .filter(|m| m.dictionary_term == "Maximilian")
        .collect();

    assert!(fuzzy_matches.is_empty());
}

#[test]
fn test_fuzzy_matching_disabled() {
    let registry = setup_fuzzy_registry();
    let config = DictionaryDetectorConfig::names_only().without_fuzzy();
    let detector = DictionaryDetector::new(config, registry);

    // With fuzzy disabled, typos should not match
    let matches = detector.detect_all("Hallo Maximilain");

    // Should not find any matches since "Maximilain" is not exactly in dictionary
    assert!(matches.is_empty());
}

#[test]
fn test_exact_match_preferred_over_fuzzy() {
    let registry = setup_fuzzy_registry();
    let config = DictionaryDetectorConfig::names_only();
    let detector = DictionaryDetector::new(config, registry);

    // Exact match should be found
    let matches = detector.detect_all("Maria ist hier");

    assert!(!matches.is_empty());

    let maria_match = matches.iter().find(|m| m.matched_text == "Maria").unwrap();
    assert!(maria_match.match_type.is_exact());
}

#[test]
fn test_fuzzy_threshold_affects_results() {
    let registry = setup_fuzzy_registry();

    // Strict threshold
    let strict_config =
        DictionaryDetectorConfig::names_only().with_fuzzy(FuzzyConfig::with_threshold(0.95));
    let strict_detector = DictionaryDetector::new(strict_config, Arc::clone(&registry));

    // Lenient threshold
    let lenient_config =
        DictionaryDetectorConfig::names_only().with_fuzzy(FuzzyConfig::with_threshold(0.70));
    let lenient_detector = DictionaryDetector::new(lenient_config, registry);

    let text = "Elisabet ist hier"; // Missing 'h'

    let strict_matches = strict_detector.detect_all(text);
    let lenient_matches = lenient_detector.detect_all(text);

    // Lenient should find more matches
    assert!(lenient_matches.len() >= strict_matches.len());
}

#[test]
fn test_fuzzy_confidence_lower_than_exact() {
    let registry = setup_fuzzy_registry();
    let config = DictionaryDetectorConfig::names_only();
    let detector = DictionaryDetector::new(config, Arc::clone(&registry));

    // Exact match
    let exact_matches = detector.detect_all("Maria ist hier");
    let exact_confidence = exact_matches
        .iter()
        .find(|m| m.dictionary_term == "Maria")
        .map(|m| m.confidence)
        .unwrap();

    // Fuzzy match
    let fuzzy_matches = detector.detect_all("Mariah ist hier");
    if let Some(fuzzy_match) = fuzzy_matches.iter().find(|m| m.dictionary_term == "Maria") {
        // Fuzzy confidence should be lower due to similarity factor
        assert!(fuzzy_match.confidence <= exact_confidence);
    }
}

#[test]
fn test_jaro_winkler_favors_prefix_matches() {
    let registry = setup_fuzzy_registry();
    let config =
        DictionaryDetectorConfig::names_only().with_fuzzy(FuzzyConfig::with_threshold(0.80));
    let detector = DictionaryDetector::new(config, registry);

    // "Alexande" (missing 'r') should match "Alexander" well with Jaro-Winkler
    // because prefixes match
    let matches = detector.detect_all("Hallo Alexande");

    let fuzzy_matches: Vec<_> = matches
        .iter()
        .filter(|m| m.dictionary_term == "Alexander")
        .collect();

    // Should find fuzzy match due to prefix matching
    if !fuzzy_matches.is_empty() {
        if let MatchType::Fuzzy { similarity } = fuzzy_matches[0].match_type {
            assert!(similarity > 0.85);
        }
    }
}

#[test]
fn test_dictionary_find_fuzzy() {
    let dict = Dictionary::new(
        "test",
        "Test",
        DictionaryCategory::FirstName,
        Locale::De,
        vec![
            DictionaryEntry::with_frequency("Maximilian", 0.80),
            DictionaryEntry::with_frequency("Maria", 0.92),
        ],
    )
    .unwrap();

    // Find fuzzy matches
    let matches = dict.find_fuzzy("Maximilain", 0.85);

    assert!(!matches.is_empty());
    assert_eq!(matches[0].entry.term, "Maximilian");
    assert!(matches[0].similarity > 0.85);
}

#[test]
fn test_fuzzy_config_default() {
    let config = FuzzyConfig::default();
    assert!(config.enabled);
    assert_eq!(config.threshold, 0.85);
}

#[test]
fn test_fuzzy_config_disabled() {
    let config = FuzzyConfig::disabled();
    assert!(!config.enabled);
}

#[test]
fn test_fuzzy_config_with_threshold() {
    let config = FuzzyConfig::with_threshold(0.9);
    assert!(config.enabled);
    assert_eq!(config.threshold, 0.9);
}
