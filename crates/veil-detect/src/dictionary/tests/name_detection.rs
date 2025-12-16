//! Tests for person name detection (User Story 1).

use std::sync::Arc;

use crate::dictionary::{
    category::{DictionaryCategory, Locale},
    config::DictionaryDetectorConfig,
    detector::DictionaryDetector,
    dictionary::Dictionary,
    entry::DictionaryEntry,
    registry::DictionaryRegistry,
};

fn setup_name_registry() -> Arc<DictionaryRegistry> {
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
            DictionaryEntry::with_frequency("Hans", 0.82),
            DictionaryEntry::with_frequency("Peter", 0.83),
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
            DictionaryEntry::with_frequency("Huber", 0.78),
            DictionaryEntry::with_frequency("Maier", 0.80),
        ],
    )
    .unwrap();

    registry.register(first_names).unwrap();
    registry.register(last_names).unwrap();

    Arc::new(registry)
}

#[test]
fn test_detect_exact_first_name() {
    let registry = setup_name_registry();
    let config = DictionaryDetectorConfig::names_only();
    let detector = DictionaryDetector::new(config, registry);

    let matches = detector.detect_all("Kontaktperson: Max aus München");

    let first_name_matches: Vec<_> = matches
        .iter()
        .filter(|m| matches!(m.category, DictionaryCategory::FirstName))
        .collect();

    assert!(!first_name_matches.is_empty());
    assert!(first_name_matches.iter().any(|m| m.matched_text == "Max"));
}

#[test]
fn test_detect_exact_last_name() {
    let registry = setup_name_registry();
    let config = DictionaryDetectorConfig::names_only();
    let detector = DictionaryDetector::new(config, registry);

    let matches = detector.detect_all("Herr Müller ist hier");

    let last_name_matches: Vec<_> = matches
        .iter()
        .filter(|m| matches!(m.category, DictionaryCategory::LastName))
        .collect();

    assert!(!last_name_matches.is_empty());
    assert!(last_name_matches.iter().any(|m| m.matched_text == "Müller"));
}

#[test]
fn test_detect_full_name() {
    let registry = setup_name_registry();
    let config = DictionaryDetectorConfig::names_only();
    let detector = DictionaryDetector::new(config, registry);

    let matches = detector.detect_all("Max Mustermann ist unser Ansprechpartner");

    assert_eq!(matches.len(), 2);
    assert!(matches.iter().any(|m| m.matched_text == "Max"));
    assert!(matches.iter().any(|m| m.matched_text == "Mustermann"));
}

#[test]
fn test_no_false_positive_on_random_words() {
    let registry = setup_name_registry();
    let config = DictionaryDetectorConfig::names_only();
    let detector = DictionaryDetector::new(config, registry);

    let matches = detector.detect_all("Der Computer ist schnell und effizient");

    assert!(matches.is_empty());
}

#[test]
fn test_case_insensitive_matching() {
    let registry = setup_name_registry();
    let config = DictionaryDetectorConfig::names_only();
    let detector = DictionaryDetector::new(config, registry);

    // All caps
    let matches1 = detector.detect_all("MARIA ist hier");
    assert!(!matches1.is_empty());

    // Lower case
    let matches2 = detector.detect_all("maria ist hier");
    assert!(!matches2.is_empty());
}

#[test]
fn test_word_boundary_prevents_substring_match() {
    let registry = setup_name_registry();
    let config = DictionaryDetectorConfig::names_only();
    let detector = DictionaryDetector::new(config, registry);

    // "Max" should not match in "Maximum"
    let matches = detector.detect_all("Der Maximum-Wert ist 100");

    // Should be empty or not contain "Max" from "Maximum"
    assert!(
        matches.is_empty()
            || !matches
                .iter()
                .any(|m| m.start == 4 && m.matched_text == "Max")
    );
}

#[test]
fn test_confidence_with_honorific() {
    let registry = setup_name_registry();
    let config = DictionaryDetectorConfig::names_only();
    let detector = DictionaryDetector::new(config, registry);

    // With honorific prefix
    let matches_with_prefix = detector.detect_all("Frau Maria ist hier");
    let maria_with_prefix = matches_with_prefix
        .iter()
        .find(|m| m.matched_text == "Maria")
        .unwrap();

    // Without honorific prefix
    let matches_without_prefix = detector.detect_all("Maria ist hier");
    let maria_without_prefix = matches_without_prefix
        .iter()
        .find(|m| m.matched_text == "Maria")
        .unwrap();

    // Confidence with honorific should be higher or equal
    assert!(maria_with_prefix.confidence >= maria_without_prefix.confidence);
}

#[test]
fn test_multiple_names_in_text() {
    let registry = setup_name_registry();
    let config = DictionaryDetectorConfig::names_only();
    let detector = DictionaryDetector::new(config, registry);

    let matches = detector.detect_all("Anna und Hans treffen Maria bei Peter");

    let first_names: Vec<_> = matches
        .iter()
        .filter(|m| matches!(m.category, DictionaryCategory::FirstName))
        .map(|m| m.matched_text.as_str())
        .collect();

    assert!(first_names.contains(&"Anna"));
    assert!(first_names.contains(&"Hans"));
    assert!(first_names.contains(&"Maria"));
    assert!(first_names.contains(&"Peter"));
}
