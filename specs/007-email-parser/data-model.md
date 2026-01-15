# Email Parser Data Model

**Feature**: 007-email-parser
**Date**: 2025-12-15

## Overview

This document defines the data structures for parsing EML and MSG email files in the veil-email crate. The model is designed to represent parsed email content in a way that enables efficient PII detection while maintaining compatibility with the veil-parsers TextSegment interface.

## Design Principles

1. **Constitution Alignment**:
   - All types use `Result<T, E>` for fallible operations
   - No `.unwrap()` on user-provided email data
   - Prefer borrowing over cloning where possible
   - Keep nesting levels ≤ 3

2. **Parser Interface Compatibility**:
   - Must produce `Vec<TextSegment>` for veil-parsers integration
   - Position metadata must identify email-specific locations
   - Support same ParseOptions and ParseResult patterns

3. **PII Detection Optimization**:
   - Separate segments for headers (high PII density)
   - Label quoted content vs. original content
   - Preserve attachment metadata without loading content

## Core Types

### EmailMessage

The top-level parsed email structure.

```rust
/// A parsed email message (EML or MSG format).
#[derive(Debug, Clone)]
pub struct EmailMessage {
    /// Email headers (From, To, Subject, etc.)
    pub headers: Vec<EmailHeader>,

    /// Email body parts (text, HTML, etc.)
    pub body_parts: Vec<EmailBodyPart>,

    /// Attachment metadata (without content)
    pub attachments: Vec<EmailAttachment>,

    /// Original email format (EML or MSG)
    pub format: EmailFormat,

    /// Total size of original email in bytes
    pub size_bytes: usize,
}
```

**Rationale**:
- `headers` as Vec allows multiple headers with same name (CC, Received, etc.)
- `body_parts` supports multipart messages (text + HTML alternatives)
- `attachments` stores metadata only (content not loaded into memory)
- `Clone` bound enables caching if needed, but parsing should avoid clones

### EmailFormat

Distinguishes between EML and MSG sources.

```rust
/// Email file format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EmailFormat {
    /// RFC 5322 email format (.eml, .msg as MIME)
    Eml,
    /// Microsoft Outlook MSG format (.msg)
    Msg,
}
```

**Rationale**: Simple enum, Copy for efficiency. Could extend to MBOX, PST in future.

### EmailHeader

Represents a single email header field.

```rust
/// A parsed email header field.
#[derive(Debug, Clone)]
pub struct EmailHeader {
    /// Header name (e.g., "From", "To", "Subject")
    pub name: String,

    /// Parsed header value
    pub value: EmailHeaderValue,

    /// Raw header value (before parsing, for debugging)
    pub raw_value: String,
}

/// Typed header values.
#[derive(Debug, Clone)]
pub enum EmailHeaderValue {
    /// Single email address (From, Sender, Reply-To)
    Address(EmailAddress),

    /// Multiple email addresses (To, CC, BCC)
    AddressList(Vec<EmailAddress>),

    /// Plain text value (Subject, Message-ID, etc.)
    Text(String),

    /// Date/time value (Date header)
    DateTime(String), // ISO 8601 formatted

    /// Unparseable or unknown header type
    Unstructured(String),
}
```

**Rationale**:
- Typed enum enables specialized PII detection per header type
- `raw_value` preserved for edge cases where parsing fails
- `AddressList` separate from `Address` for type safety
- `DateTime` as String avoids chrono dependency for P1 (parsing can be added later)

### EmailAddress

Represents a parsed email address with optional display name.

```rust
/// A parsed email address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmailAddress {
    /// Display name (e.g., "John Doe")
    pub display_name: Option<String>,

    /// Email address (e.g., "john@example.com")
    pub address: String,
}
```

**Example parsing**:
- `"john@example.com"` → `EmailAddress { display_name: None, address: "john@example.com" }`
- `"John Doe <john@example.com>"` → `EmailAddress { display_name: Some("John Doe"), address: "john@example.com" }`

**Rationale**:
- Both display_name and address are PII targets
- Simple structure, no complex address parsing (groups, comments)
- PartialEq for deduplication if needed

### EmailBodyPart

Represents a part of the email body (text or HTML).

```rust
/// A part of an email body.
#[derive(Debug, Clone)]
pub struct EmailBodyPart {
    /// Content type (e.g., "text/plain", "text/html")
    pub content_type: String,

    /// Charset encoding (e.g., "utf-8", "iso-8859-1")
    pub charset: String,

    /// Decoded text content
    pub content: String,

    /// Whether this part is quoted/replied content
    pub is_quoted: bool,

    /// Transfer encoding used (base64, quoted-printable, etc.)
    pub transfer_encoding: Option<String>,
}
```

**Rationale**:
- `content` is always String (decoded to UTF-8)
- `is_quoted` enables filtering old PII in threads
- `transfer_encoding` preserved for debugging, not needed for PII detection

### EmailAttachment

Metadata for an email attachment (content NOT included).

```rust
/// Metadata for an email attachment.
#[derive(Debug, Clone)]
pub struct EmailAttachment {
    /// Filename (may be None for unnamed attachments)
    pub filename: Option<String>,

    /// MIME content type (e.g., "application/pdf")
    pub content_type: String,

    /// Size in bytes
    pub size_bytes: usize,

    /// Content-ID header value (for inline images)
    pub content_id: Option<String>,

    /// Whether this is an inline attachment (vs. regular)
    pub inline: bool,
}
```

**Rationale**:
- No `content: Vec<u8>` field - attachments are listed only, not loaded
- `filename` is PII-relevant (may contain names, dates, etc.)
- `content_id` needed to identify inline images vs. regular attachments
- Future: attachment content can be extracted and parsed separately

## Integration with veil-parsers

### Position Enum Extension

Add new variant to `veil_parsers::Position`:

```rust
/// Email-specific position.
Email {
    /// Header name (e.g., "From", "Subject") or "body"
    field: String,

    /// For multi-value headers, index in the list (e.g., 2nd CC recipient)
    field_index: Option<usize>,

    /// For body parts, index in body_parts list
    part_index: Option<usize>,

    /// Byte offset within the field/part content
    byte_offset: usize,

    /// Length in bytes
    byte_length: usize,
},
```

**Example positions**:
- From header: `Email { field: "From", field_index: None, part_index: None, byte_offset: 0, byte_length: 24 }`
- 2nd To recipient: `Email { field: "To", field_index: Some(1), part_index: None, byte_offset: 0, byte_length: 20 }`
- Body text: `Email { field: "body", field_index: None, part_index: Some(0), byte_offset: 15, byte_length: 50 }`

### TextSegment Generation

Convert EmailMessage to `Vec<TextSegment>`:

```rust
impl EmailMessage {
    /// Convert to TextSegments for PII detection.
    pub fn to_text_segments(&self) -> Vec<TextSegment> {
        let mut segments = Vec::new();

        // 1. Extract header segments
        for header in &self.headers {
            segments.extend(header.to_text_segments());
        }

        // 2. Extract body segments
        for (idx, part) in self.body_parts.iter().enumerate() {
            segments.extend(part.to_text_segments(idx));
        }

        // 3. Extract attachment filename segments
        for attachment in &self.attachments {
            if let Some(filename) = &attachment.filename {
                segments.push(TextSegment {
                    content: filename.clone(),
                    position: Position::Email {
                        field: "attachment".to_string(),
                        field_index: None,
                        part_index: None,
                        byte_offset: 0,
                        byte_length: filename.len(),
                    },
                });
            }
        }

        segments
    }
}
```

**Rationale**:
- Each header field becomes one or more TextSegments
- Body parts are split into segments (potentially by paragraph or sentence)
- Attachment filenames included (may contain PII like "John_Doe_Resume.pdf")

## Error Types

```rust
/// Errors that can occur during email parsing.
#[derive(Debug, thiserror::Error)]
pub enum EmailParseError {
    /// Invalid email format (not EML or MSG)
    #[error("Invalid email format: {0}")]
    InvalidFormat(String),

    /// MIME parsing error
    #[error("MIME parsing failed: {0}")]
    MimeError(String),

    /// MSG OLE structure error
    #[error("MSG parsing failed: {0}")]
    MsgError(String),

    /// Character encoding error
    #[error("Encoding error: {0}")]
    EncodingError(String),

    /// I/O error reading email file
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Email file too large
    #[error("Email file too large: {size} bytes (max: {max})")]
    FileTooLarge { size: usize, max: usize },
}
```

**Rationale**:
- Use `thiserror` for library errors (per constitution)
- Distinguish between format-specific errors (MIME vs. MSG)
- `#[from]` for automatic conversion from std::io::Error
- Compatible with `veil_parsers::ParseError` structure

## Parser Options

Extend `ParseOptions` for email-specific settings:

```rust
/// Email-specific parsing options.
#[derive(Debug, Clone, Default)]
pub struct EmailParseOptions {
    /// Convert HTML body to plain text (default: true)
    pub convert_html: bool,

    /// Detect quoted/replied content (default: true)
    pub detect_quotes: bool,

    /// Extract attachment metadata (default: true)
    pub extract_attachments: bool,

    /// Maximum attachment size to list (default: None = unlimited)
    pub max_attachment_size: Option<usize>,

    /// Prefer plain text over HTML when both available (default: true)
    pub prefer_plain_text: bool,
}
```

**Integration**: These options extend `veil_parsers::ParseOptions` for FileFormat::Eml/Msg.

## Memory Considerations

### Size Estimates

For a typical email (50KB):
- Headers: ~2KB → 2KB in memory
- Body: ~45KB → 45KB in memory (single allocation)
- Attachments metadata: 3 attachments × 200 bytes = 600 bytes
- **Total**: ~48KB (< 1x file size)

For large email (5MB with 10MB attachment):
- Headers: ~5KB
- Body: ~4.9MB
- Attachments metadata: ~1KB (metadata only, no content!)
- **Total**: ~5MB (attachment content not loaded)

**Constitution compliance**: Efficient memory usage, no unnecessary clones.

### Streaming Considerations

For P1, load entire email into memory (acceptable for typical 10KB-5MB emails).

For future optimization:
- Stream body text in chunks for very large emails
- Attachment content accessed via separate API call
- MIME part iteration instead of Vec allocation

## Quote Detection Algorithm

Pseudocode for detecting quoted content:

```rust
fn detect_quotes(text: &str) -> Vec<TextSegment> {
    let lines = text.lines();
    let mut segments = Vec::new();
    let mut current_segment = String::new();
    let mut is_quoted = false;

    for line in lines {
        let line_is_quoted = line.trim_start().starts_with('>')
            || REPLY_PATTERN.is_match(line);

        if line_is_quoted != is_quoted {
            // Quote state changed, flush current segment
            if !current_segment.is_empty() {
                segments.push(create_segment(current_segment, is_quoted));
                current_segment = String::new();
            }
            is_quoted = line_is_quoted;
        }

        current_segment.push_str(line);
        current_segment.push('\n');
    }

    if !current_segment.is_empty() {
        segments.push(create_segment(current_segment, is_quoted));
    }

    segments
}
```

**Patterns to detect**:
- Lines starting with `>`, `>>`, etc.
- `On <date>, <person> wrote:`
- `-----Original Message-----`
- `From: ... Sent: ... To: ...` (Outlook-style quote headers)

## Example Data Flow

Input: `email.eml` file

1. **Parse with mailparse**: `mailparse::parse_mail(bytes)` → `ParsedMail`
2. **Extract headers**: Iterate headers, parse addresses → `Vec<EmailHeader>`
3. **Extract body**: Get text/html parts, decode → `Vec<EmailBodyPart>`
4. **Detect quotes**: Apply quote detection → Set `is_quoted` flags
5. **Extract attachments**: List parts with Content-Disposition → `Vec<EmailAttachment>`
6. **Build EmailMessage**: Combine all parts → `EmailMessage`
7. **Convert to segments**: `email.to_text_segments()` → `Vec<TextSegment>`
8. **Return ParseResult**: Wrap in `ParseResult` with metadata → Return to caller

## Testing Strategy

### Unit Tests

- Parse single header: `"From: john@example.com"` → `EmailAddress`
- Parse address list: `"To: a@ex.com, b@ex.com"` → `Vec<EmailAddress>` (length 2)
- Parse display name: `"John Doe <john@ex.com>"` → Extract both parts
- Detect quoted line: `"> previous message"` → `is_quoted = true`
- Decode base64 body: Encoded text → Decoded UTF-8

### Integration Tests

- Real EML file from Gmail export
- Real EML file from Outlook export
- Real MSG file from Outlook Save As
- Email with 10 attachments
- Email with HTML body only
- Email with both text and HTML (multipart/alternative)
- Email thread with quotes
- Non-ASCII characters (Japanese, emoji)

### Edge Case Tests

- Empty email (headers only, no body)
- Malformed headers (missing From)
- Invalid base64 encoding
- Enormous attachment (>100MB) - should list metadata only
- Encrypted email (S/MIME) - should extract headers, mark body as encrypted

## Open Design Questions

1. **DateTime Parsing**: Use String or parse to structured time?
   - **Decision**: String for P1 (ISO 8601 format). Add chrono for P2 if needed.

2. **Attachment Content Access**: Separate API or lazy loading?
   - **Decision**: Separate API. `EmailAttachment::content()` method for future.

3. **HTML Conversion Detail**: Preserve links? Images alt text?
   - **Decision**: Use html2text defaults. Links as `[text](url)`, images as `[image: alt]`.

4. **Quote Depth Tracking**: Store nesting level (>, >>, >>>)?
   - **Decision**: Boolean `is_quoted` for P1. Add `quote_depth: usize` in P3 if needed.

5. **Header Deduplication**: Multiple Received headers - keep all?
   - **Decision**: Keep all. PII detection should scan all headers.

## Conclusion

This data model balances:
- **Type safety**: Enums for header values, distinct types for addresses
- **Efficiency**: No attachment content loading, minimal clones
- **Constitution compliance**: Result types, no unwrap, ≤3 nesting levels
- **PII detection optimization**: Separate segments for headers/body/quotes
- **Parser integration**: Compatible with TextSegment/Position interface

The model supports all P1 and P2 user stories while leaving room for P3 enhancements (quote depth, calendar parsing, embedded emails).
