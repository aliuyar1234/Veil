# Data Model: PDF Parser

**Feature**: 005-pdf-parser
**Date**: 2025-12-15

## Overview

This document defines the data structures for PDF text extraction, integrating with the existing veil-parsers crate.

## Core Entities

### PdfDocument

Represents a parsed PDF file with metadata and page access.

```rust
/// A parsed PDF document.
#[derive(Debug)]
pub struct PdfDocument {
    /// Total number of pages.
    pub page_count: usize,

    /// PDF version (e.g., "1.7", "2.0").
    pub version: String,

    /// Whether the document is encrypted.
    pub is_encrypted: bool,

    /// Document title from metadata (if available).
    pub title: Option<String>,

    /// Document author from metadata (if available).
    pub author: Option<String>,

    /// List of pages (lazy-loaded for memory efficiency).
    pages: Vec<PdfPage>,
}
```

**Methods**:
```rust
impl PdfDocument {
    /// Parse a PDF from bytes.
    pub fn from_bytes(data: &[u8]) -> Result<Self, PdfError>;

    /// Get a specific page (1-indexed).
    pub fn page(&self, page_num: usize) -> Option<&PdfPage>;

    /// Iterate all pages.
    pub fn pages(&self) -> impl Iterator<Item = &PdfPage>;

    /// Extract all text as TextSegments.
    pub fn extract_text(&self) -> Vec<TextSegment>;

    /// Check if document appears to be scanned (image-only).
    pub fn is_scanned(&self) -> bool;
}
```

---

### PdfPage

Represents a single page with text blocks and form fields.

```rust
/// A single page in a PDF document.
#[derive(Debug, Clone)]
pub struct PdfPage {
    /// Page number (1-indexed).
    pub page_num: usize,

    /// Page width in PDF points (1/72 inch).
    pub width: f32,

    /// Page height in PDF points.
    pub height: f32,

    /// Rotation angle (0, 90, 180, 270).
    pub rotation: u16,

    /// Text blocks extracted from this page.
    pub text_blocks: Vec<PdfTextBlock>,

    /// Form fields on this page.
    pub form_fields: Vec<PdfFormField>,

    /// Whether this page appears to be scanned.
    pub is_scanned: bool,
}
```

**Methods**:
```rust
impl PdfPage {
    /// Get text content in reading order.
    pub fn text_content(&self) -> String;

    /// Convert to TextSegments with position metadata.
    pub fn to_segments(&self, byte_offset: &mut usize) -> Vec<TextSegment>;

    /// Check if page has minimal text (likely scanned).
    pub fn has_minimal_text(&self) -> bool;
}
```

---

### PdfTextBlock

A block of text with position and content.

```rust
/// A block of text within a PDF page.
#[derive(Debug, Clone)]
pub struct PdfTextBlock {
    /// Text content.
    pub content: String,

    /// Bounding box: left edge (x coordinate in points).
    pub x: f32,

    /// Bounding box: bottom edge (y coordinate in points).
    pub y: f32,

    /// Bounding box width in points.
    pub width: f32,

    /// Bounding box height in points.
    pub height: f32,

    /// Reading order index (0-based, sorted top-to-bottom, left-to-right).
    pub reading_order: usize,

    /// Font size in points (if available).
    pub font_size: Option<f32>,
}
```

---

### PdfFormField

An interactive form field with its value.

```rust
/// A form field in a PDF document.
#[derive(Debug, Clone)]
pub struct PdfFormField {
    /// Field name (as defined in the form).
    pub name: String,

    /// Field type.
    pub field_type: PdfFieldType,

    /// Current value.
    pub value: Option<String>,

    /// Page number where field appears.
    pub page_num: usize,

    /// Position on page.
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Types of PDF form fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PdfFieldType {
    /// Single-line or multi-line text input.
    Text,
    /// Checkbox (checked/unchecked).
    Checkbox,
    /// Radio button group.
    Radio,
    /// Dropdown list.
    Dropdown,
    /// Combo box (editable dropdown).
    ComboBox,
    /// Push button (no value).
    Button,
    /// Signature field.
    Signature,
    /// Unknown field type.
    Unknown,
}
```

---

### PdfError

Error types for PDF parsing operations.

```rust
/// Errors that can occur during PDF parsing.
#[derive(Debug, thiserror::Error)]
pub enum PdfError {
    /// PDF file is encrypted and requires a password.
    #[error("PDF is encrypted - password required")]
    Encrypted,

    /// PDF file structure is corrupted.
    #[error("PDF file is corrupted: {0}")]
    Corrupted(String),

    /// No text content found (likely scanned).
    #[error("No extractable text found - document may be scanned")]
    NoTextContent,

    /// PDF version not supported.
    #[error("Unsupported PDF version: {0}")]
    UnsupportedVersion(String),

    /// IO error reading file.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Internal parsing error.
    #[error("Parse error: {0}")]
    ParseError(String),
}
```

---

### PdfParseOptions

Configuration options for PDF parsing.

```rust
/// Options for PDF text extraction.
#[derive(Debug, Clone)]
pub struct PdfParseOptions {
    /// Extract form field values.
    pub extract_form_fields: bool,

    /// Include position metadata.
    pub include_positions: bool,

    /// Password for encrypted PDFs.
    pub password: Option<String>,

    /// Maximum pages to process (None = all).
    pub max_pages: Option<usize>,

    /// Skip pages that appear to be scanned.
    pub skip_scanned_pages: bool,
}

impl Default for PdfParseOptions {
    fn default() -> Self {
        Self {
            extract_form_fields: true,
            include_positions: true,
            password: None,
            max_pages: None,
            skip_scanned_pages: false,
        }
    }
}
```

---

## Position Extension

Extend the existing Position enum in veil-parsers:

```rust
/// Position information for a text segment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Position {
    // ... existing variants ...

    /// Position in a PDF document.
    Pdf {
        /// Page number (1-indexed).
        page: usize,
        /// Left edge in PDF points.
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
}
```

---

## Entity Relationships

```
PdfDocument
    │
    ├── metadata (version, title, author)
    │
    └── PdfPage (1..n)
            │
            ├── dimensions (width, height, rotation)
            │
            ├── PdfTextBlock (0..n)
            │       └── content + bounding box
            │
            └── PdfFormField (0..n)
                    └── name + type + value

PdfDocument.extract_text() → Vec<TextSegment>
    │
    └── TextSegment
            ├── content: String
            └── position: Position::Pdf { ... }
```

---

## Processing Flow

```
┌─────────────┐
│ PDF Bytes   │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Parse Header│ → [Error: Corrupted, UnsupportedVersion]
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Check       │ → [Error: Encrypted]
│ Encryption  │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Load Pages  │
└──────┬──────┘
       │
       ▼
┌─────────────────────────────────────┐
│ For each page:                       │
│   1. Extract text objects           │
│   2. Extract form fields            │
│   3. Sort by reading order          │
│   4. Group into text blocks         │
│   5. Detect if scanned              │
└──────┬──────────────────────────────┘
       │
       ▼
┌─────────────┐
│ PdfDocument │
│   .pages[]  │
└──────┬──────┘
       │
       ▼
┌─────────────────────┐
│ Vec<TextSegment>    │
│ (via extract_text)  │
└─────────────────────┘
```

---

## Size Limits

| Entity | Limit | Rationale |
|--------|-------|-----------|
| Page count | 10,000 | Reasonable for processing |
| Page dimensions | 14,400 pts (200 inches) | PDF spec limit |
| Text blocks per page | 10,000 | Memory constraint |
| Form fields per document | 10,000 | Memory constraint |
| Total document size | 100 MB | Per spec FR-008 |

---

## Integration with veil-parsers

The PDF parser integrates with veil-parsers through:

1. **Format Detection**: Add PDF magic bytes check (`%PDF-`)
2. **Parse Function**: `parse_pdf(data: &[u8], options: PdfParseOptions) -> Result<Vec<TextSegment>, PdfError>`
3. **Position Enum**: Extend with `Position::Pdf` variant
4. **Error Handling**: Map `PdfError` to `ParseError`

```rust
// In veil-parsers
pub fn parse_bytes(data: &[u8], options: ParseOptions) -> Result<Vec<TextSegment>, ParseError> {
    match options.format.unwrap_or_else(|| detect_format(data)) {
        Format::Pdf => pdf::parse_pdf(data, options.into()).map_err(Into::into),
        // ... other formats
    }
}
```
