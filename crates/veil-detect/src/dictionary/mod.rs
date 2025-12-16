//! Dictionary-based PII detection.
//!
//! This module provides detection of names, locations, and companies using
//! locale-specific dictionaries with fuzzy matching support.

mod category;
mod confidence;
mod config;
#[allow(clippy::module_inception)]
mod dictionary;
mod entry;
mod error;
mod fuzzy;
mod loader;
mod ngram;
mod normalize;
mod registry;

pub use category::{DictionaryCategory, Locale};
pub use config::DictionaryDetectorConfig;
pub use dictionary::Dictionary;
pub use entry::DictionaryEntry;
pub use error::DictionaryError;
pub use fuzzy::{FuzzyConfig, FuzzyMatch};
pub use loader::DictionaryLoadConfig;
pub use normalize::normalize_for_matching;
pub use registry::DictionaryRegistry;

mod boundaries;
mod detector;
pub use detector::DictionaryDetector;

#[cfg(test)]
mod tests;
