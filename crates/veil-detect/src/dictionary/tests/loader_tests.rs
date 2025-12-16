//! Tests for dictionary loading (User Story 4).

use std::io::Cursor;
use std::path::Path;
use std::sync::Arc;

use crate::dictionary::{
    category::{DictionaryCategory, Locale},
    config::DictionaryDetectorConfig,
    detector::DictionaryDetector,
    error::DictionaryError,
    loader::{load_dictionary_from_reader, load_dictionary_from_str, DictionaryLoadConfig},
    registry::DictionaryRegistry,
};

#[test]
fn test_load_simple_line_delimited() {
    let content = "ProjectX\nClientA\nSecretCode\n";
    let cursor = Cursor::new(content);

    let config = DictionaryLoadConfig::new(
        DictionaryCategory::Custom("project".to_string()),
        Locale::Generic,
    );
    let dict = load_dictionary_from_reader(cursor, Path::new("custom.txt"), config).unwrap();

    assert_eq!(dict.len(), 3);
    assert!(dict.contains("ProjectX"));
    assert!(dict.contains("ClientA"));
    assert!(dict.contains("SecretCode"));
}

#[test]
fn test_load_with_frequency_weights() {
    let content = "HighPriority\t0.95\nMediumPriority\t0.75\nLowPriority\t0.50\n";
    let cursor = Cursor::new(content);

    let config = DictionaryLoadConfig::new(
        DictionaryCategory::Custom("priority".to_string()),
        Locale::Generic,
    );
    let dict = load_dictionary_from_reader(cursor, Path::new("weights.txt"), config).unwrap();

    assert_eq!(dict.len(), 3);
    assert_eq!(dict.get("HighPriority").unwrap().frequency, 0.95);
    assert_eq!(dict.get("MediumPriority").unwrap().frequency, 0.75);
    assert_eq!(dict.get("LowPriority").unwrap().frequency, 0.50);
}

#[test]
fn test_load_with_comments_and_empty_lines() {
    let content = "# This is a comment\n\nValidEntry\n\n# Another comment\nAnotherEntry\n";
    let cursor = Cursor::new(content);

    let config = DictionaryLoadConfig::new(
        DictionaryCategory::Custom("test".to_string()),
        Locale::Generic,
    );
    let dict = load_dictionary_from_reader(cursor, Path::new("test.txt"), config).unwrap();

    assert_eq!(dict.len(), 2);
    assert!(dict.contains("ValidEntry"));
    assert!(dict.contains("AnotherEntry"));
}

#[test]
fn test_load_empty_returns_error() {
    let content = "# Only comments\n\n";
    let cursor = Cursor::new(content);

    let config = DictionaryLoadConfig::new(
        DictionaryCategory::Custom("empty".to_string()),
        Locale::Generic,
    );
    let result = load_dictionary_from_reader(cursor, Path::new("empty.txt"), config);

    assert!(matches!(result, Err(DictionaryError::EmptyDictionary)));
}

#[test]
fn test_load_invalid_frequency_returns_error() {
    let content = "Entry\tinvalid\n";
    let cursor = Cursor::new(content);

    let config = DictionaryLoadConfig::new(
        DictionaryCategory::Custom("bad".to_string()),
        Locale::Generic,
    );
    let result = load_dictionary_from_reader(cursor, Path::new("bad.txt"), config);

    assert!(matches!(result, Err(DictionaryError::ParseError { .. })));
}

#[test]
fn test_custom_dictionary_detection() {
    // Create a custom dictionary
    let content = "ProjectAlpha\nClientBeta\nSecretGamma\n";
    let dict = load_dictionary_from_str(
        content,
        "custom_projects",
        "Custom Projects",
        DictionaryCategory::Custom("project".to_string()),
        Locale::Generic,
    )
    .unwrap();

    // Set up registry with custom dictionary
    let mut registry = DictionaryRegistry::new();
    registry.register(dict).unwrap();

    // Configure detector to use custom category
    let config = DictionaryDetectorConfig::default()
        .with_categories(vec![DictionaryCategory::Custom("project".to_string())]);

    let detector = DictionaryDetector::new(config, Arc::new(registry));

    // Detect custom entries
    let matches = detector.detect_all("Das ProjectAlpha ist gestartet");

    assert!(!matches.is_empty());
    assert!(matches.iter().any(|m| m.matched_text == "ProjectAlpha"));
}

#[test]
fn test_registry_load_and_unload() {
    let content = "Entry1\nEntry2\n";
    let dict = load_dictionary_from_str(
        content,
        "temp_dict",
        "Temporary",
        DictionaryCategory::Custom("temp".to_string()),
        Locale::Generic,
    )
    .unwrap();

    let mut registry = DictionaryRegistry::new();

    // Load
    registry.register(dict).unwrap();
    assert_eq!(registry.len(), 1);
    assert!(registry.get("temp_dict").is_some());

    // Unload
    assert!(registry.unload("temp_dict"));
    assert_eq!(registry.len(), 0);
    assert!(registry.get("temp_dict").is_none());
}

#[test]
fn test_duplicate_id_returns_error() {
    let dict1 = load_dictionary_from_str(
        "Entry1\n",
        "same_id",
        "Dict 1",
        DictionaryCategory::FirstName,
        Locale::De,
    )
    .unwrap();

    let dict2 = load_dictionary_from_str(
        "Entry2\n",
        "same_id",
        "Dict 2",
        DictionaryCategory::LastName,
        Locale::De,
    )
    .unwrap();

    let mut registry = DictionaryRegistry::new();
    registry.register(dict1).unwrap();

    let result = registry.register(dict2);
    assert!(matches!(result, Err(DictionaryError::AlreadyLoaded(_))));
}

#[test]
fn test_load_config_custom_id_and_name() {
    let content = "Entry\n";
    let cursor = Cursor::new(content);

    let config = DictionaryLoadConfig::new(DictionaryCategory::FirstName, Locale::De)
        .with_id("my_custom_id")
        .with_name("My Custom Name");

    let dict = load_dictionary_from_reader(cursor, Path::new("source.txt"), config).unwrap();

    assert_eq!(dict.id, "my_custom_id");
    assert_eq!(dict.name, "My Custom Name");
}

#[test]
fn test_load_builtin_flag() {
    let content = "Entry\n";
    let cursor = Cursor::new(content);

    let config = DictionaryLoadConfig::new(DictionaryCategory::FirstName, Locale::De).as_builtin();

    let dict = load_dictionary_from_reader(cursor, Path::new("builtin.txt"), config).unwrap();

    assert!(dict.builtin);
}

#[test]
fn test_unicode_entries() {
    let content = "Müller\nSchröder\nGroß\nStraße\n";
    let dict = load_dictionary_from_str(
        content,
        "unicode_test",
        "Unicode Test",
        DictionaryCategory::LastName,
        Locale::De,
    )
    .unwrap();

    assert_eq!(dict.len(), 4);
    assert!(dict.contains("Müller"));
    assert!(dict.contains("müller")); // Case insensitive
    assert!(dict.contains("MÜLLER")); // Case insensitive
}
