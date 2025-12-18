//! Data types for WASM API.
//!
//! These types are serialized to/from JavaScript via serde-wasm-bindgen.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single PII detection finding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// PII category (e.g., "email", "phone", "iban", "credit_card")
    pub category: String,

    /// The matched text value (only included when includeValues=true with acknowledgment)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,

    /// Byte offset of match start in source
    pub start: usize,

    /// Byte offset of match end in source (exclusive)
    pub end: usize,

    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
}

impl Finding {
    /// Create a new finding.
    pub fn new(
        category: impl Into<String>,
        value: Option<String>,
        start: usize,
        end: usize,
        confidence: f64,
    ) -> Self {
        Self {
            category: category.into(),
            value,
            start,
            end,
            confidence,
        }
    }
}

/// Configuration options for scan operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanOptions {
    /// Original filename (used for format detection)
    #[serde(default)]
    pub filename: Option<String>,

    /// Categories to detect (empty = all categories)
    #[serde(default)]
    pub categories: Vec<String>,

    /// Minimum confidence threshold (default: 0.0)
    #[serde(default)]
    pub min_confidence: Option<f64>,

    /// Include matched PII values in response (default: false for security)
    #[serde(default)]
    pub include_values: bool,

    /// Acknowledge that PII values will be exposed (required when include_values=true)
    #[serde(default)]
    pub acknowledge_exposure: bool,
}

/// Statistics about a scan operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanStats {
    /// Total bytes processed
    pub bytes_processed: usize,

    /// Processing time in milliseconds
    pub duration_ms: u64,

    /// Number of findings by category
    pub category_counts: HashMap<String, usize>,
}

impl ScanStats {
    /// Create new stats.
    pub fn new(bytes_processed: usize, duration_ms: u64) -> Self {
        Self {
            bytes_processed,
            duration_ms,
            category_counts: HashMap::new(),
        }
    }

    /// Add a category count.
    pub fn add_category(&mut self, category: &str) {
        *self
            .category_counts
            .entry(category.to_string())
            .or_insert(0) += 1;
    }
}

/// Result of a scan operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    /// All findings from the scan
    pub findings: Vec<Finding>,

    /// Processing statistics
    pub stats: ScanStats,
}

impl ScanResult {
    /// Create a new scan result.
    pub fn new(findings: Vec<Finding>, stats: ScanStats) -> Self {
        Self { findings, stats }
    }
}

/// Protection/redaction style.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProtectStyle {
    /// Replace with [CATEGORY] labels (e.g., [EMAIL])
    #[default]
    Labels,

    /// Replace with block characters
    Redact,

    /// Replace with partial masking (e.g., j***@***.com)
    Mask,
}

/// Configuration options for protect operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProtectOptions {
    /// Original filename (used for format detection)
    #[serde(default)]
    pub filename: Option<String>,

    /// Protection style
    #[serde(default)]
    pub style: ProtectStyle,

    /// Categories to protect (empty = all found categories)
    #[serde(default)]
    pub categories: Vec<String>,
}

impl Default for ProtectOptions {
    fn default() -> Self {
        Self {
            filename: None,
            style: ProtectStyle::Labels,
            categories: Vec::new(),
        }
    }
}

/// Statistics about a protect operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProtectStats {
    /// Number of replacements made
    pub replacements: usize,

    /// Categories that were protected
    pub protected_categories: Vec<String>,

    /// Processing time in milliseconds
    pub duration_ms: u64,
}

impl ProtectStats {
    /// Create new stats.
    pub fn new(replacements: usize, protected_categories: Vec<String>, duration_ms: u64) -> Self {
        Self {
            replacements,
            protected_categories,
            duration_ms,
        }
    }
}

/// Result of a protect operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectResult {
    /// Protected content as bytes
    #[serde(with = "serde_bytes")]
    pub data: Vec<u8>,

    /// Statistics about the protection
    pub stats: ProtectStats,
}

impl ProtectResult {
    /// Create a new protect result.
    pub fn new(data: Vec<u8>, stats: ProtectStats) -> Self {
        Self { data, stats }
    }
}

mod serde_bytes {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(data: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize as array of numbers for JS Uint8Array compatibility
        data.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Vec::<u8>::deserialize(deserializer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finding_new() {
        let finding = Finding::new("email", Some("test@example.com".to_string()), 10, 26, 0.95);
        assert_eq!(finding.category, "email");
        assert_eq!(finding.value, Some("test@example.com".to_string()));
        assert_eq!(finding.start, 10);
        assert_eq!(finding.end, 26);
        assert!((finding.confidence - 0.95).abs() < f64::EPSILON);
    }

    #[test]
    fn test_finding_new_without_value() {
        let finding = Finding::new("email", None, 10, 26, 0.95);
        assert_eq!(finding.category, "email");
        assert_eq!(finding.value, None);
        assert_eq!(finding.start, 10);
        assert_eq!(finding.end, 26);
    }

    #[test]
    fn test_scan_options_default() {
        let options = ScanOptions::default();
        assert!(options.filename.is_none());
        assert!(options.categories.is_empty());
        assert!(options.min_confidence.is_none());
    }

    #[test]
    fn test_scan_stats_new() {
        let stats = ScanStats::new(1024, 50);
        assert_eq!(stats.bytes_processed, 1024);
        assert_eq!(stats.duration_ms, 50);
        assert!(stats.category_counts.is_empty());
    }

    #[test]
    fn test_scan_stats_add_category() {
        let mut stats = ScanStats::new(1024, 50);
        stats.add_category("email");
        stats.add_category("email");
        stats.add_category("phone");

        assert_eq!(stats.category_counts.get("email"), Some(&2));
        assert_eq!(stats.category_counts.get("phone"), Some(&1));
    }

    #[test]
    fn test_scan_result_new() {
        let findings = vec![Finding::new(
            "email",
            Some("test@example.com".to_string()),
            0,
            16,
            0.9,
        )];
        let stats = ScanStats::new(100, 10);
        let result = ScanResult::new(findings.clone(), stats);

        assert_eq!(result.findings.len(), 1);
        assert_eq!(result.stats.bytes_processed, 100);
    }

    #[test]
    fn test_protect_style_default() {
        assert_eq!(ProtectStyle::default(), ProtectStyle::Labels);
    }

    #[test]
    fn test_protect_options_default() {
        let options = ProtectOptions::default();
        assert!(options.filename.is_none());
        assert_eq!(options.style, ProtectStyle::Labels);
        assert!(options.categories.is_empty());
    }

    #[test]
    fn test_protect_stats_new() {
        let stats = ProtectStats::new(5, vec!["email".to_string(), "phone".to_string()], 100);
        assert_eq!(stats.replacements, 5);
        assert_eq!(stats.protected_categories.len(), 2);
        assert_eq!(stats.duration_ms, 100);
    }

    #[test]
    fn test_protect_result_new() {
        let data = b"protected content".to_vec();
        let stats = ProtectStats::new(1, vec!["email".to_string()], 50);
        let result = ProtectResult::new(data.clone(), stats);

        assert_eq!(result.data, data);
        assert_eq!(result.stats.replacements, 1);
    }

    #[test]
    fn test_finding_serialization_with_value() {
        let finding = Finding::new("email", Some("test@example.com".to_string()), 0, 16, 0.9);
        let json = serde_json::to_string(&finding).unwrap();
        assert!(json.contains("\"category\":\"email\""));
        assert!(json.contains("\"value\":\"test@example.com\""));
    }

    #[test]
    fn test_finding_serialization_without_value() {
        let finding = Finding::new("email", None, 0, 16, 0.9);
        let json = serde_json::to_string(&finding).unwrap();
        assert!(json.contains("\"category\":\"email\""));
        // Value should be omitted entirely, not null
        assert!(!json.contains("\"value\""));
    }

    #[test]
    fn test_scan_options_serialization() {
        let options = ScanOptions {
            filename: Some("test.txt".to_string()),
            categories: vec!["email".to_string()],
            min_confidence: Some(0.8),
            ..Default::default()
        };
        let json = serde_json::to_string(&options).unwrap();
        // Check camelCase serialization
        assert!(json.contains("\"filename\""));
        assert!(json.contains("\"minConfidence\""));
    }

    #[test]
    fn test_scan_options_include_values() {
        let options = ScanOptions {
            include_values: true,
            acknowledge_exposure: true,
            ..Default::default()
        };
        assert!(options.include_values);
        assert!(options.acknowledge_exposure);
    }

    #[test]
    fn test_protect_style_serialization() {
        let labels = serde_json::to_string(&ProtectStyle::Labels).unwrap();
        let redact = serde_json::to_string(&ProtectStyle::Redact).unwrap();
        let mask = serde_json::to_string(&ProtectStyle::Mask).unwrap();

        assert_eq!(labels, "\"labels\"");
        assert_eq!(redact, "\"redact\"");
        assert_eq!(mask, "\"mask\"");
    }
}
