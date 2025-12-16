//! PDF-specific error types.

use thiserror::Error;

/// Errors that can occur during PDF parsing.
#[derive(Debug, Error)]
pub enum PdfError {
    /// PDF file is encrypted and requires a password.
    #[error("PDF is encrypted - password required")]
    Encrypted,

    /// PDF file structure is corrupted.
    #[error("PDF file is corrupted: {0}")]
    Corrupted(String),

    /// No text content found (likely scanned).
    #[error("No extractable text found - document may be scanned")]
    NoTextContent,

    /// PDF version not supported.
    #[error("Unsupported PDF version: {0}")]
    UnsupportedVersion(String),

    /// Internal parsing error.
    #[error("PDF parse error: {0}")]
    ParseError(String),
}
