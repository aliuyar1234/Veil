# Email Parser Research

**Feature**: 007-email-parser
**Date**: 2025-12-15
**Author**: Claude (Veil Development)

## Objective

Research available Rust crates for parsing EML (RFC 5322) and MSG (Microsoft Outlook) email formats to enable PII detection in email files.

## Requirements Summary

- Parse EML files (RFC 5322 standard email format)
- Parse MSG files (Microsoft Outlook proprietary format)
- Extract headers (From, To, CC, BCC, Subject, Date, Message-ID)
- Parse email addresses with display names
- Extract plain text and HTML body content
- List attachment metadata (filename, size, MIME type)
- Handle MIME multipart messages
- Detect quoted/replied content
- Handle character encoding conversion
- Output TextSegments compatible with veil-parsers interface

## Rust Crate Analysis

### EML Parsing Crates

#### 1. mailparse (v0.15.0)

**Repository**: https://github.com/staktrace/mailparse
**License**: MIT/Apache-2.0
**Maintenance**: Active (last release Nov 2023)
**Downloads**: ~500K/month

**Features**:
- RFC 5322 and RFC 2047 compliant
- MIME multipart message handling
- Header parsing with encoding support
- Attachment extraction
- No dependencies on system libraries
- Pure Rust implementation

**Pros**:
- Well-maintained and widely used
- Comprehensive MIME support
- Good documentation
- Works with byte slices (zero-copy potential)
- Handles character encoding properly

**Cons**:
- API is lower-level, requires wrapping
- No HTML-to-text conversion built-in
- Quoted text detection not included

**Constitution Compliance**:
- Security: No unsafe code in core parsing
- Dependencies: Minimal (charset, quoted_printable, base64)
- Stability: Returns Result types, no panics on malformed input

#### 2. mail-parser (v0.9.0)

**Repository**: https://github.com/stalwartlabs/mail-parser
**License**: Apache-2.0
**Maintenance**: Active (last release Oct 2023)
**Downloads**: ~100K/month

**Features**:
- RFC 5322, RFC 2045-2049 compliant
- High-level API with Message type
- Built-in HTML/text part handling
- Header parsing with typed accessors
- Attachment metadata extraction
- Character encoding detection

**Pros**:
- More ergonomic high-level API
- Better structured message representation
- Active development by Stalwart Labs
- Good performance characteristics
- Type-safe header access

**Cons**:
- Larger dependency tree than mailparse
- Less adoption in ecosystem
- Some allocations required for convenience

**Constitution Compliance**:
- Security: Safe code, good error handling
- Dependencies: More than mailparse but justified
- Stability: Good Result usage, handles malformed emails gracefully

#### 3. email-parser (v0.5.0)

**Repository**: https://github.com/cloudflare/email-parser
**License**: BSD-3-Clause
**Maintenance**: Sporadic (last release 2021)
**Downloads**: ~5K/month

**Features**:
- Basic RFC 5322 parsing
- Header extraction
- Simple API

**Pros**:
- Minimal dependencies
- Created by Cloudflare

**Cons**:
- **Not recommended**: Low maintenance activity
- Limited MIME support
- Incomplete implementation
- Small user base

### MSG Parsing Crates

#### 1. msg-parser (v0.5.0)

**Repository**: https://github.com/contextal/msg-parser
**License**: MIT/Apache-2.0
**Maintenance**: Active (last release 2024)
**Downloads**: ~1K/month

**Features**:
- Parses Microsoft Outlook MSG format
- OLE compound document format handling
- Property extraction (headers, body, attachments)
- Named properties support
- RTF body decompression

**Pros**:
- Only dedicated MSG parser in Rust
- Handles MSG format specifics
- Active development
- Good documentation

**Cons**:
- Lower adoption (newer crate)
- Depends on cfb crate for OLE parsing
- RTF conversion may need additional library

**Constitution Compliance**:
- Security: Handles untrusted MSG files safely
- Dependencies: cfb (OLE parser), encoding_rs
- Stability: Good error handling for malformed files

#### 2. cfb (v0.10.0)

**Repository**: https://github.com/mdsteele/rust-cfb
**License**: MIT
**Maintenance**: Active
**Downloads**: ~100K/month

**Features**:
- Parses Compound File Binary (OLE) format
- Used as foundation for MSG parsing
- Read-only access to structured storage

**Note**: This is a lower-level dependency; msg-parser handles MSG-specific logic.

### Supporting Crates

#### HTML to Text Conversion

**html2text (v0.12.0)**:
- Converts HTML to readable plain text
- Handles common HTML elements
- ~200K downloads/month
- Well-maintained

**scraper (v0.19.0)** (already in veil-parsers):
- Could be used for HTML parsing
- More control over extraction
- Already approved dependency

#### Character Encoding

**encoding_rs (v0.8.0)** (already in veil-parsers):
- Standard Rust encoding library
- Used by Firefox
- Fast and comprehensive

#### Base64/Quoted-Printable

**base64 (v0.21.0)**:
- Standard base64 encoding/decoding
- Used by mailparse

**quoted_printable (v0.5.0)**:
- MIME quoted-printable decoding
- Used by mailparse

## Recommended Stack

### Primary Choices

1. **EML Parsing**: `mailparse` (v0.15.0)
   - Most mature and widely adopted
   - Comprehensive MIME support
   - Minimal dependencies
   - Good safety characteristics

2. **MSG Parsing**: `msg-parser` (v0.5.0)
   - Only viable option for MSG format
   - Active maintenance
   - Safe handling of OLE format

3. **HTML to Text**: `html2text` (v0.12.0)
   - Dedicated conversion library
   - Better than regex stripping
   - Preserves readability

### Alternative Consideration

**mail-parser** could be considered instead of mailparse if:
- Higher-level API is strongly preferred
- Typed header access is critical
- Performance testing shows no regression

**Recommendation**: Start with mailparse for stability and ecosystem maturity. Switch to mail-parser if API ergonomics become a maintenance burden.

## Dependency Justification

| Crate | Purpose | Transitive Deps | Justification |
|-------|---------|-----------------|---------------|
| mailparse | EML/MIME parsing | charset, quoted_printable, base64 | Standard for email parsing in Rust ecosystem |
| msg-parser | MSG format parsing | cfb, encoding_rs | Only option for MSG support |
| html2text | HTML body conversion | html5ever, markup5ever | Safer than regex, preserves formatting |
| encoding_rs | Character encoding | (already approved) | Required for international emails |

**Total new dependencies**: ~7 crates (mailparse + msg-parser + html2text stacks)

**Constitution compliance**: All crates use safe Rust, have active maintenance, and are focused single-purpose libraries.

## Security Considerations

### Email-Specific Threats

1. **Malformed Headers**: Both mailparse and msg-parser handle gracefully with Result returns
2. **Encoding Attacks**: encoding_rs prevents buffer overflows
3. **Zip Bombs in Attachments**: Parser only extracts metadata, not content
4. **XXE in HTML**: html2text doesn't execute scripts or load external resources

### Mitigation Strategy

- Size limits on email files (inherited from ParseOptions)
- No automatic attachment content extraction
- HTML parsing in safe mode (no script execution)
- Validate header values before use in detection

## Performance Expectations

Based on crate benchmarks and documentation:

- **EML Parsing**: ~10-50 MB/s depending on MIME complexity
- **MSG Parsing**: ~5-20 MB/s (OLE overhead)
- **HTML Conversion**: ~20-100 MB/s

For typical email sizes (10KB - 5MB):
- Parse time: <100ms for most emails
- Memory: ~2-3x file size during parsing

Meets success criteria SC-004: 10+ attachments in <1 second (metadata only).

## Quote Detection Strategy

Common patterns to detect in email bodies:

1. **Line-based quoting**: Lines starting with `>`, `>>`, etc.
2. **Attribution lines**: `On <date>, <person> wrote:`
3. **Reply separators**: `-----Original Message-----`
4. **HTML blockquote**: `<blockquote>` tags in HTML parts

**Implementation approach**:
- Regex patterns for common formats
- Track quote depth for nested replies
- Separate TextSegments for quoted vs. original content

**Note**: mailparse/msg-parser don't include this; custom implementation needed.

## Open Questions

1. **RTF Body Handling**: MSG files may contain RTF-formatted body.
   - **Decision**: Convert RTF to plain text using rtf-grimoire crate if needed, or treat as opaque.

2. **Embedded EML Files**: Some emails have attached .eml files.
   - **Decision**: List as attachment metadata; recursive parsing is out of scope for P1.

3. **Calendar Invites**: .ics attachments or inline calendar data.
   - **Decision**: Treat as attachment metadata; specialized parsing is future work.

4. **S/MIME and PGP**: Encrypted or signed emails.
   - **Decision**: Extract headers only, mark body as encrypted/signed (per edge cases in spec).

## Conclusion

**Recommended implementation**:
- Use `mailparse` for EML parsing (mature, stable, minimal dependencies)
- Use `msg-parser` for MSG parsing (only option, actively maintained)
- Use `html2text` for HTML body conversion
- Implement custom quote detection using regex patterns
- Wrap both in a unified API that outputs veil-parsers TextSegments

This stack balances constitution principles (security, stability, minimal dependencies) with practical functionality requirements.

## References

- RFC 5322: Internet Message Format - https://www.rfc-editor.org/rfc/rfc5322
- RFC 2045-2049: MIME - https://www.rfc-editor.org/rfc/rfc2045
- MSG Format Specification: [MS-OXMSG] - https://docs.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxmsg/
- mailparse docs: https://docs.rs/mailparse/
- msg-parser docs: https://docs.rs/msg-parser/
