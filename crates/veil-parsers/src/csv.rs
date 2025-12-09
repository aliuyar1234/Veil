//! CSV parser.

use std::time::Instant;

use crate::error::ParseError;
use crate::types::{
    DocumentMetadata, FileFormat, ParseOptions, ParseResult, ParseWarning, Position, TextSegment,
    WarningCode,
};

/// Parser for CSV files.
pub struct CsvParser;

impl CsvParser {
    /// Create a new CSV parser.
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
        let mut segments = Vec::new();

        let delimiter = options.csv_delimiter.unwrap_or(b',');
        let has_headers = options.csv_has_headers.unwrap_or(true);

        let mut reader = csv::ReaderBuilder::new()
            .delimiter(delimiter)
            .has_headers(has_headers)
            .flexible(true)
            .from_reader(bytes);

        // Get headers if present
        let headers: Vec<String> = if has_headers {
            reader
                .headers()
                .map(|h| h.iter().map(|s| s.to_string()).collect())
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        let mut expected_columns: Option<usize> = None;

        for (row_idx, result) in reader.records().enumerate() {
            let record = result.map_err(|e| ParseError::CsvError {
                row: row_idx + if has_headers { 2 } else { 1 },
                message: e.to_string(),
            })?;

            let row_num = row_idx + if has_headers { 2 } else { 1 };

            // Check column consistency
            if let Some(expected) = expected_columns {
                if record.len() != expected {
                    warnings.push(ParseWarning {
                        code: WarningCode::InconsistentColumns,
                        message: format!(
                            "Row {} has {} columns, expected {}",
                            row_num,
                            record.len(),
                            expected
                        ),
                        position: Some(Position::Csv {
                            row: row_num,
                            column: 0,
                            header: None,
                        }),
                    });
                }
            } else {
                expected_columns = Some(record.len());
            }

            for (col_idx, field) in record.iter().enumerate() {
                let header = headers.get(col_idx).cloned();

                segments.push(TextSegment {
                    content: field.to_string(),
                    position: Position::Csv {
                        row: row_num,
                        column: col_idx,
                        header,
                    },
                });
            }
        }

        let total_chars = segments.iter().map(|s| s.content.len()).sum();

        Ok(ParseResult {
            metadata: DocumentMetadata {
                format: FileFormat::Csv,
                encoding: "UTF-8".to_string(),
                size_bytes: Some(bytes.len()),
                filename: None,
                encoding_lossy: false,
            },
            segments,
            warnings,
            total_chars,
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }
}

impl Default for CsvParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_csv() {
        let parser = CsvParser::new();
        let content = b"name,email\nJohn,john@example.com\nJane,jane@example.com";
        let result = parser
            .parse_bytes(content, &ParseOptions::default())
            .unwrap();

        // 2 rows * 2 columns = 4 segments
        assert_eq!(result.segments.len(), 4);
        assert_eq!(result.segments[0].content, "John");
        assert_eq!(result.segments[1].content, "john@example.com");
    }

    #[test]
    fn test_csv_headers() {
        let parser = CsvParser::new();
        let content = b"name,email\nJohn,john@example.com";
        let result = parser
            .parse_bytes(content, &ParseOptions::default())
            .unwrap();

        if let Position::Csv { header, .. } = &result.segments[1].position {
            assert_eq!(header.as_deref(), Some("email"));
        }
    }

    #[test]
    fn test_semicolon_delimiter() {
        let parser = CsvParser::new();
        let content = b"name;email\nJohn;john@example.com";
        let options = ParseOptions {
            csv_delimiter: Some(b';'),
            ..Default::default()
        };
        let result = parser.parse_bytes(content, &options).unwrap();

        assert_eq!(result.segments.len(), 2);
        assert_eq!(result.segments[0].content, "John");
    }
}
