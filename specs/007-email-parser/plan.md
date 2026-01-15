# Implementation Plan: Email Parser

**Branch**: `007-email-parser` | **Date**: 2025-12-15 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/007-email-parser/spec.md`

## Summary

Implement email parsing capability for EML (RFC 5322) and MSG (Microsoft Outlook) formats to enable PII detection in email files. The parser extracts headers (From, To, CC, BCC, Subject, Date), body text (plain and HTML), and attachment metadata, outputting TextSegments compatible with the veil-parsers interface. This enables compliance teams to scan email archives for personal information.

**Technical Approach**:
- Use `mailparse` crate for EML/MIME parsing (mature, minimal dependencies)
- Use `msg-parser` crate for MSG format (only viable option for OLE-based Outlook files)
- Use `html2text` crate for HTML-to-text conversion
- Implement custom quote detection using regex patterns
- Wrap both parsers in unified API that outputs veil-parsers TextSegments
- New crate: `veil-email` as workspace member

## Technical Context

**Language/Version**: Rust stable (1.75+)
**Primary Dependencies**:
- mailparse (0.15.0) - EML/MIME parsing
- msg-parser (0.5.0) - MSG format parsing
- html2text (0.12.0) - HTML conversion
- encoding_rs (0.8.0) - Character encoding (already approved)
- thiserror (1.0) - Error types (already approved)
- serde (1.0) - Serialization (already approved)

**Storage**: N/A (stateless parsing)
**Testing**: cargo test (unit + integration tests with real email fixtures)
**Target Platform**: Cross-platform library (Linux, macOS, Windows, WASM-compatible parsers)
**Project Type**: Library crate (workspace member)
**Performance Goals**:
- Parse typical email (10KB-5MB) in <100ms
- List 10+ attachments in <1 second (metadata only)
- Process 100 emails/second in batch mode

**Constraints**:
- Memory usage: ~2-3x file size during parsing (acceptable for email sizes)
- No attachment content loading (metadata only to avoid memory bloat)
- Character encoding: 99% accuracy for international emails

**Scale/Scope**:
- Support 2 formats (EML, MSG)
- Extract 7+ standard headers
- Handle multipart MIME (text + HTML alternatives)
- List unlimited attachments (metadata only)
- Detect quoted content (best-effort)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### I. Security First
- ✅ No `unsafe` blocks required (mailparse, msg-parser are pure safe Rust)
- ✅ No `.unwrap()` on email data (use Result propagation with `?`)
- ✅ OWASP compliance: Parser doesn't execute scripts, load external resources
- ✅ Crypto libraries: N/A for P1 (encrypted emails marked as unprocessable)

### II. Stability & Error Handling
- ✅ All parsing returns `Result<EmailMessage, EmailParseError>`
- ✅ Malformed emails handled gracefully (partial extraction + warnings)
- ✅ Use `thiserror` for EmailParseError type
- ✅ No panics on invalid input (tested with fuzzing in Phase 3)

### III. Performance
- ✅ Zero-copy where possible (mailparse uses byte slices)
- ⚠️ `clone()` used for String conversions from decoded text (unavoidable)
- ✅ Attachment content NOT loaded (metadata only)
- ✅ Target: 10KB email in <10ms, 5MB email in <100ms

**Justification for clones**: Email headers/body must be decoded to UTF-8 Strings for PII detection. No zero-copy alternative exists for encoding conversion.

### IV. Simplicity & Minimalism
- ✅ Single public function: `parse_email(bytes, options) -> Result<EmailMessage>`
- ✅ No abstraction until needed (direct use of mailparse/msg-parser)
- ✅ Maximum nesting: 2 levels (EmailMessage → headers/body/attachments)
- ✅ Spec is scope: Only P1/P2 user stories implemented

### V. Test-First Development
- ✅ Integration tests with real Gmail/Outlook exports
- ✅ Unit tests for each header type parsing
- ✅ Edge case tests: empty email, malformed headers, huge attachments
- ✅ Contract tests: Verify TextSegment output format

### VI. Dependency Discipline
- ✅ mailparse: Active, 500K downloads/month, MIT/Apache-2.0
- ✅ msg-parser: Active, 1K downloads/month (only option), MIT/Apache-2.0
- ✅ html2text: Active, 200K downloads/month, MIT
- ✅ Total new dependencies: ~7 crates (justified in research.md)
- ✅ All dependencies audited for security issues

### VII. Rust Standards
- ✅ `cargo clippy -- -D warnings` enforced
- ✅ `cargo fmt` applied
- ✅ Documentation comments on all public types
- ✅ `#[must_use]` on parse functions

**PASS**: All constitution checks pass. Clone usage is justified and unavoidable.

## Project Structure

### Documentation (this feature)

```text
specs/007-email-parser/
├── plan.md              # This file
├── research.md          # Crate analysis (mailparse vs. mail-parser vs. msg-parser)
├── data-model.md        # EmailMessage, EmailHeader, EmailAddress types
├── quickstart.md        # Usage examples and API guide
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created yet)
```

### Source Code (repository root)

```text
crates/
├── veil-parsers/        # Existing: text, csv, json, html, pdf parsers
│   ├── src/
│   │   ├── lib.rs       # MODIFY: Add FileFormat::Eml, FileFormat::Msg
│   │   ├── types.rs     # MODIFY: Add Position::Email variant
│   │   └── detect.rs    # MODIFY: Detect .eml/.msg extensions
│   └── Cargo.toml       # MODIFY: Add veil-email dependency
│
└── veil-email/          # NEW CRATE
    ├── src/
    │   ├── lib.rs       # Public API: parse_email(), EmailParseOptions
    │   ├── types.rs     # EmailMessage, EmailHeader, EmailAddress, etc.
    │   ├── error.rs     # EmailParseError
    │   ├── eml.rs       # EML parser (wraps mailparse)
    │   ├── msg.rs       # MSG parser (wraps msg-parser)
    │   ├── html.rs      # HTML-to-text conversion (wraps html2text)
    │   ├── quotes.rs    # Quote detection logic
    │   └── convert.rs   # Convert EmailMessage → Vec<TextSegment>
    │
    ├── tests/
    │   ├── eml_parser_test.rs       # Unit tests for EML parsing
    │   ├── msg_parser_test.rs       # Unit tests for MSG parsing
    │   ├── integration_test.rs      # Real email files
    │   └── fixtures/
    │       ├── gmail_export.eml     # Real Gmail export
    │       ├── outlook_export.eml   # Real Outlook export
    │       ├── outlook.msg          # Real MSG file
    │       ├── with_attachments.eml # Email with 10 attachments
    │       └── email_thread.eml     # Email with quoted replies
    │
    └── Cargo.toml       # Dependencies: mailparse, msg-parser, html2text, etc.

Cargo.toml               # MODIFY: Add veil-email to workspace members
```

**Structure Decision**: New workspace crate `veil-email` keeps email-specific logic isolated from veil-parsers. This follows the pattern established by `veil-parsers` (domain-specific parsing) and allows independent versioning. The veil-parsers crate acts as the integration layer, routing .eml/.msg files to veil-email.

## Implementation Phases

### Phase 0: Research (COMPLETED)

**Outputs**:
- ✅ `research.md`: Analysis of mailparse, mail-parser, msg-parser crates
- ✅ Recommendation: Use mailparse (EML) + msg-parser (MSG) + html2text

**Key Findings**:
- mailparse is most mature EML parser (500K downloads/month)
- msg-parser is only viable MSG parser in Rust ecosystem
- html2text provides better HTML conversion than regex stripping
- Quote detection requires custom implementation (not in any crate)

### Phase 1: Design (COMPLETED)

**Outputs**:
- ✅ `data-model.md`: EmailMessage, EmailHeader, EmailAddress, EmailBodyPart, EmailAttachment types
- ✅ `quickstart.md`: Usage examples and API guide
- ✅ Position::Email variant design for veil-parsers integration

**Key Decisions**:
- EmailHeaderValue enum for typed header access
- Attachment metadata only (no content loading)
- Quote detection via regex patterns
- TextSegment conversion method on EmailMessage

### Phase 2: Tasks (NOT STARTED)

**Command**: `/speckit.tasks` (creates tasks.md with TDD task breakdown)

**Expected outputs**:
- tasks.md with Red-Green-Refactor cycles
- Test fixtures identified
- Implementation order: EML → MSG → quote detection

### Phase 3: Implementation (NOT STARTED)

**Order**:
1. Create veil-email crate skeleton
2. Implement EML parser (eml.rs wrapping mailparse)
3. Implement MSG parser (msg.rs wrapping msg-parser)
4. Implement HTML conversion (html.rs wrapping html2text)
5. Implement quote detection (quotes.rs with regex)
6. Implement TextSegment conversion (convert.rs)
7. Integrate with veil-parsers (add FileFormat variants, update detect.rs)
8. Add integration tests with real email fixtures

**TDD Workflow** (per constitution):
1. Write failing test (e.g., `test_parse_from_header`)
2. Implement minimal code to pass (e.g., extract From header)
3. Refactor (e.g., extract helper function)
4. Repeat for next feature

### Phase 4: Testing (Parallel with Phase 3)

**Test Categories**:

1. **Unit Tests** (tests/ in veil-email):
   - Parse individual headers: From, To, CC, Subject, Date
   - Parse email addresses with/without display names
   - Decode base64/quoted-printable body
   - Detect quoted lines
   - Convert HTML to text

2. **Integration Tests** (tests/integration_test.rs):
   - Real Gmail export (.eml)
   - Real Outlook export (.eml)
   - Real Outlook MSG file (.msg)
   - Email with 10 attachments
   - Email thread with quotes
   - Email with HTML body only
   - Email with both text and HTML (multipart/alternative)

3. **Edge Case Tests**:
   - Empty email (headers only)
   - Missing required headers (no From)
   - Malformed base64 encoding
   - Non-ASCII characters (Japanese, emoji)
   - Huge attachment (>100MB) - should list metadata only
   - Encrypted email (S/MIME) - should extract headers only

4. **Contract Tests** (verify veil-parsers interface):
   - parse_email() returns Vec<TextSegment>
   - Position::Email variant is valid
   - ParseResult metadata is correct
   - Warnings are emitted for malformed emails

**Fixtures** (tests/fixtures/):
- gmail_export.eml (real Gmail export)
- outlook_export.eml (real Outlook export)
- outlook.msg (real MSG file from Outlook "Save As")
- with_attachments.eml (10 PDF/Excel/image attachments)
- email_thread.eml (conversation with > quotes)
- html_only.eml (no text/plain part)
- multipart.eml (text + HTML alternatives)
- japanese.eml (non-ASCII characters)
- encrypted.eml (S/MIME encrypted body)

### Phase 5: Documentation (Parallel with Phase 3)

**Artifacts**:
1. Rustdoc comments on all public items
2. Update CLAUDE.md with veil-email crate info
3. Add email parser section to main README (if applicable)
4. CLI examples in quickstart.md (once veil-cli integrates)

### Phase 6: Integration (After Phase 3)

**Changes to veil-parsers**:

1. **types.rs**: Add Position::Email variant
   ```rust
   Email {
       field: String,
       field_index: Option<usize>,
       part_index: Option<usize>,
       byte_offset: usize,
       byte_length: usize,
   }
   ```

2. **lib.rs**: Add FileFormat::Eml and FileFormat::Msg
   ```rust
   pub enum FileFormat {
       // ... existing variants
       Eml,
       Msg,
   }
   ```

3. **detect.rs**: Add .eml and .msg extension detection
   ```rust
   match extension {
       "eml" => FileFormat::Eml,
       "msg" => FileFormat::Msg,
       // ... existing cases
   }
   ```

4. **lib.rs**: Route to veil-email parser
   ```rust
   match format {
       FileFormat::Eml | FileFormat::Msg => {
           let email = veil_email::parse_email(bytes, &email_options)?;
           email.to_parse_result()
       }
       // ... existing cases
   }
   ```

5. **Cargo.toml**: Add veil-email dependency
   ```toml
   [dependencies]
   veil-email = { path = "../veil-email", version = "0.1.0" }
   ```

## API Design

### Public API (veil-email crate)

```rust
// Main parsing function
pub fn parse_email(
    bytes: &[u8],
    options: &EmailParseOptions,
) -> Result<EmailMessage, EmailParseError>;

// Options
pub struct EmailParseOptions {
    pub convert_html: bool,         // default: true
    pub detect_quotes: bool,         // default: true
    pub extract_attachments: bool,   // default: true
    pub max_attachment_size: Option<usize>, // default: None
    pub prefer_plain_text: bool,     // default: true
}

// Main types
pub struct EmailMessage { ... }      // See data-model.md
pub struct EmailHeader { ... }
pub struct EmailAddress { ... }
pub struct EmailBodyPart { ... }
pub struct EmailAttachment { ... }

// Error type
pub enum EmailParseError { ... }    // See data-model.md
```

### Integration API (veil-parsers)

```rust
// Existing API works unchanged
let result = parse_file("email.eml", &ParseOptions::default())?;
for segment in result.segments {
    // segment.position is Position::Email for email files
}
```

## Implementation Details

### EML Parsing (eml.rs)

**Approach**: Wrap mailparse crate

```rust
use mailparse::{parse_mail, MailHeaderMap};

pub fn parse_eml(bytes: &[u8], options: &EmailParseOptions) -> Result<EmailMessage, EmailParseError> {
    let parsed = parse_mail(bytes)
        .map_err(|e| EmailParseError::MimeError(e.to_string()))?;

    let headers = extract_headers(&parsed)?;
    let body_parts = extract_body_parts(&parsed, options)?;
    let attachments = extract_attachments(&parsed, options)?;

    Ok(EmailMessage {
        headers,
        body_parts,
        attachments,
        format: EmailFormat::Eml,
        size_bytes: bytes.len(),
    })
}

fn extract_headers(parsed: &ParsedMail) -> Result<Vec<EmailHeader>, EmailParseError> {
    // Iterate parsed.headers, parse each to EmailHeader
    // Use mailparse's address parsing for From/To/CC
}

fn extract_body_parts(parsed: &ParsedMail, options: &EmailParseOptions) -> Result<Vec<EmailBodyPart>, EmailParseError> {
    // Get text/plain and text/html parts
    // Decode transfer encoding (base64, quoted-printable)
    // Convert HTML to text if needed
    // Apply quote detection if enabled
}

fn extract_attachments(parsed: &ParsedMail, options: &EmailParseOptions) -> Result<Vec<EmailAttachment>, EmailParseError> {
    // Iterate subparts with Content-Disposition: attachment
    // Extract filename, content-type, size
    // Do NOT load content into memory
}
```

### MSG Parsing (msg.rs)

**Approach**: Wrap msg-parser crate

```rust
use msg_parser::MsgParser;

pub fn parse_msg(bytes: &[u8], options: &EmailParseOptions) -> Result<EmailMessage, EmailParseError> {
    let parser = MsgParser::new(bytes)
        .map_err(|e| EmailParseError::MsgError(e.to_string()))?;

    let headers = extract_msg_headers(&parser)?;
    let body_parts = extract_msg_body(&parser, options)?;
    let attachments = extract_msg_attachments(&parser, options)?;

    Ok(EmailMessage {
        headers,
        body_parts,
        attachments,
        format: EmailFormat::Msg,
        size_bytes: bytes.len(),
    })
}

// Similar extraction functions as EML
```

### HTML Conversion (html.rs)

**Approach**: Wrap html2text crate

```rust
use html2text::from_read;

pub fn convert_html_to_text(html: &str) -> String {
    from_read(html.as_bytes(), 80) // 80 char line width
}
```

**Features**:
- Converts `<p>`, `<div>` to text paragraphs
- Preserves links as `[text](url)`
- Converts `<img>` to `[image: alt text]`
- Strips scripts, styles, comments

### Quote Detection (quotes.rs)

**Approach**: Regex pattern matching + line-by-line analysis

```rust
use regex::Regex;

lazy_static! {
    static ref QUOTE_LINE: Regex = Regex::new(r"^\s*>+\s").unwrap();
    static ref REPLY_HEADER: Regex = Regex::new(r"^On .+, .+ wrote:$").unwrap();
    static ref ORIGINAL_MESSAGE: Regex = Regex::new(r"^-+\s*Original Message\s*-+$").unwrap();
}

pub fn detect_quotes(text: &str) -> Vec<(String, bool)> {
    // Returns vec of (text, is_quoted) tuples
    let mut result = Vec::new();
    let mut current_text = String::new();
    let mut is_quoted = false;

    for line in text.lines() {
        let line_is_quoted = QUOTE_LINE.is_match(line)
            || REPLY_HEADER.is_match(line)
            || ORIGINAL_MESSAGE.is_match(line);

        if line_is_quoted != is_quoted {
            if !current_text.is_empty() {
                result.push((current_text, is_quoted));
                current_text = String::new();
            }
            is_quoted = line_is_quoted;
        }

        current_text.push_str(line);
        current_text.push('\n');
    }

    if !current_text.is_empty() {
        result.push((current_text, is_quoted));
    }

    result
}
```

### TextSegment Conversion (convert.rs)

**Approach**: Implement conversion methods

```rust
impl EmailMessage {
    pub fn to_text_segments(&self) -> Vec<TextSegment> {
        let mut segments = Vec::new();

        // Headers
        for header in &self.headers {
            segments.extend(header.to_text_segments());
        }

        // Body parts
        for (idx, part) in self.body_parts.iter().enumerate() {
            segments.extend(part.to_text_segments(idx));
        }

        // Attachment filenames
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

    pub fn to_parse_result(self) -> ParseResult {
        let start = std::time::Instant::now();
        let segments = self.to_text_segments();
        let total_chars: usize = segments.iter().map(|s| s.content.len()).sum();
        let duration_ms = start.elapsed().as_millis() as u64;

        ParseResult {
            metadata: DocumentMetadata {
                format: match self.format {
                    EmailFormat::Eml => FileFormat::Eml,
                    EmailFormat::Msg => FileFormat::Msg,
                },
                encoding: "utf-8".to_string(), // Always UTF-8 after decoding
                size_bytes: Some(self.size_bytes),
                filename: None,
                encoding_lossy: false, // TODO: track from encoding_rs
            },
            segments,
            warnings: Vec::new(), // TODO: collect warnings during parsing
            total_chars,
            duration_ms,
        }
    }
}
```

## Testing Strategy

### Unit Test Examples

**Test: Parse From header with display name**
```rust
#[test]
fn test_parse_from_with_display_name() {
    let email_bytes = b"From: John Doe <john@example.com>\r\nSubject: Test\r\n\r\nBody";
    let email = parse_email(email_bytes, &EmailParseOptions::default()).unwrap();

    let from = email.headers.iter().find(|h| h.name == "From").unwrap();
    match &from.value {
        EmailHeaderValue::Address(addr) => {
            assert_eq!(addr.display_name, Some("John Doe".to_string()));
            assert_eq!(addr.address, "john@example.com");
        }
        _ => panic!("Expected Address"),
    }
}
```

**Test: Detect quoted line**
```rust
#[test]
fn test_detect_quoted_line() {
    let text = "New message\n> Previous message\n>> Older message";
    let segments = detect_quotes(text);

    assert_eq!(segments.len(), 2);
    assert!(!segments[0].1); // "New message" is not quoted
    assert!(segments[1].1);  // "> Previous..." is quoted
}
```

### Integration Test Example

**Test: Parse real Gmail export**
```rust
#[test]
fn test_parse_gmail_export() {
    let bytes = std::fs::read("tests/fixtures/gmail_export.eml").unwrap();
    let email = parse_email(&bytes, &EmailParseOptions::default()).unwrap();

    // Verify structure
    assert!(email.headers.iter().any(|h| h.name == "From"));
    assert!(email.headers.iter().any(|h| h.name == "To"));
    assert!(email.headers.iter().any(|h| h.name == "Subject"));
    assert!(!email.body_parts.is_empty());

    // Convert to segments
    let segments = email.to_text_segments();
    assert!(!segments.is_empty());
}
```

## Error Handling Strategy

### Error Categories

1. **Invalid Format**: File is not EML or MSG
   - Detection: Magic bytes, file structure
   - Handling: Return `EmailParseError::InvalidFormat`

2. **MIME Parsing Errors**: Malformed MIME structure
   - Detection: mailparse returns Err
   - Handling: Return `EmailParseError::MimeError`, provide context

3. **MSG Parsing Errors**: Invalid OLE structure
   - Detection: msg-parser returns Err
   - Handling: Return `EmailParseError::MsgError`, provide context

4. **Encoding Errors**: Unknown or invalid character encoding
   - Detection: encoding_rs conversion fails
   - Handling: Use lossy conversion, emit warning in ParseResult

5. **I/O Errors**: File read failures
   - Detection: std::io::Error
   - Handling: Auto-convert with `#[from]` in thiserror

### Graceful Degradation

When parsing fails partially:
- Extract headers that are valid
- Skip malformed body parts
- List attachments that are accessible
- Emit warnings in ParseResult.warnings
- Never panic on malformed input

Example:
```rust
fn extract_headers(parsed: &ParsedMail) -> Vec<EmailHeader> {
    parsed.headers.iter().filter_map(|h| {
        match parse_header(h) {
            Ok(header) => Some(header),
            Err(e) => {
                eprintln!("Warning: Failed to parse header {}: {}", h.get_key(), e);
                None
            }
        }
    }).collect()
}
```

## Performance Considerations

### Optimization Strategies

1. **Lazy Decoding**: Only decode body parts when requested (for P2)
2. **Streaming**: For emails >10MB, stream body instead of loading fully (for P3)
3. **Parallel Attachment Processing**: Use rayon for batch attachment listing (for P3)
4. **Regex Caching**: Use lazy_static for quote detection patterns
5. **Zero-Copy Headers**: Use &str from mailparse when possible, only clone for storage

### Memory Profile

Typical email (50KB):
- Input bytes: 50KB
- Parsed structures: ~10KB (headers, metadata)
- Decoded body: ~40KB (UTF-8 string)
- **Peak memory**: ~100KB (2x input)

Large email (5MB):
- Input bytes: 5MB
- Parsed structures: ~50KB
- Decoded body: ~4.9MB
- Attachments: ~1KB metadata (content NOT loaded)
- **Peak memory**: ~10MB (2x input for decoding buffer)

### Performance Targets (from spec SC-004)

- Email with 10 attachments: <1 second for metadata listing
- Typical email (10KB-5MB): <100ms parse time
- Batch processing: 100 emails/second (with rayon parallelism)

## Security Considerations

### Threat Model

1. **Malicious Email Files**: Crafted to exploit parser vulnerabilities
   - Mitigation: Use safe Rust parsers (mailparse, msg-parser)
   - Mitigation: Fuzz testing with arbitrary input

2. **Encoding Attacks**: Malicious character sequences
   - Mitigation: encoding_rs handles safely
   - Mitigation: Validate UTF-8 output

3. **Resource Exhaustion**: Extremely large emails or attachments
   - Mitigation: Size limits from ParseOptions (inherited)
   - Mitigation: Attachment content not loaded into memory

4. **Script Injection**: Malicious HTML/JavaScript in email body
   - Mitigation: html2text doesn't execute scripts
   - Mitigation: Output is plain text only

5. **Path Traversal**: Malicious attachment filenames (e.g., "../../etc/passwd")
   - Mitigation: Only store filename string, don't write to filesystem
   - Mitigation: Future: Sanitize filenames if extraction is added

### Safe Parsing Practices

- No unsafe code required
- No `.unwrap()` on email data
- All parsing returns Result
- Malformed input degrades gracefully
- Size limits prevent memory exhaustion
- No external resource loading (files, network)

## Migration Path

### Phase 1: veil-email (This Feature)

- Parse EML and MSG to EmailMessage
- Output TextSegments for PII detection
- CLI integration: `veil scan email.eml`

### Phase 2: Attachment Extraction (Future)

- API to extract specific attachment content
- Parse attachments recursively (PDF, Office docs)
- CLI: `veil extract email.eml --attachment 0`

### Phase 3: Advanced Email Features (Future)

- MBOX format (Unix mailbox files)
- PST format (Outlook data files)
- Email thread reconstruction
- Calendar invite parsing (.ics)
- Contact card parsing (vCard)

### Phase 4: Email Streaming (Future)

- Stream large emails without full load
- Process IMAP/POP3 directly
- Real-time email monitoring

## Open Questions

1. **RTF Body Handling**: MSG files may contain RTF-formatted body instead of plain text.
   - **Decision**: For P1, extract RTF as-is (may contain control codes). For P2, add rtf-grimoire crate to convert RTF to text.

2. **Embedded EML Attachments**: Some emails have attached .eml files.
   - **Decision**: List as attachment metadata for P1. Recursive parsing is future enhancement.

3. **S/MIME and PGP**: Encrypted or signed emails.
   - **Decision**: Extract headers only, mark body as encrypted. Decryption is out of scope.

4. **Calendar Invites**: .ics attachments or inline calendar data.
   - **Decision**: Treat as attachment metadata. Specialized .ics parsing is future work.

5. **Performance Profiling**: Need real-world benchmarks.
   - **Decision**: Add benchmark suite in Phase 3 using criterion crate.

## Success Metrics

From spec.md success criteria:

- **SC-001**: EML files parse with 100% header extraction accuracy for standard headers.
  - **Test**: Integration tests verify From, To, CC, Subject, Date extracted.

- **SC-002**: MSG files parse with same accuracy as EML for equivalent content.
  - **Test**: Compare MSG vs. EML output for same email exported both ways.

- **SC-003**: HTML-to-text conversion preserves readable content without HTML artifacts.
  - **Test**: Verify `<p>`, `<a>`, `<img>` converted correctly, no `<script>` remnants.

- **SC-004**: Emails with 10+ attachments are parsed in under 1 second (metadata only).
  - **Test**: Benchmark with fixture containing 10 PDF attachments.

- **SC-005**: Character encoding is correctly handled for 99% of emails.
  - **Test**: Test suite includes Japanese, Russian, emoji emails.

- **SC-006**: Quoted content detection works for common reply patterns (>, On...wrote:).
  - **Test**: Verify email_thread.eml correctly marks quoted sections.

## Risks and Mitigations

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| msg-parser has bugs (low adoption) | High | Medium | Extensive testing with real Outlook MSG files; fallback to "unsupported" error |
| HTML conversion loses important PII | Medium | Low | Test with diverse HTML emails; manual review of conversion quality |
| Quote detection misses patterns | Low | Medium | Start with common patterns, add more via user feedback |
| Performance degrades on large emails | Medium | Low | Profile with 5MB+ emails, add streaming if needed |
| Encoding detection fails for rare charsets | Low | Medium | Use encoding_rs (robust), emit warnings for lossy conversions |

## Complexity Tracking

*No constitution violations to justify.*

All complexity is essential:
- 7 new dependencies justified in research.md (only viable options)
- EmailHeaderValue enum needed for type safety (not premature abstraction)
- Clone usage for String conversions is unavoidable (encoding requires owned Strings)

## Conclusion

This plan implements email parsing (EML and MSG) with:
- **Security**: Safe Rust parsers, no script execution, size limits
- **Stability**: Result-based error handling, graceful degradation
- **Performance**: <100ms for typical emails, metadata-only attachments
- **Simplicity**: Single parse_email() function, direct crate wrapping
- **Constitution compliance**: TDD workflow, minimal dependencies, Clippy enforced

Next step: Run `/speckit.tasks` to generate TDD task breakdown in tasks.md.
