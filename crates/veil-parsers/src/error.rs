//! Error types for the parsing library.

use thiserror::Error;

#[cfg(feature = "pdf")]
use crate::pdf::PdfError;

/// Errors that can occur during parsing.
#[derive(Error, Debug)]
pub enum ParseError {
    /// IO error reading the file.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid or unsupported character encoding.
    #[error("Invalid encoding: {0}")]
    Encoding(String),

    /// Malformed CSV data.
    #[error("Malformed CSV at row {row}: {message}")]
    CsvError {
        /// Row number where error occurred.
        row: usize,
        /// Error description.
        message: String,
    },

    /// Invalid JSON syntax.
    #[error("Invalid JSON: {0}")]
    JsonError(#[from] serde_json::Error),

    /// PDF parsing error.
    #[cfg(feature = "pdf")]
    #[error("PDF error: {0}")]
    PdfError(#[from] PdfError),

    /// File exceeds size limit.
    #[error("File too large: {size} bytes exceeds {max} byte limit")]
    FileTooLarge {
        /// Actual file size.
        size: usize,
        /// Maximum allowed size.
        max: usize,
    },

    /// Format not supported.
    #[error("Unsupported format: {0:?}")]
    UnsupportedFormat(Option<String>),
}
