//! Dictionary storage with FST-based lookup.

use std::collections::HashMap;

use fst::{Set, SetBuilder};

use super::category::{DictionaryCategory, Locale};
use super::entry::DictionaryEntry;
use super::error::DictionaryError;
use super::fuzzy::FuzzyMatch;
use super::ngram::NgramIndex;
use super::normalize::normalize_for_storage;

/// A named collection of entries for a specific category and locale.
#[derive(Debug)]
pub struct Dictionary {
    /// Unique identifier for this dictionary.
    pub id: String,

    /// Human-readable name.
    pub name: String,

    /// Category of entries (names, cities, etc.).
    pub category: DictionaryCategory,

    /// Locale this dictionary targets.
    pub locale: Locale,

    /// Whether this is a built-in or custom dictionary.
    pub builtin: bool,

    /// Number of entries.
    entry_count: usize,

    /// FST for fast lookups (stores normalized terms).
    fst: Set<Vec<u8>>,

    /// Entry metadata indexed by normalized term.
    entries: HashMap<String, DictionaryEntry>,

    /// Ordered list of normalized terms (for index-based lookups).
    term_list: Vec<String>,

    /// N-gram index for fast fuzzy matching.
    ngram_index: NgramIndex,
}

impl Dictionary {
    /// Create a new dictionary from entries.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        category: DictionaryCategory,
        locale: Locale,
        entries: Vec<DictionaryEntry>,
    ) -> Result<Self, DictionaryError> {
        if entries.is_empty() {
            return Err(DictionaryError::EmptyDictionary);
        }

        let id = id.into();
        let name = name.into();

        // Collect normalized terms and build entry map
        let mut normalized_entries: Vec<(String, DictionaryEntry)> = entries
            .into_iter()
            .filter(|e| e.is_valid())
            .map(|e| {
                let normalized = normalize_for_storage(&e.term);
                (normalized, e)
            })
            .collect();

        // Sort by normalized term for FST building
        normalized_entries.sort_by(|a, b| a.0.cmp(&b.0));

        // Deduplicate (keep first occurrence)
        normalized_entries.dedup_by(|a, b| a.0 == b.0);

        if normalized_entries.is_empty() {
            return Err(DictionaryError::EmptyDictionary);
        }

        // Build FST
        let mut builder = SetBuilder::memory();
        for (normalized, _) in &normalized_entries {
            builder
                .insert(normalized.as_bytes())
                .map_err(|e| DictionaryError::FstError(e.to_string()))?;
        }
        let fst = builder.into_set();

        // Build term list for index-based lookups
        let term_list: Vec<String> = normalized_entries
            .iter()
            .map(|(normalized, _)| normalized.clone())
            .collect();

        // Build n-gram index for fast fuzzy matching
        let ngram_index = NgramIndex::build(term_list.iter().map(|s| s.as_str()));

        // Build entry map
        let entry_count = normalized_entries.len();
        let entries: HashMap<String, DictionaryEntry> = normalized_entries.into_iter().collect();

        Ok(Self {
            id,
            name,
            category,
            locale,
            builtin: false,
            entry_count,
            fst,
            entries,
            term_list,
            ngram_index,
        })
    }

    /// Mark this dictionary as built-in.
    pub fn set_builtin(&mut self, builtin: bool) {
        self.builtin = builtin;
    }

    /// Get the number of entries.
    pub fn len(&self) -> usize {
        self.entry_count
    }

    /// Check if the dictionary is empty.
    pub fn is_empty(&self) -> bool {
        self.entry_count == 0
    }

    /// Check if a term exists (exact match after normalization).
    pub fn contains(&self, term: &str) -> bool {
        let normalized = normalize_for_storage(term);
        self.fst.contains(normalized.as_bytes())
    }

    /// Get entry details for a term.
    pub fn get(&self, term: &str) -> Option<&DictionaryEntry> {
        let normalized = normalize_for_storage(term);
        self.entries.get(&normalized)
    }

    /// Find fuzzy matches within threshold.
    ///
    /// Uses n-gram indexing for fast candidate selection, then
    /// Jaro-Winkler similarity for final scoring.
    /// Returns matches sorted by similarity (highest first).
    pub fn find_fuzzy(&self, term: &str, threshold: f64) -> Vec<FuzzyMatch> {
        use strsim::jaro_winkler;

        let normalized_input = normalize_for_storage(term);
        let mut matches = Vec::new();

        // Use n-gram index to find candidates (much faster for large dictionaries)
        // Use a lower overlap threshold for candidate selection since we'll filter by similarity
        let candidates = self.ngram_index.find_candidates_with_threshold(&normalized_input, 0.2);

        // For very short inputs or if ngram index returns no results, fall back to full scan
        // This ensures we don't miss matches for edge cases
        if candidates.is_empty() && normalized_input.len() >= 3 {
            // Try with even lower threshold
            let candidates = self.ngram_index.find_candidates_with_threshold(&normalized_input, 0.1);
            if candidates.is_empty() {
                // Fall back to full scan for very short queries
                return self.find_fuzzy_full_scan(term, threshold);
            }
        }

        // If candidates is empty for short inputs, do full scan
        if candidates.is_empty() {
            return self.find_fuzzy_full_scan(term, threshold);
        }

        // Check only candidates for fuzzy match
        for idx in candidates {
            if let Some(normalized_term) = self.term_list.get(idx) {
                if let Some(entry) = self.entries.get(normalized_term) {
                    let similarity = jaro_winkler(&normalized_input, normalized_term);
                    if similarity >= threshold {
                        matches.push(FuzzyMatch {
                            entry: entry.clone(),
                            similarity,
                            input: term.to_string(),
                        });
                    }
                }
            }
        }

        // Sort by similarity descending
        matches.sort_by(|a, b| {
            b.similarity
                .partial_cmp(&a.similarity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        matches
    }

    /// Find fuzzy matches using full scan (fallback for edge cases).
    fn find_fuzzy_full_scan(&self, term: &str, threshold: f64) -> Vec<FuzzyMatch> {
        use strsim::jaro_winkler;

        let normalized_input = normalize_for_storage(term);
        let mut matches = Vec::new();

        for (normalized_term, entry) in &self.entries {
            let similarity = jaro_winkler(&normalized_input, normalized_term);
            if similarity >= threshold {
                matches.push(FuzzyMatch {
                    entry: entry.clone(),
                    similarity,
                    input: term.to_string(),
                });
            }
        }

        matches.sort_by(|a, b| {
            b.similarity
                .partial_cmp(&a.similarity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        matches
    }

    /// Iterate all entries.
    pub fn iter(&self) -> impl Iterator<Item = &DictionaryEntry> {
        self.entries.values()
    }

    /// Get all normalized terms.
    pub fn terms(&self) -> impl Iterator<Item = &String> {
        self.entries.keys()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entries() -> Vec<DictionaryEntry> {
        vec![
            DictionaryEntry::with_frequency("Maximilian", 0.85),
            DictionaryEntry::with_frequency("Maria", 0.92),
            DictionaryEntry::with_frequency("Alexander", 0.78),
            DictionaryEntry::with_frequency("Anna", 0.90),
        ]
    }

    #[test]
    fn test_dictionary_new() {
        let dict = Dictionary::new(
            "test_dict",
            "Test Dictionary",
            DictionaryCategory::FirstName,
            Locale::De,
            sample_entries(),
        )
        .unwrap();

        assert_eq!(dict.id, "test_dict");
        assert_eq!(dict.len(), 4);
        assert!(!dict.is_empty());
    }

    #[test]
    fn test_dictionary_empty_error() {
        let result = Dictionary::new(
            "empty",
            "Empty",
            DictionaryCategory::FirstName,
            Locale::De,
            vec![],
        );
        assert!(matches!(result, Err(DictionaryError::EmptyDictionary)));
    }

    #[test]
    fn test_dictionary_contains() {
        let dict = Dictionary::new(
            "test",
            "Test",
            DictionaryCategory::FirstName,
            Locale::De,
            sample_entries(),
        )
        .unwrap();

        assert!(dict.contains("Maximilian"));
        assert!(dict.contains("maximilian")); // Case insensitive
        assert!(dict.contains("MARIA")); // Case insensitive
        assert!(!dict.contains("Unknown"));
    }

    #[test]
    fn test_dictionary_get() {
        let dict = Dictionary::new(
            "test",
            "Test",
            DictionaryCategory::FirstName,
            Locale::De,
            sample_entries(),
        )
        .unwrap();

        let entry = dict.get("Maria").unwrap();
        assert_eq!(entry.term, "Maria");
        assert_eq!(entry.frequency, 0.92);

        assert!(dict.get("Unknown").is_none());
    }

    #[test]
    fn test_dictionary_fuzzy() {
        let dict = Dictionary::new(
            "test",
            "Test",
            DictionaryCategory::FirstName,
            Locale::De,
            sample_entries(),
        )
        .unwrap();

        // Exact match should have similarity 1.0
        let matches = dict.find_fuzzy("Maria", 0.8);
        assert!(!matches.is_empty());
        assert!(matches[0].similarity > 0.99);

        // Typo should still match with high threshold
        let matches = dict.find_fuzzy("Mariah", 0.8);
        assert!(!matches.is_empty());
        assert!(matches[0].entry.term == "Maria");
    }

    #[test]
    fn test_dictionary_iter() {
        let dict = Dictionary::new(
            "test",
            "Test",
            DictionaryCategory::FirstName,
            Locale::De,
            sample_entries(),
        )
        .unwrap();

        let terms: Vec<_> = dict.iter().map(|e| e.term.as_str()).collect();
        assert!(terms.contains(&"Maximilian"));
        assert!(terms.contains(&"Maria"));
    }
}
