//! Error types for the policy library.

use thiserror::Error;

use crate::keyref::KeyRefError;

/// Errors that can occur with policies.
#[derive(Error, Debug)]
pub enum PolicyError {
    /// IO error reading policy file.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// YAML parsing error.
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    /// Unsupported policy version.
    #[error("Unsupported policy version: {0}")]
    UnsupportedVersion(String),

    /// Invalid policy content.
    #[error("Invalid policy: {0}")]
    Invalid(String),

    /// Unknown detector name.
    #[error("Unknown detector: {0}")]
    UnknownDetector(String),

    /// Key reference error.
    #[error("Key reference error: {0}")]
    KeyRefError(#[from] KeyRefError),

    /// Missing encryption key.
    #[error("Missing encryption key: {0}")]
    MissingKey(String),

    /// Invalid encryption key.
    #[error("Invalid encryption key: {0}")]
    InvalidKey(String),

    /// Cryptographic operation failed.
    #[error("Crypto error: {0}")]
    CryptoError(String),
}
