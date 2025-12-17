//! Protect request and response models.

use serde::{Deserialize, Serialize};

use super::scan::{Finding, ScanStats};

/// Protection method/style.
/// Compatible with WASM's ProtectStyle.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProtectionMethod {
    /// Replace with [CATEGORY] labels (e.g., [EMAIL]) - matches WASM Labels.
    #[default]
    Labels,
    /// Replace with block characters.
    Redact,
    /// Replace with partial masking (e.g., j***@***.com).
    Mask,
    /// Replace with pseudonymous values.
    Pseudonymize,
    /// Replace with tokens (requires vault).
    Tokenize,
    /// Encrypt the value.
    Encrypt,
    /// Hash the value.
    Hash,
}

/// Category-specific protection rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProtectionRule {
    /// PII category to protect.
    pub category: String,

    /// Protection method to apply.
    #[serde(default)]
    pub method: ProtectionMethod,

    /// Custom replacement text (for redact method).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replacement: Option<String>,
}

/// Protection options.
/// Compatible with WASM's ProtectOptions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProtectOptions {
    /// Original filename (used for format detection).
    #[serde(default)]
    pub filename: Option<String>,

    /// Protection style (matches WASM's style field).
    #[serde(default)]
    pub style: ProtectionMethod,

    /// Category-specific rules.
    #[serde(default)]
    pub rules: Vec<ProtectionRule>,

    /// Categories to protect (empty = all detected).
    #[serde(default)]
    pub categories: Vec<String>,

    /// Minimum confidence threshold (0.0-1.0).
    #[serde(default)]
    pub min_confidence: Option<f64>,

    /// Whether to include findings in response.
    #[serde(default = "default_include_findings")]
    pub include_findings: bool,

    /// Include matched PII values in findings (default: false for security).
    /// Requires X-Acknowledge-PII-Exposure header when true.
    #[serde(default)]
    pub include_values: bool,
}

fn default_include_findings() -> bool {
    true
}

/// Protect response (JSON metadata).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProtectResponse {
    /// Request ID for tracing.
    pub request_id: String,

    /// Whether protection was successful.
    pub success: bool,

    /// Original file format.
    pub format: String,

    /// Number of values protected (matches WASM's replacements).
    pub replacements: usize,

    /// Categories that were protected.
    #[serde(default)]
    pub protected_categories: Vec<String>,

    /// Findings that were protected.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub findings: Vec<Finding>,

    /// Processing statistics.
    pub stats: ScanStats,

    /// Warnings during processing.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,

    /// Download URL for protected file (if async).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download_url: Option<String>,
}

impl ProtectResponse {
    /// Create a successful protect response.
    pub fn success(
        request_id: String,
        format: String,
        replacements: usize,
        protected_categories: Vec<String>,
        findings: Vec<Finding>,
        stats: ScanStats,
    ) -> Self {
        Self {
            request_id,
            success: true,
            format,
            replacements,
            protected_categories,
            findings,
            stats,
            warnings: Vec::new(),
            download_url: None,
        }
    }
}
