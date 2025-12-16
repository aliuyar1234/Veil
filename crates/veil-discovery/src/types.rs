//! Core types for PII discovery.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use veil_detect::PiiCategory;

/// Options for configuring a discovery scan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryOptions {
    /// Root directory to scan
    pub root_path: PathBuf,

    /// Maximum file size to scan (in bytes). Files larger than this will be sampled.
    pub max_file_size: Option<usize>,

    /// Number of bytes to sample from large files
    pub sample_size: usize,

    /// Include patterns (glob patterns for files to include)
    pub include_patterns: Vec<String>,

    /// Exclude patterns (glob patterns for files to exclude)
    pub exclude_patterns: Vec<String>,

    /// Follow symbolic links
    pub follow_symlinks: bool,

    /// Maximum depth to traverse (None = unlimited)
    pub max_depth: Option<usize>,

    /// Report progress updates
    pub report_progress: bool,

    /// Include file content snippets in results (for debugging)
    pub include_snippets: bool,
}

impl Default for DiscoveryOptions {
    fn default() -> Self {
        Self {
            root_path: PathBuf::from("."),
            max_file_size: Some(10 * 1024 * 1024), // 10MB
            sample_size: 100 * 1024,               // 100KB
            include_patterns: vec!["**/*".to_string()],
            exclude_patterns: vec![
                "**/node_modules/**".to_string(),
                "**/.git/**".to_string(),
                "**/target/**".to_string(),
                "**/.venv/**".to_string(),
                "**/build/**".to_string(),
                "**/dist/**".to_string(),
            ],
            follow_symlinks: false,
            max_depth: None,
            report_progress: true,
            include_snippets: false,
        }
    }
}

/// Source type for discovered data.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DataSource {
    /// Local filesystem
    Filesystem,
    // Future: Database, S3, Azure, etc.
}

/// A file discovered to contain PII.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredFile {
    /// Absolute path to the file
    pub path: PathBuf,

    /// File size in bytes
    pub size: u64,

    /// Whether the file was fully scanned or sampled
    pub sampled: bool,

    /// Bytes scanned (may be less than size if sampled)
    pub bytes_scanned: usize,

    /// PII categories found in this file
    pub pii_categories: Vec<PiiCategory>,

    /// Specific PII locations in the file
    pub locations: Vec<PIILocation>,

    /// File format detected
    pub format: Option<String>,

    /// Number of PII findings
    pub finding_count: usize,
}

/// A specific location where PII was found.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PIILocation {
    /// PII category
    pub category: PiiCategory,

    /// Line number (1-indexed) or None if not applicable
    pub line: Option<usize>,

    /// Column number (1-indexed) or None if not applicable
    pub column: Option<usize>,

    /// Byte offset in the file
    pub byte_offset: usize,

    /// Length of the PII in bytes
    pub byte_length: usize,

    /// Content snippet (if include_snippets is enabled)
    pub snippet: Option<String>,

    /// Validation status
    pub validated: bool,
}

/// Result of a discovery scan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryResult {
    /// When the scan was performed
    pub timestamp: DateTime<Utc>,

    /// Source type scanned
    pub source: DataSource,

    /// Root path that was scanned
    pub root_path: PathBuf,

    /// Files containing PII
    pub discovered_files: Vec<DiscoveredFile>,

    /// Total files scanned
    pub total_files_scanned: usize,

    /// Total files skipped (due to size, permissions, etc.)
    pub total_files_skipped: usize,

    /// Total bytes scanned
    pub total_bytes_scanned: u64,

    /// Scan statistics
    pub statistics: ScanStatistics,

    /// Errors encountered during scan
    pub errors: Vec<ScanError>,
}

/// Statistics about a scan.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScanStatistics {
    /// Count of findings by PII category
    pub findings_by_category: HashMap<PiiCategory, usize>,

    /// Count of files by PII category
    pub files_by_category: HashMap<PiiCategory, usize>,

    /// Total findings across all files
    pub total_findings: usize,

    /// Total files with PII
    pub total_files_with_pii: usize,

    /// Scan duration in seconds
    pub scan_duration_secs: f64,
}

/// An error encountered during scanning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanError {
    /// Path where the error occurred
    pub path: PathBuf,

    /// Error message
    pub message: String,

    /// Error kind
    pub kind: ErrorKind,
}

/// Kind of scan error.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    /// Permission denied
    PermissionDenied,

    /// File not found
    NotFound,

    /// Parse error
    ParseError,

    /// I/O error
    IoError,

    /// Other error
    Other,
}

/// Progress information during a scan.
#[derive(Debug, Clone)]
pub struct ScanProgress {
    /// Files scanned so far
    pub files_scanned: usize,

    /// Files with PII found so far
    pub files_with_pii: usize,

    /// Bytes scanned so far
    pub bytes_scanned: u64,

    /// Current file being processed
    pub current_file: Option<PathBuf>,
}
