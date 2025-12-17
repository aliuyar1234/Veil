//! Parallel file processing with rayon.

use crate::archive;
use crate::error::BatchError;
use crate::progress::ProgressState;
use crate::types::{BatchOptions, FileEntry, FileError, FileResult, SkippedFile};
use rayon::prelude::*;
use std::sync::Arc;
use std::time::Instant;
use veil_parsers::FileFormat;

/// Build a thread pool with the specified number of threads.
/// Falls back to a single-threaded pool if construction fails.
fn build_thread_pool(num_threads: usize) -> rayon::ThreadPool {
    rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .unwrap_or_else(|e| {
            tracing::warn!("Failed to build thread pool with {} threads: {}, using single thread", num_threads, e);
            // Fallback: try with 1 thread, and if that fails, use the global pool
            rayon::ThreadPoolBuilder::new()
                .num_threads(1)
                .build()
                .expect("Failed to create even a single-threaded pool - system resource exhaustion")
        })
}

/// Thread-local accumulator for batch processing results.
/// This avoids mutex contention by accumulating results per-thread
/// and merging at the end.
#[derive(Default)]
struct BatchAccumulator {
    results: Vec<FileResult>,
    errors: Vec<FileError>,
    skipped: Vec<SkippedFile>,
}

impl BatchAccumulator {
    fn new() -> Self {
        Self::default()
    }

    fn push_result(&mut self, result: FileResult) {
        self.results.push(result);
    }

    fn push_error(&mut self, error: FileError) {
        self.errors.push(error);
    }

    fn push_skipped(&mut self, skipped: SkippedFile) {
        self.skipped.push(skipped);
    }

    /// Merge another accumulator into this one.
    fn merge(mut self, other: Self) -> Self {
        self.results.extend(other.results);
        self.errors.extend(other.errors);
        self.skipped.extend(other.skipped);
        self
    }

    /// Convert into tuple of result vectors.
    fn into_parts(self) -> (Vec<FileResult>, Vec<FileError>, Vec<SkippedFile>) {
        (self.results, self.errors, self.skipped)
    }
}

/// Process files in parallel.
///
/// # Arguments
/// * `files` - List of files to process
/// * `options` - Batch processing options
/// * `progress` - Optional progress state for tracking
///
/// # Returns
/// Tuple of (successful results, errors, skipped files)
pub fn process_files_parallel(
    files: Vec<FileEntry>,
    options: &BatchOptions,
    progress: Option<Arc<ProgressState>>,
) -> (Vec<FileResult>, Vec<FileError>, Vec<SkippedFile>) {
    // Configure thread pool
    let num_threads = options.max_parallelism.unwrap_or_else(|| {
        // Default: use available parallelism - 1
        std::thread::available_parallelism()
            .map(|n| n.get().saturating_sub(1).max(1))
            .unwrap_or(1)
    });
    let pool = build_thread_pool(num_threads);

    // Use thread-local accumulation with fold/reduce to avoid mutex contention
    let accumulator = pool.install(|| {
        files
            .par_iter()
            .fold(BatchAccumulator::new, |mut acc, entry| {
                // Update progress
                if let Some(ref prog) = progress {
                    prog.set_current_file(Some(entry.path.clone()));
                }

                // Process the file
                match process_single_file(entry, options) {
                    Ok(result) => {
                        if let Some(ref prog) = progress {
                            prog.increment_processed();
                            prog.increment_successful();
                        }
                        acc.push_result(result);
                    }
                    Err(e) => {
                        if should_skip(&e) {
                            if let Some(ref prog) = progress {
                                prog.increment_processed();
                                prog.increment_skipped();
                            }
                            acc.push_skipped(SkippedFile {
                                path: entry.path.clone(),
                                reason: e.to_string(),
                                from_archive: entry.from_archive,
                                archive_path: entry.archive_path.clone(),
                            });
                        } else {
                            if let Some(ref prog) = progress {
                                prog.increment_processed();
                                prog.increment_failed();
                            }
                            acc.push_error(FileError {
                                path: entry.path.clone(),
                                error: e.to_string(),
                                from_archive: entry.from_archive,
                                archive_path: entry.archive_path.clone(),
                            });
                        }
                    }
                }
                acc
            })
            .reduce(BatchAccumulator::new, BatchAccumulator::merge)
    });

    accumulator.into_parts()
}

/// Process a single file.
fn process_single_file(
    entry: &FileEntry,
    options: &BatchOptions,
) -> Result<FileResult, BatchError> {
    let start = Instant::now();

    // Check size limit
    if entry.size > options.max_file_size as u64 {
        return Err(BatchError::FileTooLarge {
            size: entry.size,
            max: options.max_file_size,
        });
    }

    // Check if format is supported
    let format = entry.format.ok_or_else(|| {
        BatchError::Parse(veil_parsers::ParseError::UnsupportedFormat(Some(
            "Unknown file format".to_string(),
        )))
    })?;

    if !is_supported_format(format) {
        return Err(BatchError::Parse(
            veil_parsers::ParseError::UnsupportedFormat(Some(format!("{:?}", format))),
        ));
    }

    // Parse the file
    let result = if entry.from_archive {
        // Extract from archive and parse
        let archive_path = entry.archive_path.as_ref().ok_or_else(|| {
            BatchError::Archive("Archive path missing for archived file".to_string())
        })?;
        let contents = archive::extract_file(
            archive_path,
            &entry.path,
            options.archive_password.as_deref(),
        )?;
        veil_parsers::parse_bytes(&contents, &options.parse_options)?
    } else {
        // Parse directly from file
        veil_parsers::parse_file(&entry.path, &options.parse_options)?
    };

    let duration = start.elapsed();

    Ok(FileResult {
        path: entry.path.clone(),
        result,
        duration,
        from_archive: entry.from_archive,
        archive_path: entry.archive_path.clone(),
    })
}

/// Check if a file format is supported.
fn is_supported_format(format: FileFormat) -> bool {
    matches!(
        format,
        FileFormat::Text | FileFormat::Csv | FileFormat::Json | FileFormat::Html | FileFormat::Pdf
    )
}

/// Check if an error should cause the file to be skipped rather than marked as failed.
fn should_skip(error: &BatchError) -> bool {
    matches!(
        error,
        BatchError::FileTooLarge { .. }
            | BatchError::Parse(veil_parsers::ParseError::UnsupportedFormat(_))
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_supported_format() {
        assert!(is_supported_format(FileFormat::Text));
        assert!(is_supported_format(FileFormat::Csv));
        assert!(is_supported_format(FileFormat::Json));
        assert!(is_supported_format(FileFormat::Html));
        assert!(is_supported_format(FileFormat::Pdf));
        assert!(!is_supported_format(FileFormat::Docx));
        assert!(!is_supported_format(FileFormat::Xlsx));
    }

    #[test]
    fn test_should_skip() {
        assert!(should_skip(&BatchError::FileTooLarge {
            size: 1000,
            max: 500
        }));
        assert!(should_skip(&BatchError::Parse(
            veil_parsers::ParseError::UnsupportedFormat(Some("test".to_string()))
        )));
        assert!(!should_skip(&BatchError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "test"
        ))));
    }
}
