//! Policy validation types.

use serde::{Deserialize, Serialize};

/// Result of policy validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyValidationResult {
    /// Whether the policy is valid.
    pub valid: bool,

    /// Error messages.
    pub errors: Vec<String>,

    /// Warning messages.
    pub warnings: Vec<String>,
}

impl PolicyValidationResult {
    /// Create a valid result with no issues.
    pub fn valid() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Create an invalid result with an error.
    pub fn invalid(error: impl Into<String>) -> Self {
        Self {
            valid: false,
            errors: vec![error.into()],
            warnings: Vec::new(),
        }
    }
}
