//! Parser trait and type definitions for veil-parsers
//!
//! This file defines the public API contract for the parsing library.
//! Implementation files should not deviate from these definitions.

use serde::{Deserialize, Serialize};
use std::io::Read;
use std::path::Path;

// ============================================================================
// Error Types
// ============================================================================

/// Errors that can occur during parsing
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    /// IO error reading the file
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid or unsupported character encoding
    #[error("Invalid encoding: {0}")]
    Encoding(String),

    /// Malformed CSV data
    #[error("Malformed CSV at row {row}: {message}")]
    CsvError { row: usize, message: String },

    /// Invalid JSON syntax
    #[error("Invalid JSON: {0}")]
    JsonError(#[from] serde_json::Error),

    /// File exceeds size limit
    #[error("File too large: {size} bytes exceeds {max} byte limit")]
    FileTooLarge { size: usize, max: usize },

    /// Format not supported
    #[error("Unsupported format: {0:?}")]
    UnsupportedFormat(Option<String>),
}

// ============================================================================
// Core Types
// ============================================================================

/// Supported file formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileFormat {
    Text,
    Csv,
    Json,
    Html,
}

/// Position information for a text segment
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Position {
    Text {
        line: usize,
        column: usize,
        byte_offset: usize,
        byte_length: usize,
    },
    Csv {
        row: usize,
        column: usize,
        #[serde(skip_serializing_if = "Option::is_none")]
        header: Option<String>,
    },
    Json {
        path: String,
    },
    Html {
        byte_offset: usize,
        byte_length: usize,
    },
}

/// A segment of extracted text with position metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextSegment {
    pub content: String,
    pub position: Position,
}

/// Metadata about the parsed document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub format: FileFormat,
    pub encoding: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    pub encoding_lossy: bool,
}

/// Warning codes for non-fatal issues
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WarningCode {
    LossyEncoding,
    InconsistentColumns,
    FormatMismatch,
    Truncated,
}

/// A non-fatal warning from parsing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseWarning {
    pub code: WarningCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
}

/// Complete result from a parsing operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseResult {
    pub metadata: DocumentMetadata,
    pub segments: Vec<TextSegment>,
    pub warnings: Vec<ParseWarning>,
    pub total_chars: usize,
    pub duration_ms: u64,
}

/// Options for configuring parsing behavior
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ParseOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<FileFormat>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub csv_delimiter: Option<u8>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub csv_has_headers: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_size_bytes: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_streaming: Option<bool>,
}

// ============================================================================
// Parser Trait
// ============================================================================

/// Common interface for all format-specific parsers
pub trait Parser {
    /// Parse content from a byte slice
    fn parse_bytes(&self, bytes: &[u8], options: &ParseOptions) -> Result<ParseResult, ParseError>;

    /// Parse content from a reader (for streaming large files)
    fn parse_reader<R: Read>(
        &self,
        reader: R,
        options: &ParseOptions,
    ) -> Result<ParseResult, ParseError>;

    /// Get the file formats this parser can handle
    fn supported_formats(&self) -> &[FileFormat];
}

// ============================================================================
// Public API Functions
// ============================================================================

/// Parse a file at the given path
///
/// # Arguments
/// * `path` - Path to the file to parse
/// * `options` - Optional parsing configuration
///
/// # Returns
/// * `Ok(ParseResult)` - Parsed content with metadata
/// * `Err(ParseError)` - If parsing fails
///
/// # Example
/// ```rust,ignore
/// use veil_parsers::{parse_file, ParseOptions};
///
/// let result = parse_file("document.csv", &ParseOptions::default())?;
/// for segment in result.segments {
///     println!("{}: {}", segment.position, segment.content);
/// }
/// ```
pub fn parse_file(path: impl AsRef<Path>, options: &ParseOptions) -> Result<ParseResult, ParseError>;

/// Parse content from a byte slice
///
/// # Arguments
/// * `bytes` - Raw file content
/// * `options` - Parsing configuration (should include filename for format detection)
///
/// # Returns
/// * `Ok(ParseResult)` - Parsed content with metadata
/// * `Err(ParseError)` - If parsing fails
pub fn parse_bytes(bytes: &[u8], options: &ParseOptions) -> Result<ParseResult, ParseError>;

/// Parse content from a reader
///
/// Preferred for large files as it enables streaming.
///
/// # Arguments
/// * `reader` - Any type implementing `Read`
/// * `options` - Parsing configuration
///
/// # Returns
/// * `Ok(ParseResult)` - Parsed content with metadata
/// * `Err(ParseError)` - If parsing fails
pub fn parse_reader<R: Read>(reader: R, options: &ParseOptions) -> Result<ParseResult, ParseError>;

/// Detect the format of file content
///
/// # Arguments
/// * `bytes` - First bytes of the file (at least 1KB recommended)
/// * `filename` - Optional filename for extension-based hints
///
/// # Returns
/// Detected `FileFormat`
pub fn detect_format(bytes: &[u8], filename: Option<&str>) -> FileFormat;

/// Check if a file format is supported
pub fn is_supported(format: FileFormat) -> bool;

/// Get list of supported file extensions
pub fn supported_extensions() -> &'static [&'static str];
