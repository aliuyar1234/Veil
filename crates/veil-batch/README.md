# veil-batch

Batch processing orchestration for Veil PII detection.

## Features

- **Recursive Directory Scanning**: Walk directory trees with configurable depth limits
- **ZIP Archive Processing**: Extract and process files from ZIP archives with password support
- **Parallel Processing**: Process multiple files concurrently using Rayon's work-stealing scheduler
- **Progress Tracking**: Real-time progress reporting with ETA calculation
- **Streaming Results**: Memory-efficient processing for large batches via callbacks
- **Glob Filtering**: Include/exclude files using shell-style glob patterns
- **Format Detection**: Automatic file format detection using magic bytes and extensions
- **Graceful Error Handling**: Continue processing on errors, collect failures for reporting

## Supported File Formats

- Plain text (`.txt`, `.log`)
- CSV/TSV (`.csv`, `.tsv`)
- JSON (`.json`)
- HTML (`.html`, `.htm`)
- PDF (`.pdf`)

## Usage

### Basic Batch Processing

```rust
use veil_batch::{BatchJob, BatchOptions, DefaultBatchProcessor, BatchProcessor};
use std::path::PathBuf;

let sources = vec![PathBuf::from("./data")];
let options = BatchOptions::default();
let job = BatchJob::new(sources, options);

let processor = DefaultBatchProcessor::new();
let result = processor.process(&job)?;

println!("Processed {} files", result.summary.successful);
println!("Failed: {}", result.summary.failed);
println!("Skipped: {}", result.summary.skipped);
```

### With Progress Tracking

```rust
use veil_batch::{BatchJob, BatchOptions, DefaultBatchProcessor, BatchProcessor};
use std::path::PathBuf;

let sources = vec![PathBuf::from("./data")];
let options = BatchOptions::default();
let job = BatchJob::new(sources, options);

let processor = DefaultBatchProcessor::new();
let result = processor.process_with_progress(&job, |progress| {
    println!(
        "Progress: {}/{} ({:.1}%)",
        progress.processed,
        progress.total,
        progress.percentage()
    );
})?;
```

### Streaming Results (Memory Efficient)

```rust
use veil_batch::{BatchJob, BatchOptions, DefaultBatchProcessor, BatchProcessor};
use std::path::PathBuf;

let sources = vec![PathBuf::from("./data")];
let options = BatchOptions::default();
let job = BatchJob::new(sources, options);

let processor = DefaultBatchProcessor::new();
let summary = processor.process_streaming(&job, |result| {
    println!("Processed: {:?}", result.path);
    println!("  Segments: {}", result.result.segments.len());
    println!("  Characters: {}", result.result.total_chars);
})?;

println!("Total files: {}", summary.total_files);
```

### Filtering Files

```rust
use veil_batch::{BatchJob, BatchOptions};
use std::path::PathBuf;

let mut options = BatchOptions::default();

// Include only CSV files
options.include_patterns = vec!["*.csv".to_string()];

// Exclude test files
options.exclude_patterns = vec!["test_*.csv".to_string()];

let sources = vec![PathBuf::from("./data")];
let job = BatchJob::new(sources, options);
```

### Processing Archives

```rust
use veil_batch::{BatchJob, BatchOptions};
use std::path::PathBuf;

let mut options = BatchOptions::default();

// Set password for encrypted archives
options.archive_password = Some("secret".to_string());

// Limit archive nesting depth (default: 5)
options.max_archive_depth = 3;

let sources = vec![PathBuf::from("./archive.zip")];
let job = BatchJob::new(sources, options);
```

### Configuration Options

```rust
use veil_batch::BatchOptions;

let mut options = BatchOptions::default();

// Parallel processing
options.max_parallelism = Some(4); // Use 4 threads (default: CPU count - 1)

// Directory traversal
options.recursive = true;          // Recursive scanning (default: true)
options.follow_symlinks = true;    // Follow symlinks (default: true)
options.max_depth = Some(5);       // Limit depth (default: unlimited)

// File size limits
options.max_file_size = 50 * 1024 * 1024; // 50MB (default: 100MB)

// Archive processing
options.max_archive_depth = 5;     // Max ZIP nesting (default: 5)
options.archive_password = None;   // Password for encrypted archives
```

## Architecture

The crate is organized into focused modules:

- `types.rs`: Core data structures (BatchJob, BatchOptions, BatchResult, etc.)
- `error.rs`: Error types using thiserror
- `discovery.rs`: File discovery with walkdir
- `filter.rs`: Glob-based file filtering
- `archive.rs`: ZIP archive extraction and processing
- `progress.rs`: Thread-safe progress tracking with atomic counters
- `parallel.rs`: Rayon-based parallel processing
- `streaming.rs`: Memory-efficient streaming results via channels
- `processor.rs`: Main batch processor orchestrating all components

## Performance

- **Parallel Processing**: Near-linear speedup up to 8 cores using Rayon
- **Memory Efficiency**: Streaming mode processes arbitrary batch sizes with constant memory
- **Progress Reporting**: Lock-free atomic counters for minimal overhead
- **ZIP Bomb Prevention**: Depth limits and size checks prevent resource exhaustion

## Safety

- No `unsafe` code
- Result-based error handling throughout
- Graceful degradation on errors (continue processing)
- Archive depth limits to prevent ZIP bombs
- File size limits to prevent memory exhaustion
- Thread-safe progress tracking with Arc and Mutex

## Dependencies

- `walkdir`: Safe recursive directory traversal
- `zip`: ZIP archive extraction with password support
- `rayon`: Work-stealing parallelism
- `glob`: Shell-style pattern matching
- `infer`: Magic byte file type detection
- `veil-parsers`: Document parsing (local crate)

## License

MIT OR Apache-2.0
