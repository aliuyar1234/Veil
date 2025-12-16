//! Error types for stream processing.

use thiserror::Error;

/// Errors that can occur during stream processing.
#[derive(Error, Debug)]
pub enum StreamError {
    /// IO error while reading or writing stream data.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Error during PII detection.
    #[error("Detection error: {0}")]
    Detection(#[from] veil_detect::DetectError),

    /// JSON parsing error.
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    /// UTF-8 decoding error.
    #[error("UTF-8 decode error: {0}")]
    Utf8(#[from] std::str::Utf8Error),

    /// Configuration error.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Buffer overflow (chunk too large).
    #[error("Buffer overflow: chunk size {0} exceeds maximum {1}")]
    BufferOverflow(usize, usize),

    /// Invalid data format.
    #[error("Invalid data format: {0}")]
    InvalidFormat(String),
}
