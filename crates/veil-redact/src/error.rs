//! Error types for the redaction library.

use thiserror::Error;

/// Errors that can occur during redaction.
#[derive(Error, Debug)]
pub enum RedactError {
    /// Invalid position in text.
    #[error("Invalid position: {0}..{1} in text of length {2}")]
    InvalidPosition(usize, usize, usize),

    /// Configuration error.
    #[error("Configuration error: {0}")]
    ConfigError(String),
}
