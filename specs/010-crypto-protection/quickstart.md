# Quickstart: Cryptographic Protection

**Feature**: 010-crypto-protection
**Date**: 2025-12-15

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
veil-crypto = { path = "crates/veil-crypto" }
```

## Basic Usage

### Encryption (AES-256-GCM)

```rust
use veil_crypto::{encrypt, decrypt, EncryptionConfig, OutputFormat};

// Generate or load a 32-byte key
let key = [0u8; 32]; // Use proper key management in production!

let config = EncryptionConfig {
    key: key.to_vec(),
    output_format: OutputFormat::Base64,
};

// Encrypt sensitive data
let plaintext = "DE89370400440532013000"; // IBAN
let result = encrypt(plaintext.as_bytes(), &config)?;
println!("Encrypted: {}", result.value);

// Decrypt when needed
let decrypted = decrypt(&result, &config)?;
assert_eq!(decrypted, plaintext.as_bytes());
```

### Hashing (SHA-256/SHA-512)

```rust
use veil_crypto::{hash, HashConfig, HashAlgorithm, OutputFormat};

// Basic hashing with random salt
let config = HashConfig {
    algorithm: HashAlgorithm::Sha256,
    salt: None, // Auto-generated
    output_format: OutputFormat::Hex,
    keyed: false,
};

let email = "user@example.com";
let result = hash(email.as_bytes(), &config)?;
println!("Hash: {}", result.value);

// Deterministic hashing with fixed salt
let config_deterministic = HashConfig {
    algorithm: HashAlgorithm::Sha256,
    salt: Some(b"my-fixed-salt".to_vec()),
    output_format: OutputFormat::Hex,
    keyed: false,
};

let hash1 = hash(email.as_bytes(), &config_deterministic)?;
let hash2 = hash(email.as_bytes(), &config_deterministic)?;
assert_eq!(hash1.value, hash2.value); // Same input + salt = same hash
```

### Pseudonymization

```rust
use veil_crypto::{pseudonymize, PseudonymConfig, PseudonymDataType};

// Random pseudonymization
let config = PseudonymConfig {
    locale: "de_DE".to_string(),
    seed: None,
    data_type: PseudonymDataType::Name,
    consistent: false,
};

let name = "Max Müller";
let result = pseudonymize(name, &config)?;
println!("Pseudonym: {}", result.value); // e.g., "Thomas Schmidt"

// Consistent pseudonymization (same input → same output)
let config_consistent = PseudonymConfig {
    locale: "de_DE".to_string(),
    seed: Some(12345),
    data_type: PseudonymDataType::Name,
    consistent: true,
};

let pseudo1 = pseudonymize(name, &config_consistent)?;
let pseudo2 = pseudonymize(name, &config_consistent)?;
assert_eq!(pseudo1.value, pseudo2.value);
```

### Tokenization

```rust
use veil_crypto::{tokenize, detokenize, TokenConfig, TokenFormat, InMemoryVault};
use std::sync::Arc;

// Create a vault for storing token mappings
let vault = Arc::new(InMemoryVault::default());

let config = TokenConfig {
    format: TokenFormat::Uuid,
    consistent: true,
    prefix: Some("tok_".to_string()),
};

// Tokenize a credit card number
let cc_number = "4532015112830366";
let result = tokenize(cc_number, &config, vault.clone())?;
println!("Token: {}", result.value); // e.g., "tok_a1b2c3d4-e5f6-..."

// Later, retrieve the original
let original = detokenize(&result.value, vault.clone())?;
assert_eq!(original, cc_number);
```

## High-Level Protector API

For convenient protection with automatic mode selection:

```rust
use veil_crypto::{Protector, ProtectionMode, CryptoConfig};

let config = CryptoConfig::builder()
    .encryption_key(&key)
    .hash_algorithm(HashAlgorithm::Sha256)
    .pseudonym_locale("de_DE")
    .build();

let protector = Protector::new(config);

// Protect different types of data
let encrypted_iban = protector.protect("DE89370400440532013000", ProtectionMode::Encrypt)?;
let hashed_email = protector.protect("user@example.com", ProtectionMode::Hash)?;
let fake_name = protector.protect("Max Müller", ProtectionMode::Pseudonymize)?;
let tokenized_cc = protector.protect("4532015112830366", ProtectionMode::Tokenize)?;
```

## Integration with Detection Results

Combine with `veil-detect` for automated protection:

```rust
use veil_detect::DetectorRegistry;
use veil_crypto::{Protector, ProtectionMode};
use veil_parsers::parse_bytes;

// Parse and detect PII
let content = b"Contact: user@example.com, IBAN: DE89370400440532013000";
let parsed = parse_bytes(content, &Default::default())?;
let registry = DetectorRegistry::default();
let findings = registry.detect_all(&parsed.segments);

// Protect each finding based on category
let protector = Protector::new(config);
for finding in findings {
    let mode = match finding.category.as_str() {
        "email" => ProtectionMode::Hash,
        "iban" => ProtectionMode::Encrypt,
        _ => ProtectionMode::Pseudonymize,
    };
    let protected = protector.protect(&finding.matched_text, mode)?;
    println!("{}: {} → {}", finding.category, finding.matched_text, protected.value);
}
```

## Error Handling

All operations return `Result<CryptoResult, CryptoError>`:

```rust
use veil_crypto::{encrypt, EncryptionConfig, CryptoError};

let result = encrypt(data, &config);
match result {
    Ok(encrypted) => println!("Success: {}", encrypted.value),
    Err(CryptoError::InvalidKeyLength { expected, actual }) => {
        eprintln!("Key must be {} bytes, got {}", expected, actual);
    }
    Err(CryptoError::DecryptionFailed) => {
        eprintln!("Decryption failed - wrong key or tampered data");
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

## Security Notes

1. **Key Management**: Never hardcode keys. Use environment variables, HSM, or key management services.
2. **Salt Storage**: When using deterministic hashing, store salts securely but separately from hashes.
3. **Vault Security**: In production, use a secure vault backend (Redis with TLS, encrypted database).
4. **Logging**: The library never logs plaintext values. Ensure your application follows the same practice.
