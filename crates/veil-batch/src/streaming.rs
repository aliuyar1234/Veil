//! Streaming result processing for memory efficiency.

use crate::archive;
use crate::error::BatchError;
use crate::progress::ProgressState;
use crate::types::{BatchOptions, BatchSummary, FileEntry, FileResult};
use rayon::prelude::*;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use uuid::Uuid;
use veil_parsers::FileFormat;

/// Statistics collected during streaming processing.
#[derive(Debug, Default)]
struct StreamStats {
    successful: usize,
    failed: usize,
    skipped: usize,
    total_chars: usize,
    total_segments: usize,
}

/// Process files with streaming results.
///
/// # Arguments
/// * `job_id` - Unique job identifier
/// * `files` - List of files to process
/// * `options` - Batch processing options
/// * `progress` - Optional progress state for tracking
/// * `callback` - Callback invoked for each successful result
///
/// # Returns
/// Summary statistics
pub fn process_streaming<F>(
    job_id: Uuid,
    files: Vec<FileEntry>,
    options: &BatchOptions,
    progress: Option<Arc<ProgressState>>,
    callback: F,
) -> Result<BatchSummary, BatchError>
where
    F: Fn(FileResult) + Send + Sync + 'static,
{
    let start_time = Instant::now();
    let total_files = files.len();

    // Configure thread pool
    let pool = if let Some(parallelism) = options.max_parallelism {
        rayon::ThreadPoolBuilder::new()
            .num_threads(parallelism)
            .build()
            .unwrap_or_else(|_| rayon::ThreadPoolBuilder::new().build().unwrap())
    } else {
        let num_threads = std::thread::available_parallelism()
            .map(|n| n.get().saturating_sub(1).max(1))
            .unwrap_or(1);
        rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build()
            .unwrap_or_else(|_| rayon::ThreadPoolBuilder::new().build().unwrap())
    };

    // Create channel for results
    let (tx, rx) = mpsc::channel::<FileResult>();
    let stats = Arc::new(Mutex::new(StreamStats::default()));

    // Spawn receiver thread
    let stats_clone = Arc::clone(&stats);
    let callback = Arc::new(callback);
    let receiver_handle = std::thread::spawn(move || {
        for result in rx {
            // Update stats
            if let Ok(mut s) = stats_clone.lock() {
                s.total_chars += result.result.total_chars;
                s.total_segments += result.result.segments.len();
            }
            // Invoke callback
            callback(result);
        }
    });

    // Process files in parallel
    pool.install(|| {
        files.par_iter().for_each(|entry| {
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
                    if let Ok(mut s) = stats.lock() {
                        s.successful += 1;
                    }
                    // Send result to receiver thread
                    let _ = tx.send(result);
                }
                Err(e) => {
                    if should_skip(&e) {
                        if let Some(ref prog) = progress {
                            prog.increment_processed();
                            prog.increment_skipped();
                        }
                        if let Ok(mut s) = stats.lock() {
                            s.skipped += 1;
                        }
                    } else {
                        if let Some(ref prog) = progress {
                            prog.increment_processed();
                            prog.increment_failed();
                        }
                        if let Ok(mut s) = stats.lock() {
                            s.failed += 1;
                        }
                    }
                }
            }
        });
    });

    // Close channel and wait for receiver
    drop(tx);
    receiver_handle.join().unwrap();

    // Build summary
    let duration = start_time.elapsed();
    let final_stats = stats.lock().unwrap();

    Ok(BatchSummary {
        job_id,
        total_files,
        successful: final_stats.successful,
        failed: final_stats.failed,
        skipped: final_stats.skipped,
        duration,
        total_chars: final_stats.total_chars,
        total_segments: final_stats.total_segments,
    })
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

/// Check if an error should cause the file to be skipped.
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
    fn test_stream_stats_default() {
        let stats = StreamStats::default();
        assert_eq!(stats.successful, 0);
        assert_eq!(stats.failed, 0);
        assert_eq!(stats.skipped, 0);
        assert_eq!(stats.total_chars, 0);
        assert_eq!(stats.total_segments, 0);
    }
}
