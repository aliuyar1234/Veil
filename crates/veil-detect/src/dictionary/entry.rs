//! Dictionary entry representation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single entry in a dictionary with optional metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictionaryEntry {
    /// The dictionary term (normalized form for matching).
    pub term: String,

    /// Original form(s) as they appear in source data.
    #[serde(default)]
    pub original_forms: Vec<String>,

    /// Frequency/commonality weight (0.0-1.0).
    /// Higher = more common = higher confidence when matched.
    #[serde(default = "default_frequency")]
    pub frequency: f32,

    /// Optional metadata (e.g., gender for names, population for cities).
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

fn default_frequency() -> f32 {
    0.7
}

impl DictionaryEntry {
    /// Create a new dictionary entry with just a term.
    pub fn new(term: impl Into<String>) -> Self {
        let term = term.into();
        Self {
            original_forms: vec![term.clone()],
            term,
            frequency: default_frequency(),
            metadata: HashMap::new(),
        }
    }

    /// Create an entry with a specific frequency.
    pub fn with_frequency(term: impl Into<String>, frequency: f32) -> Self {
        let term = term.into();
        Self {
            original_forms: vec![term.clone()],
            term,
            frequency: frequency.clamp(0.0, 1.0),
            metadata: HashMap::new(),
        }
    }

    /// Add an original form variant.
    pub fn add_original_form(&mut self, form: impl Into<String>) {
        self.original_forms.push(form.into());
    }

    /// Add metadata.
    pub fn add_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.insert(key.into(), value.into());
    }

    /// Check if the term is valid (non-empty).
    pub fn is_valid(&self) -> bool {
        !self.term.is_empty() && (0.0..=1.0).contains(&self.frequency)
    }
}

impl From<&str> for DictionaryEntry {
    fn from(term: &str) -> Self {
        Self::new(term)
    }
}

impl From<String> for DictionaryEntry {
    fn from(term: String) -> Self {
        Self::new(term)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_new() {
        let entry = DictionaryEntry::new("Maximilian");
        assert_eq!(entry.term, "Maximilian");
        assert_eq!(entry.frequency, 0.7);
        assert!(entry.original_forms.contains(&"Maximilian".to_string()));
    }

    #[test]
    fn test_entry_with_frequency() {
        let entry = DictionaryEntry::with_frequency("Maria", 0.95);
        assert_eq!(entry.term, "Maria");
        assert_eq!(entry.frequency, 0.95);
    }

    #[test]
    fn test_entry_frequency_clamped() {
        let entry = DictionaryEntry::with_frequency("Test", 1.5);
        assert_eq!(entry.frequency, 1.0);

        let entry = DictionaryEntry::with_frequency("Test", -0.5);
        assert_eq!(entry.frequency, 0.0);
    }

    #[test]
    fn test_entry_is_valid() {
        let entry = DictionaryEntry::new("Valid");
        assert!(entry.is_valid());

        let mut invalid = DictionaryEntry::new("");
        assert!(!invalid.is_valid());

        invalid.term = "Valid".to_string();
        invalid.frequency = 1.5;
        assert!(!invalid.is_valid());
    }
}
