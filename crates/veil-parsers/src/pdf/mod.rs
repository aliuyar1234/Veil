//! PDF document parsing for text extraction.
//!
//! This module provides text extraction from PDF documents with position
//! metadata for PII detection.

mod error;

pub use error::PdfError;

use crate::error::ParseError;
use crate::types::{
    DocumentMetadata, FileFormat, ParseOptions, ParseResult, ParseWarning, Position, TextSegment,
    WarningCode,
};

use std::time::Instant;

/// PDF parser using pdf-extract crate.
pub struct PdfParser;

impl PdfParser {
    /// Create a new PDF parser.
    pub fn new() -> Self {
        Self
    }

    /// Parse a PDF from bytes.
    pub fn parse_bytes(
        &self,
        bytes: &[u8],
        options: &ParseOptions,
    ) -> Result<ParseResult, ParseError> {
        let start = Instant::now();
        let mut warnings = Vec::new();

        // Extract text using pdf-extract
        let text = match pdf_extract::extract_text_from_mem(bytes) {
            Ok(text) => text,
            Err(e) => {
                // Check for specific error types
                let err_str = e.to_string().to_lowercase();
                if err_str.contains("encrypted") || err_str.contains("password") {
                    return Err(PdfError::Encrypted.into());
                }
                if err_str.contains("invalid") || err_str.contains("corrupt") {
                    return Err(PdfError::Corrupted(e.to_string()).into());
                }
                return Err(PdfError::ParseError(e.to_string()).into());
            }
        };

        // Check if we got any text
        let text = text.trim();
        if text.is_empty() {
            warnings.push(ParseWarning {
                code: WarningCode::FormatMismatch,
                message: "No extractable text found - document may be scanned".to_string(),
                position: None,
            });
        }

        // Split text into segments by page markers or paragraphs
        let segments = self.create_segments(text);

        let total_chars = segments.iter().map(|s| s.content.len()).sum();
        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(ParseResult {
            metadata: DocumentMetadata {
                format: FileFormat::Pdf,
                encoding: "UTF-8".to_string(),
                size_bytes: Some(bytes.len()),
                filename: options.format.map(|_| "document.pdf".to_string()),
                encoding_lossy: false,
            },
            segments,
            warnings,
            total_chars,
            duration_ms,
        })
    }

    /// Create text segments from extracted text.
    fn create_segments(&self, text: &str) -> Vec<TextSegment> {
        let mut segments = Vec::new();
        let mut byte_offset = 0;
        let mut current_page = 1;

        // Split by form feed (page break) or double newlines
        for chunk in text.split("\x0C") {
            // Skip empty chunks
            let chunk = chunk.trim();
            if chunk.is_empty() {
                current_page += 1;
                continue;
            }

            // Further split by paragraphs (double newline)
            for para in chunk.split("\n\n") {
                let para = para.trim();
                if para.is_empty() {
                    continue;
                }

                let content = para.to_string();
                let byte_length = content.len();

                segments.push(TextSegment {
                    content: content.into(),
                    position: Position::Pdf {
                        page: current_page,
                        x: 0.0,     // pdf-extract doesn't provide position
                        y: 0.0,     // pdf-extract doesn't provide position
                        width: 0.0, // pdf-extract doesn't provide position
                        height: 0.0,
                        byte_offset,
                        byte_length,
                    },
                });

                byte_offset += byte_length + 2; // +2 for paragraph separator
            }

            current_page += 1;
        }

        // If no page breaks found, create segments from the whole text
        if segments.is_empty() && !text.is_empty() {
            // Split by paragraphs only
            for para in text.split("\n\n") {
                let para = para.trim();
                if para.is_empty() {
                    continue;
                }

                let content = para.to_string();
                let byte_length = content.len();

                segments.push(TextSegment {
                    content: content.into(),
                    position: Position::Pdf {
                        page: 1,
                        x: 0.0,
                        y: 0.0,
                        width: 0.0,
                        height: 0.0,
                        byte_offset,
                        byte_length,
                    },
                });

                byte_offset += byte_length + 2;
            }
        }

        // If still empty but text exists, create one segment
        if segments.is_empty() && !text.is_empty() {
            segments.push(TextSegment {
                content: text.to_string().into(),
                position: Position::Pdf {
                    page: 1,
                    x: 0.0,
                    y: 0.0,
                    width: 0.0,
                    height: 0.0,
                    byte_offset: 0,
                    byte_length: text.len(),
                },
            });
        }

        segments
    }
}

impl Default for PdfParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdf_parser_new() {
        let parser = PdfParser::new();
        // Parser created successfully
        let _ = parser;
    }

    #[test]
    fn test_create_segments_empty() {
        let parser = PdfParser::new();
        let segments = parser.create_segments("");
        assert!(segments.is_empty());
    }

    #[test]
    fn test_create_segments_single() {
        let parser = PdfParser::new();
        let segments = parser.create_segments("Hello World");
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].content.as_str(), "Hello World");
    }

    #[test]
    fn test_create_segments_paragraphs() {
        let parser = PdfParser::new();
        let segments = parser.create_segments("First paragraph.\n\nSecond paragraph.");
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].content.as_str(), "First paragraph.");
        assert_eq!(segments[1].content.as_str(), "Second paragraph.");
    }

    #[test]
    fn test_parse_invalid_pdf() {
        let parser = PdfParser::new();
        let result = parser.parse_bytes(b"not a pdf", &ParseOptions::default());
        assert!(result.is_err());
    }
}
