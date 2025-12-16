//! File discovery and format detection.

use crate::error::BatchResult;
use crate::types::{BatchOptions, FileEntry};
use std::fs;
use std::path::{Path, PathBuf};
use veil_parsers::FileFormat;
use walkdir::WalkDir;

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
    let mut walker = WalkDir::new(path).follow_links(options.follow_symlinks);

    // Apply max depth
    if !options.recursive {
        walker = walker.max_depth(1);
    } else if let Some(depth) = options.max_depth {
        walker = walker.max_depth(depth);
    }

    let mut entries = Vec::new();

    for entry_result in walker {
        let entry = match entry_result {
            Ok(e) => e,
            Err(e) => {
                // Log error but continue processing
                eprintln!("Warning: Failed to access path: {}", e);
                continue;
            }
        };

        // Skip directories
        if entry.file_type().is_dir() {
            continue;
        }

        // Get file metadata
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(e) => {
                eprintln!(
                    "Warning: Failed to get metadata for {:?}: {}",
                    entry.path(),
                    e
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
/// # Arguments
/// * `path` - Path to the file
///
/// # Returns
/// Detected file format, or None if unknown
pub fn detect_file_format(path: &Path) -> Option<FileFormat> {
    // Try magic byte detection first
    if let Ok(mut file) = fs::File::open(path) {
        let mut buffer = vec![0u8; 1024];
        if let Ok(n) = std::io::Read::read(&mut file, &mut buffer) {
            buffer.truncate(n);

            // Use infer to detect file type
            if let Some(kind) = infer::get(&buffer) {
                return match kind.mime_type() {
                    "application/pdf" => Some(FileFormat::Pdf),
                    "application/json" => Some(FileFormat::Json),
                    "text/html" | "application/xhtml+xml" => Some(FileFormat::Html),
                    "application/zip" => {
                        // Check if it's an Office file
                        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                            match ext.to_lowercase().as_str() {
                                "docx" => return Some(FileFormat::Docx),
                                "xlsx" => return Some(FileFormat::Xlsx),
                                "pptx" => return Some(FileFormat::Pptx),
                                _ => {}
                            }
                        }
                        None // Plain ZIP, not a supported format
                    }
                    "text/csv" => Some(FileFormat::Csv),
                    mime if mime.starts_with("text/") => {
                        // Check extension for more specific type
                        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                            match ext.to_lowercase().as_str() {
                                "csv" | "tsv" => return Some(FileFormat::Csv),
                                "json" => return Some(FileFormat::Json),
                                "html" | "htm" => return Some(FileFormat::Html),
                                _ => {}
                            }
                        }
                        Some(FileFormat::Text)
                    }
                    _ => None,
                };
            }
        }
    }

    // Fallback to extension-based detection
    detect_format_by_extension(path)
}

/// Detect format based on file extension.
fn detect_format_by_extension(path: &Path) -> Option<FileFormat> {
    let ext = path.extension()?.to_str()?.to_lowercase();

    match ext.as_str() {
        "txt" | "log" => Some(FileFormat::Text),
        "csv" | "tsv" => Some(FileFormat::Csv),
        "json" => Some(FileFormat::Json),
        "html" | "htm" => Some(FileFormat::Html),
        "pdf" => Some(FileFormat::Pdf),
        "docx" => Some(FileFormat::Docx),
        "xlsx" => Some(FileFormat::Xlsx),
        "pptx" => Some(FileFormat::Pptx),
        "eml" => Some(FileFormat::Eml),
        "msg" => Some(FileFormat::Msg),
        _ => None,
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
    fn test_detect_format_by_extension() {
        assert_eq!(
            detect_format_by_extension(Path::new("test.txt")),
            Some(FileFormat::Text)
        );
        assert_eq!(
            detect_format_by_extension(Path::new("data.csv")),
            Some(FileFormat::Csv)
        );
        assert_eq!(
            detect_format_by_extension(Path::new("config.json")),
            Some(FileFormat::Json)
        );
        assert_eq!(
            detect_format_by_extension(Path::new("page.html")),
            Some(FileFormat::Html)
        );
        assert_eq!(
            detect_format_by_extension(Path::new("document.pdf")),
            Some(FileFormat::Pdf)
        );
        assert_eq!(detect_format_by_extension(Path::new("unknown.xyz")), None);
    }

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
