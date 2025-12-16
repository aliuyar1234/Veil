//! # veil-crypto
//!
//! Cryptographic protection library for PII data.
//!
//! Provides four protection strategies:
//! - **Encryption**: AES-256-GCM for reversible protection
//! - **Hashing**: SHA-256/SHA-512 for irreversible protection
//! - **Pseudonymization**: Realistic fake data generation
//! - **Tokenization**: Random tokens with vault storage
//!
//! ## Example
//!
//! ```rust,ignore
//! use veil_crypto::{encrypt, decrypt, EncryptionConfig, OutputFormat};
//!
//! let key = [0u8; 32]; // Use proper key management!
//! let config = EncryptionConfig::new(key.to_vec());
//!
//! let result = encrypt(b"sensitive data", &config)?;
//! let decrypted = decrypt(&result, &config)?;
//! ```

mod encrypt;
mod error;
mod hash;
mod pseudonym;
mod tokenize;
mod types;
pub mod vault;

pub use encrypt::{decrypt, decrypt_raw, encrypt, EncryptionConfig};
pub use error::{CryptoError, VaultError};
pub use hash::{generate_salt, hash, verify_hash, HashAlgorithm, HashConfig};
pub use pseudonym::{pseudonymize, PseudonymConfig, PseudonymDataType};
pub use tokenize::{delete_token, detokenize, tokenize, TokenConfig, TokenFormat};
pub use types::{CryptoMetadata, CryptoResult, OutputFormat, ProtectionMode};
pub use vault::{FileVault, InMemoryVault, TokenVault};
