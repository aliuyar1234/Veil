//! Dictionary file loading.

#![allow(dead_code)]

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use super::category::{DictionaryCategory, Locale};
use super::dictionary::Dictionary;
use super::entry::DictionaryEntry;
use super::error::DictionaryError;

/// Configuration for loading a dictionary from file.
#[derive(Debug, Clone)]
pub struct DictionaryLoadConfig {
    /// Category for the dictionary.
    pub category: DictionaryCategory,

    /// Locale for the dictionary.
    pub locale: Locale,

    /// Optional custom ID (defaults to filename).
    pub id: Option<String>,

    /// Optional custom name (defaults to filename).
    pub name: Option<String>,

    /// Whether to treat as built-in.
    pub builtin: bool,
}

impl DictionaryLoadConfig {
    /// Create a new load config with required fields.
    pub fn new(category: DictionaryCategory, locale: Locale) -> Self {
        Self {
            category,
            locale,
            id: None,
            name: None,
            builtin: false,
        }
    }

    /// Set a custom ID.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set a custom name.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Mark as built-in.
    pub fn as_builtin(mut self) -> Self {
        self.builtin = true;
        self
    }
}

/// Load a dictionary from a file.
///
/// Supports two formats:
/// 1. Simple: One term per line
/// 2. With frequency: term<TAB>frequency (tab-separated)
///
/// Lines starting with # are treated as comments.
/// Empty lines are skipped.
pub fn load_dictionary_from_file(
    path: &Path,
    config: DictionaryLoadConfig,
) -> Result<Dictionary, DictionaryError> {
    let file = File::open(path).map_err(|e| DictionaryError::IoError {
        path: path.to_path_buf(),
        source: e,
    })?;

    let reader = BufReader::new(file);
    load_dictionary_from_reader(reader, path, config)
}

/// Load a dictionary from a reader.
pub fn load_dictionary_from_reader<R: BufRead>(
    reader: R,
    source_path: &Path,
    config: DictionaryLoadConfig,
) -> Result<Dictionary, DictionaryError> {
    let mut entries = Vec::new();

    for (line_num, line_result) in reader.lines().enumerate() {
        let line = line_result.map_err(|e| DictionaryError::IoError {
            path: source_path.to_path_buf(),
            source: e,
        })?;

        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Parse line (tab-separated: term<TAB>frequency)
        let entry = if let Some((term, freq_str)) = line.split_once('\t') {
            let frequency: f32 = freq_str.parse().map_err(|_| DictionaryError::ParseError {
                line: line_num + 1,
                message: format!("Invalid frequency value: {}", freq_str),
            })?;
            DictionaryEntry::with_frequency(term.trim(), frequency)
        } else {
            DictionaryEntry::new(line)
        };

        if !entry.is_valid() {
            return Err(DictionaryError::ParseError {
                line: line_num + 1,
                message: "Invalid entry (empty term or invalid frequency)".to_string(),
            });
        }

        entries.push(entry);
    }

    if entries.is_empty() {
        return Err(DictionaryError::EmptyDictionary);
    }

    // Determine ID and name
    let filename = source_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");

    let id = config.id.unwrap_or_else(|| filename.to_string());
    let name = config.name.unwrap_or_else(|| filename.to_string());

    let mut dict = Dictionary::new(id, name, config.category, config.locale, entries)?;

    dict.set_builtin(config.builtin);

    Ok(dict)
}

/// Load a dictionary from a string (for embedded dictionaries).
pub fn load_dictionary_from_str(
    content: &str,
    id: impl Into<String>,
    name: impl Into<String>,
    category: DictionaryCategory,
    locale: Locale,
) -> Result<Dictionary, DictionaryError> {
    let mut entries = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Parse line
        let entry = if let Some((term, freq_str)) = line.split_once('\t') {
            let frequency: f32 = freq_str.parse().map_err(|_| DictionaryError::ParseError {
                line: line_num + 1,
                message: format!("Invalid frequency value: {}", freq_str),
            })?;
            DictionaryEntry::with_frequency(term.trim(), frequency)
        } else {
            DictionaryEntry::new(line)
        };

        if entry.is_valid() {
            entries.push(entry);
        }
    }

    if entries.is_empty() {
        return Err(DictionaryError::EmptyDictionary);
    }

    let mut dict = Dictionary::new(id, name, category, locale, entries)?;
    dict.set_builtin(true);

    Ok(dict)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_load_simple_format() {
        let content = "Max\nMaria\nAlexander\n";
        let cursor = Cursor::new(content);

        let config = DictionaryLoadConfig::new(DictionaryCategory::FirstName, Locale::De);
        let dict = load_dictionary_from_reader(cursor, Path::new("test.txt"), config).unwrap();

        assert_eq!(dict.len(), 3);
        assert!(dict.contains("Max"));
        assert!(dict.contains("Maria"));
    }

    #[test]
    fn test_load_with_frequency() {
        let content = "Max\t0.85\nMaria\t0.92\nAlexander\t0.78\n";
        let cursor = Cursor::new(content);

        let config = DictionaryLoadConfig::new(DictionaryCategory::FirstName, Locale::De);
        let dict = load_dictionary_from_reader(cursor, Path::new("test.txt"), config).unwrap();

        assert_eq!(dict.len(), 3);
        assert_eq!(dict.get("Maria").unwrap().frequency, 0.92);
    }

    #[test]
    fn test_load_with_comments() {
        let content = "# This is a comment\nMax\n\n# Another comment\nMaria\n";
        let cursor = Cursor::new(content);

        let config = DictionaryLoadConfig::new(DictionaryCategory::FirstName, Locale::De);
        let dict = load_dictionary_from_reader(cursor, Path::new("test.txt"), config).unwrap();

        assert_eq!(dict.len(), 2);
    }

    #[test]
    fn test_load_empty_error() {
        let content = "# Only comments\n\n";
        let cursor = Cursor::new(content);

        let config = DictionaryLoadConfig::new(DictionaryCategory::FirstName, Locale::De);
        let result = load_dictionary_from_reader(cursor, Path::new("test.txt"), config);

        assert!(matches!(result, Err(DictionaryError::EmptyDictionary)));
    }

    #[test]
    fn test_load_from_str() {
        let content = "Max\nMaria\nAlexander";
        let dict = load_dictionary_from_str(
            content,
            "test",
            "Test Dict",
            DictionaryCategory::FirstName,
            Locale::De,
        )
        .unwrap();

        assert_eq!(dict.len(), 3);
        assert!(dict.builtin);
    }
}
