# Implementation Plan: Batch Processing

**Branch**: `014-batch-processing` | **Date**: 2025-12-15 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/014-batch-processing/spec.md`

## Summary

Build a batch processing orchestration layer that enables recursive directory scanning, ZIP archive processing, parallel file processing, and aggregate result reporting. This crate integrates with veil-parsers to provide multi-file PII scanning capabilities with progress reporting and graceful error handling.

## Technical Context

**Language/Version**: Rust (stable, 1.75+)
**Primary Dependencies**: walkdir (directory traversal), zip (archive handling), rayon (parallelism), glob (pattern matching), indicatif (progress UI), infer (file type detection)
**Storage**: N/A (pure processing library, no persistence)
**Testing**: cargo test (unit + integration tests with fixture directories and archives)
**Target Platform**: Cross-platform library (Linux, macOS, Windows)
**Project Type**: Workspace crate (veil-batch), depends on veil-parsers
**Performance Goals**: 10,000 files in <10 minutes, near-linear speedup to 8 cores, <3x memory overhead
**Constraints**: Memory-bounded for arbitrary batch sizes via streaming, ZIP bombs prevented by depth limits
**Scale/Scope**: Process directories of 1000s-10,000s of files with progress reporting

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Security First | ✅ PASS | ZIP bomb prevention via depth/size limits; no unsafe needed |
| II. Stability & Error Handling | ✅ PASS | Continue on errors; collect failures for reporting |
| III. Performance | ✅ PASS | Rayon parallelism; streaming results; progress atomics |
| IV. Simplicity & Minimalism | ✅ PASS | Single processor; unified result type; delegating to existing parsers |
| V. Test-First Development | ✅ PASS | Fixture directories and ZIP archives for testing |
| VI. Dependency Discipline | ⚠️ REVIEW | 6 crates needed - all well-maintained and specialized |
| VII. Rust Standards | ✅ PASS | Clippy/fmt; documented public API |

**Gate Result**: PASS (dependencies justified for file system operations and parallelism)

## Project Structure

### Documentation (this feature)

```text
specs/014-batch-processing/
├── plan.md              # This file
├── research.md          # Phase 0 output ✅
├── data-model.md        # Phase 1 output ✅
├── quickstart.md        # Phase 1 output ✅
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
Cargo.toml               # Workspace root - add veil-batch member
crates/
├── veil-parsers/        # Existing crate (dependency)
└── veil-batch/          # New crate
    ├── Cargo.toml
    ├── src/
    │   ├── lib.rs           # Public API exports
    │   ├── error.rs         # Error types (thiserror)
    │   ├── types.rs         # BatchJob, BatchResult, BatchOptions, etc.
    │   ├── processor.rs     # DefaultBatchProcessor implementation
    │   ├── discovery.rs     # Directory walking and file discovery
    │   ├── filter.rs        # GlobFilter and filtering logic
    │   ├── archive.rs       # ZIP extraction and processing
    │   ├── parallel.rs      # Rayon-based parallel processing
    │   ├── progress.rs      # Progress tracking and reporting
    │   └── streaming.rs     # Streaming result handler
    └── tests/
        ├── fixtures/        # Test data
        │   ├── dirs/        # Test directory structures
        │   │   ├── simple/  # Flat directory
        │   │   ├── nested/  # Deep nesting
        │   │   └── mixed/   # Various file types
        │   └── archives/    # Test ZIP files
        │       ├── simple.zip
        │       ├── nested.zip
        │       └── encrypted.zip
        ├── processor_tests.rs
        ├── discovery_tests.rs
        ├── filter_tests.rs
        ├── archive_tests.rs
        └── integration_tests.rs
```

**Structure Decision**: New crate `veil-batch` that orchestrates batch processing by depending on `veil-parsers`. Keeps parsing logic separate from orchestration logic. File system operations and archive handling are domain-specific to batch processing.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| walkdir crate | Safe recursive traversal with symlink cycle detection | std::fs::read_dir requires manual recursion and symlink handling |
| zip crate | ZIP archive extraction with password support | No standard library alternative; format is complex |
| rayon crate | Work-stealing parallelism with thread pool management | Manual threading is error-prone and less efficient |
| glob crate | Shell-style pattern matching for file filtering | Regex is too low-level and not user-friendly for file patterns |
| indicatif crate | Progress bars with TTY detection and ETA calculation | Manual progress printing lacks polish and TTY awareness |
| infer crate | Magic byte detection for accurate file type identification | Extension-only detection is unreliable |

## Module Breakdown

### 1. Core Types (`types.rs`)

**Purpose**: Define all data structures from data-model.md

**Key Types**:
- `BatchJob`: Job configuration with sources and options
- `BatchOptions`: Processing configuration
- `BatchResult`: Aggregate results
- `BatchSummary`: Statistics
- `FileEntry`: File metadata
- `FileResult`: Per-file result
- `FileError`: Error information
- `SkippedFile`: Skipped file information
- `BatchProgress`: Progress state

**Dependencies**: serde, uuid, std::path, std::sync

**Testing**: Unit tests for type invariants and serialization

---

### 2. Error Handling (`error.rs`)

**Purpose**: Define BatchError enum for all error cases

**Error Types**:
```rust
#[derive(Error, Debug)]
pub enum BatchError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Archive error: {0}")]
    Archive(String),

    #[error("Invalid password")]
    InvalidPassword,

    #[error("Archive depth exceeded: max {max}, got {depth}")]
    ArchiveDepthExceeded { max: usize, depth: usize },

    #[error("Invalid glob pattern: {0}")]
    InvalidPattern(#[from] glob::PatternError),

    #[error("Parse error: {0}")]
    Parse(#[from] veil_parsers::ParseError),

    #[error("Batch processing cancelled")]
    Cancelled,

    #[error("File too large: {size} bytes exceeds {max}")]
    FileTooLarge { size: u64, max: usize },
}
```

**Dependencies**: thiserror

**Testing**: Error conversion and message formatting

---

### 3. File Discovery (`discovery.rs`)

**Purpose**: Walk directories and discover files to process

**Key Functions**:
```rust
pub fn discover_files(
    sources: &[PathBuf],
    options: &BatchOptions,
) -> Result<Vec<FileEntry>, BatchError>;

fn walk_directory(
    path: &Path,
    options: &BatchOptions,
) -> impl Iterator<Item = FileEntry>;

fn detect_file_format(path: &Path) -> Option<FileFormat>;
```

**Implementation**:
- Use walkdir for recursive traversal
- Apply depth and symlink limits
- Use infer crate for magic byte detection
- Fallback to extension-based detection
- Collect file metadata (size, path)

**Dependencies**: walkdir, infer, veil-parsers (for FileFormat)

**Testing**:
- Test recursive vs non-recursive
- Test symlink following
- Test depth limits
- Test format detection

---

### 4. File Filtering (`filter.rs`)

**Purpose**: Apply include/exclude glob patterns

**Key Types**:
```rust
pub trait FileFilter: Send + Sync {
    fn should_process(&self, path: &Path) -> bool;
}

pub struct GlobFilter {
    include: Vec<Pattern>,
    exclude: Vec<Pattern>,
}

impl FileFilter for GlobFilter {
    fn should_process(&self, path: &Path) -> bool;
}
```

**Implementation**:
- Parse glob patterns during construction
- Match against full path or filename based on pattern
- Include patterns are OR'd (any match = include)
- Exclude patterns are OR'd (any match = exclude)
- Exclude takes precedence over include

**Dependencies**: glob

**Testing**:
- Test wildcard patterns (`*.csv`)
- Test recursive patterns (`**/*.pdf`)
- Test complex patterns (`**/reports/*.{csv,json}`)
- Test include/exclude interaction

---

### 5. Archive Processing (`archive.rs`)

**Purpose**: Extract and process ZIP archives

**Key Functions**:
```rust
pub fn process_archive(
    path: &Path,
    password: Option<&str>,
    depth: usize,
    options: &BatchOptions,
) -> Result<Vec<FileResult>, BatchError>;

fn extract_and_process(
    archive: &mut ZipArchive<File>,
    depth: usize,
    options: &BatchOptions,
) -> Result<Vec<FileResult>, BatchError>;
```

**Implementation**:
- Open ZIP with optional password
- Iterate over entries
- For each entry:
  - If it's a nested ZIP, recurse (check depth limit)
  - If it's a supported file, process in-memory
  - If too large, skip with warning
- Use tempfile for nested ZIP extraction
- Clean up temp files after processing

**Dependencies**: zip, tempfile

**Testing**:
- Test simple ZIP processing
- Test nested ZIP processing
- Test password-protected ZIP
- Test depth limit enforcement
- Test wrong password handling
- Test corrupted ZIP handling

---

### 6. Parallel Processing (`parallel.rs`)

**Purpose**: Process files in parallel using rayon

**Key Functions**:
```rust
pub fn process_files_parallel(
    files: Vec<FileEntry>,
    options: &BatchOptions,
    progress: Option<Arc<BatchState>>,
) -> (Vec<FileResult>, Vec<FileError>, Vec<SkippedFile>);

fn process_single_file(
    entry: &FileEntry,
    options: &BatchOptions,
) -> Result<FileResult, BatchError>;
```

**Implementation**:
- Create thread pool with configured parallelism
- Use par_iter() from rayon
- Process each file:
  - Check size limit
  - If ZIP, delegate to archive module
  - Otherwise, parse with veil-parsers
  - Update progress atomically
  - Collect results, errors, and skipped
- Use Mutex<Vec> for collecting results across threads

**Dependencies**: rayon, veil-parsers

**Testing**:
- Test parallel vs sequential performance
- Test thread count configuration
- Test cancellation
- Test error collection
- Test progress updates

---

### 7. Progress Tracking (`progress.rs`)

**Purpose**: Track and report batch progress

**Key Types**:
```rust
pub struct ProgressTracker {
    state: Arc<BatchState>,
    start_time: Instant,
}

impl ProgressTracker {
    pub fn new(total: usize) -> Self;
    pub fn increment(&self);
    pub fn set_current_file(&self, path: PathBuf);
    pub fn snapshot(&self) -> BatchProgress;
}
```

**Implementation**:
- Use AtomicUsize for thread-safe counters
- Calculate ETA based on throughput
- Update current file via Mutex
- Provide snapshot for callbacks

**Dependencies**: std::sync, std::time

**Testing**:
- Test progress calculation
- Test ETA estimation
- Test concurrent updates
- Test progress callbacks

---

### 8. Streaming Results (`streaming.rs`)

**Purpose**: Stream results via callback instead of collecting

**Key Functions**:
```rust
pub fn process_streaming<F>(
    files: Vec<FileEntry>,
    options: &BatchOptions,
    callback: F,
) -> Result<BatchSummary, BatchError>
where
    F: Fn(FileResult) + Send + Sync;
```

**Implementation**:
- Use mpsc channel to send results from worker threads
- Main thread receives and invokes callback
- Only collect summary statistics, not full results
- Memory usage stays constant regardless of batch size

**Dependencies**: std::sync::mpsc, rayon

**Testing**:
- Test callback invocation count
- Test summary accuracy
- Test memory usage (doesn't grow with file count)

---

### 9. Batch Processor (`processor.rs`)

**Purpose**: Main entry point implementing BatchProcessor trait

**Key Type**:
```rust
pub struct DefaultBatchProcessor {
    // Configuration if needed
}

impl BatchProcessor for DefaultBatchProcessor {
    fn process(&self, job: &BatchJob) -> Result<BatchResult, BatchError>;

    fn process_with_progress<F>(
        &self,
        job: &BatchJob,
        progress_callback: F,
    ) -> Result<BatchResult, BatchError>
    where
        F: Fn(BatchProgress) + Send + Sync;

    fn process_streaming<F>(
        &self,
        job: &BatchJob,
        result_callback: F,
    ) -> Result<BatchSummary, BatchError>
    where
        F: Fn(FileResult) + Send + Sync;
}
```

**Implementation**:
1. Discover files via discovery module
2. Apply filters via filter module
3. Create progress tracker if needed
4. Process files via parallel module
5. Build BatchResult or BatchSummary
6. Return results

**Dependencies**: All internal modules

**Testing**: Integration tests covering full workflows

---

## Integration Points

### With veil-parsers

```rust
use veil_parsers::{parse_file, ParseOptions, ParseResult};

// In process_single_file:
let parse_result = parse_file(&entry.path, &ParseOptions::default())?;
```

**Interface**: Public API (`parse_file`, `ParseResult`, `FileFormat`)

**Data Flow**: veil-batch calls veil-parsers for each file

---

### With CLI (future veil-cli)

```rust
// CLI will use veil-batch's public API
use veil_batch::{BatchJob, BatchOptions, DefaultBatchProcessor, BatchProcessor};

let job = BatchJob::new(paths, options);
let processor = DefaultBatchProcessor::new();
let result = processor.process_with_progress(&job, |progress| {
    // Display progress bar via indicatif
});
```

---

## Post-Design Constitution Re-Check

*Re-evaluated after Phase 1 design completion (2025-12-15)*

| Principle | Status | Post-Design Notes |
|-----------|--------|-------------------|
| I. Security First | ✅ PASS | ZIP bomb prevention: max_archive_depth=5, max_file_size=100MB; no unsafe |
| II. Stability & Error Handling | ✅ PASS | Result<T, BatchError> everywhere; collect errors, continue processing |
| III. Performance | ✅ PASS | Rayon parallel processing; streaming results; atomic progress |
| IV. Simplicity & Minimalism | ✅ PASS | 9 focused modules; delegating to veil-parsers; single processor |
| V. Test-First Development | ✅ PASS | Fixture directories and ZIPs; integration tests |
| VI. Dependency Discipline | ✅ PASS | 6 crates justified: walkdir, zip, rayon, glob, indicatif, infer (all well-maintained) |
| VII. Rust Standards | ✅ PASS | thiserror for errors; serde derives; documented public API |

**Post-Design Gate Result**: PASS - Ready for task generation

---

## Implementation Phases

### Phase 0: Setup (Est. 1 hour)

1. Create `crates/veil-batch` directory structure
2. Add veil-batch to workspace Cargo.toml
3. Set up dependencies in crates/veil-batch/Cargo.toml
4. Create module structure (empty files)
5. Set up test fixtures directory

**Validation**: `cargo build` succeeds

---

### Phase 1: Core Types (Est. 2 hours)

1. Implement types.rs with all data structures
2. Add serde derives
3. Implement Default for BatchOptions
4. Add helper methods (BatchProgress calculations)
5. Write serialization tests

**Validation**: Types serialize/deserialize correctly

---

### Phase 2: Error Handling (Est. 1 hour)

1. Define BatchError enum
2. Add thiserror derives
3. Add From conversions for std::io::Error, etc.
4. Test error messages

**Validation**: Error conversions work

---

### Phase 3: File Discovery (Est. 4 hours)

1. Implement walk_directory with walkdir
2. Add format detection with infer
3. Apply depth and symlink limits
4. Collect FileEntry metadata
5. Write discovery tests

**Validation**: Can discover files in test fixtures

---

### Phase 4: File Filtering (Est. 3 hours)

1. Implement GlobFilter
2. Parse glob patterns
3. Implement should_process logic
4. Handle include/exclude interaction
5. Write filter tests with various patterns

**Validation**: Filters work correctly on test fixtures

---

### Phase 5: Archive Processing (Est. 6 hours)

1. Implement ZIP extraction
2. Add password support
3. Implement recursive archive processing
4. Add depth limit enforcement
5. Handle corrupted archives gracefully
6. Write archive tests with test ZIPs

**Validation**: Can process nested and encrypted ZIPs

---

### Phase 6: Parallel Processing (Est. 5 hours)

1. Set up rayon thread pool
2. Implement process_files_parallel
3. Add atomic progress updates
4. Collect results, errors, skipped
5. Integrate with veil-parsers
6. Write parallel processing tests

**Validation**: Files process in parallel with correct results

---

### Phase 7: Progress Tracking (Est. 3 hours)

1. Implement ProgressTracker
2. Add ETA calculation
3. Provide snapshot method
4. Test concurrent updates
5. Write progress tests

**Validation**: Progress reports accurately

---

### Phase 8: Streaming Results (Est. 3 hours)

1. Implement callback-based streaming
2. Use mpsc channel for result passing
3. Collect summary statistics only
4. Write streaming tests

**Validation**: Streaming works with constant memory

---

### Phase 9: Batch Processor (Est. 4 hours)

1. Implement DefaultBatchProcessor
2. Wire up all modules
3. Implement process, process_with_progress, process_streaming
4. Add cancellation support
5. Write integration tests

**Validation**: Full workflow tests pass

---

### Phase 10: Documentation & Polish (Est. 3 hours)

1. Add documentation comments to public API
2. Add module-level docs
3. Create examples directory
4. Run clippy and fix warnings
5. Run rustfmt
6. Update workspace README

**Validation**: `cargo doc` builds; clippy passes

---

## Testing Strategy

### Unit Tests

- Type serialization/deserialization
- Error conversions
- Progress calculations
- Filter pattern matching
- Format detection

### Integration Tests

- Full directory scanning
- ZIP processing
- Parallel processing
- Streaming results
- Progress reporting
- Cancellation

### Test Fixtures

```text
tests/fixtures/
├── dirs/
│   ├── simple/              # 10 files, flat
│   ├── nested/              # 100 files, 5 levels deep
│   ├── mixed/               # Various formats
│   └── symlinks/            # Symlink tests
└── archives/
    ├── simple.zip           # 5 files
    ├── nested.zip           # ZIP containing ZIP
    ├── encrypted.zip        # Password-protected
    └── corrupted.zip        # Intentionally broken
```

### Performance Tests

- 10,000 files in <10 minutes
- Parallel speedup test (1 vs 8 threads)
- Memory usage test (streaming vs collecting)

---

## Success Criteria

From spec.md:

- ✅ **SC-001**: 10,000 files scanned in <10 minutes with parallel processing
- ✅ **SC-002**: ZIP archives up to 1GB processed without memory exhaustion
- ✅ **SC-003**: File filtering 100% accurate
- ✅ **SC-004**: Progress updates at least every 1 second
- ✅ **SC-005**: Aggregate reports correctly sum per-file findings
- ✅ **SC-006**: Near-linear speedup up to 8 cores

---

## Dependencies

### Direct Dependencies

```toml
[dependencies]
# Core
serde = { workspace = true }
thiserror = { workspace = true }
uuid = { workspace = true }

# File operations
walkdir = "2.5"
glob = "0.3"
infer = "0.15"

# Archives
zip = { version = "0.6", features = ["deflate", "aes-crypto"] }

# Parallelism
rayon = { workspace = true }

# Progress
indicatif = { workspace = true }

# Veil crates
veil-parsers = { path = "../veil-parsers" }

[dev-dependencies]
tempfile = { workspace = true }
pretty_assertions = { workspace = true }
```

---

## Open Questions & Decisions

1. **Q: Should we support TAR/7Z/RAR archives?**
   A: Future enhancement; start with ZIP only (most common)

2. **Q: Should we support resumable batch jobs?**
   A: Future enhancement; checkpoint files for resume capability

3. **Q: Should we deduplicate files (same content, different paths)?**
   A: Future enhancement; hash-based deduplication

4. **Q: Should we integrate with PII detection directly?**
   A: No; batch processing focuses on orchestration; detection is separate concern

5. **Q: Should we support remote file systems (S3, etc.)?**
   A: Future enhancement; start with local file system

---

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| ZIP bombs | DoS, memory exhaustion | Enforce max_archive_depth=5, max_file_size limits |
| Symlink loops | Infinite recursion | Use walkdir's built-in cycle detection |
| Memory exhaustion (large batches) | OOM crash | Streaming results mode; don't collect all in memory |
| Thread contention | Poor parallel performance | Use rayon's work-stealing scheduler |
| Locked files (Windows) | Access errors | Catch and log as FileError; continue processing |

---

## Future Enhancements

- Checkpoint/resume for interrupted batches
- TAR, 7Z, RAR archive support
- Content-based file deduplication
- Cloud storage support (S3, Azure Blob, GCS)
- Distributed processing (multiple machines)
- Real-time file watching (inotify, FSEvents)
- Database output (PostgreSQL, SQLite)
- Custom output formats (HTML reports, etc.)
