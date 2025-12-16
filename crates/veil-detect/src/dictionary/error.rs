//! Error types for dictionary operations.

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during dictionary operations.
#[derive(Debug, Error)]
pub enum DictionaryError {
    /// Failed to read dictionary file.
    #[error("Failed to read dictionary file '{path}': {source}")]
    IoError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Invalid dictionary format.
    #[error("Invalid dictionary format at line {line}: {message}")]
    ParseError {
        /// Line number where the error occurred.
        line: usize,
        /// Description of the parsing error.
        message: String,
    },

    /// Dictionary not found.
    #[error("Dictionary not found: {0}")]
    NotFound(String),

    /// Dictionary already loaded.
    #[error("Dictionary already loaded: {0}")]
    AlreadyLoaded(String),

    /// Invalid configuration.
    #[error("Invalid configuration: {0}")]
    ConfigError(String),

    /// FST build error.
    #[error("FST build error: {0}")]
    FstError(String),

    /// Empty dictionary.
    #[error("Dictionary is empty")]
    EmptyDictionary,

    /// Invalid entry.
    #[error("Invalid entry: {0}")]
    InvalidEntry(String),
}

impl From<std::io::Error> for DictionaryError {
    fn from(err: std::io::Error) -> Self {
        DictionaryError::IoError {
            path: PathBuf::new(),
            source: err,
        }
    }
}
