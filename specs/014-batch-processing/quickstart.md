# Quickstart: veil-batch

This guide shows how to use the Veil batch processing library to scan directories and archives for PII.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
veil-batch = "0.1"
veil-parsers = "0.1"
```

## Basic Usage

### Scan a Directory

```rust
use veil_batch::{BatchJob, BatchOptions, DefaultBatchProcessor, BatchProcessor};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a batch job
    let job = BatchJob::new(vec![PathBuf::from("./data")], BatchOptions::default());

    // Process the batch
    let processor = DefaultBatchProcessor::new();
    let result = processor.process(&job)?;

    // Print summary
    println!("Processed: {}", result.summary.processed);
    println!("Failed: {}", result.summary.failed);
    println!("Skipped: {}", result.summary.skipped);
    println!("Duration: {}ms", result.summary.duration_ms);

    Ok(())
}
```

### Scan with Progress Reporting

```rust
use veil_batch::{BatchJob, BatchOptions, DefaultBatchProcessor, BatchProcessor};
use std::path::PathBuf;

let job = BatchJob::new(
    vec![PathBuf::from("./data")],
    BatchOptions::default(),
);

let processor = DefaultBatchProcessor::new();
let result = processor.process_with_progress(&job, |progress| {
    println!(
        "[{}/{}] {:.1}% - ETA: {}s - Current: {}",
        progress.processed,
        progress.total,
        progress.percent,
        progress.eta_seconds.unwrap_or(0),
        progress.current_file
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_default()
    );
})?;
```

### Filter Files with Patterns

```rust
use veil_batch::{BatchJob, BatchOptions};
use std::path::PathBuf;

let options = BatchOptions {
    // Only process CSV files
    include: vec!["*.csv".to_string()],
    // Exclude logs
    exclude: vec!["*.log".to_string()],
    ..Default::default()
};

let job = BatchJob::new(vec![PathBuf::from("./data")], options);
```

### Complex Glob Patterns

```rust
let options = BatchOptions {
    // Only PDFs in reports subdirectories
    include: vec!["**/reports/*.pdf".to_string()],
    exclude: vec!["**/archive/**".to_string()],
    ..Default::default()
};
```

## Archive Processing

### Process ZIP Files

```rust
use veil_batch::{BatchJob, BatchOptions};
use std::path::PathBuf;

let options = BatchOptions {
    process_archives: true,
    ..Default::default()
};

let job = BatchJob::new(vec![PathBuf::from("export.zip")], options);
let result = processor.process(&job)?;

// All files inside the ZIP are processed
println!("Files in archive: {}", result.summary.processed);
```

### Password-Protected Archives

```rust
let options = BatchOptions {
    process_archives: true,
    archive_password: Some("secret123".to_string()),
    ..Default::default()
};

let job = BatchJob::new(vec![PathBuf::from("encrypted.zip")], options);
```

### Nested Archives

```rust
let options = BatchOptions {
    process_archives: true,
    // Limit nesting depth to prevent ZIP bombs
    max_archive_depth: 5,
    ..Default::default()
};

let job = BatchJob::new(vec![PathBuf::from("nested.zip")], options);
```

## Parallel Processing

### Configure Thread Count

```rust
let options = BatchOptions {
    // Use 8 threads
    parallelism: 8,
    ..Default::default()
};

let job = BatchJob::new(vec![PathBuf::from("./data")], options);
```

### Sequential Processing

```rust
let options = BatchOptions {
    // Process files one at a time
    parallelism: 1,
    ..Default::default()
};
```

### Automatic Parallelism

```rust
// Uses (num_cpus - 1) threads by default
let options = BatchOptions::default();
println!("Using {} threads", options.parallelism);
```

## Streaming Results

For very large batches, stream results instead of collecting them all in memory:

```rust
use veil_batch::{BatchJob, BatchOptions, DefaultBatchProcessor, BatchProcessor};
use std::fs::File;
use std::io::Write;

let options = BatchOptions {
    streaming: true,
    ..Default::default()
};

let job = BatchJob::new(vec![PathBuf::from("./large-dataset")], options);

let mut output = File::create("results.jsonl")?;

let summary = processor.process_streaming(&job, |file_result| {
    // Write each result as it's processed
    let json = serde_json::to_string(&file_result).unwrap();
    writeln!(output, "{}", json).ok();
})?;

println!("Processed {} files", summary.processed);
```

## Directory Traversal Options

### Recursive Scanning

```rust
let options = BatchOptions {
    recursive: true,
    max_depth: 10,  // Limit recursion depth
    ..Default::default()
};
```

### Non-Recursive

```rust
let options = BatchOptions {
    recursive: false,  // Only scan the top-level directory
    ..Default::default()
};
```

### Symlink Handling

```rust
let options = BatchOptions {
    follow_symlinks: true,  // Follow symlinks (default)
    ..Default::default()
};

// Or disable to avoid symlink loops
let options = BatchOptions {
    follow_symlinks: false,
    ..Default::default()
};
```

## Error Handling

### Accessing Failed Files

```rust
let result = processor.process(&job)?;

for error in &result.failed {
    eprintln!(
        "Failed to process {}: {} ({})",
        error.path.display(),
        error.message,
        error.error
    );
}
```

### Accessing Skipped Files

```rust
for skipped in &result.skipped {
    println!(
        "Skipped {}: {:?}",
        skipped.path.display(),
        skipped.reason
    );
}
```

### Graceful Degradation

```rust
let result = processor.process(&job)?;

let success_rate = (result.summary.processed as f64 /
                    result.summary.total_files as f64) * 100.0;

if success_rate < 90.0 {
    eprintln!("Warning: Only {:.1}% of files processed successfully", success_rate);
}
```

## Aggregate Results

### Findings Summary

```rust
let result = processor.process(&job)?;

println!("Total findings: {}", result.summary.findings_summary.total_findings);
println!("Files with findings: {}", result.summary.findings_summary.files_with_findings);

for (category, count) in &result.summary.findings_summary.by_category {
    println!("  {}: {}", category, count);
}
```

### Per-File Results

```rust
let result = processor.process(&job)?;

for file_result in &result.file_results {
    println!(
        "{}: {} segments in {}ms",
        file_result.file.path.display(),
        file_result.parse_result.segments.len(),
        file_result.duration_ms
    );

    for warning in &file_result.warnings {
        eprintln!("  Warning: {}", warning);
    }
}
```

## Job Cancellation

```rust
use std::sync::Arc;
use std::thread;
use std::time::Duration;

let job = Arc::new(BatchJob::new(
    vec![PathBuf::from("./large-dataset")],
    BatchOptions::default(),
));

// Clone for cancellation thread
let job_clone = job.clone();

// Start processing in background
let handle = thread::spawn(move || {
    processor.process(&job_clone)
});

// Cancel after 30 seconds
thread::sleep(Duration::from_secs(30));
job.cancel();

// Wait for graceful shutdown
match handle.join().unwrap() {
    Ok(result) => println!("Completed: {} files", result.summary.processed),
    Err(e) => println!("Cancelled: {}", e),
}
```

## Output Formats

### JSON (Default)

```rust
let options = BatchOptions {
    output_format: OutputFormat::Json,
    ..Default::default()
};

let result = processor.process(&job)?;
let json = serde_json::to_string_pretty(&result)?;
println!("{}", json);
```

### JSON Lines (Streaming)

```rust
let options = BatchOptions {
    output_format: OutputFormat::JsonLines,
    streaming: true,
    ..Default::default()
};

processor.process_streaming(&job, |file_result| {
    let json = serde_json::to_string(&file_result).unwrap();
    println!("{}", json);
})?;
```

### CSV Export

```rust
let options = BatchOptions {
    output_format: OutputFormat::Csv,
    ..Default::default()
};

// Results can be exported to CSV format
let result = processor.process(&job)?;
result.to_csv("results.csv")?;
```

## Size Limits

### Set Maximum File Size

```rust
let options = BatchOptions {
    // Skip files larger than 50MB
    max_file_size: 50 * 1024 * 1024,
    ..Default::default()
};
```

### Handle Large Files

```rust
let result = processor.process(&job)?;

for skipped in &result.skipped {
    if skipped.reason == SkipReason::TooLarge {
        println!("File too large: {}", skipped.path.display());
    }
}
```

## Performance Tips

1. **Adjust parallelism**: More threads for I/O-bound workloads
   ```rust
   let options = BatchOptions {
       parallelism: 16,  // Increase for network/cloud storage
       ..Default::default()
   };
   ```

2. **Use streaming for large batches**: Prevent memory exhaustion
   ```rust
   let options = BatchOptions {
       streaming: true,
       ..Default::default()
   };
   ```

3. **Filter early**: Use include/exclude patterns to reduce processing
   ```rust
   let options = BatchOptions {
       include: vec!["*.csv".to_string(), "*.json".to_string()],
       ..Default::default()
   };
   ```

4. **Limit directory depth**: Faster discovery for shallow hierarchies
   ```rust
   let options = BatchOptions {
       recursive: true,
       max_depth: 3,
       ..Default::default()
   };
   ```

## CLI Integration Example

```rust
use veil_batch::{BatchJob, BatchOptions};
use clap::Parser;

#[derive(Parser)]
struct Args {
    /// Input paths
    #[arg(required = true)]
    paths: Vec<PathBuf>,

    /// Include pattern
    #[arg(long)]
    include: Vec<String>,

    /// Exclude pattern
    #[arg(long)]
    exclude: Vec<String>,

    /// Number of parallel threads
    #[arg(long, short = 'j')]
    parallelism: Option<usize>,

    /// Disable progress bar
    #[arg(long)]
    no_progress: bool,

    /// Password for encrypted archives
    #[arg(long)]
    password: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let options = BatchOptions {
        include: if args.include.is_empty() {
            vec!["*".to_string()]
        } else {
            args.include
        },
        exclude: args.exclude,
        parallelism: args.parallelism.unwrap_or_else(|| {
            num_cpus::get().saturating_sub(1).max(1)
        }),
        progress: !args.no_progress,
        archive_password: args.password,
        ..Default::default()
    };

    let job = BatchJob::new(args.paths, options);
    let processor = DefaultBatchProcessor::new();

    let result = if options.progress {
        processor.process_with_progress(&job, |progress| {
            println!(
                "[{}/{}] {:.1}%",
                progress.processed,
                progress.total,
                progress.percent
            );
        })?
    } else {
        processor.process(&job)?
    };

    println!("{}", serde_json::to_string_pretty(&result)?);

    Ok(())
}
```

## Testing Example

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_batch_processing() -> Result<(), Box<dyn std::error::Error>> {
        // Create temporary directory with test files
        let temp = TempDir::new()?;
        fs::write(temp.path().join("data.csv"), "name,email\nJohn,john@example.com")?;
        fs::write(temp.path().join("notes.txt"), "Contact: alice@example.com")?;

        // Process batch
        let job = BatchJob::new(
            vec![temp.path().to_path_buf()],
            BatchOptions::default(),
        );

        let processor = DefaultBatchProcessor::new();
        let result = processor.process(&job)?;

        assert_eq!(result.summary.processed, 2);
        assert_eq!(result.summary.failed, 0);

        Ok(())
    }
}
```

## Common Patterns

### Scan Multiple Directories

```rust
let job = BatchJob::new(
    vec![
        PathBuf::from("./data"),
        PathBuf::from("./backups"),
        PathBuf::from("./exports"),
    ],
    BatchOptions::default(),
);
```

### Scan Mixed Files and Directories

```rust
let job = BatchJob::new(
    vec![
        PathBuf::from("./data"),           // Directory
        PathBuf::from("./export.zip"),     // Archive
        PathBuf::from("./report.csv"),     // Single file
    ],
    BatchOptions::default(),
);
```

### Combine with Detection

```rust
// After batch processing, analyze findings
let result = processor.process(&job)?;

let high_risk_files: Vec<_> = result.file_results
    .iter()
    .filter(|r| r.parse_result.segments.len() > 100)  // Many potential PII fields
    .map(|r| &r.file.path)
    .collect();

println!("High-risk files: {:?}", high_risk_files);
```
