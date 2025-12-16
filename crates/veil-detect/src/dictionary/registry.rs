//! Dictionary registry for managing loaded dictionaries.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use super::category::{DictionaryCategory, Locale};
use super::dictionary::Dictionary;
use super::error::DictionaryError;
use super::loader::{load_dictionary_from_file, DictionaryLoadConfig};

/// Central registry managing all loaded dictionaries.
#[derive(Debug, Default)]
pub struct DictionaryRegistry {
    /// Loaded dictionaries indexed by ID.
    dictionaries: HashMap<String, Arc<Dictionary>>,

    /// Index by category for fast lookup.
    by_category: HashMap<DictionaryCategory, Vec<String>>,

    /// Index by locale.
    by_locale: HashMap<Locale, Vec<String>>,
}

impl DictionaryRegistry {
    /// Create empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a dictionary.
    ///
    /// Returns an error if a dictionary with the same ID is already loaded.
    pub fn register(&mut self, dictionary: Dictionary) -> Result<(), DictionaryError> {
        let id = dictionary.id.clone();
        let category = dictionary.category.clone();
        let locale = dictionary.locale;

        if self.dictionaries.contains_key(&id) {
            return Err(DictionaryError::AlreadyLoaded(id));
        }

        // Add to main storage
        self.dictionaries.insert(id.clone(), Arc::new(dictionary));

        // Add to category index
        self.by_category
            .entry(category)
            .or_default()
            .push(id.clone());

        // Add to locale index
        self.by_locale.entry(locale).or_default().push(id);

        Ok(())
    }

    /// Load dictionary from file.
    pub fn load(
        &mut self,
        path: &Path,
        config: DictionaryLoadConfig,
    ) -> Result<String, DictionaryError> {
        let dictionary = load_dictionary_from_file(path, config)?;
        let id = dictionary.id.clone();
        self.register(dictionary)?;
        Ok(id)
    }

    /// Unload a dictionary by ID.
    ///
    /// Returns true if the dictionary was found and removed.
    pub fn unload(&mut self, id: &str) -> bool {
        if let Some(dict) = self.dictionaries.remove(id) {
            // Remove from category index
            if let Some(ids) = self.by_category.get_mut(&dict.category) {
                ids.retain(|i| i != id);
            }

            // Remove from locale index
            if let Some(ids) = self.by_locale.get_mut(&dict.locale) {
                ids.retain(|i| i != id);
            }

            true
        } else {
            false
        }
    }

    /// Get dictionary by ID.
    pub fn get(&self, id: &str) -> Option<Arc<Dictionary>> {
        self.dictionaries.get(id).cloned()
    }

    /// Get all dictionaries for a category.
    pub fn by_category(&self, category: &DictionaryCategory) -> Vec<Arc<Dictionary>> {
        self.by_category
            .get(category)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.dictionaries.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all dictionaries for a locale.
    pub fn by_locale(&self, locale: Locale) -> Vec<Arc<Dictionary>> {
        self.by_locale
            .get(&locale)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.dictionaries.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all dictionaries matching both category and locale.
    pub fn by_category_and_locale(
        &self,
        category: &DictionaryCategory,
        locale: Locale,
    ) -> Vec<Arc<Dictionary>> {
        self.by_category(category)
            .into_iter()
            .filter(|d| d.locale == locale || d.locale == Locale::Generic)
            .collect()
    }

    /// Get all loaded dictionary IDs.
    pub fn ids(&self) -> impl Iterator<Item = &String> {
        self.dictionaries.keys()
    }

    /// Get the number of loaded dictionaries.
    pub fn len(&self) -> usize {
        self.dictionaries.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.dictionaries.is_empty()
    }

    /// Iterate all dictionaries.
    pub fn iter(&self) -> impl Iterator<Item = &Arc<Dictionary>> {
        self.dictionaries.values()
    }

    /// Reload a dictionary from its source.
    ///
    /// Note: This requires storing the source path, which is not currently implemented.
    /// For now, this just returns NotFound.
    pub fn reload(&mut self, id: &str) -> Result<(), DictionaryError> {
        if !self.dictionaries.contains_key(id) {
            return Err(DictionaryError::NotFound(id.to_string()));
        }

        // TODO: Implement reload by storing source paths
        // For now, just verify the dictionary exists
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dictionary::entry::DictionaryEntry;

    fn sample_dictionary(id: &str, category: DictionaryCategory, locale: Locale) -> Dictionary {
        Dictionary::new(
            id,
            format!("Test {}", id),
            category,
            locale,
            vec![
                DictionaryEntry::new("Entry1"),
                DictionaryEntry::new("Entry2"),
            ],
        )
        .unwrap()
    }

    #[test]
    fn test_registry_register() {
        let mut registry = DictionaryRegistry::new();
        let dict = sample_dictionary("test", DictionaryCategory::FirstName, Locale::De);

        registry.register(dict).unwrap();

        assert_eq!(registry.len(), 1);
        assert!(registry.get("test").is_some());
    }

    #[test]
    fn test_registry_duplicate_error() {
        let mut registry = DictionaryRegistry::new();
        let dict1 = sample_dictionary("test", DictionaryCategory::FirstName, Locale::De);
        let dict2 = sample_dictionary("test", DictionaryCategory::LastName, Locale::At);

        registry.register(dict1).unwrap();
        let result = registry.register(dict2);

        assert!(matches!(result, Err(DictionaryError::AlreadyLoaded(_))));
    }

    #[test]
    fn test_registry_unload() {
        let mut registry = DictionaryRegistry::new();
        let dict = sample_dictionary("test", DictionaryCategory::FirstName, Locale::De);

        registry.register(dict).unwrap();
        assert_eq!(registry.len(), 1);

        assert!(registry.unload("test"));
        assert_eq!(registry.len(), 0);
        assert!(registry.get("test").is_none());
    }

    #[test]
    fn test_registry_by_category() {
        let mut registry = DictionaryRegistry::new();

        registry
            .register(sample_dictionary(
                "first_de",
                DictionaryCategory::FirstName,
                Locale::De,
            ))
            .unwrap();
        registry
            .register(sample_dictionary(
                "first_at",
                DictionaryCategory::FirstName,
                Locale::At,
            ))
            .unwrap();
        registry
            .register(sample_dictionary(
                "last_de",
                DictionaryCategory::LastName,
                Locale::De,
            ))
            .unwrap();

        let first_names = registry.by_category(&DictionaryCategory::FirstName);
        assert_eq!(first_names.len(), 2);

        let last_names = registry.by_category(&DictionaryCategory::LastName);
        assert_eq!(last_names.len(), 1);
    }

    #[test]
    fn test_registry_by_locale() {
        let mut registry = DictionaryRegistry::new();

        registry
            .register(sample_dictionary(
                "first_de",
                DictionaryCategory::FirstName,
                Locale::De,
            ))
            .unwrap();
        registry
            .register(sample_dictionary(
                "last_de",
                DictionaryCategory::LastName,
                Locale::De,
            ))
            .unwrap();
        registry
            .register(sample_dictionary(
                "first_at",
                DictionaryCategory::FirstName,
                Locale::At,
            ))
            .unwrap();

        let de_dicts = registry.by_locale(Locale::De);
        assert_eq!(de_dicts.len(), 2);

        let at_dicts = registry.by_locale(Locale::At);
        assert_eq!(at_dicts.len(), 1);
    }
}
