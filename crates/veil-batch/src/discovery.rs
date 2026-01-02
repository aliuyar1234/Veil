//! File discovery and format detection.

use crate::error::BatchResult;
use crate::redact::{redact_path, redact_text};
use crate::types::{BatchOptions, FileEntry};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use veil_core::HEADER_BUFFER_SIZE;
use veil_fs::{walk_files, WalkFilesOptions};
use veil_parsers::FileFormat;

/// Discover files to process from the given sources.
///
/// # Arguments
/// * `sources` - Paths to files or directories to scan
/// * `options` - Batch processing options
///
/// # Returns
/// Vector of file entries to process
pub fn discover_files(sources: &[PathBuf], options: &BatchOptions) -> BatchResult<Vec<FileEntry>> {
    let mut entries = Vec::new();

    for source in sources {
        if source.is_file() {
            // Single file
            let metadata = fs::metadata(source)?;
            let size = metadata.len();
            let format = detect_file_format(source);
            entries.push(FileEntry::new(source.clone(), size, format));
        } else if source.is_dir() {
            // Directory - walk it
            let dir_entries = walk_directory(source, options)?;
            entries.extend(dir_entries);
        } else {
            // Skip invalid paths (broken symlinks, etc.)
            continue;
        }
    }

    Ok(entries)
}

/// Walk a directory and collect file entries.
fn walk_directory(path: &Path, options: &BatchOptions) -> BatchResult<Vec<FileEntry>> {
    let max_depth = if !options.recursive {
        Some(1)
    } else {
        options.max_depth
    };

    let walker = walk_files(
        path,
        WalkFilesOptions {
            follow_symlinks: options.follow_symlinks,
            max_depth,
        },
    );

    let mut entries = Vec::new();

    for entry_result in walker {
        let entry = match entry_result {
            Ok(e) => e,
            Err(e) => {
                let error_path = e
                    .path()
                    .map(redact_path)
                    .unwrap_or_else(|| "unknown".to_string());

                // Log error but continue processing
                tracing::warn!(
                    "Warning: Failed to access path {}: {}",
                    error_path,
                    redact_text(&e.to_string())
                );
                continue;
            }
        };

        // Get file metadata
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!(
                    "Warning: Failed to get metadata for {}: {}",
                    redact_path(entry.path()),
                    redact_text(&e.to_string())
                );
                continue;
            }
        };

        let size = metadata.len();
        let path = entry.path().to_path_buf();
        let format = detect_file_format(&path);

        entries.push(FileEntry::new(path, size, format));
    }

    Ok(entries)
}

/// Detect file format using magic bytes and extension.
///
/// Delegates to `veil_parsers::detect_format` for consistent format detection
/// across the codebase.
///
/// # Arguments
/// * `path` - Path to the file
///
/// # Returns
/// Detected file format, or None if unknown
pub fn detect_file_format(path: &Path) -> Option<FileFormat> {
    // Read file header for format detection
    let mut file = match fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return None,
    };

    let mut buffer = vec![0u8; HEADER_BUFFER_SIZE];
    let n = match file.read(&mut buffer) {
        Ok(n) => n,
        Err(_) => return None,
    };
    buffer.truncate(n);

    // Use veil_parsers for consistent format detection
    let filename = path.to_str();
    let format = veil_parsers::detect_format(&buffer, filename);

    // Return None for unsupported formats (those that need external crates)
    match format {
        FileFormat::Docx | FileFormat::Xlsx | FileFormat::Pptx => {
            // These are detected but not supported by veil-parsers directly
            Some(format)
        }
        FileFormat::Eml | FileFormat::Msg => {
            // Email formats also detected but handled by veil-email
            Some(format)
        }
        _ => Some(format),
    }
}

/// Check if a file format is supported for processing.
pub fn is_supported_format(format: Option<FileFormat>) -> bool {
    matches!(
        format,
        Some(FileFormat::Text)
            | Some(FileFormat::Csv)
            | Some(FileFormat::Json)
            | Some(FileFormat::Html)
            | Some(FileFormat::Pdf)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_supported_format() {
        assert!(is_supported_format(Some(FileFormat::Text)));
        assert!(is_supported_format(Some(FileFormat::Csv)));
        assert!(is_supported_format(Some(FileFormat::Json)));
        assert!(is_supported_format(Some(FileFormat::Html)));
        assert!(is_supported_format(Some(FileFormat::Pdf)));
        assert!(!is_supported_format(Some(FileFormat::Docx)));
        assert!(!is_supported_format(None));
    }
}
