# veil-discovery

Data discovery library for scanning filesystems to find personally identifiable information (PII).

## Features

- **Filesystem Scanning**: Recursively scan directories for files containing PII
- **Smart Sampling**: Automatically sample large files for efficient scanning
- **Pattern Filtering**: Include/exclude files using glob patterns
- **Multiple Report Formats**: Generate reports in JSON, text, summary, or CSV formats
- **Progress Tracking**: Monitor scan progress with callback functions
- **Comprehensive Statistics**: Track findings by category, file count, and more
- **Error Resilience**: Continue scanning even when individual files fail

## Usage

```rust
use veil_discovery::{Scanner, DiscoveryOptions, ReportGenerator, ReportFormat};
use std::path::PathBuf;

// Configure scan options
let options = DiscoveryOptions {
    root_path: PathBuf::from("/path/to/scan"),
    max_file_size: Some(10 * 1024 * 1024), // 10MB
    sample_size: 100 * 1024,                 // 100KB
    exclude_patterns: vec![
        "**/node_modules/**".to_string(),
        "**/.git/**".to_string(),
    ],
    ..Default::default()
};

// Run the scan
let scanner = Scanner::new(options);
let result = scanner.scan()?;

// Generate a report
let report = ReportGenerator::generate(&result, ReportFormat::Summary)?;
println!("{}", report);

// Access discovered files
for file in &result.discovered_files {
    println!("Found PII in: {}", file.path.display());
    println!("  Categories: {:?}", file.pii_categories);
    println!("  Findings: {}", file.finding_count);
}
```

## Progress Tracking

```rust
use veil_discovery::{Scanner, DiscoveryOptions};

let options = DiscoveryOptions {
    root_path: PathBuf::from("/path/to/scan"),
    report_progress: true,
    ..Default::default()
};

let scanner = Scanner::new(options)
    .with_progress_callback(|progress| {
        println!("Scanned {} files, found PII in {} files",
            progress.files_scanned,
            progress.files_with_pii);
    });

let result = scanner.scan()?;
```

## Report Formats

### Summary Report
Quick overview of scan results:
```
PII Discovery Summary
====================

Files Scanned: 150 | With PII: 23 | Total Findings: 87
Scan Duration: 2.5s

PII Categories Found:
  Email: 45 findings in 15 files
  Phone: 23 findings in 8 files
  CreditCard: 19 findings in 5 files
```

### JSON Report
Machine-readable format for integration:
```json
{
  "timestamp": "2025-12-15T10:30:00Z",
  "source": "filesystem",
  "discovered_files": [...],
  "statistics": {...}
}
```

### Text Report
Detailed human-readable report with file-by-file breakdown.

### CSV Report
Tabular format for spreadsheet import.

## Configuration Options

- `root_path`: Directory to scan
- `max_file_size`: Maximum file size before sampling (default: 10MB)
- `sample_size`: Number of bytes to sample from large files (default: 100KB)
- `include_patterns`: Glob patterns for files to include (default: all)
- `exclude_patterns`: Glob patterns for files to exclude (default: common build/VCS directories)
- `follow_symlinks`: Whether to follow symbolic links (default: false)
- `max_depth`: Maximum directory depth (default: unlimited)
- `report_progress`: Enable progress callbacks (default: true)
- `include_snippets`: Include PII snippets in results (default: false)

## Integration

This crate integrates with:
- `veil-parsers`: For parsing various file formats
- `veil-detect`: For PII detection using regex patterns and validators

## License

MIT OR Apache-2.0
