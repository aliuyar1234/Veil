# Research: Plaintext Parser

**Feature**: 001-plaintext-parser
**Date**: 2025-12-09

## 1. CSV Parsing Library

**Decision**: Use `csv` crate (v1.3+)

**Rationale**:
- De facto standard for CSV parsing in Rust
- Full RFC 4180 compliance with configurable delimiters
- Streaming/iterator-based API for memory efficiency
- Zero-copy parsing with `ByteRecord` for performance
- Actively maintained by BurntSushi (ripgrep author)

**Alternatives Considered**:
- `rust-csv` (older): Same crate, just older name
- Hand-rolled parser: RFC 4180 edge cases (quoted fields, embedded newlines) are tricky
- `polars`: Overkill for text extraction; optimized for data analysis

**Configuration**:
```toml
[dependencies]
csv = "1.3"
```

**Usage Pattern**:
```rust
use csv::ReaderBuilder;

let mut reader = ReaderBuilder::new()
    .delimiter(b',')
    .has_headers(true)
    .flexible(true)  // Allow variable column counts
    .from_reader(input);

for (row_idx, result) in reader.records().enumerate() {
    let record = result?;
    for (col_idx, field) in record.iter().enumerate() {
        // Extract with position
    }
}
```

## 2. Character Encoding Detection & Conversion

**Decision**: Use `encoding_rs` crate (v0.8+)

**Rationale**:
- Mozilla's encoding library, used in Firefox
- Supports all web-relevant encodings (UTF-8, UTF-16, ISO-8859-1, etc.)
- WHATWG Encoding Standard compliant
- Battle-tested and actively maintained
- Zero-copy decoding where possible

**Alternatives Considered**:
- `chardet`/`chardetng`: Detection only, no conversion
- `encoding`: Deprecated in favor of encoding_rs
- std::str: UTF-8 only

**Configuration**:
```toml
[dependencies]
encoding_rs = "0.8"
```

**Usage Pattern**:
```rust
use encoding_rs::{Encoding, UTF_8};

// Detect encoding from BOM or content analysis
fn detect_encoding(bytes: &[u8]) -> &'static Encoding {
    // Check BOM first
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return UTF_8;
    }
    if bytes.starts_with(&[0xFF, 0xFE]) {
        return encoding_rs::UTF_16LE;
    }
    if bytes.starts_with(&[0xFE, 0xFF]) {
        return encoding_rs::UTF_16BE;
    }
    // Default to UTF-8 with lossy conversion
    UTF_8
}

// Decode to UTF-8
let (decoded, _, had_errors) = encoding.decode(bytes);
if had_errors {
    // Log warning about lossy conversion
}
```

## 3. HTML Parsing Library

**Decision**: Use `scraper` crate (v0.18+) with `html5ever`

**Rationale**:
- Built on `html5ever` (Servo's HTML parser) - full HTML5 spec compliance
- CSS selector API for element targeting
- Handles malformed HTML gracefully (browser-like parsing)
- Text extraction via `.text()` iterator

**Alternatives Considered**:
- `html5ever` directly: Lower level, more verbose
- `select.rs`: Less maintained, smaller community
- `kuchiki`: Good but less intuitive API than scraper
- Regex: Incorrect for HTML parsing (nested tags, entities)

**Configuration**:
```toml
[dependencies]
scraper = "0.18"
```

**Usage Pattern**:
```rust
use scraper::{Html, Selector};

let document = Html::parse_document(html_content);

// Exclude script and style
let body_selector = Selector::parse("body").unwrap();
let exclude_selector = Selector::parse("script, style, noscript").unwrap();

for element in document.select(&body_selector) {
    // Get visible text, handling entities automatically
    let text: String = element.text().collect();
}
```

## 4. JSON Parsing Approach

**Decision**: Use `serde_json` with custom visitor for path tracking

**Rationale**:
- `serde_json` is the standard JSON library in Rust
- Already a dependency via serde
- Streaming parser available for large files
- Custom Deserializer can track JSON paths during traversal

**Alternatives Considered**:
- `simd-json`: Faster but SIMD requirements limit portability
- `json`: Unmaintained, less ergonomic
- Manual parsing: Error-prone for edge cases

**Configuration**:
```toml
[dependencies]
serde_json = "1.0"
```

**Usage Pattern**:
```rust
use serde_json::Value;

fn extract_strings(value: &Value, path: &str, results: &mut Vec<TextSegment>) {
    match value {
        Value::String(s) => {
            results.push(TextSegment {
                content: s.clone(),
                path: path.to_string(),
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
        _ => {} // Skip numbers, bools, nulls
    }
}
```

## 5. Format Auto-Detection

**Decision**: Use magic bytes + content heuristics

**Rationale**:
- File extensions are unreliable (spec requirement FR-011)
- Magic bytes provide definitive identification for some formats
- Content heuristics (JSON starts with `{` or `[`, CSV has delimiters) for others

**Detection Strategy**:
1. Check for BOM (encoding detection)
2. Check first non-whitespace characters:
   - `{` or `[` → JSON
   - `<` → HTML/XML
   - Contains comma/semicolon/tab patterns → CSV
   - Default → Plain text

**Implementation**:
```rust
pub fn detect_format(bytes: &[u8]) -> FileFormat {
    let trimmed = bytes.iter()
        .skip_while(|b| b.is_ascii_whitespace())
        .take(1)
        .next();

    match trimmed {
        Some(b'{') | Some(b'[') => FileFormat::Json,
        Some(b'<') => {
            if looks_like_html(bytes) {
                FileFormat::Html
            } else {
                FileFormat::Text // Could be XML, treat as text
            }
        }
        _ => {
            if looks_like_csv(bytes) {
                FileFormat::Csv
            } else {
                FileFormat::Text
            }
        }
    }
}
```

## 6. Streaming for Large Files

**Decision**: Use `BufReader` with chunked processing for files >10MB

**Rationale**:
- Spec requires handling files up to 100MB (FR-010)
- Memory constraint: <3x file size (SC-006)
- Line-by-line processing for text, record-by-record for CSV

**Implementation Pattern**:
```rust
use std::io::{BufRead, BufReader};

const CHUNK_SIZE: usize = 8 * 1024; // 8KB chunks

fn parse_text_streaming<R: Read>(reader: R) -> Result<ParseResult> {
    let buf_reader = BufReader::with_capacity(CHUNK_SIZE, reader);
    let mut segments = Vec::new();
    let mut offset = 0;

    for (line_num, line_result) in buf_reader.lines().enumerate() {
        let line = line_result?;
        segments.push(TextSegment {
            content: line.clone(),
            position: Position::Line {
                line: line_num + 1,
                offset,
            },
        });
        offset += line.len() + 1; // +1 for newline
    }

    Ok(ParseResult { segments, .. })
}
```

## 7. Error Handling Strategy

**Decision**: Use `thiserror` for library errors, return `Result<T, ParseError>`

**Rationale**:
- Constitution requires `thiserror` for library error types
- Graceful degradation for recoverable errors (log warning, continue)
- Hard errors only for truly unrecoverable situations

**Error Types**:
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid encoding: {0}")]
    Encoding(String),

    #[error("Malformed CSV at row {row}: {message}")]
    CsvError { row: usize, message: String },

    #[error("Invalid JSON: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("File too large: {size} bytes exceeds {max} byte limit")]
    FileTooLarge { size: usize, max: usize },

    #[error("Unsupported format")]
    UnsupportedFormat,
}
```

## 8. Position Tracking Design

**Decision**: Format-specific position enum

**Rationale**:
- Each format has different natural position units
- Unified type allows generic handling while preserving format-specific info
- Spec requires this (FR-009, Key Entities: Position)

**Design**:
```rust
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum Position {
    /// Plain text: line and character offset
    Text {
        line: usize,
        column: usize,
        byte_offset: usize,
    },
    /// CSV: row and column with optional header name
    Csv {
        row: usize,
        column: usize,
        header: Option<String>,
    },
    /// JSON: JSONPath notation
    Json {
        path: String,
    },
    /// HTML: approximate byte offset (after tag stripping, positions shift)
    Html {
        byte_offset: usize,
    },
}
```

## Summary of Decisions

| Component | Choice | Crate Version |
|-----------|--------|---------------|
| CSV parsing | csv | 1.3+ |
| Encoding | encoding_rs | 0.8+ |
| HTML parsing | scraper | 0.18+ |
| JSON parsing | serde_json | 1.0+ |
| Error types | thiserror | 1.0+ |
| Serialization | serde | 1.0+ |

## Open Questions Resolved

1. **Q: How to handle mixed encodings within a file?**
   A: Treat entire file as single encoding; detect from BOM or first bytes

2. **Q: How to report positions when encoding changes byte counts?**
   A: Report byte offsets in original encoding; provide character offsets where meaningful

3. **Q: Should HTML parsing preserve element structure?**
   A: No - only extract visible text. Structure preservation is out of scope for PII detection.
