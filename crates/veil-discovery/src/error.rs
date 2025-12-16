//! Error types for the discovery module.

use std::io;
use thiserror::Error;

/// Errors that can occur during PII discovery operations.
#[derive(Debug, Error)]
pub enum DiscoveryError {
    /// I/O error while accessing files or directories
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// Error parsing a file
    #[error("Parse error: {0}")]
    Parse(#[from] veil_parsers::ParseError),

    /// Error detecting PII
    #[error("Detection error: {0}")]
    Detection(#[from] veil_detect::DetectError),

    /// Invalid glob pattern
    #[error("Invalid glob pattern: {0}")]
    InvalidPattern(#[from] glob::PatternError),

    /// Path is not a valid directory
    #[error("Path is not a directory: {0}")]
    NotADirectory(String),

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Report generation error
    #[error("Report generation error: {0}")]
    ReportError(String),

    /// JSON serialization error
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, DiscoveryError>;
