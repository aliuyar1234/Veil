//! Format and encoding detection.

use crate::types::FileFormat;

/// Detect the format of file content.
///
/// Uses a combination of:
/// 1. Content analysis (magic bytes, structure) - PRIMARY
/// 2. File extension as fallback hint
///
/// # Security
///
/// Content-based detection takes priority over file extensions to prevent
/// format confusion attacks where a malicious file has a misleading extension.
pub fn detect_format(bytes: &[u8], filename: Option<&str>) -> FileFormat {
    // Security: Always try content-based detection first
    // This prevents format confusion attacks where extension doesn't match content
    let content_format = detect_format_from_content(bytes);

    // If content detection found a specific format (not just Text), trust it
    if content_format != FileFormat::Text {
        return content_format;
    }

    // For text-like content, use extension as a hint
    if let Some(name) = filename {
        if let Some(ext) = name.rsplit('.').next() {
            match ext.to_lowercase().as_str() {
                // Only trust extension for ambiguous text formats
                "csv" | "tsv" => {
                    // Verify it actually looks like CSV
                    if looks_like_csv(bytes) {
                        return FileFormat::Csv;
                    }
                }
                "json" => {
                    // Verify it starts with { or [
                    let trimmed = bytes
                        .iter()
                        .position(|&b| !b.is_ascii_whitespace())
                        .map(|pos| &bytes[pos..])
                        .unwrap_or(&[]);
                    if !trimmed.is_empty() && (trimmed[0] == b'{' || trimmed[0] == b'[') {
                        return FileFormat::Json;
                    }
                }
                "html" | "htm" => {
                    // Already checked in content detection
                }
                "txt" | "log" | "md" => return FileFormat::Text,
                // Binary formats must be detected by content, not extension
                // This prevents malicious files from being processed incorrectly
                _ => {}
            }
        }
    }

    content_format
}

/// Detect format from content alone.
fn detect_format_from_content(bytes: &[u8]) -> FileFormat {
    // Check for PDF magic bytes first (before BOM handling)
    if bytes.starts_with(b"%PDF-") {
        return FileFormat::Pdf;
    }

    // Check for MSG magic bytes (OLE Compound Document)
    // D0 CF 11 E0 A1 B1 1A E1
    if bytes.len() >= 8
        && bytes[0] == 0xD0
        && bytes[1] == 0xCF
        && bytes[2] == 0x11
        && bytes[3] == 0xE0
        && bytes[4] == 0xA1
        && bytes[5] == 0xB1
        && bytes[6] == 0x1A
        && bytes[7] == 0xE1
    {
        return FileFormat::Msg;
    }

    // Check for ZIP magic bytes (Office Open XML formats)
    // PK\x03\x04 is the ZIP local file header signature
    if bytes.starts_with(&[0x50, 0x4B, 0x03, 0x04]) {
        // This is a ZIP file, could be DOCX, XLSX, or PPTX
        // For content-only detection, we check internal paths
        return detect_office_format_from_zip(bytes);
    }

    // Skip BOM if present
    let content = skip_bom(bytes);

    // Skip leading whitespace
    let trimmed = content
        .iter()
        .position(|&b| !b.is_ascii_whitespace())
        .map(|pos| &content[pos..])
        .unwrap_or(&[]);

    if trimmed.is_empty() {
        return FileFormat::Text;
    }

    match trimmed[0] {
        b'{' | b'[' => {
            // Likely JSON
            if looks_like_json(trimmed) {
                FileFormat::Json
            } else {
                FileFormat::Text
            }
        }
        b'<' => {
            // Could be HTML/XML
            if looks_like_html(trimmed) {
                FileFormat::Html
            } else {
                FileFormat::Text
            }
        }
        _ => {
            // Check for CSV patterns
            if looks_like_csv(content) {
                FileFormat::Csv
            } else {
                FileFormat::Text
            }
        }
    }
}

/// Skip BOM (Byte Order Mark) if present.
fn skip_bom(bytes: &[u8]) -> &[u8] {
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        // UTF-8 BOM
        &bytes[3..]
    } else if bytes.starts_with(&[0xFF, 0xFE]) || bytes.starts_with(&[0xFE, 0xFF]) {
        // UTF-16 BOM
        &bytes[2..]
    } else {
        bytes
    }
}

/// Check if content looks like JSON.
fn looks_like_json(bytes: &[u8]) -> bool {
    // Simple heuristic: starts with { or [ and contains : or ,
    if bytes.is_empty() {
        return false;
    }

    let first = bytes[0];
    if first != b'{' && first != b'[' {
        return false;
    }

    // Look for JSON structure indicators
    bytes.iter().any(|&b| b == b':' || b == b',')
}

/// Check if content looks like HTML.
fn looks_like_html(bytes: &[u8]) -> bool {
    let content = std::str::from_utf8(bytes).unwrap_or("");
    let lower = content.to_lowercase();

    // Check for common HTML indicators
    lower.contains("<!doctype html")
        || lower.contains("<html")
        || lower.contains("<head")
        || lower.contains("<body")
        || lower.contains("<div")
        || lower.contains("<p>")
        || lower.contains("<span")
}

/// Check if content looks like CSV.
fn looks_like_csv(bytes: &[u8]) -> bool {
    let content = std::str::from_utf8(bytes).unwrap_or("");

    // Count potential delimiters in first few lines
    let mut comma_count = 0;
    let mut semicolon_count = 0;
    let mut tab_count = 0;
    let mut lines_checked = 0;

    for line in content.lines().take(5) {
        if line.trim().is_empty() {
            continue;
        }
        comma_count += line.matches(',').count();
        semicolon_count += line.matches(';').count();
        tab_count += line.matches('\t').count();
        lines_checked += 1;
    }

    if lines_checked < 2 {
        return false;
    }

    // CSV typically has consistent delimiter usage across lines
    let avg_commas = comma_count / lines_checked;
    let avg_semicolons = semicolon_count / lines_checked;
    let avg_tabs = tab_count / lines_checked;

    // At least 1 delimiter per line on average suggests CSV
    avg_commas >= 1 || avg_semicolons >= 1 || avg_tabs >= 1
}

/// Detect Office format by examining ZIP content strings.
///
/// Office Open XML files are ZIP archives containing specific marker files:
/// - DOCX: word/document.xml
/// - XLSX: xl/workbook.xml
/// - PPTX: ppt/presentation.xml
fn detect_office_format_from_zip(bytes: &[u8]) -> FileFormat {
    // Simple heuristic: search for signature paths in the raw bytes
    // This avoids parsing the full ZIP structure for detection
    let content = bytes;

    // Look for Office-specific paths in the ZIP data
    if content.windows(17).any(|w| w == b"word/document.xml") {
        return FileFormat::Docx;
    }
    if content.windows(14).any(|w| w == b"xl/workbook.xml") {
        return FileFormat::Xlsx;
    }
    if content.windows(20).any(|w| w == b"ppt/presentation.xml") {
        return FileFormat::Pptx;
    }

    // If we can't determine the specific type, default to XLSX
    // (most common Office format for data processing)
    FileFormat::Xlsx
}

/// Detect character encoding from BOM.
pub fn detect_encoding(bytes: &[u8]) -> &'static str {
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        "UTF-8"
    } else if bytes.starts_with(&[0xFF, 0xFE]) {
        "UTF-16LE"
    } else if bytes.starts_with(&[0xFE, 0xFF]) {
        "UTF-16BE"
    } else {
        // Default to UTF-8
        "UTF-8"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_json() {
        assert_eq!(
            detect_format(b"{\"key\": \"value\"}", None),
            FileFormat::Json
        );
        assert_eq!(detect_format(b"[1, 2, 3]", None), FileFormat::Json);
    }

    #[test]
    fn test_detect_html() {
        assert_eq!(
            detect_format(b"<!DOCTYPE html><html>", None),
            FileFormat::Html
        );
        assert_eq!(detect_format(b"<html><body>", None), FileFormat::Html);
    }

    #[test]
    fn test_detect_csv() {
        assert_eq!(detect_format(b"a,b,c\n1,2,3\n4,5,6", None), FileFormat::Csv);
    }

    #[test]
    fn test_detect_by_extension() {
        // Security: Content detection now takes priority over extension
        // Extension is only used as a hint for ambiguous text formats

        // JSON: Extension + valid JSON-like content required
        assert_eq!(
            detect_format(b"{\"key\": \"value\"}", Some("file.json")),
            FileFormat::Json
        );
        // JSON extension with non-JSON content should NOT trust extension alone
        assert_eq!(
            detect_format(b"plain text content", Some("file.json")),
            FileFormat::Text
        );

        // CSV: Extension + CSV-like content required
        assert_eq!(
            detect_format(b"a,b,c\n1,2,3", Some("file.csv")),
            FileFormat::Csv
        );

        // HTML: Detected by content magic bytes
        assert_eq!(
            detect_format(b"<html><body>", Some("file.html")),
            FileFormat::Html
        );

        // TXT: Extension trusted for explicit text files
        assert_eq!(
            detect_format(b"content", Some("file.txt")),
            FileFormat::Text
        );
    }

    #[test]
    fn test_content_priority_over_extension() {
        // Security: Content detection should override misleading extensions

        // PDF content with wrong extension should still detect as PDF
        assert_eq!(
            detect_format(b"%PDF-1.4 test content", Some("fake.txt")),
            FileFormat::Pdf
        );

        // JSON content without extension should detect as JSON
        assert_eq!(detect_format(b"{\"data\": 123}", None), FileFormat::Json);
    }
}
