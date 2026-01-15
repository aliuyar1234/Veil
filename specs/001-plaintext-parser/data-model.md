# Data Model: Plaintext Parser

**Feature**: 001-plaintext-parser
**Date**: 2025-12-09

## Overview

This document defines the core data structures for the Veil parsing library. These types form
the foundation for all document parsing and are consumed by the detection engine.

## Core Entities

### FileFormat

Enumeration of supported file formats.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileFormat {
    /// Plain text file (.txt, .log, etc.)
    Text,
    /// Comma-separated values (.csv, .tsv)
    Csv,
    /// JSON data (.json)
    Json,
    /// HTML document (.html, .htm)
    Html,
}
```

**Validation Rules**:
- Auto-detection may override extension-based assumption
- Unknown extensions default to `Text`

---

### Position

Location metadata for a text segment, format-specific.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Position {
    /// Plain text position
    Text {
        /// 1-indexed line number
        line: usize,
        /// 1-indexed column number (character position in line)
        column: usize,
        /// 0-indexed byte offset from file start
        byte_offset: usize,
        /// Length in bytes
        byte_length: usize,
    },

    /// CSV cell position
    Csv {
        /// 1-indexed row number (header row is 1 if present)
        row: usize,
        /// 0-indexed column index
        column: usize,
        /// Column header name if headers are present
        header: Option<String>,
    },

    /// JSON value position
    Json {
        /// JSONPath notation (e.g., "$.users[0].email")
        path: String,
    },

    /// HTML text position
    Html {
        /// Approximate byte offset in original HTML
        byte_offset: usize,
        /// Byte length in original HTML
        byte_length: usize,
    },
}
```

**Invariants**:
- `line` and `row` are 1-indexed (human-readable)
- `column` in Position::Csv is 0-indexed (programmatic)
- `byte_offset` is always 0-indexed
- JSON `path` uses JSONPath syntax starting with `$`

---

### TextSegment

A piece of extracted text with its location.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextSegment {
    /// The extracted text content
    pub content: String,

    /// Position in the original document
    pub position: Position,
}
```

**Validation Rules**:
- `content` may be empty (empty cells, empty lines)
- `content` is always valid UTF-8 (lossy conversion applied if needed)

---

### DocumentMetadata

Information about the parsed document.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    /// Detected or specified file format
    pub format: FileFormat,

    /// Detected character encoding
    pub encoding: String,

    /// Original file size in bytes (if known)
    pub size_bytes: Option<usize>,

    /// Original filename (if provided)
    pub filename: Option<String>,

    /// Whether encoding conversion had errors (lossy)
    pub encoding_lossy: bool,
}
```

---

### ParseWarning

Non-fatal issues encountered during parsing.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseWarning {
    /// Warning code for programmatic handling
    pub code: WarningCode,

    /// Human-readable message
    pub message: String,

    /// Location where warning occurred (if applicable)
    pub position: Option<Position>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum WarningCode {
    /// Encoding conversion was lossy
    LossyEncoding,
    /// CSV row had inconsistent column count
    InconsistentColumns,
    /// File extension didn't match detected format
    FormatMismatch,
    /// Content was truncated (file too large)
    Truncated,
}
```

---

### ParseResult

The complete output of a parsing operation.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseResult {
    /// Document metadata
    pub metadata: DocumentMetadata,

    /// Extracted text segments
    pub segments: Vec<TextSegment>,

    /// Warnings encountered during parsing
    pub warnings: Vec<ParseWarning>,

    /// Total characters extracted
    pub total_chars: usize,

    /// Processing time in milliseconds
    pub duration_ms: u64,
}
```

**Invariants**:
- `segments` is never null (empty Vec for empty documents)
- `total_chars` equals sum of all segment content lengths
- `warnings` may be empty

---

### ParseOptions

Configuration for parsing operations.

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ParseOptions {
    /// Override format auto-detection
    pub format: Option<FileFormat>,

    /// Override encoding auto-detection
    pub encoding: Option<String>,

    /// CSV-specific: delimiter character (default: ',')
    pub csv_delimiter: Option<u8>,

    /// CSV-specific: treat first row as headers (default: true)
    pub csv_has_headers: Option<bool>,

    /// Maximum file size to process (default: 100MB)
    pub max_size_bytes: Option<usize>,

    /// Enable streaming for large files (default: true for >10MB)
    pub enable_streaming: Option<bool>,
}
```

---

## Entity Relationships

```
ParseOptions ──────┐
                   │
                   ▼
              ┌─────────┐
Input Data ───┤  parse  ├───▶ ParseResult
              └─────────┘          │
                                   ├── metadata: DocumentMetadata
                                   ├── segments: Vec<TextSegment>
                                   │                    │
                                   │                    └── position: Position
                                   └── warnings: Vec<ParseWarning>
```

---

## Trait Definition

### Parser

The common interface for all format-specific parsers.

```rust
pub trait Parser {
    /// Parse content from a byte slice
    fn parse_bytes(&self, bytes: &[u8], options: &ParseOptions) -> Result<ParseResult, ParseError>;

    /// Parse content from a reader (streaming)
    fn parse_reader<R: Read>(&self, reader: R, options: &ParseOptions) -> Result<ParseResult, ParseError>;

    /// Get the formats this parser handles
    fn supported_formats(&self) -> &[FileFormat];
}
```

---

## State Transitions

### Parsing Flow

```
┌──────────────┐
│ Input Bytes  │
└──────┬───────┘
       │
       ▼
┌──────────────┐
│ Detect       │ → Format, Encoding
│ Format       │
└──────┬───────┘
       │
       ▼
┌──────────────┐
│ Validate     │ → [Error: FileTooLarge, UnsupportedFormat]
│ Input        │
└──────┬───────┘
       │
       ▼
┌──────────────┐
│ Convert      │ → [Warning: LossyEncoding]
│ Encoding     │
└──────┬───────┘
       │
       ▼
┌──────────────┐
│ Parse by     │ → [Warning: InconsistentColumns]
│ Format       │
└──────┬───────┘
       │
       ▼
┌──────────────┐
│ Build Result │
└──────────────┘
```

---

## Size Limits

| Entity | Limit | Rationale |
|--------|-------|-----------|
| Input file | 100 MB | Memory constraints (spec FR-010) |
| Single segment content | 10 MB | Prevent oversized allocations |
| Segments count | 10,000,000 | Practical limit for large CSV files |
| JSON nesting depth | 100 | Prevent stack overflow (spec SC-003) |
| CSV columns | 10,000 | Reasonable spreadsheet limit |

---

## Serialization

All types derive `Serialize` and `Deserialize` via serde:

```rust
use serde::{Serialize, Deserialize};
```

JSON output example:

```json
{
  "metadata": {
    "format": "csv",
    "encoding": "UTF-8",
    "size_bytes": 1024,
    "filename": "data.csv",
    "encoding_lossy": false
  },
  "segments": [
    {
      "content": "john@example.com",
      "position": {
        "type": "csv",
        "row": 2,
        "column": 1,
        "header": "email"
      }
    }
  ],
  "warnings": [],
  "total_chars": 16,
  "duration_ms": 5
}
```
