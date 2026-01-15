# Implementation Tasks: Batch Processing

**Feature**: `014-batch-processing` | **Branch**: `014-batch-processing` | **Generated**: 2025-12-15
**Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md)

## Implementation Status: ✅ COMPLETE

All phases have been implemented and 22 unit tests + 3 integration tests are passing.

### Completion Summary

| Phase | Status | Notes |
|-------|--------|-------|
| Phase 0: Project Setup | ✅ Complete | Crate structure, Cargo.toml, modules |
| Phase 1: Core Types & Errors | ✅ Complete | BatchError, BatchOptions, all types |
| Phase 2: File Discovery | ✅ Complete | Directory walking, format detection |
| Phase 3: File Filtering | ✅ Complete | GlobFilter, include/exclude patterns |
| Phase 4: Archive Processing | ✅ Complete | ZIP extraction, password support |
| Phase 5: Parallel Processing | ✅ Complete | Rayon integration, thread pool |
| Phase 6: Progress Tracking | ✅ Complete | ProgressTracker, callbacks |
| Phase 7: Streaming Results | ✅ Complete | Memory-efficient processing |
| Phase 8: Batch Processor | ✅ Complete | DefaultBatchProcessor trait impl |
| Phase 9: Integration Testing | ✅ Complete | End-to-end tests passing |
| Phase 10: Documentation & Polish | ✅ Complete | Clippy clean, formatted |

### Implemented Modules

- `src/error.rs` - BatchError enum with thiserror
- `src/types.rs` - All data types (BatchOptions, FileEntry, FileResult, etc.)
- `src/discovery.rs` - File discovery with walkdir
- `src/filter.rs` - Glob pattern filtering
- `src/archive.rs` - ZIP processing with password support
- `src/parallel.rs` - Rayon parallel processing
- `src/progress.rs` - Progress tracking
- `src/streaming.rs` - Streaming result processing
- `src/processor.rs` - BatchProcessor trait and implementation
- `src/lib.rs` - Public API exports

---

## Overview

This document breaks down the implementation of the batch processing feature into actionable, dependency-ordered tasks. Each task includes acceptance criteria, testing requirements, and estimated effort.

**Total Estimated Effort**: ~35 hours

---

## Phase 0: Project Setup

### Task 0.1: Create crate structure

**Effort**: 30 minutes

**Description**: Set up the veil-batch crate directory structure and integrate with workspace.

**Steps**:
1. Create `crates/veil-batch/` directory
2. Create `crates/veil-batch/src/` directory
3. Create `crates/veil-batch/tests/` directory
4. Create `crates/veil-batch/tests/fixtures/` directory structure

**Acceptance Criteria**:
- Directory structure matches plan.md specification
- All directories exist and are ready for code

**Dependencies**: None

---

### Task 0.2: Initialize Cargo.toml files

**Effort**: 30 minutes

**Description**: Configure workspace and crate manifests with dependencies.

**Steps**:
1. Add `veil-batch` member to workspace `Cargo.toml`
2. Create `crates/veil-batch/Cargo.toml` with dependencies:
   - serde (workspace)
   - thiserror (workspace)
   - uuid (workspace)
   - walkdir = "2.5"
   - glob = "0.3"
   - infer = "0.15"
   - zip = { version = "0.6", features = ["deflate", "aes-crypto"] }
   - rayon (workspace)
   - indicatif (workspace)
   - veil-parsers = { path = "../veil-parsers" }
3. Add dev-dependencies:
   - tempfile (workspace)
   - pretty_assertions (workspace)
4. Set package metadata (name, version, edition, authors)

**Acceptance Criteria**:
- `cargo build` succeeds in workspace root
- All dependencies resolve correctly
- Crate appears in workspace members

**Dependencies**: Task 0.1

**Testing**: Run `cargo build` and verify success

---

### Task 0.3: Create module skeleton

**Effort**: 30 minutes

**Description**: Create empty module files with basic structure.

**Steps**:
1. Create `src/lib.rs` with module declarations
2. Create empty files:
   - `src/error.rs`
   - `src/types.rs`
   - `src/processor.rs`
   - `src/discovery.rs`
   - `src/filter.rs`
   - `src/archive.rs`
   - `src/parallel.rs`
   - `src/progress.rs`
   - `src/streaming.rs`
3. Add `#![warn(missing_docs)]` to lib.rs
4. Add basic crate-level documentation

**Acceptance Criteria**:
- All modules compile (even if empty)
- `cargo build` succeeds
- `cargo clippy` passes

**Dependencies**: Task 0.2

**Testing**: Run `cargo build && cargo clippy`

---

### Task 0.4: Create test fixtures

**Effort**: 1 hour

**Description**: Set up test directory structures and archive files for integration testing.

**Steps**:
1. Create `tests/fixtures/dirs/` subdirectories:
   - `simple/` - 10 text files with sample content
   - `nested/` - 5 levels of nesting with mixed file types
   - `mixed/` - Various formats (txt, csv, json, html)
   - `symlinks/` - Test files with symlinks
2. Create `tests/fixtures/archives/` with ZIP files:
   - `simple.zip` - Archive of simple directory
   - `nested.zip` - Archive containing another ZIP
   - `encrypted.zip` - Password-protected archive (password: "test")
   - `corrupted.zip` - Intentionally broken ZIP
3. Add `.gitkeep` files to preserve directory structure
4. Document fixtures in `tests/fixtures/README.md`

**Acceptance Criteria**:
- All fixture directories exist
- All fixture archives are valid (except corrupted.zip)
- Fixtures contain realistic test data
- README documents each fixture's purpose

**Dependencies**: Task 0.1

**Testing**: Manually verify fixtures can be opened/extracted

---

## Phase 1: Core Types & Errors

### Task 1.1: Implement error types

**Effort**: 1 hour

**Description**: Define BatchError enum with all error cases using thiserror.

**Steps**:
1. In `src/error.rs`, define `BatchError` enum with variants:
   - `Io(#[from] std::io::Error)`
   - `Archive(String)`
   - `InvalidPassword`
   - `ArchiveDepthExceeded { max: usize, depth: usize }`
   - `InvalidPattern(#[from] glob::PatternError)`
   - `Parse(#[from] veil_parsers::ParseError)`
   - `Cancelled`
   - `FileTooLarge { size: u64, max: usize }`
2. Add `#[derive(Error, Debug)]` attribute
3. Add error message strings with `#[error("...")]`
4. Implement custom Display if needed
5. Add unit tests for error formatting

**Acceptance Criteria**:
- All error types compile
- Error messages are clear and actionable
- From conversions work for wrapped errors
- Tests verify error formatting

**Dependencies**: Task 0.3

**Testing**:
```rust
#[test]
fn test_error_messages() {
    let err = BatchError::ArchiveDepthExceeded { max: 5, depth: 6 };
    assert_eq!(err.to_string(), "Archive depth exceeded: max 5, got 6");
}
```

---

### Task 1.2: Implement core data types

**Effort**: 2 hours

**Description**: Implement all data structures from data-model.md in types.rs.

**Steps**:
1. Define `BatchOptions` struct with fields:
   - `recursive: bool`
   - `follow_symlinks: bool`
   - `max_depth: Option<usize>`
   - `max_file_size: usize`
   - `max_archive_depth: usize`
   - `parallelism: usize`
   - `include_patterns: Vec<String>`
   - `exclude_patterns: Vec<String>`
   - `archive_password: Option<String>`
2. Define `FileEntry` struct
3. Define `FileResult` struct
4. Define `FileError` struct
5. Define `SkippedFile` struct
6. Define `BatchSummary` struct
7. Define `BatchResult` struct
8. Define `BatchProgress` struct
9. Define `BatchJob` struct
10. Add `#[derive(Debug, Clone, Serialize, Deserialize)]` to all types
11. Implement `Default` for `BatchOptions`

**Acceptance Criteria**:
- All types compile with serde derives
- Default values are sensible (recursive=true, parallelism=num_cpus-1, etc.)
- Types match data-model.md specification

**Dependencies**: Task 1.1

**Testing**:
```rust
#[test]
fn test_batch_options_default() {
    let opts = BatchOptions::default();
    assert!(opts.recursive);
    assert_eq!(opts.max_archive_depth, 5);
    assert_eq!(opts.max_file_size, 100_000_000); // 100MB
}
```

---

### Task 1.3: Add type helper methods

**Effort**: 1 hour

**Description**: Implement helper methods for BatchOptions, BatchProgress, and BatchJob.

**Steps**:
1. Implement `BatchOptions::builder()` method
2. Implement `BatchProgress::percentage()` method
3. Implement `BatchProgress::eta_seconds()` method
4. Implement `BatchJob::new(sources, options)` constructor
5. Add validation methods where appropriate

**Acceptance Criteria**:
- Builder pattern works for BatchOptions
- Progress calculations are correct
- BatchJob validates inputs

**Dependencies**: Task 1.2

**Testing**:
```rust
#[test]
fn test_progress_percentage() {
    let progress = BatchProgress {
        processed: 50,
        total: 100,
        // ... other fields
    };
    assert_eq!(progress.percentage(), 50.0);
}
```

---

### Task 1.4: Test type serialization

**Effort**: 30 minutes

**Description**: Verify all types serialize/deserialize correctly with serde.

**Steps**:
1. Create test module in `src/types.rs`
2. Write serialization tests for each type
3. Test round-trip (serialize → deserialize → compare)
4. Test JSON and binary formats if needed

**Acceptance Criteria**:
- All types serialize to JSON
- Round-trip serialization preserves data
- No panics during serialization

**Dependencies**: Task 1.2

**Testing**:
```rust
#[test]
fn test_batch_result_serialization() {
    let result = BatchResult { /* ... */ };
    let json = serde_json::to_string(&result).unwrap();
    let deserialized: BatchResult = serde_json::from_str(&json).unwrap();
    assert_eq!(result, deserialized);
}
```

---

## Phase 2: File Discovery

### Task 2.1: Implement directory walking

**Effort**: 2 hours

**Description**: Implement recursive directory traversal using walkdir.

**Steps**:
1. In `src/discovery.rs`, implement `walk_directory()` function
2. Configure walkdir with:
   - `follow_links` from options
   - `max_depth` from options
   - Filter out hidden files (configurable)
3. Collect file paths and metadata
4. Handle permission errors gracefully (log and continue)
5. Return iterator of PathBuf

**Acceptance Criteria**:
- Recursively walks directories when enabled
- Respects depth limits
- Handles symlinks according to options
- Gracefully handles permission errors

**Dependencies**: Task 1.2

**Testing**:
```rust
#[test]
fn test_walk_directory_recursive() {
    let opts = BatchOptions { recursive: true, ..Default::default() };
    let files: Vec<_> = walk_directory(Path::new("tests/fixtures/dirs/nested"), &opts).collect();
    assert!(files.len() > 10);
}

#[test]
fn test_walk_directory_depth_limit() {
    let opts = BatchOptions { recursive: true, max_depth: Some(2), ..Default::default() };
    let files: Vec<_> = walk_directory(Path::new("tests/fixtures/dirs/nested"), &opts).collect();
    // Verify max depth respected
}
```

---

### Task 2.2: Implement file format detection

**Effort**: 2 hours

**Description**: Detect file formats using magic bytes (infer crate) with extension fallback.

**Steps**:
1. Implement `detect_file_format(path: &Path) -> Option<FileFormat>` function
2. Try magic byte detection first using infer crate
3. Fall back to extension-based detection
4. Map detected types to veil-parsers FileFormat enum
5. Return None for unsupported formats
6. Handle I/O errors during detection

**Acceptance Criteria**:
- Correctly detects common formats (txt, csv, json, html, pdf)
- Extension fallback works when magic bytes fail
- Returns None for unsupported formats
- No panics on unreadable files

**Dependencies**: Task 2.1

**Testing**:
```rust
#[test]
fn test_detect_format_by_magic_bytes() {
    // Create temp file with PDF header
    let format = detect_file_format(Path::new("test.dat")).unwrap();
    assert_eq!(format, FileFormat::Pdf);
}

#[test]
fn test_detect_format_by_extension() {
    let format = detect_file_format(Path::new("file.csv")).unwrap();
    assert_eq!(format, FileFormat::Csv);
}
```

---

### Task 2.3: Build FileEntry metadata

**Effort**: 1 hour

**Description**: Collect file metadata and create FileEntry objects.

**Steps**:
1. Extend `walk_directory` to create `FileEntry` for each file
2. Collect metadata:
   - `path: PathBuf`
   - `size: u64`
   - `format: Option<FileFormat>`
   - `modified: Option<SystemTime>`
3. Skip files larger than `max_file_size`
4. Filter by detected format (if parser available)

**Acceptance Criteria**:
- FileEntry contains accurate metadata
- Large files are skipped with warning
- Unsupported formats are excluded

**Dependencies**: Task 2.2

**Testing**:
```rust
#[test]
fn test_file_entry_metadata() {
    let entries = discover_files(&[PathBuf::from("tests/fixtures/dirs/simple")], &opts);
    assert!(entries.iter().all(|e| e.size > 0));
    assert!(entries.iter().all(|e| e.format.is_some()));
}
```

---

### Task 2.4: Implement discover_files public API

**Effort**: 1 hour

**Description**: Create main entry point for file discovery.

**Steps**:
1. Implement `pub fn discover_files(sources: &[PathBuf], options: &BatchOptions) -> Result<Vec<FileEntry>, BatchError>`
2. Handle multiple source paths
3. Handle both files and directories as sources
4. Deduplicate if same file appears multiple times
5. Sort by path for deterministic ordering

**Acceptance Criteria**:
- Accepts both individual files and directories
- Returns deduplicated file list
- Respects all BatchOptions settings
- Returns detailed errors on failure

**Dependencies**: Task 2.3

**Testing**:
```rust
#[test]
fn test_discover_files_mixed_sources() {
    let sources = vec![
        PathBuf::from("tests/fixtures/dirs/simple"),
        PathBuf::from("tests/fixtures/dirs/mixed/file.txt"),
    ];
    let entries = discover_files(&sources, &opts).unwrap();
    assert!(entries.len() > 10);
}
```

---

## Phase 3: File Filtering

### Task 3.1: Implement GlobFilter type

**Effort**: 1.5 hours

**Description**: Create filter type that applies glob patterns to file paths.

**Steps**:
1. In `src/filter.rs`, define `FileFilter` trait
2. Define `GlobFilter` struct with `include` and `exclude` pattern lists
3. Implement `GlobFilter::new(include: Vec<String>, exclude: Vec<String>) -> Result<Self, BatchError>`
4. Parse glob patterns during construction (fail fast on invalid patterns)
5. Store compiled Pattern objects

**Acceptance Criteria**:
- GlobFilter compiles patterns at construction
- Invalid patterns return error (not panic)
- Stores both include and exclude patterns

**Dependencies**: Task 1.1

**Testing**:
```rust
#[test]
fn test_glob_filter_construction() {
    let filter = GlobFilter::new(
        vec!["*.csv".to_string()],
        vec!["*test*".to_string()],
    ).unwrap();
    // Should succeed
}

#[test]
fn test_glob_filter_invalid_pattern() {
    let result = GlobFilter::new(vec!["[invalid".to_string()], vec![]);
    assert!(result.is_err());
}
```

---

### Task 3.2: Implement filter matching logic

**Effort**: 2 hours

**Description**: Implement should_process method with include/exclude logic.

**Steps**:
1. Implement `FileFilter` trait for `GlobFilter`
2. Implement `should_process(&self, path: &Path) -> bool` method
3. Logic:
   - If include patterns exist, path must match at least one
   - If exclude patterns exist, path must not match any
   - Exclude takes precedence over include
4. Support both full path matching and filename-only matching
5. Handle edge cases (empty pattern lists, etc.)

**Acceptance Criteria**:
- Include patterns work (OR logic)
- Exclude patterns work (OR logic)
- Exclude overrides include
- Works with both full paths and filenames

**Dependencies**: Task 3.1

**Testing**:
```rust
#[test]
fn test_include_patterns() {
    let filter = GlobFilter::new(vec!["*.csv".to_string()], vec![]).unwrap();
    assert!(filter.should_process(Path::new("data.csv")));
    assert!(!filter.should_process(Path::new("data.txt")));
}

#[test]
fn test_exclude_overrides_include() {
    let filter = GlobFilter::new(
        vec!["*.csv".to_string()],
        vec!["*test*.csv".to_string()],
    ).unwrap();
    assert!(filter.should_process(Path::new("data.csv")));
    assert!(!filter.should_process(Path::new("test_data.csv")));
}
```

---

### Task 3.3: Integrate filters with discovery

**Effort**: 1 hour

**Description**: Apply filters during file discovery phase.

**Steps**:
1. Modify `discover_files` to create and apply GlobFilter
2. Build filter from BatchOptions include/exclude patterns
3. Filter FileEntry list before returning
4. Track filtered files in statistics

**Acceptance Criteria**:
- Filters are applied correctly during discovery
- Only matching files are returned
- Statistics track filtered count

**Dependencies**: Task 3.2, Task 2.4

**Testing**:
```rust
#[test]
fn test_discovery_with_filters() {
    let mut opts = BatchOptions::default();
    opts.include_patterns = vec!["*.txt".to_string()];
    let entries = discover_files(&[PathBuf::from("tests/fixtures/dirs/mixed")], &opts).unwrap();
    assert!(entries.iter().all(|e| e.path.extension() == Some(OsStr::new("txt"))));
}
```

---

## Phase 4: Archive Processing

### Task 4.1: Implement basic ZIP extraction

**Effort**: 2 hours

**Description**: Extract and process files from ZIP archives.

**Steps**:
1. In `src/archive.rs`, implement `process_archive(path: &Path, password: Option<&str>, depth: usize, options: &BatchOptions) -> Result<Vec<FileResult>, BatchError>`
2. Open ZIP archive with zip crate
3. Iterate over archive entries
4. Extract each entry to memory (for small files)
5. Detect format and process with veil-parsers
6. Handle extraction errors gracefully

**Acceptance Criteria**:
- Successfully extracts files from ZIP
- Processes each extracted file
- Returns results for all files
- Handles corrupted ZIPs gracefully

**Dependencies**: Task 1.2

**Testing**:
```rust
#[test]
fn test_process_simple_archive() {
    let results = process_archive(
        Path::new("tests/fixtures/archives/simple.zip"),
        None,
        0,
        &opts,
    ).unwrap();
    assert!(results.len() > 0);
}
```

---

### Task 4.2: Add password support

**Effort**: 1 hour

**Description**: Handle password-protected ZIP archives.

**Steps**:
1. Pass password to ZIP archive opener
2. Handle invalid password error
3. Return `BatchError::InvalidPassword` on auth failure
4. Test with encrypted fixtures

**Acceptance Criteria**:
- Correct password allows extraction
- Wrong password returns InvalidPassword error
- No password on encrypted ZIP returns error

**Dependencies**: Task 4.1

**Testing**:
```rust
#[test]
fn test_encrypted_archive_correct_password() {
    let results = process_archive(
        Path::new("tests/fixtures/archives/encrypted.zip"),
        Some("test"),
        0,
        &opts,
    ).unwrap();
    assert!(results.len() > 0);
}

#[test]
fn test_encrypted_archive_wrong_password() {
    let result = process_archive(
        Path::new("tests/fixtures/archives/encrypted.zip"),
        Some("wrong"),
        0,
        &opts,
    );
    assert!(matches!(result, Err(BatchError::InvalidPassword)));
}
```

---

### Task 4.3: Implement nested archive processing

**Effort**: 2 hours

**Description**: Recursively process ZIPs within ZIPs with depth limits.

**Steps**:
1. Detect if extracted file is a ZIP (by magic bytes)
2. If ZIP and depth < max_archive_depth, recurse
3. If depth limit exceeded, return ArchiveDepthExceeded error
4. Track depth through recursive calls
5. Clean up temporary extracted files

**Acceptance Criteria**:
- Nested ZIPs are extracted recursively
- Depth limit is enforced
- Temporary files are cleaned up
- Exceeding depth returns proper error

**Dependencies**: Task 4.2

**Testing**:
```rust
#[test]
fn test_nested_archive() {
    let results = process_archive(
        Path::new("tests/fixtures/archives/nested.zip"),
        None,
        0,
        &opts,
    ).unwrap();
    // Should include files from both outer and inner ZIP
}

#[test]
fn test_archive_depth_limit() {
    let mut opts = BatchOptions::default();
    opts.max_archive_depth = 1;
    let result = process_archive(
        Path::new("tests/fixtures/archives/deeply_nested.zip"),
        None,
        0,
        &opts,
    );
    assert!(matches!(result, Err(BatchError::ArchiveDepthExceeded { .. })));
}
```

---

### Task 4.4: Handle large files in archives

**Effort**: 1 hour

**Description**: Skip or stream very large files from archives.

**Steps**:
1. Check uncompressed size before extraction
2. If size > max_file_size, skip and add to skipped list
3. Add warning message to SkippedFile
4. Track skipped files in results

**Acceptance Criteria**:
- Large files are skipped, not extracted
- Skipped files are reported in results
- Processing continues after skipping

**Dependencies**: Task 4.1

**Testing**:
```rust
#[test]
fn test_skip_large_files_in_archive() {
    let mut opts = BatchOptions::default();
    opts.max_file_size = 1024; // 1KB limit
    let results = process_archive(
        Path::new("tests/fixtures/archives/with_large_file.zip"),
        None,
        0,
        &opts,
    ).unwrap();
    // Verify large file was skipped
}
```

---

## Phase 5: Parallel Processing

### Task 5.1: Set up rayon thread pool

**Effort**: 1 hour

**Description**: Configure rayon for parallel processing with configurable thread count.

**Steps**:
1. In `src/parallel.rs`, implement thread pool configuration
2. Use `rayon::ThreadPoolBuilder` to create pool
3. Configure number of threads from BatchOptions.parallelism
4. Handle pool creation errors
5. Default to num_cpus - 1 if parallelism = 0

**Acceptance Criteria**:
- Thread pool created with correct thread count
- Parallelism = 1 works (sequential processing)
- Default parallelism is reasonable
- Pool creation errors are handled

**Dependencies**: Task 1.2

**Testing**:
```rust
#[test]
fn test_thread_pool_configuration() {
    let opts = BatchOptions { parallelism: 4, ..Default::default() };
    // Verify thread pool uses 4 threads
}
```

---

### Task 5.2: Implement parallel file processing

**Effort**: 3 hours

**Description**: Process multiple files concurrently using rayon's par_iter.

**Steps**:
1. Implement `process_files_parallel(files: Vec<FileEntry>, options: &BatchOptions, progress: Option<Arc<BatchState>>) -> (Vec<FileResult>, Vec<FileError>, Vec<SkippedFile>)`
2. Use `par_iter()` to process files in parallel
3. For each file:
   - Check if archive (delegate to archive module)
   - Otherwise parse with veil-parsers
   - Collect result, error, or skipped
4. Use thread-safe collections (Mutex<Vec>) for results
5. Update progress atomically

**Acceptance Criteria**:
- Files process in parallel
- Results are collected correctly
- No race conditions
- Progress updates are thread-safe

**Dependencies**: Task 5.1, Task 4.3

**Testing**:
```rust
#[test]
fn test_parallel_processing() {
    let files = vec![/* test FileEntry objects */];
    let (results, errors, skipped) = process_files_parallel(files, &opts, None);
    assert_eq!(results.len() + errors.len() + skipped.len(), files.len());
}
```

---

### Task 5.3: Implement single file processing

**Effort**: 2 hours

**Description**: Process individual files with proper error handling.

**Steps**:
1. Implement `process_single_file(entry: &FileEntry, options: &BatchOptions) -> Result<FileResult, BatchError>`
2. Check file size against limits
3. Detect if file is archive
4. If archive, delegate to `process_archive`
5. Otherwise, read file and parse with veil-parsers
6. Handle parse errors gracefully
7. Return FileResult with findings

**Acceptance Criteria**:
- Correctly processes text files
- Correctly processes archives
- Enforces size limits
- Handles parse errors without panicking

**Dependencies**: Task 4.3

**Testing**:
```rust
#[test]
fn test_process_text_file() {
    let entry = FileEntry {
        path: PathBuf::from("tests/fixtures/dirs/simple/file.txt"),
        size: 100,
        format: Some(FileFormat::PlainText),
        modified: None,
    };
    let result = process_single_file(&entry, &opts).unwrap();
    assert_eq!(result.path, entry.path);
}
```

---

### Task 5.4: Test parallel performance

**Effort**: 1 hour

**Description**: Verify parallel processing provides speedup.

**Steps**:
1. Create performance test with 100+ files
2. Benchmark with parallelism = 1 (sequential)
3. Benchmark with parallelism = 4
4. Benchmark with parallelism = 8
5. Verify speedup is reasonable (not slower)
6. Measure CPU utilization

**Acceptance Criteria**:
- Parallel processing is faster than sequential
- Speedup increases with thread count (up to CPU limit)
- No deadlocks or race conditions

**Dependencies**: Task 5.2

**Testing**:
```rust
#[test]
fn test_parallel_speedup() {
    let files = create_test_files(100);

    let start = Instant::now();
    let opts_seq = BatchOptions { parallelism: 1, ..Default::default() };
    process_files_parallel(files.clone(), &opts_seq, None);
    let seq_duration = start.elapsed();

    let start = Instant::now();
    let opts_par = BatchOptions { parallelism: 4, ..Default::default() };
    process_files_parallel(files, &opts_par, None);
    let par_duration = start.elapsed();

    assert!(par_duration < seq_duration);
}
```

---

## Phase 6: Progress Tracking

### Task 6.1: Implement ProgressTracker

**Effort**: 2 hours

**Description**: Create thread-safe progress tracking with ETA calculation.

**Steps**:
1. In `src/progress.rs`, define `BatchState` struct with AtomicUsize counters
2. Implement `ProgressTracker` wrapper around Arc<BatchState>
3. Add methods:
   - `new(total: usize) -> Self`
   - `increment()`
   - `set_current_file(path: PathBuf)`
   - `snapshot() -> BatchProgress`
4. Calculate ETA based on throughput
5. Track start time for duration calculation

**Acceptance Criteria**:
- Thread-safe increment operations
- Accurate progress percentage
- Reasonable ETA estimates
- No race conditions

**Dependencies**: Task 1.2

**Testing**:
```rust
#[test]
fn test_progress_tracking() {
    let tracker = ProgressTracker::new(100);
    tracker.increment();
    tracker.increment();
    let snapshot = tracker.snapshot();
    assert_eq!(snapshot.processed, 2);
    assert_eq!(snapshot.total, 100);
    assert_eq!(snapshot.percentage(), 2.0);
}
```

---

### Task 6.2: Integrate progress with parallel processing

**Effort**: 1 hour

**Description**: Update progress during parallel file processing.

**Steps**:
1. Modify `process_files_parallel` to accept ProgressTracker
2. Increment after each file completes
3. Set current file before processing
4. Handle progress in both success and error cases

**Acceptance Criteria**:
- Progress updates during processing
- Updates are thread-safe
- Final count matches total files

**Dependencies**: Task 6.1, Task 5.2

**Testing**:
```rust
#[test]
fn test_progress_during_processing() {
    let files = create_test_files(10);
    let tracker = ProgressTracker::new(files.len());
    process_files_parallel(files, &opts, Some(tracker.clone()));
    let snapshot = tracker.snapshot();
    assert_eq!(snapshot.processed, 10);
}
```

---

### Task 6.3: Add progress callbacks

**Effort**: 1 hour

**Description**: Support callback functions for progress updates.

**Steps**:
1. Add callback parameter to processing functions
2. Invoke callback periodically (not too frequently to avoid overhead)
3. Pass BatchProgress snapshot to callback
4. Handle callback errors gracefully (don't fail batch)

**Acceptance Criteria**:
- Callbacks invoked during processing
- Callback frequency is reasonable (e.g., every 1%)
- Callback errors don't crash batch
- Callback can be None (optional)

**Dependencies**: Task 6.2

**Testing**:
```rust
#[test]
fn test_progress_callback() {
    let files = create_test_files(100);
    let callback_count = Arc::new(AtomicUsize::new(0));
    let counter = callback_count.clone();

    process_files_with_progress(files, &opts, move |_progress| {
        counter.fetch_add(1, Ordering::SeqCst);
    });

    assert!(callback_count.load(Ordering::SeqCst) > 0);
}
```

---

## Phase 7: Streaming Results

### Task 7.1: Implement streaming processor

**Effort**: 2 hours

**Description**: Process files with callback instead of collecting all results.

**Steps**:
1. In `src/streaming.rs`, implement `process_streaming<F>(files: Vec<FileEntry>, options: &BatchOptions, callback: F) -> Result<BatchSummary, BatchError>`
2. Use mpsc channel to send results from worker threads
3. Main thread receives and invokes callback
4. Collect only summary statistics (counts, not full results)
5. Ensure callback is called for each result

**Acceptance Criteria**:
- Results streamed via callback
- Memory usage doesn't grow with file count
- Summary statistics are accurate
- All results delivered to callback

**Dependencies**: Task 5.2

**Testing**:
```rust
#[test]
fn test_streaming_results() {
    let files = create_test_files(100);
    let result_count = Arc::new(AtomicUsize::new(0));
    let counter = result_count.clone();

    let summary = process_streaming(files, &opts, move |_result| {
        counter.fetch_add(1, Ordering::SeqCst);
    }).unwrap();

    assert_eq!(result_count.load(Ordering::SeqCst), 100);
    assert_eq!(summary.total_files, 100);
}
```

---

### Task 7.2: Test streaming memory usage

**Effort**: 1 hour

**Description**: Verify streaming mode uses constant memory.

**Steps**:
1. Create test with large file count (1000+)
2. Process in streaming mode
3. Process in collecting mode
4. Compare memory usage (streaming should be much lower)
5. Use memory profiling tools

**Acceptance Criteria**:
- Streaming memory usage is bounded
- Collecting mode memory grows with file count
- Streaming is significantly more memory efficient

**Dependencies**: Task 7.1

**Testing**:
```rust
#[test]
fn test_streaming_memory_efficiency() {
    let files = create_test_files(1000);

    // Measure memory during streaming
    let summary = process_streaming(files.clone(), &opts, |_| {}).unwrap();

    // Compare with collecting mode
    // (Use memory profiling or allocation tracking)
}
```

---

## Phase 8: Batch Processor

### Task 8.1: Implement BatchProcessor trait

**Effort**: 1 hour

**Description**: Define public trait for batch processing.

**Steps**:
1. In `src/processor.rs`, define `BatchProcessor` trait with methods:
   - `process(&self, job: &BatchJob) -> Result<BatchResult, BatchError>`
   - `process_with_progress<F>(&self, job: &BatchJob, progress_callback: F) -> Result<BatchResult, BatchError>`
   - `process_streaming<F>(&self, job: &BatchJob, result_callback: F) -> Result<BatchSummary, BatchError>`
2. Document trait and methods
3. Add examples to documentation

**Acceptance Criteria**:
- Trait is well-documented
- Methods have clear signatures
- Examples are provided

**Dependencies**: Task 1.2

**Testing**: N/A (trait definition)

---

### Task 8.2: Implement DefaultBatchProcessor

**Effort**: 3 hours

**Description**: Implement main batch processor orchestrating all modules.

**Steps**:
1. Define `DefaultBatchProcessor` struct
2. Implement `BatchProcessor` trait for `DefaultBatchProcessor`
3. In `process` method:
   - Discover files
   - Apply filters
   - Process in parallel
   - Build BatchResult
   - Return results
4. In `process_with_progress`:
   - Same as process, but with progress tracking
   - Invoke callback periodically
5. In `process_streaming`:
   - Use streaming module
   - Return summary only

**Acceptance Criteria**:
- All three methods work correctly
- Proper error handling throughout
- Results are accurate
- Progress callbacks work

**Dependencies**: Task 8.1, Task 2.4, Task 3.3, Task 5.2, Task 6.3, Task 7.1

**Testing**:
```rust
#[test]
fn test_batch_processor_basic() {
    let job = BatchJob::new(
        vec![PathBuf::from("tests/fixtures/dirs/simple")],
        BatchOptions::default(),
    );
    let processor = DefaultBatchProcessor::new();
    let result = processor.process(&job).unwrap();
    assert!(result.summary.total_files > 0);
}
```

---

### Task 8.3: Add cancellation support

**Effort**: 2 hours

**Description**: Allow batch jobs to be cancelled during processing.

**Steps**:
1. Add `CancellationToken` to BatchOptions or as separate parameter
2. Check token periodically during processing
3. Return `BatchError::Cancelled` if cancellation requested
4. Clean up resources on cancellation
5. Ensure partial results are discarded safely

**Acceptance Criteria**:
- Cancellation stops processing promptly
- Resources are cleaned up
- Cancelled error is returned
- No resource leaks

**Dependencies**: Task 8.2

**Testing**:
```rust
#[test]
fn test_batch_cancellation() {
    let job = BatchJob::new(
        vec![PathBuf::from("tests/fixtures/dirs/large")],
        BatchOptions::default(),
    );
    let processor = DefaultBatchProcessor::new();
    let token = CancellationToken::new();

    // Cancel after 100ms
    let token_clone = token.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(100));
        token_clone.cancel();
    });

    let result = processor.process_with_cancellation(&job, token);
    assert!(matches!(result, Err(BatchError::Cancelled)));
}
```

---

### Task 8.4: Wire up all modules in lib.rs

**Effort**: 1 hour

**Description**: Export public API from lib.rs.

**Steps**:
1. Re-export public types from each module
2. Organize exports logically
3. Add crate-level documentation with examples
4. Add usage examples
5. Document feature flags if any

**Acceptance Criteria**:
- Public API is clean and intuitive
- All necessary types are exported
- Documentation includes examples
- No unnecessary internals exposed

**Dependencies**: Task 8.2

**Testing**:
```rust
// In integration test
use veil_batch::{BatchJob, BatchOptions, DefaultBatchProcessor, BatchProcessor};

#[test]
fn test_public_api_usage() {
    let job = BatchJob::new(/* ... */);
    let processor = DefaultBatchProcessor::new();
    let result = processor.process(&job).unwrap();
    // Should compile and work
}
```

---

## Phase 9: Integration Testing

### Task 9.1: Write end-to-end integration tests

**Effort**: 3 hours

**Description**: Test complete workflows with realistic scenarios.

**Steps**:
1. Create `tests/integration_tests.rs`
2. Test scenarios:
   - Scan directory with mixed file types
   - Process ZIP archive
   - Apply include/exclude filters
   - Parallel processing with progress
   - Streaming results
   - Error handling (permission denied, corrupted files)
3. Use test fixtures extensively
4. Verify results match expectations

**Acceptance Criteria**:
- All user stories from spec.md are covered
- Tests use realistic data
- Tests are deterministic and repeatable
- Edge cases are tested

**Dependencies**: Task 8.4

**Testing**: Run `cargo test --test integration_tests`

---

### Task 9.2: Write performance benchmarks

**Effort**: 2 hours

**Description**: Measure performance against success criteria.

**Steps**:
1. Create benchmark for 10,000 files (SC-001)
2. Create benchmark for 1GB ZIP archive (SC-002)
3. Measure parallel speedup (SC-006)
4. Measure progress update frequency (SC-004)
5. Use criterion crate for accurate measurements
6. Document results

**Acceptance Criteria**:
- 10,000 files process in <10 minutes
- 1GB ZIP doesn't exhaust memory
- Parallel speedup is near-linear up to 8 cores
- Progress updates >= 1/second

**Dependencies**: Task 9.1

**Testing**: Run benchmarks and verify against success criteria

---

### Task 9.3: Test error handling scenarios

**Effort**: 2 hours

**Description**: Verify graceful error handling for all failure modes.

**Steps**:
1. Test permission denied errors
2. Test corrupted file handling
3. Test invalid passwords
4. Test depth limit exceeded
5. Test file size limit exceeded
6. Test invalid glob patterns
7. Verify batch continues after errors
8. Verify errors are reported in results

**Acceptance Criteria**:
- All error cases handled gracefully
- Batch doesn't crash on errors
- Errors are reported to user
- Processing continues after errors

**Dependencies**: Task 8.4

**Testing**:
```rust
#[test]
fn test_continue_on_error() {
    // Create directory with mix of valid and invalid files
    let result = processor.process(&job).unwrap();
    assert!(result.errors.len() > 0);
    assert!(result.successful.len() > 0);
}
```

---

## Phase 10: Documentation & Polish

### Task 10.1: Add documentation comments

**Effort**: 2 hours

**Description**: Document all public API items with examples.

**Steps**:
1. Add `///` doc comments to all public items
2. Include usage examples in doc comments
3. Document error conditions
4. Document performance characteristics
5. Add module-level documentation
6. Cross-reference related items

**Acceptance Criteria**:
- All public items have doc comments
- Examples are included and tested
- Documentation is clear and helpful
- `cargo doc` builds without warnings

**Dependencies**: Task 8.4

**Testing**: Run `cargo doc --no-deps --open` and review

---

### Task 10.2: Create usage examples

**Effort**: 1 hour

**Description**: Add example programs demonstrating key features.

**Steps**:
1. Create `examples/basic_scan.rs` - simple directory scan
2. Create `examples/parallel_processing.rs` - parallel processing with progress
3. Create `examples/archive_processing.rs` - process ZIP files
4. Create `examples/streaming.rs` - streaming results
5. Ensure examples run successfully

**Acceptance Criteria**:
- All examples compile and run
- Examples demonstrate key features
- Examples include comments explaining usage
- Examples use realistic scenarios

**Dependencies**: Task 10.1

**Testing**: Run `cargo run --example basic_scan`

---

### Task 10.3: Run clippy and fix warnings

**Effort**: 1 hour

**Description**: Ensure code passes clippy lints.

**Steps**:
1. Run `cargo clippy -- -D warnings`
2. Fix all warnings
3. Enable pedantic lints where appropriate
4. Document any allowed lints with justification

**Acceptance Criteria**:
- No clippy warnings
- Code follows Rust idioms
- Allowed lints are documented

**Dependencies**: Task 8.4

**Testing**: Run `cargo clippy -- -D warnings`

---

### Task 10.4: Run rustfmt

**Effort**: 15 minutes

**Description**: Format all code consistently.

**Steps**:
1. Run `cargo fmt`
2. Verify formatting is consistent
3. Add rustfmt.toml if custom formatting needed
4. Check formatting in CI

**Acceptance Criteria**:
- All code is formatted
- Formatting is consistent
- `cargo fmt --check` passes

**Dependencies**: Task 10.3

**Testing**: Run `cargo fmt --check`

---

### Task 10.5: Update workspace documentation

**Effort**: 30 minutes

**Description**: Update project-level documentation.

**Steps**:
1. Update root `README.md` to mention veil-batch
2. Update `CLAUDE.md` with veil-batch info
3. Add veil-batch to project structure diagram
4. Document testing strategy
5. Document commands for batch crate

**Acceptance Criteria**:
- README mentions veil-batch
- CLAUDE.md includes veil-batch details
- Documentation is accurate

**Dependencies**: Task 10.1

**Testing**: Review documentation for accuracy

---

## Success Criteria Validation

After completing all tasks, verify these criteria from spec.md:

- [ ] **SC-001**: Directory with 10,000 files scanned in <10 minutes with parallel processing
- [ ] **SC-002**: ZIP archives up to 1GB processed without memory exhaustion
- [ ] **SC-003**: File filtering reduces processing to only matching files with 100% accuracy
- [ ] **SC-004**: Progress reporting updates at least every 1 second during active processing
- [ ] **SC-005**: Aggregate reports correctly sum per-file findings
- [ ] **SC-006**: Parallel processing achieves near-linear speedup up to 8 cores

---

## Task Dependencies Summary

```
Phase 0: Setup
  0.1 → 0.2 → 0.3
  0.1 → 0.4

Phase 1: Core Types
  0.3 → 1.1 → 1.2 → 1.3 → 1.4
           ↘  3.1

Phase 2: Discovery
  1.2 → 2.1 → 2.2 → 2.3 → 2.4

Phase 3: Filtering
  1.1 → 3.1 → 3.2 → 3.3
  2.4 → 3.3

Phase 4: Archives
  1.2 → 4.1 → 4.2 → 4.3 → 4.4
                       ↓
Phase 5: Parallel      ↓
  1.2 → 5.1 → 5.2 ← 5.3 ← 4.3
           ↓     ↓
           5.4   6.2

Phase 6: Progress
  1.2 → 6.1 → 6.2 → 6.3
  5.2 → 6.2

Phase 7: Streaming
  5.2 → 7.1 → 7.2

Phase 8: Processor
  1.2 → 8.1 → 8.2 → 8.3 → 8.4
  2.4, 3.3, 5.2, 6.3, 7.1 → 8.2

Phase 9: Integration
  8.4 → 9.1 → 9.2, 9.3

Phase 10: Documentation
  8.4 → 10.1 → 10.2, 10.3 → 10.4 → 10.5
```

---

## Estimated Timeline

- **Phase 0**: 2.5 hours
- **Phase 1**: 4.5 hours
- **Phase 2**: 6 hours
- **Phase 3**: 4.5 hours
- **Phase 4**: 6 hours
- **Phase 5**: 7 hours
- **Phase 6**: 4 hours
- **Phase 7**: 3 hours
- **Phase 8**: 7 hours
- **Phase 9**: 7 hours
- **Phase 10**: 4.75 hours

**Total**: ~56 hours

---

## Notes

- Tasks are designed to be completed in order within each phase
- Some tasks can be parallelized across phases if multiple developers are working
- Each task includes specific acceptance criteria and testing requirements
- All tasks follow TDD principles from constitution
- Integration tests validate against spec.md user stories
- Performance tests validate against spec.md success criteria
