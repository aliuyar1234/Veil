//! Error types for the audit library.

use thiserror::Error;

/// Errors that can occur with audit logging.
#[derive(Error, Debug)]
pub enum AuditError {
    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Checksum verification failed.
    #[error("Checksum verification failed for entry {0}")]
    ChecksumMismatch(String),

    /// Hash chain broken.
    #[error("Hash chain broken at entry {0}")]
    ChainBroken(String),

    /// Log directory not found.
    #[error("Log directory not found: {0}")]
    DirectoryNotFound(String),

    /// Invalid encryption key.
    #[error("Invalid encryption key: expected {expected} bytes, got {actual} bytes")]
    InvalidKey {
        /// Expected key length.
        expected: usize,
        /// Actual key length provided.
        actual: usize,
    },

    /// Encryption feature is disabled.
    #[error("Encryption is disabled; enable the `encryption` feature to use encrypted audit logs")]
    EncryptionUnavailable,
}
