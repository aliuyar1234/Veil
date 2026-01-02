//! Error types for cryptographic operations.

use std::path::PathBuf;

use thiserror::Error;

/// Errors that can occur during cryptographic operations.
#[derive(Debug, Error)]
pub enum CryptoError {
    /// Invalid encryption key length.
    #[error("Invalid key length: expected {expected} bytes, got {actual}")]
    InvalidKeyLength {
        /// Expected key length.
        expected: usize,
        /// Actual key length.
        actual: usize,
    },

    /// Invalid initialization vector length.
    #[error("Invalid IV length: expected {expected} bytes, got {actual}")]
    InvalidIvLength {
        /// Expected IV length.
        expected: usize,
        /// Actual IV length.
        actual: usize,
    },

    /// Encryption operation failed.
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    /// Decryption operation failed (authentication error).
    #[error("Decryption failed: authentication error")]
    DecryptionFailed,

    /// Invalid ciphertext format.
    #[error("Invalid ciphertext format: {0}")]
    InvalidCiphertext(String),

    /// Hashing operation failed.
    #[error("Hashing failed: {0}")]
    HashingFailed(String),

    /// Pseudonymization operation failed.
    #[error("Pseudonymization failed: {0}")]
    PseudonymFailed(String),

    /// Tokenization operation failed.
    #[error("Tokenization failed: {0}")]
    TokenFailed(String),

    /// Token vault error.
    #[error("Vault error: {0}")]
    Vault(#[from] VaultError),

    /// Invalid configuration.
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Plaintext key storage is disabled.
    #[error("Plaintext key storage is disabled")]
    PlaintextStorageDisabled,

    /// Key not found in provider.
    #[error("Key not found: {0}")]
    KeyNotFound(String),

    /// Key management operation failed.
    #[error("Key management error: {0}")]
    KeyManagement(String),
}

/// Errors that can occur during vault operations.
#[derive(Debug, Error)]
pub enum VaultError {
    /// I/O error while interacting with storage.
    #[error("I/O error during {action} for {path}: {source}")]
    Io {
        action: &'static str,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Serialization/deserialization error while interacting with storage.
    #[error("Serialization error during {action}: {source}")]
    Json {
        action: &'static str,
        #[source]
        source: serde_json::Error,
    },

    /// Lock poisoning or synchronization error.
    #[error("Lock poisoned during {action}")]
    LockPoisoned { action: &'static str },

    /// Storage operation failed.
    #[error("Storage error: {0}")]
    Storage(String),

    /// Token not found in vault.
    #[error("Token not found: {0}")]
    NotFound(String),

    /// Token already exists.
    #[error("Token already exists: {0}")]
    Duplicate(String),

    /// Plaintext vault storage is disabled.
    #[error("Plaintext vault storage is disabled")]
    PlaintextStorageDisabled,
}

impl VaultError {
    pub(crate) fn io(
        action: &'static str,
        path: impl Into<PathBuf>,
        source: std::io::Error,
    ) -> Self {
        Self::Io {
            action,
            path: path.into(),
            source,
        }
    }

    pub(crate) fn json(action: &'static str, source: serde_json::Error) -> Self {
        Self::Json { action, source }
    }

    pub(crate) fn lock_poisoned(action: &'static str) -> Self {
        Self::LockPoisoned { action }
    }
}
