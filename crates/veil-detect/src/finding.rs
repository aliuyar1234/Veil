//! PII finding types.

use serde::{Deserialize, Serialize};
use veil_core::SensitiveString;

use crate::category::PiiCategory;

/// Validation status of a detected PII match.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationStatus {
    /// Pattern matched and validation passed (e.g., checksum verified).
    Valid,
    /// Pattern matched but validation failed.
    Invalid {
        /// Reason for validation failure.
        reason: String,
    },
    /// Pattern matched, no validation available.
    Unvalidated,
}

/// A detected PII instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// The matched text (securely zeroed on drop).
    pub matched_text: SensitiveString,

    /// PII category.
    pub category: PiiCategory,

    /// Start position in the segment's content (byte offset).
    pub start: usize,

    /// End position (exclusive) in the segment's content (byte offset).
    pub end: usize,

    /// Confidence score (0.0 - 1.0).
    pub confidence: f32,

    /// Validation status.
    pub validation: ValidationStatus,

    /// Index of the source segment (from ParseResult).
    pub segment_index: usize,

    /// Context reasoning (if context analysis was applied).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_reasoning: Option<Vec<String>>,
}

impl Finding {
    /// Create a new finding.
    pub fn new(
        matched_text: impl Into<String>,
        category: PiiCategory,
        start: usize,
        end: usize,
        confidence: f32,
        validation: ValidationStatus,
        segment_index: usize,
    ) -> Self {
        Self {
            matched_text: SensitiveString::new(matched_text.into()),
            category,
            start,
            end,
            confidence,
            validation,
            segment_index,
            context_reasoning: None,
        }
    }

    /// Set context reasoning for this finding.
    pub fn with_context_reasoning(mut self, reasoning: Vec<String>) -> Self {
        self.context_reasoning = Some(reasoning);
        self
    }

    /// Add context reasoning to this finding.
    pub fn add_context_reasoning(&mut self, reasoning: Vec<String>) {
        self.context_reasoning = Some(reasoning);
    }

    /// Check if the finding is valid (validation passed or unvalidated).
    pub fn is_valid(&self) -> bool {
        !matches!(self.validation, ValidationStatus::Invalid { .. })
    }
}
