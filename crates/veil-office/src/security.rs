//! Security utilities for Office document parsing.
//!
//! This module provides protection against common security threats:
//! - ZIP bomb attacks (decompression bombs)
//! - Path traversal attacks
//! - Encrypted document detection

use std::io::{Read, Seek};

use zip::ZipArchive;

use crate::error::{OfficeError, Result};

/// Security limits for archive processing.
#[derive(Debug, Clone)]
pub struct SecurityLimits {
    /// Maximum compression ratio allowed (decompressed/compressed size).
    /// Default: 100x (a 1MB compressed file can expand to at most 100MB).
    pub max_compression_ratio: usize,

    /// Maximum total uncompressed size in bytes.
    /// Default: 1GB.
    pub max_uncompressed_size: u64,

    /// Maximum number of files in the archive.
    /// Default: 10,000.
    pub max_file_count: usize,

    /// Maximum size of a single file entry in bytes.
    /// Default: 100MB.
    pub max_single_file_size: u64,
}

impl Default for SecurityLimits {
    fn default() -> Self {
        Self {
            max_compression_ratio: 100,
            max_uncompressed_size: 1024 * 1024 * 1024, // 1GB
            max_file_count: 10_000,
            max_single_file_size: 100 * 1024 * 1024, // 100MB
        }
    }
}

/// Validate an archive against security limits.
///
/// This function checks for:
/// - ZIP bomb attacks (excessive compression ratios)
/// - Excessive file counts
/// - Path traversal attacks
/// - Single file size limits
///
/// # Arguments
/// * `archive` - The ZIP archive to validate
/// * `limits` - Security limits to apply
///
/// # Returns
/// * `Ok(())` if the archive passes all checks
/// * `Err(OfficeError)` if any security check fails
pub fn validate_archive<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    limits: &SecurityLimits,
) -> Result<()> {
    // Check file count
    if archive.len() > limits.max_file_count {
        return Err(OfficeError::Corrupted(format!(
            "Archive contains too many files: {} (max: {})",
            archive.len(),
            limits.max_file_count
        )));
    }

    let mut total_compressed: u64 = 0;
    let mut total_uncompressed: u64 = 0;

    for i in 0..archive.len() {
        let file = archive.by_index(i)?;
        let name = file.name();

        // Check for path traversal
        validate_path(name)?;

        // Accumulate sizes
        total_compressed += file.compressed_size();
        total_uncompressed += file.size();

        // Check single file size
        if file.size() > limits.max_single_file_size {
            return Err(OfficeError::FileTooLarge {
                max_mb: (limits.max_single_file_size / (1024 * 1024)) as usize,
                actual_mb: file.size() as f64 / (1024.0 * 1024.0),
            });
        }
    }

    // Check total uncompressed size
    if total_uncompressed > limits.max_uncompressed_size {
        return Err(OfficeError::FileTooLarge {
            max_mb: (limits.max_uncompressed_size / (1024 * 1024)) as usize,
            actual_mb: total_uncompressed as f64 / (1024.0 * 1024.0),
        });
    }

    // Check compression ratio (ZIP bomb detection)
    if total_compressed > 0 {
        let ratio = total_uncompressed as f64 / total_compressed as f64;
        if ratio > limits.max_compression_ratio as f64 {
            return Err(OfficeError::ZipBomb {
                compressed_mb: total_compressed as f64 / (1024.0 * 1024.0),
                uncompressed_mb: total_uncompressed as f64 / (1024.0 * 1024.0),
                ratio,
                max_ratio: limits.max_compression_ratio,
            });
        }
    }

    Ok(())
}

/// Validate a path for traversal attacks.
///
/// Checks for:
/// - Absolute paths
/// - Parent directory references (..)
/// - Null bytes
/// - Suspicious patterns
fn validate_path(path: &str) -> Result<()> {
    // Check for null bytes
    if path.contains('\0') {
        return Err(OfficeError::PathTraversal(format!(
            "Path contains null byte: {}",
            path.replace('\0', "\\0")
        )));
    }

    // Check for absolute paths
    if path.starts_with('/') || path.starts_with('\\') {
        return Err(OfficeError::PathTraversal(format!(
            "Absolute path not allowed: {}",
            path
        )));
    }

    // Check for Windows drive letters
    if path.len() >= 2 && path.chars().nth(1) == Some(':') {
        return Err(OfficeError::PathTraversal(format!(
            "Windows absolute path not allowed: {}",
            path
        )));
    }

    // Check for parent directory traversal
    for component in path.split(['/', '\\']) {
        if component == ".." {
            return Err(OfficeError::PathTraversal(format!(
                "Parent directory traversal not allowed: {}",
                path
            )));
        }
    }

    Ok(())
}

/// Check if an Office document is encrypted.
///
/// This function looks for encryption indicators in Office Open XML documents:
/// - EncryptedPackage in compound documents
/// - Encryption metadata in XML
pub fn is_encrypted<R: Read + Seek>(archive: &mut ZipArchive<R>) -> bool {
    // Check for encrypted package indicator
    for i in 0..archive.len() {
        if let Ok(file) = archive.by_index(i) {
            let name = file.name().to_lowercase();

            // Check for encryption-related files
            if name.contains("encryptedpackage")
                || name.contains("encryptioninfo")
                || name.contains("dataintegrity")
            {
                return true;
            }

            // Check for encryption in settings
            if name == "word/settings.xml"
                || name == "xl/workbook.xml"
                || name == "ppt/presentation.xml"
            {
                // Would need to parse XML to check for encryption settings
                // For now, the presence of encryption-specific files is the indicator
            }
        }
    }

    false
}

/// Check for legacy Office formats (pre-2007).
///
/// Returns the format extension if it's a legacy format, None otherwise.
pub fn detect_legacy_format(bytes: &[u8]) -> Option<&'static str> {
    // Check for OLE Compound Document header (D0 CF 11 E0 A1 B1 1A E1)
    const OLE_HEADER: &[u8] = &[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];

    if bytes.len() >= 8 && &bytes[0..8] == OLE_HEADER {
        // This is a compound document - likely legacy Office
        // Could be .doc, .xls, .ppt, .msg, etc.
        // Without further parsing, we can't determine which one
        return Some("doc/xls/ppt");
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_path_normal() {
        assert!(validate_path("word/document.xml").is_ok());
        assert!(validate_path("_rels/.rels").is_ok());
        assert!(validate_path("[Content_Types].xml").is_ok());
    }

    #[test]
    fn test_validate_path_traversal() {
        assert!(validate_path("../../../etc/passwd").is_err());
        assert!(validate_path("word/..\\..\\secret.txt").is_err());
        assert!(validate_path("foo/../bar").is_err());
    }

    #[test]
    fn test_validate_path_absolute() {
        assert!(validate_path("/etc/passwd").is_err());
        assert!(validate_path("\\Windows\\System32").is_err());
        assert!(validate_path("C:\\Windows\\System32").is_err());
    }

    #[test]
    fn test_validate_path_null_byte() {
        assert!(validate_path("file.txt\0.exe").is_err());
    }

    #[test]
    fn test_security_limits_default() {
        let limits = SecurityLimits::default();
        assert_eq!(limits.max_compression_ratio, 100);
        assert_eq!(limits.max_uncompressed_size, 1024 * 1024 * 1024);
        assert_eq!(limits.max_file_count, 10_000);
    }

    #[test]
    fn test_detect_legacy_format() {
        // OLE header
        let ole_header = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];
        assert!(detect_legacy_format(&ole_header).is_some());

        // ZIP header (modern Office)
        let zip_header = [0x50, 0x4B, 0x03, 0x04, 0x00, 0x00, 0x00, 0x00];
        assert!(detect_legacy_format(&zip_header).is_none());

        // Random data
        let random = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07];
        assert!(detect_legacy_format(&random).is_none());
    }
}
