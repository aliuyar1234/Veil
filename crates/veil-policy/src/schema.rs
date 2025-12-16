//! Policy schema definition.

use serde::{Deserialize, Serialize};

use crate::locale::Locale;
use crate::rules::{DetectionRule, ProtectionRule};

/// A complete policy definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    /// Policy format version (semver).
    pub version: String,

    /// Human-readable name.
    pub name: String,

    /// Optional locale for region-specific detection.
    #[serde(default)]
    pub locale: Option<Locale>,

    /// Rules for filtering detection results.
    #[serde(default)]
    pub detection: Vec<DetectionRule>,

    /// Rules for applying protection.
    #[serde(default)]
    pub protection: Vec<ProtectionRule>,
}

impl Default for Policy {
    fn default() -> Self {
        crate::defaults::default_policy()
    }
}

impl Policy {
    /// Get the policy name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the policy version.
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Check if the policy version is supported.
    pub fn is_version_supported(&self) -> bool {
        self.version.starts_with("1.")
    }
}
