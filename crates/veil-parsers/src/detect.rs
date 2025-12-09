//! Format and encoding detection.

use crate::types::FileFormat;

/// Detect the format of file content.
///
/// Uses a combination of:
/// 1. File extension (if provided)
/// 2. Content analysis (magic bytes, structure)
pub fn detect_format(bytes: &[u8], filename: Option<&str>) -> FileFormat {
    // First, check extension if filename provided
    if let Some(name) = filename {
        if let Some(ext) = name.rsplit('.').next() {
            match ext.to_lowercase().as_str() {
                "csv" | "tsv" => return FileFormat::Csv,
                "json" => return FileFormat::Json,
                "html" | "htm" => return FileFormat::Html,
                "txt" | "log" | "md" => return FileFormat::Text,
                _ => {}
            }
        }
    }

    // Content-based detection
    detect_format_from_content(bytes)
}

/// Detect format from content alone.
fn detect_format_from_content(bytes: &[u8]) -> FileFormat {
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
        assert_eq!(
            detect_format(b"content", Some("file.json")),
            FileFormat::Json
        );
        assert_eq!(detect_format(b"content", Some("file.csv")), FileFormat::Csv);
        assert_eq!(
            detect_format(b"content", Some("file.html")),
            FileFormat::Html
        );
    }
}
