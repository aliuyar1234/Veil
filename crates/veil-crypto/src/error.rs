//! Error types for cryptographic operations.

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
    /// Storage operation failed.
    #[error("Storage error: {0}")]
    Storage(String),

    /// Token not found in vault.
    #[error("Token not found: {0}")]
    NotFound(String),

    /// Token already exists.
    #[error("Token already exists: {0}")]
    Duplicate(String),
}
