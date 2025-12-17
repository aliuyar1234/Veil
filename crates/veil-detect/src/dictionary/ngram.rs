//! N-gram indexing for fast fuzzy matching.
//!
//! Uses trigrams (3-character sequences) to quickly find candidate
//! matches before computing expensive similarity scores.

use std::collections::{HashMap, HashSet};

/// Default n-gram size (trigrams).
const DEFAULT_NGRAM_SIZE: usize = 3;

/// Minimum candidate overlap ratio for fuzzy matching.
/// If input shares at least this fraction of n-grams with a term,
/// it's considered a candidate for full similarity check.
///
/// Note: Used by `find_candidates()` which is part of the public API
/// for advanced fuzzy matching use cases.
#[allow(dead_code)] // Public API for advanced fuzzy matching - used by find_candidates()
const MIN_OVERLAP_RATIO: f64 = 0.3;

/// N-gram index for fast fuzzy matching.
#[derive(Debug)]
pub struct NgramIndex {
    /// Size of n-grams (usually 2 or 3).
    ngram_size: usize,

    /// Index mapping n-grams to term indices.
    index: HashMap<String, Vec<usize>>,

    /// Number of terms indexed.
    term_count: usize,
}

impl NgramIndex {
    /// Create a new empty n-gram index.
    pub fn new() -> Self {
        Self::with_size(DEFAULT_NGRAM_SIZE)
    }

    /// Create an n-gram index with a specific n-gram size.
    pub fn with_size(ngram_size: usize) -> Self {
        Self {
            ngram_size: ngram_size.max(2),
            index: HashMap::new(),
            term_count: 0,
        }
    }

    /// Build an index from a list of terms.
    pub fn build<'a, I>(terms: I) -> Self
    where
        I: IntoIterator<Item = &'a str>,
    {
        let mut index = Self::new();
        for (idx, term) in terms.into_iter().enumerate() {
            index.add_term(term, idx);
        }
        index
    }

    /// Add a term to the index at a given index position.
    pub fn add_term(&mut self, term: &str, idx: usize) {
        let ngrams = Self::compute_ngrams(term, self.ngram_size);
        for ngram in ngrams {
            self.index.entry(ngram).or_default().push(idx);
        }
        self.term_count = self.term_count.max(idx + 1);
    }

    /// Find candidate term indices for a query.
    ///
    /// Returns indices of terms that share enough n-grams with the query
    /// to be potential fuzzy matches.
    #[allow(dead_code)] // Public API for advanced fuzzy matching scenarios
    pub fn find_candidates(&self, query: &str) -> Vec<usize> {
        let query_ngrams = Self::compute_ngrams(query, self.ngram_size);
        if query_ngrams.is_empty() {
            return Vec::new();
        }

        // Count how many query n-grams each term shares
        let mut term_counts: HashMap<usize, usize> = HashMap::new();
        for ngram in &query_ngrams {
            if let Some(term_indices) = self.index.get(ngram) {
                for &idx in term_indices {
                    *term_counts.entry(idx).or_default() += 1;
                }
            }
        }

        // Filter to terms with sufficient overlap
        let min_matches = ((query_ngrams.len() as f64) * MIN_OVERLAP_RATIO).ceil() as usize;
        let min_matches = min_matches.max(1);

        let mut candidates: Vec<usize> = term_counts
            .into_iter()
            .filter(|&(_, count)| count >= min_matches)
            .map(|(idx, _)| idx)
            .collect();

        candidates.sort_unstable();
        candidates.dedup();
        candidates
    }

    /// Find candidates with a custom minimum overlap ratio.
    pub fn find_candidates_with_threshold(&self, query: &str, overlap_ratio: f64) -> Vec<usize> {
        let query_ngrams = Self::compute_ngrams(query, self.ngram_size);
        if query_ngrams.is_empty() {
            return Vec::new();
        }

        let mut term_counts: HashMap<usize, usize> = HashMap::new();
        for ngram in &query_ngrams {
            if let Some(term_indices) = self.index.get(ngram) {
                for &idx in term_indices {
                    *term_counts.entry(idx).or_default() += 1;
                }
            }
        }

        let min_matches = ((query_ngrams.len() as f64) * overlap_ratio).ceil() as usize;
        let min_matches = min_matches.max(1);

        let mut candidates: Vec<usize> = term_counts
            .into_iter()
            .filter(|&(_, count)| count >= min_matches)
            .map(|(idx, _)| idx)
            .collect();

        candidates.sort_unstable();
        candidates.dedup();
        candidates
    }

    /// Compute n-grams for a string.
    pub fn compute_ngrams(s: &str, n: usize) -> HashSet<String> {
        let s = s.to_lowercase();
        let chars: Vec<char> = s.chars().collect();

        if chars.len() < n {
            // For very short strings, return the string itself as a single "n-gram"
            let mut set = HashSet::new();
            set.insert(s);
            return set;
        }

        chars
            .windows(n)
            .map(|window| window.iter().collect::<String>())
            .collect()
    }

    /// Get the number of unique n-grams in the index.
    #[allow(dead_code)] // Public API for index statistics
    pub fn ngram_count(&self) -> usize {
        self.index.len()
    }

    /// Get the number of terms indexed.
    #[allow(dead_code)] // Public API for index statistics
    pub fn term_count(&self) -> usize {
        self.term_count
    }
}

impl Default for NgramIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_ngrams() {
        let ngrams = NgramIndex::compute_ngrams("hello", 3);
        assert!(ngrams.contains("hel"));
        assert!(ngrams.contains("ell"));
        assert!(ngrams.contains("llo"));
        assert_eq!(ngrams.len(), 3);
    }

    #[test]
    fn test_compute_ngrams_short_string() {
        let ngrams = NgramIndex::compute_ngrams("hi", 3);
        assert_eq!(ngrams.len(), 1);
        assert!(ngrams.contains("hi"));
    }

    #[test]
    fn test_compute_ngrams_case_insensitive() {
        let ngrams1 = NgramIndex::compute_ngrams("Hello", 3);
        let ngrams2 = NgramIndex::compute_ngrams("HELLO", 3);
        assert_eq!(ngrams1, ngrams2);
    }

    #[test]
    fn test_build_index() {
        let terms = vec!["hello", "world", "help"];
        let index = NgramIndex::build(terms.iter().copied());

        assert_eq!(index.term_count(), 3);
        assert!(index.ngram_count() > 0);
    }

    #[test]
    fn test_find_candidates_exact() {
        let terms = vec!["hello", "world", "help", "held"];
        let index = NgramIndex::build(terms.iter().copied());

        // "hello" should match "hello"
        let candidates = index.find_candidates("hello");
        assert!(candidates.contains(&0)); // hello
    }

    #[test]
    fn test_find_candidates_similar() {
        let terms = vec!["hello", "world", "hallo", "held"];
        let index = NgramIndex::build(terms.iter().copied());

        // "hallo" should match both "hello" and "hallo" due to shared ngrams
        let candidates = index.find_candidates("hallo");
        assert!(candidates.contains(&2)); // hallo (exact)
        // hello may or may not be included depending on overlap
    }

    #[test]
    fn test_find_candidates_no_match() {
        let terms = vec!["hello", "world"];
        let index = NgramIndex::build(terms.iter().copied());

        // "xyz" has no shared ngrams
        let candidates = index.find_candidates("xyz");
        assert!(candidates.is_empty());
    }

    #[test]
    fn test_find_candidates_with_threshold() {
        let terms = vec!["hello", "hallo", "help", "world"];
        let index = NgramIndex::build(terms.iter().copied());

        // With high overlap requirement, should find fewer matches
        let high_threshold = index.find_candidates_with_threshold("hello", 0.8);
        let low_threshold = index.find_candidates_with_threshold("hello", 0.2);

        // Low threshold should have >= candidates as high threshold
        assert!(low_threshold.len() >= high_threshold.len());
    }

    #[test]
    fn test_add_term() {
        let mut index = NgramIndex::new();
        index.add_term("hello", 0);
        index.add_term("world", 1);

        assert_eq!(index.term_count(), 2);

        let candidates = index.find_candidates("hello");
        assert!(candidates.contains(&0));
    }

    #[test]
    fn test_bigrams() {
        let _index = NgramIndex::with_size(2); // Verify construction with custom size
        let ngrams = NgramIndex::compute_ngrams("hello", 2);

        assert!(ngrams.contains("he"));
        assert!(ngrams.contains("el"));
        assert!(ngrams.contains("ll"));
        assert!(ngrams.contains("lo"));
        assert_eq!(ngrams.len(), 4);
    }
}
