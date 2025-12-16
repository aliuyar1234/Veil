//! Security utilities for the Veil API.
//!
//! This module provides security-related utilities including filename sanitization
//! to prevent path traversal and header injection attacks.

use std::path::Path;

/// Maximum allowed length for sanitized filenames.
const MAX_FILENAME_LENGTH: usize = 255;

/// Default filename when sanitization results in an empty name.
const DEFAULT_FILENAME: &str = "document";

/// Sanitize a filename to prevent security issues.
///
/// This function:
/// - Removes path separators to prevent path traversal
/// - Removes null bytes and control characters
/// - Removes or escapes characters problematic in HTTP headers
/// - Truncates to a maximum length
/// - Defaults to a safe filename if the result is empty
///
/// # Arguments
/// * `filename` - The raw filename to sanitize
///
/// # Returns
/// A sanitized filename safe for use in file operations and HTTP headers.
///
/// # Examples
/// ```
/// use veil_api::security::sanitize_filename;
///
/// assert_eq!(sanitize_filename("document.txt"), "document.txt");
/// // Path traversal attempts are blocked - only the filename is extracted
/// assert_eq!(sanitize_filename("../../../etc/passwd"), "passwd");
/// // Null bytes and control characters are removed
/// assert_eq!(sanitize_filename("file\x00name.txt"), "filename.txt");
/// assert_eq!(sanitize_filename("file\nname.txt"), "filename.txt");
/// ```
pub fn sanitize_filename(filename: &str) -> String {
    // Handle empty input
    if filename.is_empty() {
        return DEFAULT_FILENAME.to_string();
    }

    // Extract just the filename component (no path)
    let base_name = Path::new(filename)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(filename);

    // Remove dangerous characters
    let sanitized: String = base_name
        .chars()
        .filter(|c| {
            // Allow alphanumeric, common safe punctuation, and unicode letters
            c.is_alphanumeric()
                || *c == '.'
                || *c == '-'
                || *c == '_'
                || *c == ' '
                // Allow common non-ASCII characters for internationalization
                || (c.is_alphabetic() && !c.is_ascii())
        })
        .map(|c| {
            // Replace spaces with underscores for safer handling
            if c == ' ' { '_' } else { c }
        })
        .collect();

    // Remove leading/trailing dots and dashes (security concern)
    let trimmed = sanitized.trim_matches(|c| c == '.' || c == '-' || c == '_');

    // Handle empty result after sanitization
    if trimmed.is_empty() {
        return DEFAULT_FILENAME.to_string();
    }

    // Truncate to max length, trying to preserve extension
    let result = if trimmed.len() > MAX_FILENAME_LENGTH {
        // Try to preserve extension
        if let Some(ext_pos) = trimmed.rfind('.') {
            let ext = &trimmed[ext_pos..];
            if ext.len() < 20 && ext_pos > 0 {
                // Reasonable extension length
                let name_max = MAX_FILENAME_LENGTH - ext.len();
                format!("{}{}", &trimmed[..name_max.min(ext_pos)], ext)
            } else {
                trimmed[..MAX_FILENAME_LENGTH].to_string()
            }
        } else {
            trimmed[..MAX_FILENAME_LENGTH].to_string()
        }
    } else {
        trimmed.to_string()
    };

    // Final validation - ensure no path separators slipped through
    result.replace(['/', '\\'], "_")
}

/// Sanitize a filename and extract the extension.
///
/// Returns (sanitized_basename, extension) where extension includes the dot.
/// If no valid extension exists, returns (sanitized_filename, "").
pub fn sanitize_filename_with_extension(filename: &str) -> (String, String) {
    let sanitized = sanitize_filename(filename);

    if let Some(ext_pos) = sanitized.rfind('.') {
        let ext = &sanitized[ext_pos..];
        // Validate extension looks reasonable
        if ext.len() <= 10 && ext.chars().skip(1).all(|c| c.is_alphanumeric()) {
            return (sanitized[..ext_pos].to_string(), ext.to_string());
        }
    }

    (sanitized, String::new())
}

/// Create a safe Content-Disposition header value for a filename.
///
/// Properly encodes the filename for HTTP headers using RFC 5987 encoding
/// when necessary (for non-ASCII characters).
pub fn safe_content_disposition(filename: &str, disposition: &str) -> String {
    let sanitized = sanitize_filename(filename);

    // Check if filename is pure ASCII
    if sanitized.is_ascii() {
        // Simple case - just quote the filename
        format!("{}; filename=\"{}\"", disposition, sanitized)
    } else {
        // Use RFC 5987 encoding for non-ASCII
        let encoded: String = sanitized
            .bytes()
            .map(|b| {
                if b.is_ascii_alphanumeric() || b == b'.' || b == b'-' || b == b'_' {
                    format!("{}", b as char)
                } else {
                    format!("%{:02X}", b)
                }
            })
            .collect();

        // Provide both ASCII fallback and UTF-8 encoded version
        let ascii_fallback: String = sanitized
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' { c } else { '_' })
            .collect();

        format!(
            "{}; filename=\"{}\"; filename*=UTF-8''{}",
            disposition, ascii_fallback, encoded
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_filename() {
        assert_eq!(sanitize_filename("document.txt"), "document.txt");
        assert_eq!(sanitize_filename("my_file.pdf"), "my_file.pdf");
        assert_eq!(sanitize_filename("test-file.json"), "test-file.json");
    }

    #[test]
    fn test_path_traversal_prevention() {
        // Path components are stripped - only the final filename is kept
        assert_eq!(sanitize_filename("../../../etc/passwd"), "passwd");
        assert_eq!(sanitize_filename("/etc/passwd"), "passwd");
        // Windows paths too
        let result = sanitize_filename("C:\\Windows\\System32\\config.sys");
        // The result should only be the filename, no path components
        assert!(!result.contains('/'));
        assert!(!result.contains('\\'));
        assert!(result.contains("config"));
    }

    #[test]
    fn test_null_byte_removal() {
        assert_eq!(sanitize_filename("file\x00name.txt"), "filename.txt");
        assert_eq!(sanitize_filename("test\x00.php"), "test.php");
    }

    #[test]
    fn test_control_character_removal() {
        assert_eq!(sanitize_filename("file\nname.txt"), "filename.txt");
        assert_eq!(sanitize_filename("file\rname.txt"), "filename.txt");
        assert_eq!(sanitize_filename("file\tname.txt"), "filename.txt");
    }

    #[test]
    fn test_header_injection_prevention() {
        // Newlines could inject new headers - they get stripped out
        let result = sanitize_filename("file\r\nX-Injected: bad");
        assert!(!result.contains('\r'));
        assert!(!result.contains('\n'));
        assert!(!result.contains(':'));

        // Quotes could break out of quoted strings - they get stripped
        let result = sanitize_filename("file\"name.txt");
        assert!(!result.contains('"'));
        assert_eq!(result, "filename.txt");
    }

    #[test]
    fn test_empty_filename() {
        assert_eq!(sanitize_filename(""), DEFAULT_FILENAME);
        assert_eq!(sanitize_filename("..."), DEFAULT_FILENAME);
        assert_eq!(sanitize_filename("---"), DEFAULT_FILENAME);
        assert_eq!(sanitize_filename("///"), DEFAULT_FILENAME);
    }

    #[test]
    fn test_length_truncation() {
        let long_name = "a".repeat(300) + ".txt";
        let result = sanitize_filename(&long_name);
        assert!(result.len() <= MAX_FILENAME_LENGTH);
        assert!(result.ends_with(".txt"));
    }

    #[test]
    fn test_extension_preservation() {
        let (name, ext) = sanitize_filename_with_extension("document.pdf");
        assert_eq!(name, "document");
        assert_eq!(ext, ".pdf");
    }

    #[test]
    fn test_space_replacement() {
        assert_eq!(sanitize_filename("my file name.txt"), "my_file_name.txt");
    }

    #[test]
    fn test_unicode_preservation() {
        // Valid unicode should be preserved
        let result = sanitize_filename("documento.txt");
        assert_eq!(result, "documento.txt");
    }

    #[test]
    fn test_content_disposition_ascii() {
        let header = safe_content_disposition("document.txt", "attachment");
        assert_eq!(header, "attachment; filename=\"document.txt\"");
    }

    #[test]
    fn test_content_disposition_special_chars() {
        let header = safe_content_disposition("../bad.txt", "attachment");
        // Should sanitize the filename
        assert!(header.contains("filename="));
        assert!(!header.contains(".."));
    }
}
