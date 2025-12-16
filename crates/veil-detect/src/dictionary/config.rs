//! Dictionary detector configuration.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::category::{DictionaryCategory, Locale};
use super::fuzzy::FuzzyConfig;

/// Configuration for the dictionary detector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictionaryDetectorConfig {
    /// Enabled dictionary categories.
    #[serde(default = "default_categories")]
    pub categories: Vec<DictionaryCategory>,

    /// Enabled locales.
    #[serde(default = "default_locales")]
    pub locales: Vec<Locale>,

    /// Fuzzy matching configuration.
    #[serde(default)]
    pub fuzzy: FuzzyConfig,

    /// Paths to custom dictionary files.
    #[serde(default)]
    pub custom_dictionaries: Vec<PathBuf>,

    /// Minimum confidence threshold for reporting.
    #[serde(default = "default_min_confidence")]
    pub min_confidence: f32,

    /// Require word boundaries for matches.
    #[serde(default = "default_require_boundaries")]
    pub require_word_boundaries: bool,
}

fn default_categories() -> Vec<DictionaryCategory> {
    vec![DictionaryCategory::FirstName, DictionaryCategory::LastName]
}

fn default_locales() -> Vec<Locale> {
    vec![Locale::At, Locale::De, Locale::Ch]
}

fn default_min_confidence() -> f32 {
    0.5
}

fn default_require_boundaries() -> bool {
    true
}

impl Default for DictionaryDetectorConfig {
    fn default() -> Self {
        Self {
            categories: default_categories(),
            locales: default_locales(),
            fuzzy: FuzzyConfig::default(),
            custom_dictionaries: Vec::new(),
            min_confidence: default_min_confidence(),
            require_word_boundaries: default_require_boundaries(),
        }
    }
}

impl DictionaryDetectorConfig {
    /// Create a new config with all categories enabled.
    pub fn all_categories() -> Self {
        Self {
            categories: vec![
                DictionaryCategory::FirstName,
                DictionaryCategory::LastName,
                DictionaryCategory::City,
                DictionaryCategory::Company,
            ],
            ..Default::default()
        }
    }

    /// Create a config for name detection only.
    pub fn names_only() -> Self {
        Self {
            categories: vec![DictionaryCategory::FirstName, DictionaryCategory::LastName],
            ..Default::default()
        }
    }

    /// Create a config for location detection only.
    pub fn locations_only() -> Self {
        Self {
            categories: vec![DictionaryCategory::City, DictionaryCategory::Street],
            ..Default::default()
        }
    }

    /// Add a custom dictionary path.
    pub fn with_custom_dictionary(mut self, path: impl Into<PathBuf>) -> Self {
        self.custom_dictionaries.push(path.into());
        self
    }

    /// Set the minimum confidence threshold.
    pub fn with_min_confidence(mut self, confidence: f32) -> Self {
        self.min_confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Set the fuzzy matching configuration.
    pub fn with_fuzzy(mut self, fuzzy: FuzzyConfig) -> Self {
        self.fuzzy = fuzzy;
        self
    }

    /// Disable fuzzy matching.
    pub fn without_fuzzy(mut self) -> Self {
        self.fuzzy = FuzzyConfig::disabled();
        self
    }

    /// Disable word boundary requirement.
    pub fn without_word_boundaries(mut self) -> Self {
        self.require_word_boundaries = false;
        self
    }

    /// Set specific locales.
    pub fn with_locales(mut self, locales: Vec<Locale>) -> Self {
        self.locales = locales;
        self
    }

    /// Set specific categories.
    pub fn with_categories(mut self, categories: Vec<DictionaryCategory>) -> Self {
        self.categories = categories;
        self
    }

    /// Check if a category is enabled.
    pub fn is_category_enabled(&self, category: &DictionaryCategory) -> bool {
        self.categories.contains(category)
    }

    /// Check if a locale is enabled.
    pub fn is_locale_enabled(&self, locale: Locale) -> bool {
        self.locales.contains(&locale) || locale == Locale::Generic
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = DictionaryDetectorConfig::default();

        assert!(config.categories.contains(&DictionaryCategory::FirstName));
        assert!(config.categories.contains(&DictionaryCategory::LastName));
        assert!(config.locales.contains(&Locale::At));
        assert!(config.locales.contains(&Locale::De));
        assert!(config.fuzzy.enabled);
        assert!(config.require_word_boundaries);
    }

    #[test]
    fn test_all_categories() {
        let config = DictionaryDetectorConfig::all_categories();

        assert!(config.is_category_enabled(&DictionaryCategory::FirstName));
        assert!(config.is_category_enabled(&DictionaryCategory::City));
        assert!(config.is_category_enabled(&DictionaryCategory::Company));
    }

    #[test]
    fn test_names_only() {
        let config = DictionaryDetectorConfig::names_only();

        assert!(config.is_category_enabled(&DictionaryCategory::FirstName));
        assert!(config.is_category_enabled(&DictionaryCategory::LastName));
        assert!(!config.is_category_enabled(&DictionaryCategory::City));
    }

    #[test]
    fn test_with_custom_dictionary() {
        let config = DictionaryDetectorConfig::default()
            .with_custom_dictionary("custom.txt")
            .with_custom_dictionary("another.txt");

        assert_eq!(config.custom_dictionaries.len(), 2);
    }

    #[test]
    fn test_without_fuzzy() {
        let config = DictionaryDetectorConfig::default().without_fuzzy();

        assert!(!config.fuzzy.enabled);
    }
}
