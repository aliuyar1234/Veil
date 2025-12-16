//! Fuzzy matching configuration and results.

use serde::{Deserialize, Serialize};

use super::entry::DictionaryEntry;

/// Configuration for fuzzy/approximate matching.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuzzyConfig {
    /// Enable fuzzy matching.
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Similarity threshold (0.0-1.0).
    /// Higher = stricter matching.
    #[serde(default = "default_threshold")]
    pub threshold: f64,

    /// Maximum edit distance for candidates (optimization).
    #[serde(default = "default_max_distance")]
    pub max_distance: usize,
}

fn default_enabled() -> bool {
    true
}

fn default_threshold() -> f64 {
    0.85
}

fn default_max_distance() -> usize {
    2
}

impl Default for FuzzyConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            threshold: default_threshold(),
            max_distance: default_max_distance(),
        }
    }
}

impl FuzzyConfig {
    /// Create a disabled fuzzy config (exact matching only).
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Default::default()
        }
    }

    /// Create a fuzzy config with a custom threshold.
    pub fn with_threshold(threshold: f64) -> Self {
        Self {
            enabled: true,
            threshold: threshold.clamp(0.0, 1.0),
            ..Default::default()
        }
    }

    /// Check if fuzzy matching is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

/// Result of a fuzzy match operation.
#[derive(Debug, Clone)]
pub struct FuzzyMatch {
    /// The matched dictionary entry.
    pub entry: DictionaryEntry,

    /// Similarity score (0.0-1.0).
    pub similarity: f64,

    /// The input term that was matched.
    pub input: String,
}

impl FuzzyMatch {
    /// Check if this is an exact match (similarity >= 0.999).
    pub fn is_exact(&self) -> bool {
        self.similarity >= 0.999
    }

    /// Get the confidence adjustment factor based on similarity.
    ///
    /// For exact matches, returns 1.0.
    /// For fuzzy matches, returns the similarity score.
    pub fn confidence_factor(&self) -> f32 {
        if self.is_exact() {
            1.0
        } else {
            self.similarity as f32
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_config_default() {
        let config = FuzzyConfig::default();
        assert!(config.enabled);
        assert_eq!(config.threshold, 0.85);
        assert_eq!(config.max_distance, 2);
    }

    #[test]
    fn test_fuzzy_config_disabled() {
        let config = FuzzyConfig::disabled();
        assert!(!config.enabled);
    }

    #[test]
    fn test_fuzzy_config_threshold() {
        let config = FuzzyConfig::with_threshold(0.9);
        assert!(config.enabled);
        assert_eq!(config.threshold, 0.9);

        // Clamping
        let config = FuzzyConfig::with_threshold(1.5);
        assert_eq!(config.threshold, 1.0);
    }

    #[test]
    fn test_fuzzy_match_is_exact() {
        let entry = DictionaryEntry::new("Test");

        let exact = FuzzyMatch {
            entry: entry.clone(),
            similarity: 1.0,
            input: "Test".to_string(),
        };
        assert!(exact.is_exact());

        let fuzzy = FuzzyMatch {
            entry,
            similarity: 0.95,
            input: "Tset".to_string(),
        };
        assert!(!fuzzy.is_exact());
    }

    #[test]
    fn test_fuzzy_match_confidence_factor() {
        let entry = DictionaryEntry::new("Test");

        let exact = FuzzyMatch {
            entry: entry.clone(),
            similarity: 1.0,
            input: "Test".to_string(),
        };
        assert_eq!(exact.confidence_factor(), 1.0);

        let fuzzy = FuzzyMatch {
            entry,
            similarity: 0.85,
            input: "Tset".to_string(),
        };
        assert!((fuzzy.confidence_factor() - 0.85).abs() < 0.001);
    }
}
