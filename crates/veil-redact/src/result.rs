//! Redaction result type.

use serde::{Deserialize, Serialize};

use crate::applied::AppliedRedaction;
use crate::position::PositionMap;

/// Result of a redaction operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedactionResult {
    /// The redacted text.
    pub text: String,

    /// List of applied redactions.
    pub redactions: Vec<AppliedRedaction>,

    /// Position mapping for downstream use.
    pub position_map: PositionMap,
}

impl RedactionResult {
    /// Create a new redaction result.
    pub fn new(text: String, redactions: Vec<AppliedRedaction>, position_map: PositionMap) -> Self {
        Self {
            text,
            redactions,
            position_map,
        }
    }

    /// Get the number of redactions applied.
    pub fn redaction_count(&self) -> usize {
        self.redactions.len()
    }

    /// Check if any redactions were applied.
    pub fn has_redactions(&self) -> bool {
        !self.redactions.is_empty()
    }
}
