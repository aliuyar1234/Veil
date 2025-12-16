//! Core types for batch processing.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;
use veil_parsers::{FileFormat, ParseResult};

/// Options for configuring batch processing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchOptions {
    /// Maximum number of parallel workers (default: CPU count - 1).
    pub max_parallelism: Option<usize>,

    /// Follow symbolic links (default: true).
    pub follow_symlinks: bool,

    /// Maximum directory depth to traverse (default: unlimited).
    pub max_depth: Option<usize>,

    /// Maximum archive nesting depth (default: 5).
    pub max_archive_depth: usize,

    /// Maximum file size to process in bytes (default: 100MB).
    pub max_file_size: usize,

    /// Include patterns (glob format).
    pub include_patterns: Vec<String>,

    /// Exclude patterns (glob format).
    pub exclude_patterns: Vec<String>,

    /// Password for encrypted archives.
    pub archive_password: Option<String>,

    /// Enable recursive directory scanning (default: true).
    pub recursive: bool,

    /// Parse options to pass to veil-parsers.
    pub parse_options: veil_parsers::ParseOptions,
}

impl Default for BatchOptions {
    fn default() -> Self {
        Self {
            max_parallelism: None, // Will default to CPU count - 1
            follow_symlinks: true,
            max_depth: None,
            max_archive_depth: 5,
            max_file_size: 100 * 1024 * 1024, // 100MB
            include_patterns: Vec::new(),
            exclude_patterns: Vec::new(),
            archive_password: None,
            recursive: true,
            parse_options: veil_parsers::ParseOptions::default(),
        }
    }
}

/// A batch processing job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchJob {
    /// Unique job identifier.
    pub id: Uuid,

    /// Source paths (files or directories).
    pub sources: Vec<PathBuf>,

    /// Processing options.
    pub options: BatchOptions,
}

impl BatchJob {
    /// Create a new batch job.
    pub fn new(sources: Vec<PathBuf>, options: BatchOptions) -> Self {
        Self {
            id: Uuid::new_v4(),
            sources,
            options,
        }
    }
}

/// Entry for a file to be processed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    /// File path.
    pub path: PathBuf,

    /// File size in bytes.
    pub size: u64,

    /// Detected file format.
    pub format: Option<FileFormat>,

    /// Whether this file is from an archive.
    pub from_archive: bool,

    /// Archive path if from an archive.
    pub archive_path: Option<PathBuf>,
}

impl FileEntry {
    /// Create a new file entry.
    pub fn new(path: PathBuf, size: u64, format: Option<FileFormat>) -> Self {
        Self {
            path,
            size,
            format,
            from_archive: false,
            archive_path: None,
        }
    }

    /// Create a file entry from an archive.
    pub fn from_archive(
        path: PathBuf,
        size: u64,
        format: Option<FileFormat>,
        archive_path: PathBuf,
    ) -> Self {
        Self {
            path,
            size,
            format,
            from_archive: true,
            archive_path: Some(archive_path),
        }
    }
}

/// Result of processing a single file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileResult {
    /// File path.
    pub path: PathBuf,

    /// Parse result.
    pub result: ParseResult,

    /// Processing duration.
    pub duration: Duration,

    /// Whether this file is from an archive.
    pub from_archive: bool,

    /// Archive path if from an archive.
    pub archive_path: Option<PathBuf>,
}

/// Information about a file that encountered an error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileError {
    /// File path.
    pub path: PathBuf,

    /// Error message.
    pub error: String,

    /// Whether this file is from an archive.
    pub from_archive: bool,

    /// Archive path if from an archive.
    pub archive_path: Option<PathBuf>,
}

/// Information about a skipped file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkippedFile {
    /// File path.
    pub path: PathBuf,

    /// Reason for skipping.
    pub reason: String,

    /// Whether this file is from an archive.
    pub from_archive: bool,

    /// Archive path if from an archive.
    pub archive_path: Option<PathBuf>,
}

/// Progress information for a batch job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchProgress {
    /// Number of files processed so far.
    pub processed: usize,

    /// Total number of files to process.
    pub total: usize,

    /// Currently processing file.
    pub current_file: Option<PathBuf>,

    /// Estimated time remaining.
    pub eta: Option<Duration>,

    /// Elapsed time.
    pub elapsed: Duration,

    /// Number of successful files.
    pub successful: usize,

    /// Number of failed files.
    pub failed: usize,

    /// Number of skipped files.
    pub skipped: usize,
}

impl BatchProgress {
    /// Calculate percentage complete.
    pub fn percentage(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.processed as f64 / self.total as f64) * 100.0
        }
    }

    /// Check if processing is complete.
    pub fn is_complete(&self) -> bool {
        self.processed >= self.total
    }
}

/// Summary statistics for a batch job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchSummary {
    /// Job ID.
    pub job_id: Uuid,

    /// Total files discovered.
    pub total_files: usize,

    /// Successfully processed files.
    pub successful: usize,

    /// Failed files.
    pub failed: usize,

    /// Skipped files.
    pub skipped: usize,

    /// Total processing duration.
    pub duration: Duration,

    /// Total characters extracted.
    pub total_chars: usize,

    /// Total segments extracted.
    pub total_segments: usize,
}

/// Complete result of a batch processing job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResult {
    /// Summary statistics.
    pub summary: BatchSummary,

    /// Successful file results.
    pub results: Vec<FileResult>,

    /// Files that encountered errors.
    pub errors: Vec<FileError>,

    /// Skipped files.
    pub skipped: Vec<SkippedFile>,
}

impl BatchResult {
    /// Create a new batch result.
    pub fn new(
        job_id: Uuid,
        results: Vec<FileResult>,
        errors: Vec<FileError>,
        skipped: Vec<SkippedFile>,
        duration: Duration,
    ) -> Self {
        let total_chars: usize = results.iter().map(|r| r.result.total_chars).sum();
        let total_segments: usize = results.iter().map(|r| r.result.segments.len()).sum();

        let summary = BatchSummary {
            job_id,
            total_files: results.len() + errors.len() + skipped.len(),
            successful: results.len(),
            failed: errors.len(),
            skipped: skipped.len(),
            duration,
            total_chars,
            total_segments,
        };

        Self {
            summary,
            results,
            errors,
            skipped,
        }
    }
}

/// Progress callback type.
pub type ProgressCallback = Arc<dyn Fn(BatchProgress) + Send + Sync>;

/// Result callback type for streaming.
pub type ResultCallback = Arc<dyn Fn(FileResult) + Send + Sync>;
