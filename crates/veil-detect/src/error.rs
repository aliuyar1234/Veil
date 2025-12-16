//! Error types for the detection library.

use thiserror::Error;

/// Errors that can occur during PII detection.
#[derive(Error, Debug)]
pub enum DetectError {
    /// Invalid regex pattern.
    #[error("Invalid regex pattern: {0}")]
    InvalidPattern(#[from] regex::Error),

    /// Detector not found.
    #[error("Detector not found: {0}")]
    DetectorNotFound(String),

    /// Configuration error.
    #[error("Configuration error: {0}")]
    ConfigError(String),
}
