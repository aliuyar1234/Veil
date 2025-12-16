//! # veil-batch
//!
//! Batch processing orchestration for Veil PII detection.
//!
//! This crate provides functionality for:
//! - Recursive directory scanning
//! - ZIP archive processing (with password support)
//! - Parallel file processing
//! - Progress tracking and reporting
//! - Streaming results for memory efficiency
//! - Glob-based file filtering
//!
//! ## Example
//!
//! ```rust,no_run
//! use veil_batch::{BatchJob, BatchOptions, DefaultBatchProcessor, BatchProcessor};
//! use std::path::PathBuf;
//!
//! let sources = vec![PathBuf::from("./data")];
//! let options = BatchOptions::default();
//! let job = BatchJob::new(sources, options);
//!
//! let processor = DefaultBatchProcessor::new();
//! let result = processor.process(&job).unwrap();
//!
//! println!("Processed {} files", result.summary.successful);
//! println!("Failed: {}", result.summary.failed);
//! println!("Skipped: {}", result.summary.skipped);
//! ```
//!
//! ## Progress Tracking
//!
//! ```rust,no_run
//! use veil_batch::{BatchJob, BatchOptions, DefaultBatchProcessor, BatchProcessor};
//! use std::path::PathBuf;
//!
//! let sources = vec![PathBuf::from("./data")];
//! let options = BatchOptions::default();
//! let job = BatchJob::new(sources, options);
//!
//! let processor = DefaultBatchProcessor::new();
//! let result = processor.process_with_progress(&job, |progress| {
//!     println!(
//!         "Progress: {}/{} ({:.1}%)",
//!         progress.processed,
//!         progress.total,
//!         progress.percentage()
//!     );
//! }).unwrap();
//! ```
//!
//! ## Streaming Results
//!
//! ```rust,no_run
//! use veil_batch::{BatchJob, BatchOptions, DefaultBatchProcessor, BatchProcessor};
//! use std::path::PathBuf;
//!
//! let sources = vec![PathBuf::from("./data")];
//! let options = BatchOptions::default();
//! let job = BatchJob::new(sources, options);
//!
//! let processor = DefaultBatchProcessor::new();
//! let summary = processor.process_streaming(&job, |result| {
//!     println!("Processed: {:?}", result.path);
//!     println!("  Segments: {}", result.result.segments.len());
//!     println!("  Characters: {}", result.result.total_chars);
//! }).unwrap();
//!
//! println!("Total files: {}", summary.total_files);
//! ```

mod archive;
mod discovery;
mod error;
mod filter;
mod parallel;
mod processor;
mod progress;
mod streaming;
mod types;

// Re-export public API
pub use error::{BatchError, BatchResult};
pub use processor::{BatchProcessor, DefaultBatchProcessor};
pub use types::{
    BatchJob, BatchOptions, BatchProgress, BatchResult as BatchResultType, BatchSummary, FileEntry,
    FileError, FileResult, ProgressCallback, ResultCallback, SkippedFile,
};

// Re-export commonly used types from veil-parsers
pub use veil_parsers::{FileFormat, ParseOptions, ParseResult};
