//! Progress tracking for batch processing.

use crate::types::BatchProgress;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Internal state for progress tracking.
#[derive(Debug)]
pub struct ProgressState {
    /// Total number of files to process.
    total: usize,
    /// Number of files processed.
    processed: AtomicUsize,
    /// Number of successful files.
    successful: AtomicUsize,
    /// Number of failed files.
    failed: AtomicUsize,
    /// Number of skipped files.
    skipped: AtomicUsize,
    /// Current file being processed.
    current_file: Mutex<Option<PathBuf>>,
    /// Start time.
    start_time: Instant,
}

impl ProgressState {
    /// Create new progress state.
    pub fn new(total: usize) -> Self {
        Self {
            total,
            processed: AtomicUsize::new(0),
            successful: AtomicUsize::new(0),
            failed: AtomicUsize::new(0),
            skipped: AtomicUsize::new(0),
            current_file: Mutex::new(None),
            start_time: Instant::now(),
        }
    }

    /// Increment processed count.
    pub fn increment_processed(&self) {
        self.processed.fetch_add(1, Ordering::SeqCst);
    }

    /// Increment successful count.
    pub fn increment_successful(&self) {
        self.successful.fetch_add(1, Ordering::SeqCst);
    }

    /// Increment failed count.
    pub fn increment_failed(&self) {
        self.failed.fetch_add(1, Ordering::SeqCst);
    }

    /// Increment skipped count.
    pub fn increment_skipped(&self) {
        self.skipped.fetch_add(1, Ordering::SeqCst);
    }

    /// Set current file being processed.
    pub fn set_current_file(&self, path: Option<PathBuf>) {
        if let Ok(mut current) = self.current_file.lock() {
            *current = path;
        }
    }

    /// Get current progress snapshot.
    pub fn snapshot(&self) -> BatchProgress {
        let processed = self.processed.load(Ordering::SeqCst);
        let successful = self.successful.load(Ordering::SeqCst);
        let failed = self.failed.load(Ordering::SeqCst);
        let skipped = self.skipped.load(Ordering::SeqCst);
        let elapsed = self.start_time.elapsed();

        let current_file = self.current_file.lock().ok().and_then(|f| f.clone());

        let eta = if processed > 0 {
            let avg_time_per_file = elapsed.as_secs_f64() / processed as f64;
            let remaining = self.total.saturating_sub(processed);
            let eta_secs = avg_time_per_file * remaining as f64;
            Some(Duration::from_secs_f64(eta_secs))
        } else {
            None
        };

        BatchProgress {
            processed,
            total: self.total,
            current_file,
            eta,
            elapsed,
            successful,
            failed,
            skipped,
        }
    }
}

/// Progress tracker for batch processing.
pub struct ProgressTracker {
    state: Arc<ProgressState>,
}

impl ProgressTracker {
    /// Create a new progress tracker.
    pub fn new(total: usize) -> Self {
        Self {
            state: Arc::new(ProgressState::new(total)),
        }
    }

    /// Get shared state for use in parallel processing.
    pub fn state(&self) -> Arc<ProgressState> {
        Arc::clone(&self.state)
    }

    /// Get current progress snapshot.
    #[allow(dead_code)]
    pub fn snapshot(&self) -> BatchProgress {
        self.state.snapshot()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_progress_tracking() {
        let tracker = ProgressTracker::new(100);
        let state = tracker.state();

        state.increment_processed();
        state.increment_successful();

        let progress = tracker.snapshot();
        assert_eq!(progress.processed, 1);
        assert_eq!(progress.successful, 1);
        assert_eq!(progress.total, 100);
        assert_eq!(progress.percentage(), 1.0);
    }

    #[test]
    fn test_progress_eta() {
        let tracker = ProgressTracker::new(100);
        let state = tracker.state();

        // Process some files with a small delay to get meaningful ETA
        for _ in 0..10 {
            thread::sleep(Duration::from_millis(1));
            state.increment_processed();
            state.increment_successful();
        }

        let progress = tracker.snapshot();
        assert_eq!(progress.processed, 10);
        assert!(progress.eta.is_some());
    }

    #[test]
    fn test_current_file() {
        let tracker = ProgressTracker::new(10);
        let state = tracker.state();

        let path = PathBuf::from("test.txt");
        state.set_current_file(Some(path.clone()));

        let progress = tracker.snapshot();
        assert_eq!(progress.current_file, Some(path));
    }

    #[test]
    fn test_concurrent_updates() {
        let tracker = ProgressTracker::new(100);
        let state = tracker.state();

        let mut handles = vec![];
        for _ in 0..10 {
            let state_clone = Arc::clone(&state);
            let handle = thread::spawn(move || {
                for _ in 0..10 {
                    state_clone.increment_processed();
                    state_clone.increment_successful();
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let progress = tracker.snapshot();
        assert_eq!(progress.processed, 100);
        assert_eq!(progress.successful, 100);
    }
}
