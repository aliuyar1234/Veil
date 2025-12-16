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
pub mod pdf;
pub mod stream;
mod text;
mod types;

pub use error::ParseError;
pub use types::{
    DocumentMetadata, FileFormat, ParseOptions, ParseResult, ParseWarning, Position, TableCell,
    TextSegment, WarningCode,
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
        FileFormat::Pdf => pdf::PdfParser::new().parse_bytes(bytes, options),
        // Office formats are handled by veil-office crate
        FileFormat::Docx | FileFormat::Xlsx | FileFormat::Pptx => {
            Err(ParseError::UnsupportedFormat(Some(format!(
                "{:?} format requires veil-office crate",
                format
            ))))
        }
        // Email formats are handled by veil-email crate
        FileFormat::Eml | FileFormat::Msg => Err(ParseError::UnsupportedFormat(Some(format!(
            "{:?} format requires veil-email crate",
            format
        )))),
    }
}

/// Parse content from a reader.
///
/// For large files, consider using [`parse_reader_streaming`] instead,
/// which processes content without loading everything into memory.
///
/// # Arguments
/// * `reader` - Any type implementing `Read`
/// * `options` - Parsing configuration
///
/// # Returns
/// * `Ok(ParseResult)` - Parsed content with metadata
/// * `Err(ParseError)` - If parsing fails
pub fn parse_reader<R: Read>(reader: R, options: &ParseOptions) -> Result<ParseResult, ParseError> {
    // Check if streaming is enabled and format supports it
    let streaming_enabled = options.enable_streaming.unwrap_or(false);
    let format = options.format;

    // Try streaming if enabled and format supports it
    if streaming_enabled {
        if let Some(fmt) = format {
            if stream::ParseStream::<R>::supports_streaming(fmt) {
                return parse_reader_streaming_impl(reader, options);
            }
        }
    }

    // Fall back to buffered parsing
    let mut buf = Vec::new();
    let mut reader = reader;
    reader.read_to_end(&mut buf)?;
    parse_bytes(&buf, options)
}

/// Parse content from a reader using streaming.
///
/// Processes the reader incrementally without loading the entire file into memory.
/// This is ideal for large text or CSV files. Falls back to buffered parsing for
/// formats that don't support streaming (JSON, HTML, PDF).
///
/// # Arguments
/// * `reader` - Any type implementing `Read`
/// * `options` - Parsing configuration
///
/// # Returns
/// * `Ok(ParseResult)` - Parsed content with metadata
/// * `Err(ParseError)` - If parsing fails
///
/// # Example
/// ```rust,ignore
/// use std::fs::File;
/// use veil_parsers::{parse_reader_streaming, ParseOptions, FileFormat};
///
/// let file = File::open("large_file.csv")?;
/// let options = ParseOptions {
///     format: Some(FileFormat::Csv),
///     ..Default::default()
/// };
/// let result = parse_reader_streaming(file, &options)?;
/// ```
pub fn parse_reader_streaming<R: Read>(
    reader: R,
    options: &ParseOptions,
) -> Result<ParseResult, ParseError> {
    parse_reader_streaming_impl(reader, options)
}

fn parse_reader_streaming_impl<R: Read>(
    reader: R,
    options: &ParseOptions,
) -> Result<ParseResult, ParseError> {
    use std::io::BufReader;
    use std::time::Instant;

    let start = Instant::now();

    // Detect format from first chunk if not specified
    let mut buf_reader = BufReader::new(reader);
    let mut header = vec![0u8; 1024];

    // We need to read the header for detection, then continue
    let n = std::io::Read::read(&mut buf_reader, &mut header)?;
    header.truncate(n);

    let format = options
        .format
        .unwrap_or_else(|| detect::detect_format(&header, None));

    // Check if format supports streaming
    if !stream::ParseStream::<R>::supports_streaming(format) {
        // Fall back to buffered for unsupported formats
        let mut rest = Vec::new();
        std::io::Read::read_to_end(&mut buf_reader, &mut rest)?;
        let mut full = header;
        full.extend(rest);
        return parse_bytes(&full, options);
    }

    // Use streaming
    let chained = std::io::Cursor::new(header).chain(buf_reader);
    let stream = stream::ParseStream::new(chained, format, options)?
        .ok_or_else(|| ParseError::UnsupportedFormat(Some("Format does not support streaming".to_string())))?;

    let mut segments = Vec::new();
    let mut warnings = Vec::new();
    let mut total_chars = 0;

    for result in stream {
        match result {
            Ok(segment) => {
                total_chars += segment.content.len();
                segments.push(segment);
            }
            Err(e) => {
                // For recoverable errors, log warning and continue
                warnings.push(ParseWarning {
                    code: WarningCode::Truncated,
                    message: format!("Streaming error: {}", e),
                    position: None,
                });
            }
        }
    }

    let encoding = match format {
        FileFormat::Text => options.encoding.clone().unwrap_or_else(|| "UTF-8".to_string()),
        _ => "UTF-8".to_string(),
    };

    Ok(ParseResult {
        metadata: DocumentMetadata {
            format,
            encoding,
            size_bytes: None, // Unknown when streaming
            filename: None,
            encoding_lossy: false,
        },
        segments,
        warnings,
        total_chars,
        duration_ms: start.elapsed().as_millis() as u64,
    })
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
        FileFormat::Text | FileFormat::Csv | FileFormat::Json | FileFormat::Html | FileFormat::Pdf
    )
}

/// Get list of supported file extensions.
pub fn supported_extensions() -> &'static [&'static str] {
    &["txt", "log", "csv", "tsv", "json", "html", "htm", "pdf"]
}
