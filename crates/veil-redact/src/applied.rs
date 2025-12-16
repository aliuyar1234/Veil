//! Applied redaction tracking.

use serde::{Deserialize, Serialize};
use veil_detect::PiiCategory;

/// Record of an applied redaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppliedRedaction {
    /// Original text that was redacted.
    pub original: String,

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
    pub fn new(
        original: impl Into<String>,
        replacement: impl Into<String>,
        original_position: (usize, usize),
        new_position: (usize, usize),
        category: PiiCategory,
    ) -> Self {
        Self {
            original: original.into(),
            replacement: replacement.into(),
            original_position,
            new_position,
            category,
        }
    }
}
