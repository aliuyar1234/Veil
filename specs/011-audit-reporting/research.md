# Research: Audit Trail & Reporting

**Feature**: 011-audit-reporting
**Date**: 2025-12-15

## Existing Implementation Analysis

The `veil-audit` crate already has a solid foundation:

### Core Infrastructure (Complete)
- **AuditEntry**: Full struct with ID, timestamp, operation, parameters, outcome, checksum
- **AuditLogger**: JSONL file-based logging with daily rotation (audit-YYYY-MM-DD.jsonl)
- **Hash Chain**: SHA-256 checksums linking entries via `previous_checksum`
- **AuditFilter**: Query support for date ranges, operation types, file paths
- **FindingsSummary**: Aggregates PII findings by category
- **RedactionsSummary**: Aggregates redactions by category
- **Tamper Detection**: `verify_chain()` function validates checksums

### Current File Structure
```text
crates/veil-audit/src/
├── lib.rs          # Public exports
├── entry.rs        # AuditEntry, AuditParameters, AuditOutcome
├── logger.rs       # AuditLogger with JSONL append
├── operation.rs    # AuditOperation enum (Scan, Protect, etc.)
├── summary.rs      # FindingsSummary, RedactionsSummary
├── checksum.rs     # calculate_checksum, verify_chain
└── error.rs        # AuditError enum
```

### Dependencies (Already in Cargo.toml)
- serde, serde_json: Serialization
- chrono: Timestamps
- sha2: Checksums
- uuid: Entry IDs
- thiserror: Error types
- veil-detect: PiiCategory for summaries
- veil-redact: AppliedRedaction for summaries

## Missing Features (per spec)

### 1. Data Inventory Reports (FR-004, Priority P1)

**Need**: Generate reports showing PII distribution across files.

**Current State**: We have per-operation findings in audit log, but no aggregation.

**Required**:
- `InventoryReport` struct with per-file and per-category breakdowns
- `AuditLogger::generate_inventory()` method
- Output formats: JSON, CSV, human-readable text

### 2. Compliance Reports (FR-005, Priority P2)

**Need**: Map findings to GDPR compliance requirements.

**Current State**: No compliance framework knowledge.

**Required**:
- `ComplianceReport` struct
- `ComplianceFramework` enum (GDPR, with future extensibility)
- Mapping of PII categories to GDPR articles
- Pass/fail status for each requirement
- `AuditLogger::generate_compliance_report()` method

### 3. DSAR Support (FR-009, FR-010, Priority P2)

**Need**: Search audit logs and findings for specific identifiers (email, name, phone).

**Current State**: Basic query filtering exists, but no identifier-specific search.

**Required**:
- `DsarRequest` struct with identifier and type
- `DsarResponse` struct with search results and file excerpts
- `AuditLogger::search_dsar()` method
- Search across both audit metadata AND finding content

### 4. Export Formats (FR-006, FR-007)

**Need**: Export in multiple formats (JSON, JSONL, CSV, text).

**Current State**: Logs are already JSONL; need formatters for reports.

**Required**:
- `ReportFormat` enum
- Trait-based formatters for each report type
- CSV writer for tabular data
- Human-readable text formatter

### 5. Log Rotation & Retention (FR-012)

**Need**: Configurable retention periods.

**Current State**: Daily rotation works; no automatic cleanup.

**Required**:
- `RetentionPolicy` config
- `AuditLogger::rotate_logs()` method
- Delete logs older than retention period

## Decision 1: Report Generation Architecture

**Question**: Should reports be generated in-memory or streamed?

**Decision**: In-memory aggregation for initial implementation.

**Rationale**:
- Simpler implementation
- Audit logs are append-only; can cache aggregations
- Performance target: 1M entries in <10s is achievable with in-memory HashMap
- Streaming can be added later if needed

**Interface**:
```rust
pub struct InventoryReport {
    pub generated_at: DateTime<Utc>,
    pub total_files_scanned: usize,
    pub total_findings: usize,
    pub by_file: HashMap<PathBuf, FileSummary>,
    pub by_category: HashMap<String, usize>,
}

impl InventoryReport {
    pub fn to_json(&self) -> Result<String, AuditError>;
    pub fn to_csv(&self) -> Result<String, AuditError>;
    pub fn to_text(&self) -> String;
}
```

## Decision 2: GDPR Compliance Mapping

**Question**: How to map PII categories to GDPR articles?

**Decision**: Static mapping in code (const data structure).

**Rationale**:
- GDPR is stable; no need for external config
- Simpler than loading from file
- Future frameworks can add similar mappings

**Mapping Strategy**:
```rust
pub struct GdprMapping {
    article: &'static str,
    requirement: &'static str,
    applicable_categories: &'static [&'static str],
}

const GDPR_MAPPINGS: &[GdprMapping] = &[
    GdprMapping {
        article: "Art. 32 (Security)",
        requirement: "PII must be encrypted or pseudonymized",
        applicable_categories: &["email", "phone", "iban", "credit_card"],
    },
    // ...
];
```

## Decision 3: DSAR Search Implementation

**Question**: How to search for identifiers in findings?

**Decision**: Two-phase search: audit metadata + findings content.

**Rationale**:
- Phase 1: Filter audit entries by file paths (fast)
- Phase 2: Regex/exact match on finding content (detailed)
- Return both: files containing identifier + specific excerpts

**Interface**:
```rust
pub struct DsarRequest {
    pub identifier: String,
    pub identifier_type: IdentifierType, // Email, Name, Phone, Custom
    pub date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
}

pub struct DsarResponse {
    pub request: DsarRequest,
    pub generated_at: DateTime<Utc>,
    pub files_found: Vec<PathBuf>,
    pub excerpts: Vec<DsarExcerpt>,
}

pub struct DsarExcerpt {
    pub file_path: PathBuf,
    pub operation_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub context: String, // Surrounding text
}
```

## Decision 4: Report Format Abstraction

**Question**: How to support multiple output formats?

**Decision**: Implement `Display` and dedicated methods per format.

**Rationale**:
- `Display` trait for human-readable text
- Dedicated methods (`to_json()`, `to_csv()`) for structured formats
- Avoids over-engineering with trait objects

## Decision 5: Retention Policy

**Question**: How to configure retention periods?

**Decision**: Builder pattern with defaults.

**Rationale**:
- Default: 7 years (GDPR Article 5)
- Configurable per deployment
- Manual trigger for rotation (no automatic background task)

**Interface**:
```rust
pub struct RetentionPolicy {
    pub duration_days: u32,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self { duration_days: 365 * 7 } // 7 years
    }
}

impl AuditLogger {
    pub fn rotate_logs(&self, policy: &RetentionPolicy) -> Result<usize, AuditError>;
}
```

## Implementation Phases

1. **Phase 1: Inventory Reports** (P1)
   - Add `InventoryReport` struct
   - Implement `generate_inventory()` in `AuditLogger`
   - Add JSON, CSV, text formatters

2. **Phase 2: Compliance Reports** (P2)
   - Add `ComplianceReport` struct
   - Create GDPR mapping data
   - Implement `generate_compliance_report()`

3. **Phase 3: DSAR Support** (P2)
   - Add `DsarRequest` and `DsarResponse` structs
   - Implement `search_dsar()` in `AuditLogger`
   - Add excerpt extraction logic

4. **Phase 4: Log Rotation** (P2)
   - Add `RetentionPolicy` config
   - Implement `rotate_logs()` method

## Dependencies to Add

| Crate | Purpose | Justification |
|-------|---------|---------------|
| csv | CSV export | Standard library for CSV writing |
| regex | DSAR identifier search | Pattern matching for flexible search |

## Deferred Features

- **Real-time log streaming**: Requires async; defer to future iteration
- **External storage (S3, database)**: Local filesystem sufficient for initial release
- **Policy inheritance for retention**: Complex; single policy is enough
- **Automatic background rotation**: Requires daemon; manual rotation is sufficient

## Performance Considerations

- **Target**: Generate inventory report from 1M entries in <10 seconds
- **Strategy**: Single pass through logs with HashMap aggregation
- **Memory**: Assume ~500 bytes per entry; 1M entries = ~500MB in memory (acceptable)
- **Optimization**: Can add indices for common queries if needed (defer)

## Security Considerations

- **Log tampering**: Hash chain already provides detection
- **DSAR privacy**: Ensure excerpts don't leak additional PII
- **Key storage**: No keys stored in audit logs (only references)
- **Access control**: Out of scope (filesystem permissions sufficient)
