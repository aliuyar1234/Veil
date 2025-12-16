//! Main batch processor implementation.

use crate::archive;
use crate::discovery::{discover_files, is_supported_format};
use crate::error::BatchResult;
use crate::filter::{apply_filter, GlobFilter};
use crate::parallel::process_files_parallel;
use crate::progress::ProgressTracker;
use crate::streaming::process_streaming;
use crate::types::{
    BatchJob, BatchProgress, BatchResult as BatchResultType, BatchSummary, FileEntry,
};
use std::sync::Arc;
use std::time::Instant;

/// Trait for batch processing.
pub trait BatchProcessor {
    /// Process a batch job.
    ///
    /// # Arguments
    /// * `job` - The batch job to process
    ///
    /// # Returns
    /// Complete batch result with all file results, errors, and skipped files
    fn process(&self, job: &BatchJob) -> BatchResult<BatchResultType>;

    /// Process a batch job with progress reporting.
    ///
    /// # Arguments
    /// * `job` - The batch job to process
    /// * `progress_callback` - Callback invoked with progress updates
    ///
    /// # Returns
    /// Complete batch result
    fn process_with_progress<F>(
        &self,
        job: &BatchJob,
        progress_callback: F,
    ) -> BatchResult<BatchResultType>
    where
        F: Fn(BatchProgress) + Send + Sync + 'static;

    /// Process a batch job with streaming results.
    ///
    /// # Arguments
    /// * `job` - The batch job to process
    /// * `result_callback` - Callback invoked for each successful file result
    ///
    /// # Returns
    /// Summary statistics only (results are not collected)
    fn process_streaming<F>(&self, job: &BatchJob, result_callback: F) -> BatchResult<BatchSummary>
    where
        F: Fn(crate::types::FileResult) + Send + Sync + 'static;
}

/// Default implementation of batch processor.
#[derive(Debug, Default)]
pub struct DefaultBatchProcessor;

impl DefaultBatchProcessor {
    /// Create a new batch processor.
    pub fn new() -> Self {
        Self
    }

    /// Discover and filter files for processing.
    fn discover_and_filter(&self, job: &BatchJob) -> BatchResult<Vec<FileEntry>> {
        // Discover all files
        let mut entries = discover_files(&job.sources, &job.options)?;

        // Process archives if any
        let mut archive_entries = Vec::new();
        for entry in &entries {
            if archive::is_zip_archive(&entry.path) {
                match archive::process_archive(&entry.path, &job.options, 0) {
                    Ok(mut ae) => archive_entries.append(&mut ae),
                    Err(e) => {
                        eprintln!("Warning: Failed to process archive {:?}: {}", entry.path, e);
                    }
                }
            }
        }
        entries.extend(archive_entries);

        // Filter by supported formats
        entries.retain(|entry| is_supported_format(entry.format));

        // Apply glob filters
        let filter = GlobFilter::new(&job.options.include_patterns, &job.options.exclude_patterns)?;
        let filtered = apply_filter(entries, &filter, |e| &e.path);

        Ok(filtered)
    }
}

impl BatchProcessor for DefaultBatchProcessor {
    fn process(&self, job: &BatchJob) -> BatchResult<BatchResultType> {
        let start_time = Instant::now();

        // Discover and filter files
        let files = self.discover_and_filter(job)?;

        // Process files in parallel
        let (results, errors, skipped) = process_files_parallel(files, &job.options, None);

        let duration = start_time.elapsed();
        let result = BatchResultType::new(job.id, results, errors, skipped, duration);

        Ok(result)
    }

    fn process_with_progress<F>(
        &self,
        job: &BatchJob,
        progress_callback: F,
    ) -> BatchResult<BatchResultType>
    where
        F: Fn(BatchProgress) + Send + Sync + 'static,
    {
        let start_time = Instant::now();

        // Discover and filter files
        let files = self.discover_and_filter(job)?;

        // Create progress tracker
        let tracker = ProgressTracker::new(files.len());
        let progress_state = tracker.state();

        // Spawn progress reporting thread
        let progress_state_clone = Arc::clone(&progress_state);
        let callback = Arc::new(progress_callback);
        let progress_handle = std::thread::spawn(move || loop {
            let snapshot = progress_state_clone.snapshot();
            let is_complete = snapshot.is_complete();
            callback(snapshot);

            if is_complete {
                break;
            }

            std::thread::sleep(std::time::Duration::from_millis(100));
        });

        // Process files in parallel
        let (results, errors, skipped) =
            process_files_parallel(files, &job.options, Some(progress_state));

        // Wait for progress thread to finish
        progress_handle.join().unwrap();

        let duration = start_time.elapsed();
        let result = BatchResultType::new(job.id, results, errors, skipped, duration);

        Ok(result)
    }

    fn process_streaming<F>(&self, job: &BatchJob, result_callback: F) -> BatchResult<BatchSummary>
    where
        F: Fn(crate::types::FileResult) + Send + Sync + 'static,
    {
        // Discover and filter files
        let files = self.discover_and_filter(job)?;

        // Process with streaming
        process_streaming(job.id, files, &job.options, None, result_callback)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::BatchOptions;

    #[test]
    fn test_processor_creation() {
        let _processor = DefaultBatchProcessor::new();
        // Just verify it can be created without panicking
    }

    #[test]
    fn test_empty_job() {
        let processor = DefaultBatchProcessor::new();
        let job = BatchJob::new(vec![], BatchOptions::default());

        let result = processor.process(&job);
        assert!(result.is_ok());

        let batch_result = result.unwrap();
        assert_eq!(batch_result.summary.total_files, 0);
        assert_eq!(batch_result.summary.successful, 0);
    }
}
