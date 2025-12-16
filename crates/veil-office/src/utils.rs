//! Utility functions for Office document parsing.

use crate::error::{OfficeError, Result};

/// Maximum allowed uncompressed size (50MB).
pub const MAX_UNCOMPRESSED_SIZE: u64 = 50 * 1024 * 1024;

/// Maximum allowed single ZIP entry size (50MB).
pub const MAX_ENTRY_SIZE: u64 = 50 * 1024 * 1024;

/// Sanitize a ZIP entry path to prevent path traversal attacks.
///
/// Returns `None` if the path is unsafe (contains `..` or absolute paths).
pub fn sanitize_zip_path(path: &str) -> Option<String> {
    // Reject empty paths
    if path.is_empty() {
        return None;
    }

    // Reject absolute paths
    if path.starts_with('/') || path.starts_with('\\') {
        return None;
    }

    // Reject paths with drive letters (Windows)
    if path.len() >= 2 && path.chars().nth(1) == Some(':') {
        return None;
    }

    // Reject paths with parent directory references
    for component in path.split(['/', '\\']) {
        if component == ".." {
            return None;
        }
    }

    // Normalize path separators to forward slashes
    let normalized = path.replace('\\', "/");

    Some(normalized)
}

/// Check if a ZIP entry size is within allowed limits.
///
/// Returns an error if the entry exceeds the maximum size.
pub fn check_zip_entry_size(_entry_name: &str, size: u64) -> Result<()> {
    if size > MAX_ENTRY_SIZE {
        return Err(OfficeError::FileTooLarge {
            max_mb: (MAX_ENTRY_SIZE / (1024 * 1024)) as usize,
            actual_mb: size as f64 / (1024.0 * 1024.0),
        });
    }
    Ok(())
}

/// Check cumulative uncompressed size to protect against ZIP bombs.
pub fn check_cumulative_size(total_size: u64) -> Result<()> {
    if total_size > MAX_UNCOMPRESSED_SIZE {
        return Err(OfficeError::FileTooLarge {
            max_mb: (MAX_UNCOMPRESSED_SIZE / (1024 * 1024)) as usize,
            actual_mb: total_size as f64 / (1024.0 * 1024.0),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_valid_paths() {
        assert_eq!(
            sanitize_zip_path("word/document.xml"),
            Some("word/document.xml".to_string())
        );
        assert_eq!(
            sanitize_zip_path("xl/worksheets/sheet1.xml"),
            Some("xl/worksheets/sheet1.xml".to_string())
        );
        assert_eq!(
            sanitize_zip_path("[Content_Types].xml"),
            Some("[Content_Types].xml".to_string())
        );
    }

    #[test]
    fn test_sanitize_path_traversal() {
        assert_eq!(sanitize_zip_path("../etc/passwd"), None);
        assert_eq!(sanitize_zip_path("word/../../../etc/passwd"), None);
        assert_eq!(sanitize_zip_path(".."), None);
    }

    #[test]
    fn test_sanitize_absolute_paths() {
        assert_eq!(sanitize_zip_path("/etc/passwd"), None);
        assert_eq!(sanitize_zip_path("\\Windows\\System32"), None);
        assert_eq!(sanitize_zip_path("C:\\Windows"), None);
    }

    #[test]
    fn test_sanitize_empty_path() {
        assert_eq!(sanitize_zip_path(""), None);
    }

    #[test]
    fn test_sanitize_windows_paths() {
        // Windows paths should be normalized to forward slashes
        assert_eq!(
            sanitize_zip_path("word\\document.xml"),
            Some("word/document.xml".to_string())
        );
    }

    #[test]
    fn test_check_entry_size_valid() {
        assert!(check_zip_entry_size("test.xml", 1024).is_ok());
        assert!(check_zip_entry_size("test.xml", MAX_ENTRY_SIZE).is_ok());
    }

    #[test]
    fn test_check_entry_size_too_large() {
        let result = check_zip_entry_size("test.xml", MAX_ENTRY_SIZE + 1);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            OfficeError::FileTooLarge { .. }
        ));
    }
}
