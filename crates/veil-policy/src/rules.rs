//! Policy rules for detection and protection.

use serde::{Deserialize, Serialize};
use veil_detect::PiiCategory;

use crate::keyref::KeyRef;

/// A rule for filtering detection results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionRule {
    /// PII types this rule applies to.
    pub types: Vec<PiiCategory>,

    /// Minimum confidence threshold (0.0 - 1.0).
    #[serde(default = "default_confidence")]
    pub confidence_threshold: f32,

    /// Whether this rule is enabled.
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_confidence() -> f32 {
    0.5
}

fn default_enabled() -> bool {
    true
}

impl Default for DetectionRule {
    fn default() -> Self {
        Self {
            types: Vec::new(),
            confidence_threshold: default_confidence(),
            enabled: true,
        }
    }
}

/// Action to take for protection.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtectionAction {
    /// Redact with label, bars, or mask.
    #[default]
    Redact,
    /// Apply partial masking.
    Mask,
    /// Hash the value.
    Hash,
    /// Replace with consistent pseudonym.
    Pseudonymize,
    /// Encrypt the value.
    Encrypt,
    /// Replace with token.
    Tokenize,
}

/// A rule for applying protection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectionRule {
    /// PII types this rule applies to.
    pub types: Vec<PiiCategory>,

    /// Protection action to apply.
    #[serde(default)]
    pub action: ProtectionAction,

    /// Style for redaction (label, bar, mask).
    #[serde(default)]
    pub style: Option<String>,

    /// Whether to use consistent replacement (for pseudonymization).
    #[serde(default)]
    pub consistent: bool,

    /// Key reference for encryption (env:// or file://).
    #[serde(default)]
    pub key_ref: Option<KeyRef>,
}

impl Default for ProtectionRule {
    fn default() -> Self {
        Self {
            types: Vec::new(),
            action: ProtectionAction::Redact,
            style: Some("label".to_string()),
            consistent: false,
            key_ref: None,
        }
    }
}
