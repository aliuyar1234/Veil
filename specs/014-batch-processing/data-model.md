# Data Model: Batch Processing

**Feature**: 014-batch-processing
**Date**: 2025-12-15

## Overview

This document defines the core data structures for batch file processing. These types orchestrate
multi-file scanning, directory traversal, ZIP archive processing, and result aggregation.

## Core Entities

### BatchJob

A batch processing job with configuration and execution state.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchJob {
    /// Job identifier
    pub id: Uuid,

    /// Input paths (files or directories)
    pub sources: Vec<PathBuf>,

    /// Processing options
    pub options: BatchOptions,

    /// Current execution state
    #[serde(skip)]
    pub state: Arc<BatchState>,
}

#[derive(Debug)]
pub struct BatchState {
    /// Cancellation flag
    pub cancelled: AtomicBool,

    /// Files processed so far
    pub processed: AtomicUsize,

    /// Total files discovered
    pub total: AtomicUsize,

    /// Current file being processed (for progress)
    pub current_file: Mutex<Option<PathBuf>>,
}
```

**Invariants**:
- `sources` must not be empty
- `id` is unique per job instance
- `state.processed <= state.total`

---

### BatchOptions

Configuration for batch processing operations.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchOptions {
    /// Recursively scan subdirectories (default: true)
    pub recursive: bool,

    /// Follow symbolic links (default: true)
    pub follow_symlinks: bool,

    /// Maximum directory depth (default: 100)
    pub max_depth: usize,

    /// Include patterns (glob syntax, e.g., "*.csv")
    pub include: Vec<String>,

    /// Exclude patterns (glob syntax, e.g., "*.log")
    pub exclude: Vec<String>,

    /// Number of parallel threads (default: num_cpus - 1)
    pub parallelism: usize,

    /// Enable progress reporting (default: true)
    pub progress: bool,

    /// Process ZIP archives (default: true)
    pub process_archives: bool,

    /// Password for encrypted ZIPs
    pub archive_password: Option<String>,

    /// Maximum archive nesting depth (default: 5)
    pub max_archive_depth: usize,

    /// Maximum single file size (default: 100MB)
    pub max_file_size: usize,

    /// Output format for results (default: Json)
    pub output_format: OutputFormat,

    /// Enable streaming results (default: false)
    pub streaming: bool,
}

impl Default for BatchOptions {
    fn default() -> Self {
        Self {
            recursive: true,
            follow_symlinks: true,
            max_depth: 100,
            include: vec!["*".to_string()],
            exclude: Vec::new(),
            parallelism: num_cpus::get().saturating_sub(1).max(1),
            progress: true,
            process_archives: true,
            archive_password: None,
            max_archive_depth: 5,
            max_file_size: 100 * 1024 * 1024, // 100MB
            output_format: OutputFormat::Json,
            streaming: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputFormat {
    Json,
    JsonLines,
    Csv,
}
```

---

### FileEntry

Metadata about a file to be processed.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    /// Full path to the file
    pub path: PathBuf,

    /// File size in bytes
    pub size: u64,

    /// Detected file format
    pub format: Option<FileFormat>,

    /// Processing status
    pub status: FileStatus,

    /// Source archive (if extracted from ZIP)
    pub archive_source: Option<PathBuf>,

    /// Archive nesting depth (0 for regular files)
    pub archive_depth: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileStatus {
    /// Not yet processed
    Pending,

    /// Currently being processed
    Processing,

    /// Successfully processed
    Completed,

    /// Processing failed
    Failed,

    /// Skipped (filtered out or too large)
    Skipped,
}
```

**Validation Rules**:
- `path` must exist and be a file (not directory)
- `size` matches actual file size
- `archive_depth` <= `BatchOptions::max_archive_depth`

---

### FileResult

Result of processing a single file.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileResult {
    /// File metadata
    pub file: FileEntry,

    /// Parse result from veil-parsers
    pub parse_result: ParseResult,

    /// Processing duration in milliseconds
    pub duration_ms: u64,

    /// Any warnings or non-fatal errors
    pub warnings: Vec<String>,
}
```

---

### BatchProgress

Progress state for active batch job.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchProgress {
    /// Number of files processed
    pub processed: usize,

    /// Total files to process
    pub total: usize,

    /// Percentage complete (0-100)
    pub percent: f64,

    /// Currently processing file
    pub current_file: Option<PathBuf>,

    /// Estimated time remaining in seconds
    pub eta_seconds: Option<u64>,

    /// Elapsed time in seconds
    pub elapsed_seconds: u64,

    /// Processing rate (files per second)
    pub throughput: f64,
}

impl BatchProgress {
    pub fn percent(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.processed as f64 / self.total as f64) * 100.0
        }
    }

    pub fn eta_seconds(&self, elapsed: u64) -> Option<u64> {
        if self.processed == 0 {
            return None;
        }

        let rate = self.processed as f64 / elapsed as f64;
        let remaining = self.total.saturating_sub(self.processed);
        Some((remaining as f64 / rate) as u64)
    }
}
```

**Invariants**:
- `processed <= total`
- `percent` is in range [0.0, 100.0]
- `throughput >= 0.0`

---

### BatchResult

Aggregate result of batch processing.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResult {
    /// Job identifier
    pub job_id: Uuid,

    /// Summary statistics
    pub summary: BatchSummary,

    /// Per-file results (empty if streaming enabled)
    pub file_results: Vec<FileResult>,

    /// Files that failed to process
    pub failed: Vec<FileError>,

    /// Files that were skipped
    pub skipped: Vec<SkippedFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchSummary {
    /// Total files discovered
    pub total_files: usize,

    /// Files successfully processed
    pub processed: usize,

    /// Files that failed
    pub failed: usize,

    /// Files skipped
    pub skipped: usize,

    /// Total processing duration in milliseconds
    pub duration_ms: u64,

    /// Total bytes processed
    pub bytes_processed: u64,

    /// Aggregate findings summary
    pub findings_summary: FindingsSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindingsSummary {
    /// Total findings across all files
    pub total_findings: usize,

    /// Findings grouped by category
    pub by_category: HashMap<String, usize>,

    /// Files containing findings
    pub files_with_findings: usize,
}
```

**Invariants**:
- `total_files = processed + failed + skipped`
- `file_results.len() == processed` when streaming disabled
- `file_results.len() == 0` when streaming enabled

---

### FileError

Information about a file that failed to process.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileError {
    /// Path to the file
    pub path: PathBuf,

    /// Error type
    pub error: ErrorKind,

    /// Error message
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorKind {
    /// File not found or inaccessible
    NotFound,

    /// Permission denied
    PermissionDenied,

    /// File too large
    FileTooLarge,

    /// Unsupported format
    UnsupportedFormat,

    /// Parse error
    ParseError,

    /// Archive error (corrupted, wrong password)
    ArchiveError,

    /// I/O error
    IoError,

    /// Other error
    Other,
}
```

---

### SkippedFile

Information about a file that was skipped.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkippedFile {
    /// Path to the file
    pub path: PathBuf,

    /// Reason for skipping
    pub reason: SkipReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkipReason {
    /// Filtered out by include/exclude patterns
    Filtered,

    /// File too large
    TooLarge,

    /// Unsupported file type
    UnsupportedType,

    /// Archive nesting too deep
    ArchiveDepthExceeded,
}
```

---

## Entity Relationships

```
BatchJob ────────────┐
    │                │
    ├─ sources       │
    ├─ options ──────┼──▶ BatchOptions
    └─ state ────────┘         │
         │                     │
         ├─ cancelled          ├─ include/exclude patterns
         ├─ processed          ├─ parallelism
         ├─ total              └─ archive_password
         └─ current_file
              │
              ▼
         FileEntry ────────▶ FileResult ────┐
              │                  │           │
              ├─ path            ├─ parse_result (from veil-parsers)
              ├─ size            └─ duration_ms
              ├─ format                │
              ├─ status                │
              └─ archive_source        │
                                       ▼
                                  BatchResult
                                       │
                                       ├─ summary: BatchSummary
                                       ├─ file_results: Vec<FileResult>
                                       ├─ failed: Vec<FileError>
                                       └─ skipped: Vec<SkippedFile>
```

---

## Trait Definitions

### BatchProcessor

The main interface for batch processing operations.

```rust
pub trait BatchProcessor {
    /// Process a batch job
    fn process(&self, job: &BatchJob) -> Result<BatchResult, BatchError>;

    /// Process with progress callback
    fn process_with_progress<F>(
        &self,
        job: &BatchJob,
        progress_callback: F,
    ) -> Result<BatchResult, BatchError>
    where
        F: Fn(BatchProgress) + Send + Sync;

    /// Process with streaming results
    fn process_streaming<F>(
        &self,
        job: &BatchJob,
        result_callback: F,
    ) -> Result<BatchSummary, BatchError>
    where
        F: Fn(FileResult) + Send + Sync;
}
```

---

### FileFilter

Interface for file filtering logic.

```rust
pub trait FileFilter: Send + Sync {
    /// Check if file should be processed
    fn should_process(&self, path: &Path) -> bool;
}

/// Glob pattern based filter
pub struct GlobFilter {
    include: Vec<Pattern>,
    exclude: Vec<Pattern>,
}

impl FileFilter for GlobFilter {
    fn should_process(&self, path: &Path) -> bool {
        let matches_include = self.include.is_empty()
            || self.include.iter().any(|p| p.matches_path(path));

        let matches_exclude = self.exclude.iter().any(|p| p.matches_path(path));

        matches_include && !matches_exclude
    }
}
```

---

## State Transitions

### Batch Processing Flow

```
┌──────────────┐
│ Create Job   │
└──────┬───────┘
       │
       ▼
┌──────────────┐
│ Discover     │ → Walk directories, expand archives
│ Files        │ → Apply filters
└──────┬───────┘
       │
       ▼
┌──────────────┐
│ Initialize   │ → Set total count
│ Progress     │ → Create thread pool
└──────┬───────┘
       │
       ▼
┌──────────────┐
│ Process      │ → Parallel file processing
│ Files        │ → Update progress atomically
└──────┬───────┘
       │
       ├─▶ [Cancelled] ──▶ Return partial results
       │
       ▼
┌──────────────┐
│ Aggregate    │ → Collect results
│ Results      │ → Build summary
└──────┬───────┘
       │
       ▼
┌──────────────┐
│ Return       │
│ BatchResult  │
└──────────────┘
```

### File Processing Flow

```
┌──────────────┐
│ FileEntry    │
│ (Pending)    │
└──────┬───────┘
       │
       ▼
┌──────────────┐
│ Check Size   │ → [Too Large] ──▶ Skipped
└──────┬───────┘
       │
       ▼
┌──────────────┐
│ Detect Type  │ → [Unsupported] ──▶ Skipped
└──────┬───────┘
       │
       ▼
┌──────────────┐
│ FileEntry    │
│ (Processing) │
└──────┬───────┘
       │
       ├─▶ [ZIP] ──▶ Extract ──▶ Recurse
       │
       ▼
┌──────────────┐
│ Parse File   │ → [Error] ──▶ Failed
└──────┬───────┘
       │
       ▼
┌──────────────┐
│ FileEntry    │
│ (Completed)  │
└──────┬───────┘
       │
       ▼
┌──────────────┐
│ FileResult   │
└──────────────┘
```

---

## Size Limits

| Entity | Limit | Rationale |
|--------|-------|-----------|
| Max file size | 100 MB (default) | Memory constraints (spec FR-008) |
| Max archive depth | 5 levels | Prevent ZIP bombs |
| Max directory depth | 100 levels | Prevent infinite recursion |
| Max parallelism | 64 threads | Reasonable concurrency limit |
| Max total files | No hard limit | Streaming prevents memory issues |

---

## Serialization

All public types derive `Serialize` and `Deserialize` via serde.

### JSON Output Example

```json
{
  "job_id": "550e8400-e29b-41d4-a716-446655440000",
  "summary": {
    "total_files": 1000,
    "processed": 985,
    "failed": 5,
    "skipped": 10,
    "duration_ms": 45000,
    "bytes_processed": 524288000,
    "findings_summary": {
      "total_findings": 1523,
      "by_category": {
        "email": 450,
        "ssn": 23,
        "phone": 1050
      },
      "files_with_findings": 127
    }
  },
  "file_results": [],
  "failed": [
    {
      "path": "/data/corrupted.csv",
      "error": "ParseError",
      "message": "Invalid CSV: unexpected end of file"
    }
  ],
  "skipped": [
    {
      "path": "/data/huge.log",
      "reason": "TooLarge"
    }
  ]
}
```

### Progress Event Example

```json
{
  "processed": 450,
  "total": 1000,
  "percent": 45.0,
  "current_file": "/data/reports/2024-Q3.csv",
  "eta_seconds": 55,
  "elapsed_seconds": 45,
  "throughput": 10.0
}
```
