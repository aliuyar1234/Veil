//! ZIP archive processing.

use crate::error::{BatchError, BatchResult};
use crate::redact::redact_text;
use crate::types::{BatchOptions, FileEntry};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use zip::ZipArchive;

/// Maximum compression ratio allowed (ZIP bomb protection).
const MAX_COMPRESSION_RATIO: u64 = 100;

/// Maximum total uncompressed size (1GB).
const MAX_TOTAL_UNCOMPRESSED_SIZE: u64 = 1024 * 1024 * 1024;

/// Validate archive path for traversal attacks.
fn validate_archive_path(path: &str) -> BatchResult<()> {
    // Check for null bytes
    if path.contains('\0') {
        return Err(BatchError::Archive(format!(
            "Path traversal detected: null byte in path '{}'",
            path.replace('\0', "\\0")
        )));
    }

    // Check for absolute paths
    if path.starts_with('/') || path.starts_with('\\') {
        return Err(BatchError::Archive(format!(
            "Path traversal detected: absolute path '{}'",
            path
        )));
    }

    // Check for Windows drive letters
    if path.len() >= 2 && path.chars().nth(1) == Some(':') {
        return Err(BatchError::Archive(format!(
            "Path traversal detected: Windows absolute path '{}'",
            path
        )));
    }

    // Check for parent directory traversal
    for component in path.split(['/', '\\']) {
        if component == ".." {
            return Err(BatchError::Archive(format!(
                "Path traversal detected: parent directory reference in '{}'",
                path
            )));
        }
    }

    Ok(())
}

/// Check for ZIP bomb (decompression bomb).
fn check_zip_bomb(archive: &mut ZipArchive<File>) -> BatchResult<()> {
    let mut total_compressed: u64 = 0;
    let mut total_uncompressed: u64 = 0;

    for i in 0..archive.len() {
        if let Ok(file) = archive.by_index_raw(i) {
            total_compressed += file.compressed_size();
            total_uncompressed += file.size();
        }
    }

    // Check total size limit
    if total_uncompressed > MAX_TOTAL_UNCOMPRESSED_SIZE {
        return Err(BatchError::Archive(format!(
            "Archive too large: would expand to {} MB (max: {} MB)",
            total_uncompressed / (1024 * 1024),
            MAX_TOTAL_UNCOMPRESSED_SIZE / (1024 * 1024)
        )));
    }

    // Check compression ratio
    if total_compressed > 0 {
        let ratio = total_uncompressed / total_compressed;
        if ratio > MAX_COMPRESSION_RATIO {
            return Err(BatchError::Archive(format!(
                "Potential ZIP bomb detected: compression ratio {}x exceeds limit of {}x",
                ratio, MAX_COMPRESSION_RATIO
            )));
        }
    }

    Ok(())
}

/// Process a ZIP archive and extract file entries.
///
/// # Arguments
/// * `path` - Path to the ZIP file
/// * `options` - Batch processing options
/// * `current_depth` - Current archive nesting depth
///
/// # Returns
/// Vector of file entries found in the archive
pub fn process_archive(
    path: &Path,
    options: &BatchOptions,
    current_depth: usize,
) -> BatchResult<Vec<FileEntry>> {
    // Check depth limit
    if current_depth >= options.max_archive_depth {
        return Err(BatchError::ArchiveDepthExceeded {
            max: options.max_archive_depth,
            depth: current_depth,
        });
    }

    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file)?;

    // Security check: ZIP bomb protection
    let file_for_check = File::open(path)?;
    let mut archive_for_check = ZipArchive::new(file_for_check)?;
    check_zip_bomb(&mut archive_for_check)?;

    let mut entries = Vec::new();

    for i in 0..archive.len() {
        let mut zip_file = archive.by_index(i)?;

        // Skip directories
        if zip_file.is_dir() {
            continue;
        }

        let file_name = zip_file.name().to_string();

        // Security check: path traversal
        validate_archive_path(&file_name)?;

        let size = zip_file.size();

        // Check size limit
        if size > options.max_file_size as u64 {
            tracing::warn!(
                "Warning: Skipping large file in archive: {} ({} bytes)",
                redact_text(&file_name),
                size
            );
            continue;
        }

        // Create a virtual path for the archived file
        let virtual_path = PathBuf::from(file_name.clone());

        // Detect format using veil_parsers
        let mut buffer = vec![0u8; 1024.min(size as usize)];
        let bytes_read = zip_file.read(&mut buffer)?;
        buffer.truncate(bytes_read);

        let format = if bytes_read > 0 {
            // Check for nested ZIP (PK signature)
            if buffer.starts_with(&[0x50, 0x4B, 0x03, 0x04]) {
                // Nested ZIP - skip for now, handle in batch processor
                None
            } else {
                // Use veil_parsers for consistent detection
                Some(veil_parsers::detect_format(&buffer, Some(&file_name)))
            }
        } else {
            // Empty file - try extension-based detection
            Some(veil_parsers::detect_format(&[], Some(&file_name)))
        };

        let mut entry = FileEntry::new(virtual_path, size, format);
        entry.from_archive = true;
        entry.archive_path = Some(path.to_path_buf());

        entries.push(entry);
    }

    Ok(entries)
}

/// Extract and read a file from a ZIP archive.
///
/// # Arguments
/// * `archive_path` - Path to the ZIP archive
/// * `file_path` - Path within the archive
/// * `password` - Optional password for encrypted archives
///
/// # Returns
/// File contents as bytes
pub fn extract_file(
    archive_path: &Path,
    file_path: &Path,
    password: Option<&str>,
) -> BatchResult<Vec<u8>> {
    let file_path_str = file_path.to_string_lossy();

    // Security check: path traversal
    validate_archive_path(&file_path_str)?;

    let file = File::open(archive_path)?;
    let mut archive = ZipArchive::new(file)?;

    let mut zip_file = if let Some(pwd) = password {
        archive
            .by_name_decrypt(&file_path_str, pwd.as_bytes())?
            .map_err(|_| BatchError::InvalidPassword)?
    } else {
        archive.by_name(&file_path_str)?
    };

    let mut contents = Vec::new();
    zip_file.read_to_end(&mut contents)?;

    Ok(contents)
}

/// Check if a file is a ZIP archive.
pub fn is_zip_archive(path: &Path) -> bool {
    if let Ok(file) = File::open(path) {
        ZipArchive::new(file).is_ok()
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_zip_archive() {
        // This test would need actual ZIP files to work
        // For now, just test that it doesn't panic on non-existent files
        assert!(!is_zip_archive(Path::new("nonexistent.zip")));
    }

    #[test]
    fn test_validate_archive_path_normal() {
        assert!(validate_archive_path("file.txt").is_ok());
        assert!(validate_archive_path("dir/file.txt").is_ok());
        assert!(validate_archive_path("dir/subdir/file.txt").is_ok());
        assert!(validate_archive_path("[Content_Types].xml").is_ok());
    }

    #[test]
    fn test_validate_archive_path_traversal() {
        assert!(validate_archive_path("../etc/passwd").is_err());
        assert!(validate_archive_path("foo/../bar").is_err());
        assert!(validate_archive_path("..\\Windows\\System32").is_err());
    }

    #[test]
    fn test_validate_archive_path_absolute() {
        assert!(validate_archive_path("/etc/passwd").is_err());
        assert!(validate_archive_path("\\Windows\\System32").is_err());
        assert!(validate_archive_path("C:\\Windows\\System32").is_err());
    }

    #[test]
    fn test_validate_archive_path_null_byte() {
        assert!(validate_archive_path("file.txt\0.exe").is_err());
    }
}
