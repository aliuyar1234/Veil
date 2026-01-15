# Research: Office Document Parsing in Rust

**Feature**: 006-office-parser
**Research Date**: 2025-12-15
**Researcher**: Claude

## Objective

Identify suitable Rust crates for parsing Office Open XML formats (DOCX, XLSX, PPTX) to extract text content and metadata for PII detection.

## Requirements Context

- **Target formats**: DOCX, XLSX, PPTX (Office Open XML only)
- **Output interface**: TextSegments compatible with veil-parsers (Position enum)
- **Performance target**: 10MB files in <5 seconds; 100K row Excel without memory issues
- **Constitutional constraints**: Prefer single-purpose crates, active maintenance, minimal dependencies, security-audited for crypto

## Available Rust Crates

### DOCX Parsing

#### 1. `docx-rs` (https://crates.io/crates/docx-rs)
- **Version**: 0.4.x (active development)
- **Purpose**: Read and write DOCX files
- **Dependencies**: Moderate (xml-rs, zip, serde)
- **Pros**:
  - Pure Rust implementation
  - Read and write support
  - Active maintenance (last updated 2024)
  - Handles paragraphs, tables, headers/footers
- **Cons**:
  - API designed for writing; reading API less mature
  - Limited documentation for extraction use cases
  - May require manual XML navigation for complex structures
- **Constitution alignment**: ✅ Single-purpose, active, reasonable deps

#### 2. `docx` (https://crates.io/crates/docx)
- **Version**: 0.3.x
- **Purpose**: DOCX parsing library
- **Dependencies**: Light (zip, xml-rs)
- **Pros**:
  - Simpler API focused on reading
  - Lightweight
- **Cons**:
  - Less active (last update 2023)
  - Limited feature set compared to docx-rs
  - Sparse documentation
- **Constitution alignment**: ⚠️ Maintenance concerns

#### 3. Manual Approach: `zip` + `quick-xml`
- **Approach**: Parse DOCX as ZIP, extract XML, parse with quick-xml
- **Pros**:
  - Complete control over extraction
  - Minimal dependencies (zip 0.6, quick-xml 0.31)
  - High performance (quick-xml is event-driven)
  - DOCX format is well-documented (ECMA-376)
- **Cons**:
  - Requires understanding DOCX XML structure (document.xml, styles.xml, etc.)
  - More implementation work upfront
  - Must handle relationships, parts, content types manually
- **Constitution alignment**: ✅✅ Minimal deps, full control, security auditable

**Recommendation**: **Manual approach with zip + quick-xml** for maximum control and minimal dependencies.

### XLSX Parsing

#### 1. `calamine` (https://crates.io/crates/calamine)
- **Version**: 0.24.x (very active)
- **Purpose**: Excel file reader (XLS, XLSX, ODS)
- **Dependencies**: Moderate (zip, quick-xml, encoding_rs, serde)
- **Pros**:
  - **Industry standard** for Excel parsing in Rust
  - Excellent performance (streaming support)
  - Active maintenance (updated monthly)
  - Handles shared strings, formulas, multiple sheets
  - Used in production by multiple projects
  - Supports both legacy XLS and modern XLSX
- **Cons**:
  - Slightly heavier (multi-format support adds complexity)
  - Returns cell values, not formulas (but this is what we want)
- **Constitution alignment**: ✅✅ Well-maintained, trusted, appropriate scope

#### 2. `xlsx_reader` (https://crates.io/crates/xlsx_reader)
- **Version**: 0.2.x
- **Purpose**: Simple XLSX reader
- **Dependencies**: Light
- **Pros**:
  - Minimalist API
- **Cons**:
  - Less active maintenance
  - Fewer features than calamine
  - Limited handling of complex Excel files
- **Constitution alignment**: ⚠️ Less proven, maintenance concerns

#### 3. Manual Approach: `zip` + `quick-xml`
- Similar to DOCX, but Excel XML is more complex (shared strings table, relationships, cell references)
- **Not recommended**: Calamine already exists and is well-tested

**Recommendation**: **`calamine`** - proven, performant, actively maintained, correct scope.

### PPTX Parsing

#### 1. No mature Rust crates exist

- **Research findings**: No dedicated PPTX parsing library found on crates.io
- **Explanation**: PPTX is the least common format for data processing use cases
- **PPTX structure**: Similar to DOCX (ZIP archive with XML), but slide content in `ppt/slides/slideN.xml` and notes in `ppt/notesSlides/notesSlideN.xml`

#### 2. Manual Approach: `zip` + `quick-xml`
- **Approach**: Extract ZIP, parse slide XML files
- **Pros**:
  - PPTX format is documented (ECMA-376 Part 1, Section 19)
  - Slides have simpler structure than Word documents
  - Can reuse learnings from DOCX manual parsing
- **Cons**:
  - Must understand DrawingML (shapes, text boxes)
  - Slide layouts and master slides add complexity
- **Constitution alignment**: ✅ Only viable option, keeps dependencies minimal

**Recommendation**: **Manual approach with zip + quick-xml** (necessary, no alternative).

## Dependency Analysis

### Core Dependencies (All Approaches)

| Crate | Version | Purpose | Transitive Deps | Security Concerns |
|-------|---------|---------|----------------|-------------------|
| `zip` | 0.6 | Extract OOXML archives | flate2, crc32fast | ✅ Widely used |
| `quick-xml` | 0.31 | Parse Office XML | memchr | ✅ Fast, safe, popular |
| `serde` | 1.0 | Serialization | Already in workspace | ✅ Audited |

### Additional for Calamine (XLSX)

| Crate | Version | Purpose | Transitive Deps | Security Concerns |
|-------|---------|---------|----------------|-------------------|
| `calamine` | 0.24 | Excel parsing | Uses zip, quick-xml, encoding_rs | ✅ Mature, active |
| `encoding_rs` | 0.8 | Character encoding | Already in workspace | ✅ Mozilla project |

### Total Dependency Count
- **Manual DOCX + Manual PPTX + Calamine XLSX**: ~8 crates (zip, quick-xml, calamine, encoding_rs + their transitive deps)
- **Acceptable**: All dependencies are single-purpose, actively maintained, and widely used

## Performance Considerations

### Streaming vs. In-Memory

- **XLSX via Calamine**: Supports streaming row-by-row (critical for 100K row requirement)
- **DOCX/PPTX (manual)**: Office XML files are typically <10MB even for large documents (text compresses well); in-memory parsing is acceptable
- **ZIP extraction**: Can stream individual files from archive without extracting entire ZIP to disk

### Benchmarking Targets (from spec)

| Metric | Target | Approach |
|--------|--------|----------|
| 10MB Office file | <5 seconds | Streaming XLSX, buffered XML parsing |
| 100K row Excel | No memory issues | Calamine's streaming API |
| DOCX text extraction | 99% accuracy vs copy-paste | Full XML parsing (no shortcuts) |

## Format Detection

### File Signatures

- **Office Open XML**: All formats are ZIP archives with specific directory structures
  - Magic bytes: `PK\x03\x04` (ZIP signature)
  - Detection: Unzip and check for `[Content_Types].xml` + format-specific folders:
    - DOCX: `word/document.xml`
    - XLSX: `xl/workbook.xml`
    - PPTX: `ppt/presentation.xml`

### Encrypted Files

- **Requirement**: FR-007 - Report encrypted files as unprocessable
- **Detection**: Encrypted OOXML uses `EncryptionInfo` and `EncryptedPackage` streams
  - Check `[Content_Types].xml` for `EncryptedPackage` content type
  - Or attempt ZIP extraction and catch encryption errors
- **Implementation**: Return `ParseError::Encrypted` with helpful message

### Legacy Formats (.doc, .xls, .ppt)

- **Requirement**: FR-008 - Reject with clear error message
- **Detection**: Binary formats have different magic bytes:
  - DOC/XLS/PPT: `\xD0\xCF\x11\xE0` (OLE2 compound file)
- **Implementation**: Return `ParseError::UnsupportedFormat` suggesting conversion

## Metadata Extraction

### Office Open XML Metadata (All Formats)

- **Location**: `docProps/core.xml` (Dublin Core properties)
- **Standard fields**:
  - `dc:creator` - Author
  - `dc:title` - Document title
  - `dc:subject` - Subject
  - `cp:lastModifiedBy` - Last editor
  - `dcterms:created` - Creation date
  - `dcterms:modified` - Modification date
- **Extended properties**: `docProps/app.xml` (company, manager, etc.)

### Implementation Strategy

1. Extract `docProps/core.xml` and `docProps/app.xml` from ZIP
2. Parse XML for metadata fields
3. Return as TextSegments with `Position::OfficeMetadata` variant (new)

## Risk Assessment

### Security Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| ZIP bomb (compressed bomb) | Medium | High (DoS) | Limit uncompressed size (50MB per spec) |
| XXE injection in XML | Low | Medium | quick-xml defaults are safe, no external entity resolution |
| Malicious macros in .xlsm/.docm | Low | None | Macros are ignored (VBA code not extracted) |
| Path traversal in ZIP | Low | Medium | Sanitize ZIP entry paths |

### Compatibility Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Unsupported Office features | Medium | Low | Best-effort extraction; log warnings for unknown elements |
| Corrupted OOXML files | Medium | Medium | Graceful error handling with `ParseError::Corrupted` |
| Non-standard Office variants (e.g., Google Docs exports) | Low | Low | Follow ECMA-376 strictly; may partially work |

## Recommendations

### Selected Approach

1. **XLSX**: Use **`calamine` 0.24** - proven, performant, correct scope
2. **DOCX**: **Manual parsing** with `zip` + `quick-xml` - full control, minimal deps
3. **PPTX**: **Manual parsing** with `zip` + `quick-xml` - only viable option

### Rationale

- **Constitutional compliance**: Minimal dependencies, all single-purpose and actively maintained
- **Performance**: Calamine's streaming meets 100K row requirement; manual parsing allows optimization
- **Security**: All dependencies are widely used and audited; manual parsing eliminates unknown code paths
- **Maintainability**: Calamine handles Excel complexity; manual parsing for simpler DOCX/PPTX is understandable

### Implementation Priority

1. **Phase 1**: XLSX parsing (highest business value - bulk PII data)
2. **Phase 2**: DOCX parsing (most common format)
3. **Phase 3**: PPTX parsing (lower priority - P2 in spec)
4. **Phase 4**: Metadata extraction (all formats)

### Dependency Additions to `Cargo.toml`

```toml
[workspace.dependencies]
# Office document parsing
calamine = "0.24"           # XLSX parsing
zip = { version = "0.6", default-features = false, features = ["deflate"] }
quick-xml = "0.31"          # XML parsing for DOCX/PPTX
```

## Open Questions

1. **Comments in Office documents**: Should track changes and comments be extracted?
   - **Recommendation**: Yes (Priority P2) - they may contain PII
   - **Implementation**: Additional XML files (`word/comments.xml`, `xl/comments.xml`)

2. **Hidden content**: Hidden sheets in Excel, hidden text in Word?
   - **Recommendation**: Yes - hidden content may contain PII that was "hidden" rather than removed
   - **Implementation**: Calamine handles hidden sheets by default; DOCX requires checking `w:vanish` attribute

3. **Embedded objects**: Excel charts embedded in Word, etc.?
   - **Recommendation**: No (out of scope for v1) - extract text from main document only
   - **Implementation**: Log warning if `embeddings/` folder detected in ZIP

4. **Unicode normalization**: Should text be normalized (NFC/NFD)?
   - **Recommendation**: No - preserve original text exactly for accurate PII detection
   - **Implementation**: Pass through as-is

## References

- [Office Open XML (ECMA-376)](https://www.ecma-international.org/publications-and-standards/standards/ecma-376/)
- [calamine documentation](https://docs.rs/calamine/)
- [quick-xml documentation](https://docs.rs/quick-xml/)
- [ZIP specification (PKWARE)](https://pkware.cachefly.net/webdocs/casestudies/APPNOTE.TXT)
- [OOXML Explained (Microsoft)](https://learn.microsoft.com/en-us/office/open-xml/structure-of-a-spreadsheetml-document)

## Conclusion

The combination of **calamine for XLSX** and **manual ZIP+XML parsing for DOCX/PPTX** provides:
- Constitutional compliance (minimal, justified dependencies)
- Performance targets met (streaming support)
- Security (auditable, no hidden complexity)
- Maintainability (well-understood approach)

This approach aligns with Veil's constitution principle of "Ship less, ship solid."
