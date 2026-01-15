# Data Model: Audit Trail & Reporting

**Feature**: 011-audit-reporting
**Date**: 2025-12-15

## Entity Relationship Overview

```text
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  AuditLogger    │────▶│   AuditEntry    │────▶│  AuditOutcome   │
│─────────────────│     │─────────────────│     │─────────────────│
│ log_dir         │     │ id              │     │ success         │
│ last_checksum   │     │ timestamp       │     │ error           │
│ log()           │     │ operation       │     │ findings        │
│ query()         │     │ parameters      │     │ redactions      │
│ generate_*()    │     │ outcome         │     └─────────────────┘
└────────┬────────┘     │ checksum        │
         │              │ previous_hash   │
         │              └─────────────────┘
         │
         ▼
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│ InventoryReport │     │ComplianceReport │     │  DsarResponse   │
│─────────────────│     │─────────────────│     │─────────────────│
│ generated_at    │     │ framework       │     │ request         │
│ total_files     │     │ generated_at    │     │ generated_at    │
│ total_findings  │     │ requirements[]  │     │ files_found[]   │
│ by_file         │     │ overall_status  │     │ excerpts[]      │
│ by_category     │     │ gaps[]          │     └─────────────────┘
│ to_json()       │     │ to_json()       │
│ to_csv()        │     │ to_text()       │
│ to_text()       │     └─────────────────┘
└─────────────────┘
```

## Existing Types (in veil-audit)

These types already exist and work well:

### AuditEntry (entry.rs)

```rust
/// A single audit log entry (EXISTS).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Unique identifier.
    pub id: Uuid,
    /// When the operation occurred.
    pub timestamp: DateTime<Utc>,
    /// Type of operation.
    pub operation: AuditOperation,
    /// Operation-specific parameters.
    pub parameters: AuditParameters,
    /// Operation outcome.
    pub outcome: AuditOutcome,
    /// Checksum for tamper detection.
    pub checksum: String,
    /// Previous entry's checksum (hash chain).
    pub previous_checksum: Option<String>,
}
```

### AuditParameters (entry.rs)

```rust
/// Parameters for an audit operation (EXISTS).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditParameters {
    /// Input file path(s).
    pub input: Vec<PathBuf>,
    /// Output file path (for protect).
    pub output: Option<PathBuf>,
    /// Policy used.
    pub policy: Option<String>,
    /// Additional metadata.
    pub metadata: HashMap<String, String>,
}
```

### AuditOutcome (entry.rs)

```rust
/// Outcome of an audit operation (EXISTS).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditOutcome {
    /// Whether operation succeeded.
    pub success: bool,
    /// Error message if failed.
    pub error: Option<String>,
    /// Findings summary (for scan).
    pub findings: Option<FindingsSummary>,
    /// Redactions summary (for protect).
    pub redactions: Option<RedactionsSummary>,
}
```

### AuditOperation (operation.rs)

```rust
/// Types of auditable operations (EXISTS - EXTEND).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditOperation {
    Scan,
    Protect,
    PolicyValidate,
    ReportGenerate, // Already includes report generation
}
```

### AuditLogger (logger.rs)

```rust
/// Audit logger for recording PII operations (EXISTS - EXTEND).
pub struct AuditLogger {
    log_dir: PathBuf,
    last_checksum: Option<String>,
}
```

### AuditFilter (logger.rs)

```rust
/// Filter for querying audit logs (EXISTS).
#[derive(Debug, Clone, Default)]
pub struct AuditFilter {
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub operations: Option<Vec<AuditOperation>>,
    pub paths: Option<Vec<PathBuf>>,
}
```

## New Types for Reporting

### InventoryReport

Data inventory report showing PII distribution.

```rust
/// Report of PII inventory across scanned files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryReport {
    /// When the report was generated.
    pub generated_at: DateTime<Utc>,
    /// Date range covered by the report.
    pub date_range: (DateTime<Utc>, DateTime<Utc>),
    /// Total unique files scanned.
    pub total_files: usize,
    /// Total PII findings across all files.
    pub total_findings: usize,
    /// Breakdown by file.
    pub by_file: HashMap<PathBuf, FileSummary>,
    /// Breakdown by PII category.
    pub by_category: HashMap<String, CategorySummary>,
}

/// Summary for a single file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSummary {
    /// File path.
    pub path: PathBuf,
    /// Number of findings in this file.
    pub findings_count: usize,
    /// Breakdown by category.
    pub by_category: HashMap<String, usize>,
    /// Last scanned timestamp.
    pub last_scanned: DateTime<Utc>,
}

/// Summary for a PII category.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategorySummary {
    /// PII category name.
    pub category: String,
    /// Total occurrences across all files.
    pub total_count: usize,
    /// Files containing this category.
    pub files: Vec<PathBuf>,
}

impl InventoryReport {
    /// Export as JSON.
    pub fn to_json(&self) -> Result<String, AuditError>;

    /// Export as CSV (file-level summary).
    pub fn to_csv(&self) -> Result<String, AuditError>;

    /// Format as human-readable text.
    pub fn to_text(&self) -> String;
}
```

**Validation Rules**:
- `total_files` must equal `by_file.len()`
- `total_findings` must equal sum of all `FileSummary.findings_count`
- `date_range.0` must be less than or equal to `date_range.1`

### ComplianceReport

Compliance assessment against regulatory frameworks.

```rust
/// Compliance report for a regulatory framework.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    /// Framework being assessed.
    pub framework: ComplianceFramework,
    /// When the report was generated.
    pub generated_at: DateTime<Utc>,
    /// Date range covered.
    pub date_range: (DateTime<Utc>, DateTime<Utc>),
    /// Individual compliance requirements.
    pub requirements: Vec<ComplianceRequirement>,
    /// Overall compliance status.
    pub overall_status: ComplianceStatus,
    /// Summary of gaps.
    pub gaps: Vec<ComplianceGap>,
}

/// Regulatory compliance framework.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ComplianceFramework {
    /// EU General Data Protection Regulation.
    Gdpr,
    // Future: Ccpa, Hipaa, etc.
}

/// A single compliance requirement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceRequirement {
    /// Article/section reference.
    pub article: String,
    /// Human-readable requirement.
    pub description: String,
    /// Applicable PII categories.
    pub categories: Vec<String>,
    /// Compliance status for this requirement.
    pub status: ComplianceStatus,
}

/// Compliance status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ComplianceStatus {
    /// Fully compliant.
    Compliant,
    /// Partially compliant (some gaps).
    Partial,
    /// Not compliant.
    NonCompliant,
    /// Not applicable (no data found).
    NotApplicable,
}

/// A compliance gap.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceGap {
    /// Requirement that is not met.
    pub requirement: String,
    /// PII category affected.
    pub category: String,
    /// Files with unprotected PII.
    pub affected_files: Vec<PathBuf>,
    /// Recommended action.
    pub recommendation: String,
}

impl ComplianceReport {
    /// Export as JSON.
    pub fn to_json(&self) -> Result<String, AuditError>;

    /// Format as human-readable text.
    pub fn to_text(&self) -> String;
}
```

**Validation Rules**:
- `overall_status` is Compliant only if all `requirements` are Compliant
- `overall_status` is NonCompliant if any requirement is NonCompliant
- `gaps` should contain entries only for Partial or NonCompliant requirements

### DsarRequest & DsarResponse

Data Subject Access Request support.

```rust
/// A data subject access request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DsarRequest {
    /// The identifier being searched for.
    pub identifier: String,
    /// Type of identifier.
    pub identifier_type: IdentifierType,
    /// Optional date range to search.
    pub date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
}

/// Type of identifier in a DSAR.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IdentifierType {
    /// Email address.
    Email,
    /// Person's name.
    Name,
    /// Phone number.
    Phone,
    /// Custom regex pattern.
    Custom,
}

/// Response to a data subject access request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DsarResponse {
    /// Original request.
    pub request: DsarRequest,
    /// When the search was performed.
    pub generated_at: DateTime<Utc>,
    /// Files containing the identifier.
    pub files_found: Vec<PathBuf>,
    /// Total number of matches.
    pub total_matches: usize,
    /// Excerpts showing context.
    pub excerpts: Vec<DsarExcerpt>,
}

/// A text excerpt from a DSAR search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DsarExcerpt {
    /// File containing the match.
    pub file_path: PathBuf,
    /// Audit entry ID.
    pub operation_id: Uuid,
    /// When the file was scanned.
    pub timestamp: DateTime<Utc>,
    /// Text context around the match.
    pub context: String,
    /// Position of match within context.
    pub match_offset: usize,
}

impl DsarResponse {
    /// Export as JSON.
    pub fn to_json(&self) -> Result<String, AuditError>;

    /// Format as human-readable text.
    pub fn to_text(&self) -> String;
}
```

**Validation Rules**:
- `total_matches` must equal `excerpts.len()`
- `files_found` should be deduplicated
- `context` should have reasonable length (e.g., 200 chars max)

### RetentionPolicy

Configuration for log retention.

```rust
/// Policy for audit log retention.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RetentionPolicy {
    /// Number of days to retain logs.
    pub duration_days: u32,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            duration_days: 365 * 7, // 7 years (GDPR default)
        }
    }
}

impl RetentionPolicy {
    /// Create a policy with custom duration.
    pub fn new(duration_days: u32) -> Self {
        Self { duration_days }
    }

    /// Check if a date is within retention period.
    pub fn is_retained(&self, date: NaiveDate) -> bool {
        let cutoff = Utc::now().date_naive() - Duration::days(self.duration_days as i64);
        date >= cutoff
    }
}
```

### ReportFormat

Output format for reports.

```rust
/// Output format for reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportFormat {
    /// JSON format.
    Json,
    /// CSV format (for inventory reports).
    Csv,
    /// Human-readable text.
    Text,
}
```

## Extended AuditLogger Methods

```rust
impl AuditLogger {
    // Existing methods (from logger.rs)
    pub fn new(log_dir: impl Into<PathBuf>) -> Result<Self, AuditError>;
    pub fn log(&mut self, entry: AuditEntry) -> Result<(), AuditError>;
    pub fn query(&self, filter: &AuditFilter) -> Result<Vec<AuditEntry>, AuditError>;

    // NEW: Report generation methods

    /// Generate a data inventory report.
    pub fn generate_inventory(
        &self,
        filter: &AuditFilter,
    ) -> Result<InventoryReport, AuditError>;

    /// Generate a compliance report.
    pub fn generate_compliance_report(
        &self,
        framework: ComplianceFramework,
        filter: &AuditFilter,
    ) -> Result<ComplianceReport, AuditError>;

    /// Search for a data subject's information.
    pub fn search_dsar(
        &self,
        request: &DsarRequest,
    ) -> Result<DsarResponse, AuditError>;

    /// Rotate old logs according to retention policy.
    pub fn rotate_logs(
        &self,
        policy: &RetentionPolicy,
    ) -> Result<usize, AuditError>;
}
```

## Extended Error Types

```rust
/// Errors that can occur with audit logging (EXISTS - EXTEND).
#[derive(Error, Debug)]
pub enum AuditError {
    // Existing errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Checksum verification failed for entry {0}")]
    ChecksumMismatch(String),

    #[error("Hash chain broken at entry {0}")]
    ChainBroken(String),

    #[error("Log directory not found: {0}")]
    DirectoryNotFound(String),

    // NEW errors for reporting

    #[error("CSV serialization error: {0}")]
    CsvError(String),

    #[error("Invalid DSAR identifier: {0}")]
    InvalidIdentifier(String),

    #[error("Unsupported compliance framework: {0}")]
    UnsupportedFramework(String),

    #[error("Regex error: {0}")]
    RegexError(#[from] regex::Error),
}
```

## Module Organization

```text
crates/veil-audit/src/
├── lib.rs                # Public exports (EXTEND)
├── entry.rs              # AuditEntry, AuditParameters, AuditOutcome (EXISTS)
├── logger.rs             # AuditLogger (EXISTS - EXTEND with report methods)
├── operation.rs          # AuditOperation (EXISTS)
├── summary.rs            # FindingsSummary, RedactionsSummary (EXISTS)
├── checksum.rs           # Checksum and chain verification (EXISTS)
├── error.rs              # AuditError (EXISTS - EXTEND)
├── filter.rs             # NEW: Move AuditFilter here, extend
├── reports/              # NEW: Report generation module
│   ├── mod.rs            # Report exports
│   ├── inventory.rs      # InventoryReport and generation
│   ├── compliance.rs     # ComplianceReport and GDPR mapping
│   ├── dsar.rs           # DsarRequest, DsarResponse, search logic
│   ├── format.rs         # Format conversion (JSON, CSV, text)
│   └── retention.rs      # RetentionPolicy and log rotation
```

## State Transitions

### Inventory Report Generation Flow

```text
AuditFilter ──query──▶ Vec<AuditEntry>
                            │
                            ▼
                   Aggregate by file and category
                            │
                            ▼
                      InventoryReport
                            │
                ┌───────────┼───────────┐
                ▼           ▼           ▼
            to_json()   to_csv()    to_text()
```

### Compliance Report Generation Flow

```text
ComplianceFramework + AuditFilter
                │
                ▼
         Load GDPR mappings
                │
                ▼
         Query audit entries
                │
                ▼
    Check each requirement:
    - Are affected PII types found?
    - Are they protected?
                │
                ▼
         ComplianceReport
         (with gaps identified)
```

### DSAR Search Flow

```text
DsarRequest ──parse identifier──▶ Regex pattern
                                        │
                                        ▼
                             Query audit entries in range
                                        │
                                        ▼
                       Filter by findings containing pattern
                                        │
                                        ▼
                            Extract context excerpts
                                        │
                                        ▼
                                 DsarResponse
```

## Data Flow Example

```text
User performs scan:
    ↓
AuditLogger.log(entry with FindingsSummary)
    ↓
Written to: audit-2025-12-15.jsonl
    ↓
Later, user requests inventory report:
    ↓
AuditLogger.generate_inventory(filter)
    ↓
Reads all matching JSONL files
    ↓
Aggregates FindingsSummary data
    ↓
Returns InventoryReport
    ↓
User calls report.to_csv()
    ↓
CSV output ready for SIEM/Excel
```
