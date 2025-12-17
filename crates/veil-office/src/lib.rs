//! Office document parsing for the Veil privacy platform.
//!
//! This crate provides parsing capabilities for Microsoft Office Open XML formats:
//! - DOCX (Word documents)
//! - XLSX (Excel spreadsheets)
//! - PPTX (PowerPoint presentations)
//!
//! # Features
//!
//! - Extract text content from all Office document types
//! - Preserve position information (page, cell reference, slide number)
//! - Extract document metadata (author, company, etc.)
//! - Handle large Excel files with streaming support
//! - Security protections against ZIP bombs and path traversal
//!
//! # Example
//!
//! ```ignore
//! use veil_office::{parse_docx, parse_pptx};
//! use std::fs::File;
//! use std::io::BufReader;
//!
//! // Parse a Word document
//! let file = File::open("document.docx").unwrap();
//! let result = parse_docx(file).unwrap();
//!
//! for segment in result.segments {
//!     // Use .as_str() for intentional output (content is redacted by default)
//!     println!("{:?}: {}", segment.position, segment.content.as_str());
//! }
//! ```
//!
//! # Supported Formats
//!
//! | Format | Extension | Support |
//! |--------|-----------|---------|
//! | Word | .docx | Full |
//! | Excel | .xlsx | Full (with streaming) |
//! | PowerPoint | .pptx | Full |
//! | Legacy Word | .doc | Not supported |
//! | Legacy Excel | .xls | Not supported |
//! | Legacy PowerPoint | .ppt | Not supported |

pub mod detect;
pub mod error;
pub mod metadata;
pub mod security;
pub mod utils;

// Format-specific modules
pub mod docx;
pub mod pptx;
pub mod xlsx;

// Re-export main types
pub use detect::{detect_office_type, is_encrypted, is_legacy_format, OfficeType};
pub use error::{OfficeError, Result};
pub use metadata::OfficeMetadata;
pub use security::{detect_legacy_format, validate_archive, SecurityLimits};
pub use xlsx::{parse_xlsx_large, parse_xlsx_streaming, StreamingConfig, StreamingStats};

// Re-export veil-parsers types for convenience
pub use veil_parsers::{ParseResult, Position, TextSegment};

use std::io::{BufRead, Read, Seek};

/// Parse an Excel spreadsheet (.xlsx) and extract text content.
///
/// # Arguments
///
/// * `reader` - A reader for the XLSX file data
///
/// # Returns
///
/// A `ParseResult` containing extracted text segments with cell references.
///
/// # Errors
///
/// Returns an error if:
/// - The file is not a valid XLSX document
/// - The file is encrypted
/// - The file exceeds size limits
pub fn parse_xlsx<R: Read + Seek + BufRead + Clone>(reader: R) -> Result<ParseResult> {
    xlsx::parse_xlsx(reader)
}

/// Parse a Word document (.docx) and extract text content.
///
/// # Arguments
///
/// * `reader` - A reader for the DOCX file data
///
/// # Returns
///
/// A `ParseResult` containing extracted text segments with position metadata.
///
/// # Errors
///
/// Returns an error if:
/// - The file is not a valid DOCX document
/// - The file is encrypted
/// - The file exceeds size limits
pub fn parse_docx<R: Read + Seek>(reader: R) -> Result<ParseResult> {
    docx::parse_docx(reader)
}

/// Parse a PowerPoint presentation (.pptx) and extract text content.
///
/// # Arguments
///
/// * `reader` - A reader for the PPTX file data
///
/// # Returns
///
/// A `ParseResult` containing extracted text segments with slide information.
///
/// # Errors
///
/// Returns an error if:
/// - The file is not a valid PPTX document
/// - The file is encrypted
/// - The file exceeds size limits
pub fn parse_pptx<R: Read + Seek>(reader: R) -> Result<ParseResult> {
    pptx::parse_pptx(reader)
}
