# Data Model: Cryptographic Protection

**Feature**: 010-crypto-protection
**Date**: 2025-12-15

## Entity Relationship Overview

```text
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│ ProtectionMode  │     │  CryptoConfig   │     │  CryptoResult   │
│─────────────────│     │─────────────────│     │─────────────────│
│ Encrypt         │────▶│ encryption_cfg  │────▶│ protected_value │
│ Hash            │     │ hash_cfg        │     │ metadata        │
│ Pseudonymize    │     │ pseudonym_cfg   │     │ mode            │
│ Tokenize        │     │ token_cfg       │     └─────────────────┘
└─────────────────┘     └─────────────────┘
                               │
                               ▼
                        ┌─────────────────┐
                        │   TokenVault    │
                        │─────────────────│
                        │ store()         │
                        │ lookup()        │
                        │ delete()        │
                        │ exists()        │
                        └─────────────────┘
```

## Core Types

### ProtectionMode

Enumeration of available protection strategies.

```rust
/// Protection strategy for PII data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProtectionMode {
    /// AES-256-GCM encryption (reversible with key)
    Encrypt,
    /// SHA-256/SHA-512 hashing (irreversible)
    Hash,
    /// Replace with realistic fake data
    Pseudonymize,
    /// Replace with random token (reversible via vault)
    Tokenize,
}
```

### EncryptionConfig

Configuration for AES-256-GCM encryption operations.

```rust
/// Configuration for encryption operations.
#[derive(Debug, Clone)]
pub struct EncryptionConfig {
    /// Encryption key (must be 32 bytes for AES-256)
    pub key: Vec<u8>,
    /// Output encoding format
    pub output_format: OutputFormat,
}

/// Output encoding format for binary data.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputFormat {
    #[default]
    Base64,
    Hex,
}
```

**Validation Rules**:
- `key` must be exactly 32 bytes
- Key is zeroized on drop

### HashConfig

Configuration for SHA-256/SHA-512 hashing operations.

```rust
/// Configuration for hashing operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashConfig {
    /// Hash algorithm to use
    pub algorithm: HashAlgorithm,
    /// Salt for the hash (optional for deterministic mode)
    pub salt: Option<Vec<u8>>,
    /// Output encoding format
    pub output_format: OutputFormat,
    /// If true, use HMAC with salt as key
    pub keyed: bool,
}

/// Supported hash algorithms.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum HashAlgorithm {
    #[default]
    Sha256,
    Sha512,
}
```

**Validation Rules**:
- If `keyed` is true, `salt` must be provided
- Salt should be at least 16 bytes for security

### PseudonymConfig

Configuration for pseudonymization operations.

```rust
/// Configuration for pseudonymization operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PseudonymConfig {
    /// Locale for generating fake data (e.g., "de_DE", "en_US")
    pub locale: String,
    /// Seed for deterministic pseudonymization (optional)
    pub seed: Option<u64>,
    /// Data type for generating appropriate fake data
    pub data_type: PseudonymDataType,
    /// Whether to maintain consistency (same input → same output)
    pub consistent: bool,
}

/// Type of data to generate for pseudonymization.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PseudonymDataType {
    #[default]
    Name,
    FirstName,
    LastName,
    Email,
    Phone,
    Address,
    City,
    PostalCode,
    Company,
    Generic,
}
```

**Validation Rules**:
- If `consistent` is true, `seed` should be provided or derived from input
- `locale` must be a valid locale code

### TokenConfig

Configuration for tokenization operations.

```rust
/// Configuration for tokenization operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenConfig {
    /// Token format to generate
    pub format: TokenFormat,
    /// Whether same input should produce same token
    pub consistent: bool,
    /// Prefix for generated tokens (optional)
    pub prefix: Option<String>,
}

/// Format for generated tokens.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenFormat {
    #[default]
    Uuid,
    Hex16,
    Hex32,
    Alphanumeric,
}
```

### CryptoResult

Result of a cryptographic protection operation.

```rust
/// Result of a cryptographic protection operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoResult {
    /// The protected value
    pub value: String,
    /// Protection mode used
    pub mode: ProtectionMode,
    /// Additional metadata
    pub metadata: CryptoMetadata,
}

/// Metadata about the cryptographic operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CryptoMetadata {
    /// Initialization vector (for encryption)
    pub iv: Option<String>,
    /// Authentication tag (for encryption)
    pub tag: Option<String>,
    /// Algorithm used
    pub algorithm: Option<String>,
    /// Salt used (for hashing)
    pub salt: Option<String>,
    /// Original data type (for pseudonymization)
    pub data_type: Option<String>,
}
```

### TokenVault Trait

Storage interface for token mappings.

```rust
/// Error type for vault operations.
#[derive(Debug, Error)]
pub enum VaultError {
    #[error("Storage error: {0}")]
    Storage(String),
    #[error("Token not found: {0}")]
    NotFound(String),
    #[error("Token already exists: {0}")]
    Duplicate(String),
}

/// Trait for token vault storage backends.
pub trait TokenVault: Send + Sync {
    /// Store a token-to-original mapping.
    fn store(&self, token: &str, original: &str) -> Result<(), VaultError>;

    /// Look up the original value for a token.
    fn lookup(&self, token: &str) -> Result<Option<String>, VaultError>;

    /// Delete a token mapping.
    fn delete(&self, token: &str) -> Result<bool, VaultError>;

    /// Check if original value already has a token.
    fn find_by_original(&self, original: &str) -> Result<Option<String>, VaultError>;
}
```

### InMemoryVault

Default in-memory implementation of TokenVault.

```rust
/// In-memory token vault for testing and simple use cases.
#[derive(Debug, Default)]
pub struct InMemoryVault {
    /// Token to original mapping
    tokens: RwLock<HashMap<String, String>>,
    /// Original to token mapping (for consistency)
    reverse: RwLock<HashMap<String, String>>,
}
```

### CryptoError

Error types for cryptographic operations.

```rust
/// Errors that can occur during cryptographic operations.
#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("Invalid key length: expected {expected}, got {actual}")]
    InvalidKeyLength { expected: usize, actual: usize },

    #[error("Invalid IV length: expected {expected}, got {actual}")]
    InvalidIvLength { expected: usize, actual: usize },

    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Decryption failed: authentication error")]
    DecryptionFailed,

    #[error("Invalid ciphertext format")]
    InvalidCiphertext,

    #[error("Hashing failed: {0}")]
    HashingFailed(String),

    #[error("Pseudonymization failed: {0}")]
    PseudonymFailed(String),

    #[error("Tokenization failed: {0}")]
    TokenFailed(String),

    #[error("Vault error: {0}")]
    Vault(#[from] VaultError),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}
```

## State Transitions

### Encryption Flow

```text
plaintext ──encrypt(key)──▶ CryptoResult {
                              value: base64(iv || ciphertext || tag),
                              metadata: { iv, tag, algorithm: "AES-256-GCM" }
                           }
                                    │
                                    ▼
CryptoResult ──decrypt(key)──▶ plaintext
```

### Tokenization Flow

```text
original ──tokenize()──▶ CryptoResult {
   │                        value: "tok_a1b2c3d4...",
   │                        metadata: {}
   │                     }
   │                           │
   └───store───▶ TokenVault ◀──┘
                     │
                     ▼
         token ──detokenize()──▶ original
```

## Module Organization

```text
crates/veil-crypto/
├── src/
│   ├── lib.rs           # Public API exports
│   ├── encrypt.rs       # AES-256-GCM encryption/decryption
│   ├── hash.rs          # SHA-256/SHA-512 hashing
│   ├── pseudonym.rs     # Fake data generation
│   ├── tokenize.rs      # Token generation and management
│   ├── vault/
│   │   ├── mod.rs       # TokenVault trait
│   │   └── memory.rs    # InMemoryVault implementation
│   ├── types.rs         # Shared types and configs
│   ├── error.rs         # CryptoError type
│   └── protector.rs     # High-level Protector interface
└── tests/
    ├── encrypt_tests.rs
    ├── hash_tests.rs
    ├── pseudonym_tests.rs
    └── tokenize_tests.rs
```
