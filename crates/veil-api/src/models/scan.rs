//! Scan request and response models.

use serde::{Deserialize, Serialize};

/// Scan request options (from query params or JSON body).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanOptions {
    /// Original filename (used for format detection).
    #[serde(default)]
    pub filename: Option<String>,

    /// Categories to detect (empty = all).
    #[serde(default)]
    pub categories: Vec<String>,

    /// Minimum confidence threshold (0.0-1.0).
    #[serde(default)]
    pub min_confidence: Option<f64>,

    /// Include raw text context around findings.
    #[serde(default)]
    pub include_context: bool,

    /// Context characters before/after finding.
    #[serde(default = "default_context_chars")]
    pub context_chars: usize,

    /// Include matched PII values in response (default: false for security).
    /// Requires X-Acknowledge-PII-Exposure header when true.
    #[serde(default)]
    pub include_values: bool,
}

fn default_context_chars() -> usize {
    50
}

/// A single PII finding.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Finding {
    /// PII category (email, phone, credit_card, etc.).
    pub category: String,

    /// The matched text (only included when include_values=true with acknowledgment).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,

    /// Start position in the document.
    pub start: usize,

    /// End position in the document.
    pub end: usize,

    /// Confidence score (0.0-1.0).
    pub confidence: f64,

    /// Text context around the finding.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,

    /// Position details (format-specific).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<PositionInfo>,
}

/// Format-specific position information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PositionInfo {
    /// Text file position.
    Text { line: usize, column: usize },
    /// CSV cell position.
    Csv {
        row: usize,
        column: usize,
        header: Option<String>,
    },
    /// JSON path position.
    Json { path: String },
    /// PDF position.
    Pdf { page: usize },
    /// Email field position.
    Email { field: String },
}

/// Scan statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanStats {
    /// Total bytes processed.
    pub bytes_processed: usize,

    /// Total findings count.
    pub total_findings: usize,

    /// Findings by category (matches WASM's category_counts).
    pub category_counts: std::collections::HashMap<String, usize>,

    /// Processing time in milliseconds.
    pub duration_ms: u64,
}

/// Scan response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanResponse {
    /// Request ID for tracing.
    pub request_id: String,

    /// Whether the scan was successful.
    pub success: bool,

    /// Detected file format.
    pub format: String,

    /// List of findings.
    pub findings: Vec<Finding>,

    /// Scan statistics.
    pub stats: ScanStats,

    /// Warnings during processing.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
}

impl ScanStats {
    /// Create new scan stats.
    pub fn new(bytes_processed: usize, duration_ms: u64) -> Self {
        Self {
            bytes_processed,
            total_findings: 0,
            category_counts: std::collections::HashMap::new(),
            duration_ms,
        }
    }

    /// Add a finding to the stats.
    pub fn add_finding(&mut self, category: &str) {
        self.total_findings += 1;
        *self
            .category_counts
            .entry(category.to_string())
            .or_insert(0) += 1;
    }
}

impl ScanResponse {
    /// Create a successful scan response.
    pub fn success(
        request_id: String,
        format: String,
        findings: Vec<Finding>,
        stats: ScanStats,
    ) -> Self {
        Self {
            request_id,
            success: true,
            format,
            findings,
            stats,
            warnings: Vec::new(),
        }
    }
}
