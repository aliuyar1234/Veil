# Research: Batch Processing

**Feature**: 014-batch-processing
**Date**: 2025-12-15

## 1. Directory Traversal Library

**Decision**: Use `walkdir` crate (v2.5+)

**Rationale**:
- Industry standard for directory walking in Rust
- Efficient recursive traversal with configurable depth
- Built-in symlink handling with cycle detection
- Lazy iteration for memory efficiency
- Cross-platform support (Windows, Unix)
- Actively maintained by BurntSushi

**Alternatives Considered**:
- `std::fs::read_dir`: Requires manual recursion; no symlink safety
- `jwalk`: Parallel traversal but overkill for most use cases; adds complexity
- `ignore`: Designed for git-aware traversal; too specialized

**Configuration**:
```toml
[dependencies]
walkdir = "2.5"
```

**Usage Pattern**:
```rust
use walkdir::WalkDir;

let walker = WalkDir::new(path)
    .follow_links(true)
    .max_depth(100)
    .into_iter()
    .filter_entry(|e| !is_excluded(e));

for entry in walker.filter_map(|e| e.ok()) {
    if entry.file_type().is_file() {
        // Process file
    }
}
```

## 2. ZIP Archive Handling

**Decision**: Use `zip` crate (v0.6+)

**Rationale**:
- Pure Rust implementation with no C dependencies
- Read and write support for ZIP archives
- Password-protected archive support (AES-256)
- Streaming decompression for memory efficiency
- Supports ZIP64 for large archives
- Wide adoption in Rust ecosystem

**Alternatives Considered**:
- `zip-rs`: Same crate, older name
- `compress-tools`: Supports multiple formats but requires external tools
- `libarchive-sys`: FFI bindings, adds platform dependencies

**Configuration**:
```toml
[dependencies]
zip = { version = "0.6", features = ["deflate", "aes-crypto"] }
```

**Usage Pattern**:
```rust
use zip::ZipArchive;
use std::io::Read;

let file = File::open("archive.zip")?;
let mut archive = ZipArchive::new(file)?;

for i in 0..archive.len() {
    let mut file = archive.by_index(i)?;

    if file.is_file() {
        let mut contents = Vec::new();
        file.read_to_end(&mut contents)?;
        // Process file contents
    }
}
```

**Password Protection**:
```rust
let mut file = archive.by_index_decrypt(i, password.as_bytes())?
    .map_err(|_| BatchError::InvalidPassword)?;
```

## 3. Parallel Processing

**Decision**: Use `rayon` crate (v1.10+)

**Rationale**:
- Constitution-approved for data parallelism
- Work-stealing scheduler for efficient load balancing
- Simple parallel iterator API
- Thread pool management built-in
- Configurable thread count
- Zero-cost for sequential execution when not needed

**Alternatives Considered**:
- `tokio`: Async runtime; not needed for CPU-bound tasks
- Manual threading: Complex, error-prone
- `threadpool`: Lower level, requires manual work distribution

**Configuration**:
```toml
[dependencies]
rayon = "1.10"
```

**Usage Pattern**:
```rust
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;

let pool = ThreadPoolBuilder::new()
    .num_threads(num_threads)
    .build()?;

pool.install(|| {
    files.par_iter()
        .map(|file| process_file(file))
        .collect()
});
```

**Progress Tracking**:
```rust
use std::sync::atomic::{AtomicUsize, Ordering};

let counter = AtomicUsize::new(0);
let results: Vec<_> = files.par_iter()
    .map(|file| {
        let result = process_file(file);
        let current = counter.fetch_add(1, Ordering::Relaxed);
        progress_callback(current, total);
        result
    })
    .collect();
```

## 4. Glob Pattern Matching

**Decision**: Use `glob` crate (v0.3+)

**Rationale**:
- Standard glob pattern syntax (`*.csv`, `**/*.txt`)
- Shell-style pattern matching
- Cross-platform path handling
- Simple API for include/exclude filtering
- Well-tested and stable

**Alternatives Considered**:
- `globset`: More features but heavier; designed for `.gitignore` use cases
- `regex`: Too low-level for file patterns; not user-friendly
- Manual pattern matching: Error-prone for edge cases

**Configuration**:
```toml
[dependencies]
glob = "0.3"
```

**Usage Pattern**:
```rust
use glob::Pattern;

let include_pattern = Pattern::new("*.csv")?;
let exclude_pattern = Pattern::new("*.log")?;

for entry in walker {
    let path = entry.path();
    let filename = path.file_name().unwrap().to_str().unwrap();

    if include_pattern.matches(filename) && !exclude_pattern.matches(filename) {
        // Process file
    }
}
```

**Complex Patterns**:
```rust
// Supports `**` for recursive matching
let pattern = Pattern::new("**/reports/*.pdf")?;

// Check full path against pattern
if pattern.matches_path(path) {
    // Process file
}
```

## 5. Progress Reporting

**Decision**: Use `indicatif` crate (v0.17+)

**Rationale**:
- Constitution-approved for CLI progress UI
- Multiple progress bar styles (spinner, bar, multi-progress)
- Automatic terminal detection (TTY vs piped output)
- ETA calculation based on throughput
- Thread-safe progress updates
- Rich formatting with templates

**Alternatives Considered**:
- `pbr`: Simpler but less feature-rich
- Manual printing: No ETA, no TTY detection
- `tqdm-rs`: Less maintained than indicatif

**Configuration**:
```toml
[dependencies]
indicatif = "0.17"
```

**Usage Pattern**:
```rust
use indicatif::{ProgressBar, ProgressStyle};

let pb = ProgressBar::new(total as u64);
pb.set_style(
    ProgressStyle::default_bar()
        .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) ETA: {eta}")
        .unwrap()
        .progress_chars("#>-")
);

for file in files {
    process_file(file);
    pb.inc(1);
}

pb.finish_with_message("Scan complete");
```

**Multi-Progress (Parallel)**:
```rust
use indicatif::MultiProgress;

let multi = MultiProgress::new();
let pb_main = multi.add(ProgressBar::new(total));
let pb_current = multi.add(ProgressBar::new_spinner());

pb_current.set_message(format!("Processing: {}", current_file));
pb_main.inc(1);
```

## 6. File Type Detection

**Decision**: Use `infer` crate (v0.15+) with fallback to extension

**Rationale**:
- Magic byte detection for accurate type identification
- Supports 100+ file types
- Zero dependencies
- Fast detection (first 8KB of file)
- Works when extensions are missing or wrong

**Alternatives Considered**:
- Extension-only: Unreliable when files renamed
- `mime_guess`: Extension-based only
- `tree_magic`: Requires complex MIME database

**Configuration**:
```toml
[dependencies]
infer = "0.15"
```

**Usage Pattern**:
```rust
use infer;

fn detect_format(path: &Path) -> Result<FileFormat> {
    let mut file = File::open(path)?;
    let mut buffer = [0u8; 8192];
    file.read(&mut buffer)?;

    if let Some(kind) = infer::get(&buffer) {
        match kind.mime_type() {
            "application/pdf" => Ok(FileFormat::Pdf),
            "application/zip" => Ok(FileFormat::Zip),
            "text/plain" => Ok(FileFormat::Text),
            _ => fallback_to_extension(path),
        }
    } else {
        fallback_to_extension(path)
    }
}
```

## 7. Error Aggregation Strategy

**Decision**: Continue on errors, collect failures for summary report

**Rationale**:
- Spec requires graceful handling of file access errors (FR-008)
- One corrupted file shouldn't stop entire batch
- Users need visibility into what failed and why
- Matches Unix tool philosophy (partial success)

**Pattern**:
```rust
use std::sync::Mutex;

#[derive(Debug)]
pub struct FileError {
    pub path: PathBuf,
    pub error: BatchError,
}

pub struct BatchResult {
    pub processed: Vec<FileResult>,
    pub failed: Vec<FileError>,
    pub skipped: Vec<PathBuf>,
}

let failures = Mutex::new(Vec::new());

files.par_iter().for_each(|file| {
    match process_file(file) {
        Ok(result) => results.push(result),
        Err(e) => {
            failures.lock().unwrap().push(FileError {
                path: file.clone(),
                error: e,
            });
        }
    }
});
```

## 8. Streaming Results

**Decision**: Use callback pattern for result streaming

**Rationale**:
- Spec requires streaming output for large batches (FR-011)
- Avoid holding all results in memory
- Enables real-time processing of findings
- Works with parallel processing via channels

**Pattern**:
```rust
use std::sync::mpsc::channel;

pub type ResultCallback = Box<dyn Fn(FileResult) + Send + Sync>;

pub fn process_batch_streaming<F>(
    paths: Vec<PathBuf>,
    options: &BatchOptions,
    callback: F,
) -> Result<BatchSummary>
where
    F: Fn(FileResult) + Send + Sync + Clone + 'static,
{
    let (tx, rx) = channel();

    let callback_clone = callback.clone();
    let handle = std::thread::spawn(move || {
        for result in rx {
            callback_clone(result);
        }
    });

    paths.par_iter().for_each(|path| {
        if let Ok(result) = process_file(path) {
            tx.send(result).ok();
        }
    });

    drop(tx);
    handle.join().unwrap();

    Ok(BatchSummary { /* stats */ })
}
```

## 9. Nested Archive Handling

**Decision**: Recursive extraction with depth limit

**Rationale**:
- Spec requires processing nested ZIPs (User Story 3)
- Depth limit prevents ZIP bombs
- Temporary extraction to avoid excessive memory use

**Pattern**:
```rust
const MAX_ARCHIVE_DEPTH: usize = 5;

fn process_zip_recursive(
    path: &Path,
    depth: usize,
    options: &BatchOptions,
) -> Result<Vec<FileResult>> {
    if depth > MAX_ARCHIVE_DEPTH {
        return Err(BatchError::ArchiveDepthExceeded);
    }

    let mut archive = ZipArchive::new(File::open(path)?)?;
    let mut results = Vec::new();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name().to_string();

        if name.ends_with(".zip") {
            // Extract to temp and recurse
            let temp_path = extract_to_temp(&mut file)?;
            let nested = process_zip_recursive(&temp_path, depth + 1, options)?;
            results.extend(nested);
        } else if is_supported_format(&name) {
            // Process directly from archive
            let result = process_archive_file(&mut file, &name)?;
            results.push(result);
        }
    }

    Ok(results)
}
```

## 10. Cancellation Support

**Decision**: Use `AtomicBool` for cooperative cancellation

**Rationale**:
- Spec requires cancellation support (FR-010)
- Thread-safe, lock-free cancellation
- Works with rayon parallel iterators
- Clean shutdown without orphaned threads

**Pattern**:
```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct BatchJob {
    cancelled: Arc<AtomicBool>,
}

impl BatchJob {
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }

    pub fn run(&self, files: Vec<PathBuf>) -> Result<BatchResult> {
        let cancelled = self.cancelled.clone();

        let results: Vec<_> = files
            .par_iter()
            .take_any_while(|_| !cancelled.load(Ordering::Relaxed))
            .map(|file| process_file(file))
            .collect();

        if cancelled.load(Ordering::Relaxed) {
            Err(BatchError::Cancelled)
        } else {
            Ok(BatchResult { results, .. })
        }
    }
}
```

## Summary of Decisions

| Component | Choice | Crate Version |
|-----------|--------|---------------|
| Directory traversal | walkdir | 2.5+ |
| ZIP handling | zip | 0.6+ |
| Parallel processing | rayon | 1.10+ |
| Glob patterns | glob | 0.3+ |
| Progress UI | indicatif | 0.17+ |
| File type detection | infer | 0.15+ |
| Error handling | thiserror | 1.0+ |
| Serialization | serde | 1.0+ |

## Open Questions Resolved

1. **Q: How to handle symbolic links?**
   A: Follow symlinks by default with cycle detection; `--no-follow-symlinks` flag to disable

2. **Q: How to prevent ZIP bombs?**
   A: Enforce max archive depth (5 levels), max extracted size per file, max total extracted size

3. **Q: How to handle progress with parallel processing?**
   A: Use atomic counter shared across threads; update progress bar from main thread

4. **Q: How to integrate with existing parsers?**
   A: veil-batch depends on veil-parsers; dispatches to appropriate parser based on file type

5. **Q: How to handle very large directories (millions of files)?**
   A: Stream processing with callback pattern; don't collect all results in memory

6. **Q: How to handle file access permissions?**
   A: Catch and log permission errors; add to failed files list; continue with other files
