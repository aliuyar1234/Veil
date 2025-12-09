//! JSON parser.

use std::time::Instant;

use serde_json::Value;

use crate::error::ParseError;
use crate::types::{
    DocumentMetadata, FileFormat, ParseOptions, ParseResult, Position, TextSegment,
};

/// Parser for JSON files.
pub struct JsonParser;

impl JsonParser {
    /// Create a new JSON parser.
    pub fn new() -> Self {
        Self
    }

    /// Parse bytes into a ParseResult.
    pub fn parse_bytes(
        &self,
        bytes: &[u8],
        _options: &ParseOptions,
    ) -> Result<ParseResult, ParseError> {
        let start = Instant::now();

        let value: Value = serde_json::from_slice(bytes)?;

        let mut segments = Vec::new();
        extract_strings(&value, "$", &mut segments);

        let total_chars = segments.iter().map(|s| s.content.len()).sum();

        Ok(ParseResult {
            metadata: DocumentMetadata {
                format: FileFormat::Json,
                encoding: "UTF-8".to_string(),
                size_bytes: Some(bytes.len()),
                filename: None,
                encoding_lossy: false,
            },
            segments,
            warnings: Vec::new(),
            total_chars,
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }
}

impl Default for JsonParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Recursively extract string values with their JSON paths.
fn extract_strings(value: &Value, path: &str, results: &mut Vec<TextSegment>) {
    match value {
        Value::String(s) => {
            results.push(TextSegment {
                content: s.clone(),
                position: Position::Json {
                    path: path.to_string(),
                },
            });
        }
        Value::Object(map) => {
            for (key, val) in map {
                let new_path = format!("{}.{}", path, key);
                extract_strings(val, &new_path, results);
            }
        }
        Value::Array(arr) => {
            for (idx, val) in arr.iter().enumerate() {
                let new_path = format!("{}[{}]", path, idx);
                extract_strings(val, &new_path, results);
            }
        }
        // Skip numbers, booleans, nulls
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_json() {
        let parser = JsonParser::new();
        let content = br#"{"name": "John", "email": "john@example.com"}"#;
        let result = parser
            .parse_bytes(content, &ParseOptions::default())
            .unwrap();

        assert_eq!(result.segments.len(), 2);
    }

    #[test]
    fn test_json_paths() {
        let parser = JsonParser::new();
        let content = br#"{"user": {"name": "John"}}"#;
        let result = parser
            .parse_bytes(content, &ParseOptions::default())
            .unwrap();

        assert_eq!(result.segments.len(), 1);
        if let Position::Json { path } = &result.segments[0].position {
            assert_eq!(path, "$.user.name");
        }
    }

    #[test]
    fn test_json_array_paths() {
        let parser = JsonParser::new();
        let content = br#"{"users": ["Alice", "Bob"]}"#;
        let result = parser
            .parse_bytes(content, &ParseOptions::default())
            .unwrap();

        assert_eq!(result.segments.len(), 2);

        if let Position::Json { path } = &result.segments[0].position {
            assert_eq!(path, "$.users[0]");
        }
        if let Position::Json { path } = &result.segments[1].position {
            assert_eq!(path, "$.users[1]");
        }
    }

    #[test]
    fn test_skip_non_strings() {
        let parser = JsonParser::new();
        let content = br#"{"name": "John", "age": 30, "active": true}"#;
        let result = parser
            .parse_bytes(content, &ParseOptions::default())
            .unwrap();

        // Only "John" should be extracted
        assert_eq!(result.segments.len(), 1);
        assert_eq!(result.segments[0].content, "John");
    }
}
