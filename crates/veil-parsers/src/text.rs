//! Plain text parser.

use std::time::Instant;

use crate::detect::detect_encoding;
use crate::error::ParseError;
use crate::types::{
    DocumentMetadata, FileFormat, ParseOptions, ParseResult, ParseWarning, Position, TextSegment,
    WarningCode,
};

/// Parser for plain text files.
pub struct TextParser;

impl TextParser {
    /// Create a new text parser.
    pub fn new() -> Self {
        Self
    }

    /// Parse bytes into a ParseResult.
    pub fn parse_bytes(
        &self,
        bytes: &[u8],
        options: &ParseOptions,
    ) -> Result<ParseResult, ParseError> {
        let start = Instant::now();
        let mut warnings = Vec::new();

        // Detect and convert encoding
        let encoding_name = options
            .encoding
            .as_deref()
            .unwrap_or_else(|| detect_encoding(bytes));

        let (content, encoding_lossy) = decode_content(bytes, encoding_name, &mut warnings)?;

        // Parse into segments (one per line)
        let segments = parse_lines(&content);

        let total_chars = segments.iter().map(|s| s.content.len()).sum();

        Ok(ParseResult {
            metadata: DocumentMetadata {
                format: FileFormat::Text,
                encoding: encoding_name.to_string(),
                size_bytes: Some(bytes.len()),
                filename: None,
                encoding_lossy,
            },
            segments,
            warnings,
            total_chars,
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }
}

impl Default for TextParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Decode bytes to UTF-8 string, handling different encodings.
/// Returns (content, was_lossy).
fn decode_content(
    bytes: &[u8],
    encoding_name: &str,
    warnings: &mut Vec<ParseWarning>,
) -> Result<(String, bool), ParseError> {
    let encoding = match encoding_name.to_uppercase().as_str() {
        "UTF-8" | "UTF8" => encoding_rs::UTF_8,
        "UTF-16LE" | "UTF16LE" => encoding_rs::UTF_16LE,
        "UTF-16BE" | "UTF16BE" => encoding_rs::UTF_16BE,
        "ISO-8859-1" | "LATIN1" | "ISO88591" => encoding_rs::WINDOWS_1252,
        _ => encoding_rs::UTF_8,
    };

    let (decoded, _, had_errors) = encoding.decode(bytes);

    if had_errors {
        warnings.push(ParseWarning {
            code: WarningCode::LossyEncoding,
            message: format!(
                "Some characters could not be decoded from {}",
                encoding_name
            ),
            position: None,
        });
    }

    Ok((decoded.into_owned(), had_errors))
}

/// Parse text content into line segments.
fn parse_lines(content: &str) -> Vec<TextSegment> {
    let mut segments = Vec::new();
    let mut byte_offset = 0;

    for (line_idx, line) in content.lines().enumerate() {
        let line_num = line_idx + 1;

        segments.push(TextSegment {
            content: line.to_string().into(),
            position: Position::Text {
                line: line_num,
                column: 1,
                byte_offset,
                byte_length: line.len(),
            },
        });

        // Account for line content plus newline
        byte_offset += line.len() + 1;
    }

    segments
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_text() {
        let parser = TextParser::new();
        let content = b"Hello\nWorld\n";
        let result = parser
            .parse_bytes(content, &ParseOptions::default())
            .unwrap();

        assert_eq!(result.segments.len(), 2);
        assert_eq!(result.segments[0].content.as_str(), "Hello");
        assert_eq!(result.segments[1].content.as_str(), "World");
    }

    #[test]
    fn test_parse_empty() {
        let parser = TextParser::new();
        let content = b"";
        let result = parser
            .parse_bytes(content, &ParseOptions::default())
            .unwrap();

        assert_eq!(result.segments.len(), 0);
    }

    #[test]
    fn test_line_positions() {
        let parser = TextParser::new();
        let content = b"Line 1\nLine 2";
        let result = parser
            .parse_bytes(content, &ParseOptions::default())
            .unwrap();

        if let Position::Text { line, column, .. } = &result.segments[0].position {
            assert_eq!(*line, 1);
            assert_eq!(*column, 1);
        } else {
            panic!("Expected Text position");
        }

        if let Position::Text { line, column, .. } = &result.segments[1].position {
            assert_eq!(*line, 2);
            assert_eq!(*column, 1);
        } else {
            panic!("Expected Text position");
        }
    }
}
