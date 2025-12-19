//! Applied redaction tracking.
//!
//! # Security Design
//!
//! This module intentionally does NOT store the original PII text in the
//! `AppliedRedaction` struct to prevent data leakage. Instead, it stores:
//! - A one-way hash of the original for verification purposes
//! - The original length for auditing
//!
//! If you need the original text, you must explicitly enable it via
//! `AppliedRedactionBuilder::with_original()` and acknowledge the security
//! implications.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use veil_detect::PiiCategory;

/// Record of an applied redaction.
///
/// # Security
///
/// By default, this struct does NOT store the original PII text to prevent
/// accidental data leakage. The `original_hash` field contains a SHA-256 hash
/// for verification purposes only.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppliedRedaction {
    /// SHA-256 hash of the original text (for verification, not reconstruction).
    /// Format: hex-encoded hash.
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
    /// Create a new applied redaction record.
    ///
    /// The original text is hashed (not stored) to prevent data leakage.
    pub fn new(
        original: impl AsRef<str>,
        replacement: impl Into<String>,
        original_position: (usize, usize),
        new_position: (usize, usize),
        category: PiiCategory,
    ) -> Self {
        let original_str = original.as_ref();
        let hash = compute_hash(original_str);

        Self {
            original_hash: hash,
            original_length: original_str.len(),
            replacement: replacement.into(),
            original_position,
            new_position,
            category,
        }
    }

    /// Verify if a given text matches the original that was redacted.
    ///
    /// This allows verification without storing the original PII.
    pub fn verify_original(&self, text: &str) -> bool {
        if text.len() != self.original_length {
            return false;
        }
        compute_hash(text) == self.original_hash
    }
}

/// Compute SHA-256 hash of text, returning hex-encoded string.
fn compute_hash(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}
