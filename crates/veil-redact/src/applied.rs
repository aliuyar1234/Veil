//! Applied redaction tracking.
//!
//! # Security Design
//!
//! This module intentionally does NOT store the original PII text in the
//! `AppliedRedaction` struct to prevent data leakage. Instead, it stores:
//! - A one-way hash of the original for verification purposes
//! - The original length for auditing
//!
//! The hash is stored in one of the following formats:
//! - `<hex>` or `sha256:<hex>` (legacy / unkeyed SHA-256)
//! - `hmac-sha256:<hex>` (keyed HMAC-SHA256; provide a 32-byte key for
//!   tamper-resistant verification)

use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use veil_detect::PiiCategory;

const HASH_SHA256_PREFIX: &str = "sha256:";
const HASH_HMAC_SHA256_PREFIX: &str = "hmac-sha256:";
type HmacSha256 = Hmac<Sha256>;

/// Record of an applied redaction.
///
/// # Security
///
/// By default, this struct does NOT store the original PII text to prevent
/// accidental data leakage. The `original_hash` field contains a one-way hash
/// for verification purposes only.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppliedRedaction {
    /// One-way hash of the original text (for verification, not reconstruction).
    ///
    /// Format:
    /// - `<hex>` or `sha256:<hex>` (unkeyed SHA-256)
    /// - `hmac-sha256:<hex>` (keyed HMAC-SHA256)
    pub original_hash: String,

    /// Length of the original text in bytes.
    pub original_length: usize,

    /// Replacement text.
    pub replacement: String,

    /// Original position (start, end).
    pub original_position: (usize, usize),

    /// New position after redaction.
    pub new_position: (usize, usize),

    /// PII category.
    pub category: PiiCategory,
}

impl AppliedRedaction {
    /// Create a new applied redaction record using unkeyed SHA-256 hashing.
    ///
    /// The original text is hashed (not stored) to prevent data leakage.
    pub fn new(
        original: impl AsRef<str>,
        replacement: impl Into<String>,
        original_position: (usize, usize),
        new_position: (usize, usize),
        category: PiiCategory,
    ) -> Self {
        Self::new_with_optional_hmac_key(
            original,
            replacement,
            original_position,
            new_position,
            category,
            None,
        )
    }

    /// Create a new applied redaction record using HMAC-SHA256 hashing.
    pub fn new_with_hmac_key(
        original: impl AsRef<str>,
        replacement: impl Into<String>,
        original_position: (usize, usize),
        new_position: (usize, usize),
        category: PiiCategory,
        hmac_key: &[u8; 32],
    ) -> Self {
        Self::new_with_optional_hmac_key(
            original,
            replacement,
            original_position,
            new_position,
            category,
            Some(hmac_key),
        )
    }

    /// Verify if a given text matches the original that was redacted.
    ///
    /// This allows verification without storing the original PII.
    pub fn verify_original(&self, text: &str) -> bool {
        if text.len() != self.original_length {
            return false;
        }

        verify_stored_hash(text, &self.original_hash, None)
    }

    /// Verify if a given text matches the original that was redacted, using a provided HMAC key.
    pub fn verify_original_with_hmac_key(&self, text: &str, hmac_key: &[u8; 32]) -> bool {
        if text.len() != self.original_length {
            return false;
        }

        verify_stored_hash(text, &self.original_hash, Some(hmac_key))
    }

    fn new_with_optional_hmac_key(
        original: impl AsRef<str>,
        replacement: impl Into<String>,
        original_position: (usize, usize),
        new_position: (usize, usize),
        category: PiiCategory,
        hmac_key: Option<&[u8; 32]>,
    ) -> Self {
        let original_str = original.as_ref();
        let hash = compute_hash_for_storage(original_str, hmac_key);

        Self {
            original_hash: hash,
            original_length: original_str.len(),
            replacement: replacement.into(),
            original_position,
            new_position,
            category,
        }
    }
}

fn sha256_hex(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    hex::encode(hasher.finalize())
}

fn hmac_sha256_hex(text: &str, key: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC accepts keys of arbitrary size");
    mac.update(text.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

fn compute_hash_for_storage(text: &str, hmac_key: Option<&[u8; 32]>) -> String {
    if let Some(key) = hmac_key {
        format!("{}{}", HASH_HMAC_SHA256_PREFIX, hmac_sha256_hex(text, key))
    } else {
        format!("{}{}", HASH_SHA256_PREFIX, sha256_hex(text))
    }
}

fn verify_stored_hash(text: &str, stored_hash: &str, hmac_key: Option<&[u8; 32]>) -> bool {
    if let Some(expected) = stored_hash.strip_prefix(HASH_HMAC_SHA256_PREFIX) {
        let Some(key) = hmac_key else {
            return false;
        };

        hmac_sha256_hex(text, key) == expected
    } else if let Some(expected) = stored_hash.strip_prefix(HASH_SHA256_PREFIX) {
        sha256_hex(text) == expected
    } else {
        // Legacy, unprefixed hash.
        sha256_hex(text) == stored_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_original_works_without_storing_pii() {
        let redaction = AppliedRedaction::new(
            "secret@example.com",
            "[EMAIL]",
            (0, 18),
            (0, 7),
            PiiCategory::Email,
        );

        assert!(redaction.original_hash.starts_with(HASH_SHA256_PREFIX));
        assert!(redaction.verify_original("secret@example.com"));
        assert!(!redaction.verify_original("other@example.com"));
    }

    #[test]
    fn verify_original_uses_hmac_when_key_set() {
        let key = [0u8; 32];
        let redaction = AppliedRedaction::new_with_hmac_key(
            "secret@example.com",
            "[EMAIL]",
            (0, 18),
            (0, 7),
            PiiCategory::Email,
            &key,
        );

        assert!(redaction.original_hash.starts_with(HASH_HMAC_SHA256_PREFIX));
        assert!(!redaction.verify_original("secret@example.com"));
        assert!(redaction.verify_original_with_hmac_key("secret@example.com", &key));
    }

    #[test]
    fn verify_original_accepts_legacy_unprefixed_hashes() {
        let mut redaction = AppliedRedaction::new(
            "secret@example.com",
            "[EMAIL]",
            (0, 18),
            (0, 7),
            PiiCategory::Email,
        );
        redaction.original_hash = sha256_hex("secret@example.com");

        assert!(redaction.verify_original("secret@example.com"));
    }
}
