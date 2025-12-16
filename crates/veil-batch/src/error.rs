//! Error types for batch processing.

use thiserror::Error;

/// Errors that can occur during batch processing.
#[derive(Error, Debug)]
pub enum BatchError {
    /// IO error occurred.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Archive processing error.
    #[error("Archive error: {0}")]
    Archive(String),

    /// Invalid password for encrypted archive.
    #[error("Invalid password for archive")]
    InvalidPassword,

    /// Archive nesting depth exceeded.
    #[error("Archive depth exceeded: max {max}, got {depth}")]
    ArchiveDepthExceeded {
        /// Maximum allowed depth.
        max: usize,
        /// Actual depth encountered.
        depth: usize,
    },

    /// Invalid glob pattern.
    #[error("Invalid glob pattern: {0}")]
    InvalidPattern(#[from] glob::PatternError),

    /// Parse error from veil-parsers.
    #[error("Parse error: {0}")]
    Parse(#[from] veil_parsers::ParseError),

    /// Batch processing was cancelled.
    #[error("Batch processing cancelled")]
    Cancelled,

    /// File too large to process.
    #[error("File too large: {size} bytes exceeds {max}")]
    FileTooLarge {
        /// File size in bytes.
        size: u64,
        /// Maximum allowed size.
        max: usize,
    },

    /// ZIP archive error.
    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),

    /// Invalid configuration.
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Walkdir error.
    #[error("Directory walk error: {0}")]
    WalkDir(#[from] walkdir::Error),
}

/// Result type for batch operations.
pub type BatchResult<T> = std::result::Result<T, BatchError>;
