//! Shared constants for the Veil workspace.
//!
//! These constants define limits and defaults used across multiple crates.

/// Default maximum file size for parsing (100 MB).
pub const DEFAULT_MAX_FILE_SIZE: usize = 100 * 1024 * 1024;

/// Default sample size for file content sampling (100 KB).
pub const DEFAULT_SAMPLE_SIZE: usize = 100 * 1024;

/// Header buffer size for format detection (1 KB).
pub const HEADER_BUFFER_SIZE: usize = 1024;

/// Default chunk size for streaming operations (64 KB).
pub const DEFAULT_CHUNK_SIZE: usize = 64 * 1024;

/// Maximum number of concurrent file operations.
pub const DEFAULT_MAX_CONCURRENCY: usize = 8;

/// Minimum confidence threshold for PII detection (0-1 scale).
pub const MIN_CONFIDENCE_THRESHOLD: f64 = 0.5;

/// Default confidence threshold for PII detection (0-1 scale).
pub const DEFAULT_CONFIDENCE_THRESHOLD: f64 = 0.8;

const _: () = assert!(MIN_CONFIDENCE_THRESHOLD < DEFAULT_CONFIDENCE_THRESHOLD);
const _: () = assert!(MIN_CONFIDENCE_THRESHOLD >= 0.0);
const _: () = assert!(DEFAULT_CONFIDENCE_THRESHOLD <= 1.0);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_size_constants() {
        assert_eq!(DEFAULT_MAX_FILE_SIZE, 104_857_600);
        assert_eq!(DEFAULT_SAMPLE_SIZE, 102_400);
        assert_eq!(HEADER_BUFFER_SIZE, 1024);
        assert_eq!(DEFAULT_CHUNK_SIZE, 65_536);
        assert_eq!(DEFAULT_MAX_CONCURRENCY, 8);
    }
}
