//! # veil-parsers
//!
//! Document parsing library for PII detection.
//! Supports plain text, CSV, JSON, and HTML file formats.
//!
//! ## Example
//!
//! ```rust,ignore
//! use veil_parsers::{parse_file, ParseOptions};
//!
//! let result = parse_file("document.txt", &ParseOptions::default())?;
//! for segment in result.segments {
//!     println!("{}", segment.content);
//! }
//! ```

mod csv;
mod detect;
mod error;
mod html;
mod json;
mod text;
mod types;

pub use error::ParseError;
pub use types::{
    DocumentMetadata, FileFormat, ParseOptions, ParseResult, ParseWarning, Position, TextSegment,
    WarningCode,
};

use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Parse a file at the given path.
///
/// # Arguments
/// * `path` - Path to the file to parse
/// * `options` - Parsing configuration
///
/// # Returns
/// * `Ok(ParseResult)` - Parsed content with metadata
/// * `Err(ParseError)` - If parsing fails
pub fn parse_file(
    path: impl AsRef<Path>,
    options: &ParseOptions,
) -> Result<ParseResult, ParseError> {
    let path = path.as_ref();
    let file = File::open(path)?;
    let metadata = file.metadata()?;
    let size = metadata.len() as usize;

    // Check file size limit
    let max_size = options.max_size_bytes.unwrap_or(100 * 1024 * 1024);
    if size > max_size {
        return Err(ParseError::FileTooLarge {
            size,
            max: max_size,
        });
    }

    let mut options = options.clone();
    if options.format.is_none() {
        // Read first 1KB for format detection
        let mut buf = vec![0u8; 1024.min(size)];
        let mut f = File::open(path)?;
        let n = f.read(&mut buf)?;
        buf.truncate(n);
        options.format = Some(detect::detect_format(&buf, path.to_str()));
    }

    let mut reader = File::open(path)?;
    parse_reader(&mut reader, &options)
}

/// Parse content from a byte slice.
///
/// # Arguments
/// * `bytes` - Raw file content
/// * `options` - Parsing configuration
///
/// # Returns
/// * `Ok(ParseResult)` - Parsed content with metadata
/// * `Err(ParseError)` - If parsing fails
pub fn parse_bytes(bytes: &[u8], options: &ParseOptions) -> Result<ParseResult, ParseError> {
    let max_size = options.max_size_bytes.unwrap_or(100 * 1024 * 1024);
    if bytes.len() > max_size {
        return Err(ParseError::FileTooLarge {
            size: bytes.len(),
            max: max_size,
        });
    }

    let format = options
        .format
        .unwrap_or_else(|| detect::detect_format(bytes, None));

    match format {
        FileFormat::Text => text::TextParser::new().parse_bytes(bytes, options),
        FileFormat::Csv => csv::CsvParser::new().parse_bytes(bytes, options),
        FileFormat::Json => json::JsonParser::new().parse_bytes(bytes, options),
        FileFormat::Html => html::HtmlParser::new().parse_bytes(bytes, options),
    }
}

/// Parse content from a reader.
///
/// # Arguments
/// * `reader` - Any type implementing `Read`
/// * `options` - Parsing configuration
///
/// # Returns
/// * `Ok(ParseResult)` - Parsed content with metadata
/// * `Err(ParseError)` - If parsing fails
pub fn parse_reader<R: Read>(reader: R, options: &ParseOptions) -> Result<ParseResult, ParseError> {
    // For simplicity, read all content into memory
    // Future: implement true streaming for large files
    let mut buf = Vec::new();
    let mut reader = reader;
    reader.read_to_end(&mut buf)?;
    parse_bytes(&buf, options)
}

/// Detect the format of file content.
///
/// # Arguments
/// * `bytes` - First bytes of the file (at least 1KB recommended)
/// * `filename` - Optional filename for extension-based hints
///
/// # Returns
/// Detected `FileFormat`
pub fn detect_format(bytes: &[u8], filename: Option<&str>) -> FileFormat {
    detect::detect_format(bytes, filename)
}

/// Check if a file format is supported.
pub fn is_supported(format: FileFormat) -> bool {
    matches!(
        format,
        FileFormat::Text | FileFormat::Csv | FileFormat::Json | FileFormat::Html
    )
}

/// Get list of supported file extensions.
pub fn supported_extensions() -> &'static [&'static str] {
    &["txt", "log", "csv", "tsv", "json", "html", "htm"]
}
