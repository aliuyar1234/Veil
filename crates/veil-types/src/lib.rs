//! Shared types used across Veil crates.
//!
//! This crate contains the core data model for extracted text, position
//! metadata, and parse results. It is intentionally kept free of any parsing
//! implementation details so that downstream crates (e.g. detection) don't need
//! to depend on full parser implementations.

use serde::{Deserialize, Serialize};
use veil_core::SensitiveString;

/// Supported file formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileFormat {
    /// Plain text file (.txt, .log, etc.)
    Text,
    /// Comma-separated values (.csv, .tsv)
    Csv,
    /// JSON data (.json)
    Json,
    /// HTML document (.html, .htm)
    Html,
    /// PDF document (.pdf)
    Pdf,
    /// Word document (.docx)
    Docx,
    /// Excel spreadsheet (.xlsx)
    Xlsx,
    /// PowerPoint presentation (.pptx)
    Pptx,
    /// Email message (.eml)
    Eml,
    /// Microsoft Outlook message (.msg)
    Msg,
}

/// Position information for a text segment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Position {
    /// Plain text position.
    Text {
        /// 1-indexed line number.
        line: usize,
        /// 1-indexed column number.
        column: usize,
        /// 0-indexed byte offset from file start.
        byte_offset: usize,
        /// Length in bytes.
        byte_length: usize,
    },
    /// CSV cell position.
    Csv {
        /// 1-indexed row number.
        row: usize,
        /// 0-indexed column index.
        column: usize,
        /// Column header name if headers are present.
        #[serde(skip_serializing_if = "Option::is_none")]
        header: Option<String>,
    },
    /// JSON value position.
    Json {
        /// JSONPath notation (e.g., `$.users[0].email`).
        path: String,
    },
    /// HTML text position.
    Html {
        /// Approximate byte offset in original HTML.
        byte_offset: usize,
        /// Byte length in original HTML.
        byte_length: usize,
    },
    /// PDF document position.
    Pdf {
        /// Page number (1-indexed).
        page: usize,
        /// Left edge in PDF points (1/72 inch).
        x: f32,
        /// Bottom edge in PDF points.
        y: f32,
        /// Width in PDF points.
        width: f32,
        /// Height in PDF points.
        height: f32,
        /// Cumulative byte offset (for Finding compatibility).
        byte_offset: usize,
        /// Byte length of the text.
        byte_length: usize,
    },
    /// Word document (.docx) position.
    Docx {
        /// Section type (body, header, footer, table, etc.).
        section: String,
        /// Paragraph number (1-indexed).
        paragraph: usize,
        /// Character offset within the paragraph.
        char_offset: usize,
        /// Character length.
        char_length: usize,
        /// Table cell location if within a table.
        #[serde(skip_serializing_if = "Option::is_none")]
        table_cell: Option<TableCell>,
    },
    /// Excel spreadsheet (.xlsx) position.
    Xlsx {
        /// Sheet name.
        sheet: String,
        /// Row number (1-indexed).
        row: usize,
        /// Column index (0-indexed).
        column: usize,
        /// Column letter (A, B, ..., Z, AA, AB, ...).
        column_letter: String,
        /// Full cell reference (e.g., "Sheet1!A1").
        cell_ref: String,
        /// Whether the sheet is hidden.
        hidden_sheet: bool,
    },
    /// PowerPoint presentation (.pptx) position.
    Pptx {
        /// Slide number (1-indexed).
        slide: usize,
        /// Element type (title, body, note, shape, table).
        element: String,
        /// Text element index within the slide.
        text_index: usize,
        /// Character offset within the text element.
        char_offset: usize,
        /// Character length.
        char_length: usize,
    },
    /// Office document metadata field.
    OfficeMetadata {
        /// Field name (creator, title, company, etc.).
        field: String,
    },
    /// Email message position.
    Email {
        /// Field name (from_address, to_name, subject, body, etc.).
        field: String,
        /// Field index for repeated headers.
        #[serde(skip_serializing_if = "Option::is_none")]
        field_index: Option<usize>,
        /// Part index for multipart bodies or address lists.
        #[serde(skip_serializing_if = "Option::is_none")]
        part_index: Option<usize>,
        /// Byte offset within the field.
        byte_offset: usize,
        /// Byte length.
        byte_length: usize,
    },
}

/// Table cell position in a Word document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableCell {
    /// Table index in the document (1-indexed).
    pub table_index: usize,
    /// Row number (1-indexed).
    pub row: usize,
    /// Column number (1-indexed).
    pub column: usize,
}

/// A segment of extracted text with position metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextSegment {
    /// The extracted text content (securely zeroed on drop).
    pub content: SensitiveString,
    /// Position in the original document.
    pub position: Position,
}

/// Metadata about the parsed document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    /// Detected or specified file format.
    pub format: FileFormat,
    /// Detected character encoding.
    pub encoding: String,
    /// Original file size in bytes (if known).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<usize>,
    /// Original filename (if provided).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    /// Whether encoding conversion had errors (lossy).
    pub encoding_lossy: bool,
}

/// Warning codes for non-fatal issues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WarningCode {
    /// Encoding conversion was lossy.
    LossyEncoding,
    /// CSV row had inconsistent column count.
    InconsistentColumns,
    /// File extension didn't match detected format.
    FormatMismatch,
    /// Content was truncated (file too large).
    Truncated,
    /// XML parsing failed (partial extraction).
    XmlParseError,
}

/// A non-fatal warning from parsing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseWarning {
    /// Warning code for programmatic handling.
    pub code: WarningCode,
    /// Human-readable message.
    pub message: String,
    /// Location where warning occurred (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
}

/// Complete result from a parsing operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseResult {
    /// Document metadata.
    pub metadata: DocumentMetadata,
    /// Extracted text segments.
    pub segments: Vec<TextSegment>,
    /// Warnings encountered during parsing.
    pub warnings: Vec<ParseWarning>,
    /// Total characters extracted.
    pub total_chars: usize,
    /// Processing time in milliseconds.
    pub duration_ms: u64,
}

/// Options for configuring parsing behavior.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ParseOptions {
    /// Override format auto-detection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<FileFormat>,

    /// Override encoding auto-detection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding: Option<String>,

    /// CSV-specific: delimiter character (default: ',').
    #[serde(skip_serializing_if = "Option::is_none")]
    pub csv_delimiter: Option<u8>,

    /// CSV-specific: treat first row as headers (default: true).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub csv_has_headers: Option<bool>,

    /// Maximum file size to process (default: 100MB).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_size_bytes: Option<usize>,

    /// Enable streaming for large files (default: true for >10MB).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_streaming: Option<bool>,
}
