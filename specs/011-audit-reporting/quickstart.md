# Quickstart: Audit Trail & Reporting

**Feature**: 011-audit-reporting
**Date**: 2025-12-15

## Basic Audit Logging

### Setting Up the Logger

```rust
use veil_audit::{AuditLogger, AuditEntry, AuditOperation, AuditParameters, AuditOutcome};

// Create logger (will create directory if needed)
let mut logger = AuditLogger::new("/var/log/veil/audit")?;

// Logs are stored as: /var/log/veil/audit/audit-2025-12-15.jsonl
```

### Logging a Scan Operation

```rust
use veil_audit::FindingsSummary;
use veil_detect::{DetectorRegistry, PiiCategory};
use veil_parsers::parse_bytes;

// Perform scan
let content = b"Email: john@example.com, Phone: +43 123 456789";
let parsed = parse_bytes(content, &Default::default())?;
let registry = DetectorRegistry::default();
let findings = registry.detect_all(&parsed.segments);

// Create findings summary
let summary = FindingsSummary::from_findings(&findings);

// Create audit entry
let mut entry = AuditEntry::new(
    AuditOperation::Scan,
    AuditParameters {
        input: vec![PathBuf::from("document.txt")],
        ..Default::default()
    },
    AuditOutcome::success().with_findings(summary),
);

// Log it
logger.log(entry)?;
```

### Logging a Protection Operation

```rust
use veil_audit::RedactionsSummary;
use veil_redact::{RedactionEngine, RedactionConfig};

// Perform redaction
let config = RedactionConfig::default();
let engine = RedactionEngine::new(config);
let result = engine.redact_findings(&parsed.segments, &findings)?;

// Create redactions summary
let summary = RedactionsSummary::from_redactions(&result.redactions);

// Create audit entry
let entry = AuditEntry::new(
    AuditOperation::Protect,
    AuditParameters {
        input: vec![PathBuf::from("document.txt")],
        output: Some(PathBuf::from("document-protected.txt")),
        policy: Some("gdpr-standard".to_string()),
        ..Default::default()
    },
    AuditOutcome::success().with_redactions(summary),
);

logger.log(entry)?;
```

## Querying Audit Logs

### Query All Entries

```rust
use veil_audit::AuditFilter;

let filter = AuditFilter::default();
let entries = logger.query(&filter)?;

for entry in entries {
    println!("{}: {} on {:?}",
        entry.timestamp,
        entry.operation,
        entry.parameters.input
    );
}
```

### Query by Date Range

```rust
use chrono::{Duration, Utc};

let filter = AuditFilter {
    from: Some(Utc::now() - Duration::days(7)),
    to: Some(Utc::now()),
    ..Default::default()
};

let entries = logger.query(&filter)?;
println!("Entries in last 7 days: {}", entries.len());
```

### Query by Operation Type

```rust
use veil_audit::{AuditFilter, AuditOperation};

let filter = AuditFilter {
    operations: Some(vec![AuditOperation::Scan]),
    ..Default::default()
};

let scan_entries = logger.query(&filter)?;
println!("Total scans: {}", scan_entries.len());
```

### Query by File Path

```rust
use std::path::PathBuf;

let filter = AuditFilter {
    paths: Some(vec![PathBuf::from("sensitive-docs/")]),
    ..Default::default()
};

let entries = logger.query(&filter)?;
println!("Operations on sensitive docs: {}", entries.len());
```

## Data Inventory Reports

### Generate Inventory Report

```rust
use veil_audit::AuditFilter;

// Generate report for last 30 days
let filter = AuditFilter {
    from: Some(Utc::now() - Duration::days(30)),
    ..Default::default()
};

let report = logger.generate_inventory(&filter)?;

println!("Data Inventory Report");
println!("=====================");
println!("Files scanned: {}", report.total_files);
println!("Total PII findings: {}", report.total_findings);
println!();

// Show breakdown by category
println!("PII by Category:");
for (category, summary) in &report.by_category {
    println!("  {}: {} occurrences in {} files",
        category,
        summary.total_count,
        summary.files.len()
    );
}
```

### Export as JSON

```rust
// Export as JSON for programmatic use
let json = report.to_json()?;
std::fs::write("inventory-report.json", json)?;
```

### Export as CSV

```rust
// Export as CSV for Excel/analysis
let csv = report.to_csv()?;
std::fs::write("inventory-report.csv", csv)?;
```

**CSV Format**:
```csv
file_path,total_findings,email,phone,person_name,last_scanned
/docs/contract.pdf,12,3,2,7,2025-12-15T10:30:00Z
/docs/cv.pdf,8,1,1,6,2025-12-14T15:20:00Z
```

### Export as Human-Readable Text

```rust
// Export as text for reports
let text = report.to_text();
println!("{}", text);
```

**Text Format**:
```text
Data Inventory Report
Generated: 2025-12-15 14:30:00 UTC
Period: 2025-11-15 to 2025-12-15

Summary:
  Files scanned:      147
  Total PII findings: 1,234

PII by Category:
  email          342 occurrences (89 files)
  phone          156 occurrences (45 files)
  person_name    523 occurrences (102 files)
  address        98 occurrences (34 files)
  iban           67 occurrences (23 files)
  credit_card    48 occurrences (18 files)

Top 10 Files by PII Count:
  1. /docs/customer-database.csv    87 findings
  2. /docs/contracts/2025-Q1.pdf    64 findings
  ...
```

## Compliance Reports

### Generate GDPR Compliance Report

```rust
use veil_audit::ComplianceFramework;

let report = logger.generate_compliance_report(
    ComplianceFramework::Gdpr,
    &AuditFilter::default(),
)?;

println!("GDPR Compliance Report");
println!("======================");
println!("Overall status: {:?}", report.overall_status);
println!();

// Show requirements
for req in &report.requirements {
    println!("  {} - {}", req.article, req.description);
    println!("    Status: {:?}", req.status);
    println!("    Categories: {:?}", req.categories);
}

// Show gaps
if !report.gaps.is_empty() {
    println!("\nCompliance Gaps:");
    for gap in &report.gaps {
        println!("  ⚠ {}", gap.requirement);
        println!("    Category: {}", gap.category);
        println!("    Affected files: {}", gap.affected_files.len());
        println!("    Recommendation: {}", gap.recommendation);
    }
}
```

### Export Compliance Report

```rust
// Export as JSON
let json = report.to_json()?;
std::fs::write("compliance-report.json", json)?;

// Or as readable text
let text = report.to_text();
std::fs::write("compliance-report.txt", text)?;
```

**Example Output**:
```text
GDPR Compliance Report
Framework: GDPR
Generated: 2025-12-15 14:45:00 UTC
Overall Status: PARTIAL

Requirements:
✅ Art. 30 (Records of Processing) - COMPLIANT
   Data inventory maintained for: email, phone, person_name

⚠ Art. 32 (Security) - PARTIAL
   Encryption required for: iban, credit_card
   3 files with unprotected IBAN found

Compliance Gaps:
1. Art. 32 Security Requirement
   Category: iban
   Affected files: 3
   Files:
     - /docs/invoices/2025-03.pdf
     - /docs/contracts/supplier-a.pdf
     - /docs/payments.csv
   Recommendation: Apply encryption or tokenization protection
```

## DSAR (Data Subject Access Request)

### Search for a Data Subject

```rust
use veil_audit::{DsarRequest, IdentifierType};

// Create DSAR request
let request = DsarRequest {
    identifier: "john@example.com".to_string(),
    identifier_type: IdentifierType::Email,
    date_range: None, // Search all logs
};

// Execute search
let response = logger.search_dsar(&request)?;

println!("DSAR Response for: {}", request.identifier);
println!("===================");
println!("Files found: {}", response.files_found.len());
println!("Total matches: {}", response.total_matches);
println!();

// Show excerpts
for excerpt in &response.excerpts {
    println!("File: {:?}", excerpt.file_path);
    println!("Date: {}", excerpt.timestamp);
    println!("Context: {}", excerpt.context);
    println!();
}
```

### Search by Name

```rust
let request = DsarRequest {
    identifier: "Max Müller".to_string(),
    identifier_type: IdentifierType::Name,
    date_range: Some((
        Utc::now() - Duration::days(365),
        Utc::now(),
    )),
};

let response = logger.search_dsar(&request)?;
```

### Search by Phone

```rust
let request = DsarRequest {
    identifier: "+43 123 456789".to_string(),
    identifier_type: IdentifierType::Phone,
    date_range: None,
};

let response = logger.search_dsar(&request)?;
```

### Search with Custom Pattern

```rust
// Use custom regex pattern
let request = DsarRequest {
    identifier: r"\b\d{4}-\d{4}-\d{4}-\d{4}\b".to_string(),
    identifier_type: IdentifierType::Custom,
    date_range: None,
};

let response = logger.search_dsar(&request)?;
```

### Export DSAR Response

```rust
// Export as JSON for compliance documentation
let json = response.to_json()?;
std::fs::write("dsar-response.json", json)?;

// Or as readable text for data subject
let text = response.to_text();
std::fs::write("dsar-response.txt", text)?;
```

**Example DSAR Response**:
```text
Data Subject Access Request Response
Identifier: john@example.com
Type: Email
Generated: 2025-12-15 15:00:00 UTC

Summary:
  Files containing data: 12
  Total matches: 23

Files:
  1. /docs/contracts/2025-contract.pdf (Scanned: 2025-12-10)
  2. /docs/emails/inbox.mbox (Scanned: 2025-12-12)
  ...

Excerpts:
1. File: /docs/contracts/2025-contract.pdf
   Date: 2025-12-10 09:15:00 UTC
   Context: "...please contact John Doe at john@example.com for further..."

2. File: /docs/emails/inbox.mbox
   Date: 2025-12-12 14:30:00 UTC
   Context: "From: john@example.com\nSubject: Meeting request..."
```

## Log Rotation

### Configure Retention Policy

```rust
use veil_audit::RetentionPolicy;

// Default: 7 years (GDPR requirement)
let policy = RetentionPolicy::default();

// Or custom duration
let policy = RetentionPolicy::new(365 * 3); // 3 years
```

### Rotate Old Logs

```rust
// Delete logs older than retention period
let deleted_count = logger.rotate_logs(&policy)?;

println!("Deleted {} old log files", deleted_count);
```

### Check if Date is Retained

```rust
use chrono::NaiveDate;

let date = NaiveDate::from_ymd_opt(2018, 12, 15).unwrap();

if policy.is_retained(date) {
    println!("Date is within retention period");
} else {
    println!("Date is outside retention period (will be deleted)");
}
```

## Tamper Detection

### Verify Hash Chain

```rust
use veil_audit::verify_chain;

// Query all entries
let entries = logger.query(&AuditFilter::default())?;

// Verify the hash chain
match verify_chain(&entries) {
    Ok(()) => println!("✓ Audit log integrity verified"),
    Err(e) => eprintln!("✗ Tampering detected: {}", e),
}
```

### Verify Single Day

```rust
use chrono::NaiveDate;

let date = NaiveDate::from_ymd_opt(2025, 12, 15).unwrap();

let filter = AuditFilter {
    from: Some(date.and_hms_opt(0, 0, 0).unwrap().and_utc()),
    to: Some(date.and_hms_opt(23, 59, 59).unwrap().and_utc()),
    ..Default::default()
};

let entries = logger.query(&filter)?;

match verify_chain(&entries) {
    Ok(()) => println!("✓ Log integrity verified for {}", date),
    Err(e) => eprintln!("✗ Tampering detected on {}: {}", date, e),
}
```

## Integration Example: Full Workflow

```rust
use veil_audit::{
    AuditLogger, AuditEntry, AuditOperation, AuditParameters, AuditOutcome,
    ComplianceFramework, DsarRequest, IdentifierType, RetentionPolicy,
};
use veil_detect::DetectorRegistry;
use veil_parsers::parse_bytes;

// 1. Set up audit logger
let mut logger = AuditLogger::new("/var/log/veil/audit")?;

// 2. Process documents
let files = vec!["doc1.txt", "doc2.pdf", "doc3.csv"];

for file in files {
    let content = std::fs::read(file)?;
    let parsed = parse_bytes(&content, &Default::default())?;

    let registry = DetectorRegistry::default();
    let findings = registry.detect_all(&parsed.segments);

    let summary = FindingsSummary::from_findings(&findings);

    let entry = AuditEntry::new(
        AuditOperation::Scan,
        AuditParameters {
            input: vec![PathBuf::from(file)],
            ..Default::default()
        },
        AuditOutcome::success().with_findings(summary),
    );

    logger.log(entry)?;
}

// 3. Generate data inventory
let inventory = logger.generate_inventory(&AuditFilter::default())?;
std::fs::write("inventory.csv", inventory.to_csv()?)?;

// 4. Check GDPR compliance
let compliance = logger.generate_compliance_report(
    ComplianceFramework::Gdpr,
    &AuditFilter::default(),
)?;
std::fs::write("compliance.txt", compliance.to_text())?;

// 5. Handle DSAR request
let dsar_request = DsarRequest {
    identifier: "data-subject@example.com".to_string(),
    identifier_type: IdentifierType::Email,
    date_range: None,
};

let dsar_response = logger.search_dsar(&dsar_request)?;
std::fs::write("dsar-response.json", dsar_response.to_json()?)?;

// 6. Rotate old logs
let policy = RetentionPolicy::default();
let deleted = logger.rotate_logs(&policy)?;
println!("Maintenance: Deleted {} old log files", deleted);

// 7. Verify integrity
let entries = logger.query(&AuditFilter::default())?;
verify_chain(&entries)?;
println!("✓ Audit log integrity verified");
```

## Error Handling

```rust
use veil_audit::{AuditLogger, AuditError};

match AuditLogger::new("/var/log/veil/audit") {
    Ok(mut logger) => {
        // Use logger
    }
    Err(AuditError::Io(e)) => {
        eprintln!("IO error: {}", e);
        // Check permissions, disk space, etc.
    }
    Err(AuditError::DirectoryNotFound(path)) => {
        eprintln!("Directory not found: {}", path);
        // Create directory or fix path
    }
    Err(e) => {
        eprintln!("Error: {}", e);
    }
}
```

## CLI Integration Examples

These examples show how veil-cli would use the audit library:

```bash
# Generate inventory report
veil audit inventory --format csv --output inventory.csv

# Check GDPR compliance
veil audit compliance --framework gdpr --format text

# Search for data subject
veil audit dsar --email "john@example.com" --output dsar-response.json

# Rotate old logs
veil audit rotate --retention-days 2555  # 7 years

# Verify integrity
veil audit verify --date 2025-12-15
```
