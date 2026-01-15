# Implementation Plan: Office Document Parser

**Branch**: `006-office-parser` | **Date**: 2025-12-15 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/006-office-parser/spec.md`

## Summary

Build a parser for Microsoft Office Open XML documents (DOCX, XLSX, PPTX) that extracts text content, tables, metadata, and provides precise location information for PII detection. The parser integrates with veil-parsers, returning `TextSegment`s with Office-specific `Position` variants (cell references for Excel, paragraph/page for Word, slide numbers for PowerPoint). Implementation uses calamine for XLSX parsing and manual ZIP+XML parsing for DOCX/PPTX to minimize dependencies while maintaining full control over extraction.

## Technical Context

**Language/Version**: Rust (stable, 1.75+)
**Primary Dependencies**: calamine (XLSX), zip (ZIP extraction), quick-xml (XML parsing), serde, encoding_rs (already in workspace)
**Storage**: N/A (pure library, no persistence)
**Testing**: cargo test (unit + integration tests with real Office files)
**Target Platform**: Cross-platform library (Linux, macOS, Windows, WASM-compatible)
**Project Type**: New crate (veil-office) integrated with veil-parsers workspace
**Performance Goals**: 10MB files <5s, 100K row Excel without memory issues, streaming for large XLSX
**Constraints**: 50MB max file size (per spec), Office Open XML only (no legacy .doc/.xls/.ppt), encrypted files rejected
**Scale/Scope**: Single-file parsing, all sheets/slides/sections extracted, precise position metadata

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Security First | ✅ PASS | No unsafe needed; ZIP path sanitization; encrypted file detection; no XXE (quick-xml safe defaults) |
| II. Stability & Error Handling | ✅ PASS | Result types for all fallible operations; graceful handling of corrupted Office files; specific errors for encryption/legacy formats |
| III. Performance | ✅ PASS | Streaming for large Excel (calamine); zero-copy XML parsing (quick-xml borrows); lazy sheet evaluation |
| IV. Simplicity & Minimalism | ✅ PASS | Manual parsing avoids unnecessary abstraction; one parser per format; reuses veil-parsers types |
| V. Test-First Development | ✅ PASS | Real Office test files for each format; edge cases (empty, encrypted, corrupted, large) |
| VI. Dependency Discipline | ⚠️ REVIEW | calamine (justified - Excel is complex), zip (required for OOXML), quick-xml (efficient XML) - all actively maintained |
| VII. Rust Standards | ✅ PASS | Clippy/fmt; documented public API; thiserror for errors |

**Gate Result**: PASS (dependencies justified - calamine is industry standard for Excel, manual parsing for others minimizes deps)

## Project Structure

### Documentation (this feature)

```text
specs/006-office-parser/
├── plan.md              # This file
├── research.md          # Phase 0 output (crate analysis)
├── data-model.md        # Phase 1 output (types, Position variants)
├── quickstart.md        # Phase 1 output (usage examples)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created yet)
```

### Source Code (repository root)

```text
Cargo.toml               # Workspace root (add zip, quick-xml, calamine)

crates/veil-parsers/     # Extended with Office support
├── Cargo.toml           # Add veil-office dependency
├── src/
│   ├── lib.rs           # Add Office formats to parse_bytes() dispatch
│   ├── types.rs         # Add Position::Docx, Position::Xlsx, Position::Pptx, Position::OfficeMetadata
│   ├── error.rs         # Add ParseError::Encrypted, ParseError::UnsupportedFormat
│   └── detect.rs        # Add Office format detection (ZIP + content types)

crates/veil-office/      # NEW CRATE
├── Cargo.toml
├── src/
│   ├── lib.rs           # Public API: parse_docx, parse_xlsx, parse_pptx
│   ├── error.rs         # OfficeError (thiserror)
│   ├── metadata.rs      # OfficeMetadata extraction (docProps/core.xml, app.xml)
│   ├── detect.rs        # Detect DOCX/XLSX/PPTX from ZIP contents
│   ├── utils.rs         # Shared utilities (ZIP extraction, cell reference conversion)
│   │
│   ├── docx/
│   │   ├── mod.rs       # DOCX parser entry point
│   │   ├── parser.rs    # Main document.xml parser
│   │   ├── styles.rs    # Paragraph styles (for context)
│   │   ├── tables.rs    # Table extraction (w:tbl elements)
│   │   ├── headers.rs   # Header/footer extraction
│   │   └── rels.rs      # Relationship resolution (if needed)
│   │
│   ├── xlsx/
│   │   ├── mod.rs       # XLSX parser entry point
│   │   ├── parser.rs    # Wrapper around calamine
│   │   ├── streaming.rs # Streaming row iterator for large files
│   │   └── cell_ref.rs  # CellReference utility (column letters, cell refs)
│   │
│   └── pptx/
│       ├── mod.rs       # PPTX parser entry point
│       ├── parser.rs    # Slide XML parser
│       ├── slides.rs    # Slide content extraction (p:txBody)
│       └── notes.rs     # Speaker notes extraction
│
└── tests/
    ├── fixtures/
    │   ├── docx/
    │   │   ├── simple.docx          # Basic paragraph text
    │   │   ├── table.docx           # Document with tables
    │   │   ├── header_footer.docx   # Headers and footers
    │   │   ├── metadata.docx        # Rich metadata
    │   │   ├── encrypted.docx       # Password-protected
    │   │   └── corrupted.docx       # Malformed ZIP
    │   ├── xlsx/
    │   │   ├── simple.xlsx          # Single sheet, basic cells
    │   │   ├── multi_sheet.xlsx     # Multiple sheets
    │   │   ├── formulas.xlsx        # Cells with formulas (test display values)
    │   │   ├── large.xlsx           # 100K rows (streaming test)
    │   │   ├── hidden_sheet.xlsx    # Hidden sheet with data
    │   │   └── metadata.xlsx        # Rich metadata
    │   ├── pptx/
    │   │   ├── simple.pptx          # Title + body slides
    │   │   ├── notes.pptx           # Speaker notes
    │   │   ├── shapes.pptx          # Text in shapes/text boxes
    │   │   └── metadata.pptx        # Rich metadata
    │   └── legacy/
    │       ├── old.doc              # Legacy Word (should reject)
    │       ├── old.xls              # Legacy Excel (should reject)
    │       └── old.ppt              # Legacy PowerPoint (should reject)
    ├── docx_tests.rs
    ├── xlsx_tests.rs
    ├── pptx_tests.rs
    ├── metadata_tests.rs
    ├── error_tests.rs          # Encrypted, legacy, corrupted
    └── integration_tests.rs    # Full parse_file() tests
```

**Structure Decision**: Separate `veil-office` crate keeps Office-specific dependencies isolated from veil-parsers. Manual parsing for DOCX/PPTX gives full control and minimal deps. Calamine for XLSX is industry-proven and handles complexity correctly.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| calamine crate | Excel format is complex (shared strings, formulas, relationships, streaming) | Manual XLSX parsing would be 1000+ lines and error-prone; calamine is well-tested |
| zip crate | Office Open XML is ZIP-based; extraction required | No alternative - OOXML spec mandates ZIP |
| quick-xml crate | Event-driven XML parsing for DOCX/PPTX | serde_xml too rigid; roxmltree loads entire tree (memory issue); quick-xml is zero-copy |

## Implementation Phases

### Phase 0: Research (COMPLETE)

- [x] Analyze Rust crates for Office parsing
- [x] Evaluate calamine vs manual XLSX parsing
- [x] Research DOCX/PPTX XML structure (ECMA-376)
- [x] Document security concerns (ZIP bombs, XXE, encrypted files)
- **Output**: [research.md](./research.md)

### Phase 1: Design (COMPLETE)

- [x] Define Position variants (Docx, Xlsx, Pptx, OfficeMetadata)
- [x] Design OfficeMetadata struct (author, company, dates)
- [x] Design DOCX types (DocxParagraph, DocxTable, DocxSection enum)
- [x] Design XLSX types (XlsxSheet, CellReference utility)
- [x] Design PPTX types (PptxSlide, PptxElement enum)
- [x] Define OfficeError types
- **Output**: [data-model.md](./data-model.md), [quickstart.md](./quickstart.md)

### Phase 2: Tasks (PENDING - use /speckit.tasks)

- Will generate detailed implementation tasks
- **Output**: tasks.md

### Phase 3: Implementation (PENDING)

Implementation order prioritized by business value and dependency:

#### 3.1: Foundation (Week 1)

1. Create veil-office crate structure
2. Add workspace dependencies (zip, quick-xml, calamine)
3. Implement OfficeError types with thiserror
4. Implement Office format detection (ZIP + [Content_Types].xml)
5. Implement metadata extraction (docProps/*.xml)
6. Tests: Format detection, metadata extraction

#### 3.2: XLSX Parser (Week 2) - Priority P1

7. Implement XLSX parser using calamine
8. Implement CellReference utility (column letters, cell refs)
9. Implement streaming support for large files
10. Extract all sheets (including hidden sheets)
11. Convert cells to TextSegment with Position::Xlsx
12. Tests: Simple XLSX, multi-sheet, formulas, 100K rows, hidden sheets

#### 3.3: DOCX Parser (Week 3) - Priority P1

13. Implement ZIP extraction for document.xml
14. Implement paragraph extraction (w:p → w:t elements)
15. Implement table extraction (w:tbl → w:tc)
16. Implement header/footer extraction
17. Convert to TextSegment with Position::Docx
18. Tests: Simple DOCX, tables, headers/footers

#### 3.4: PPTX Parser (Week 4) - Priority P2

19. Implement slide extraction (ppt/slides/slideN.xml)
20. Implement speaker notes extraction (ppt/notesSlides/notesSlideN.xml)
21. Implement shape text extraction (DrawingML a:t elements)
22. Convert to TextSegment with Position::Pptx
23. Tests: Simple PPTX, speaker notes, text in shapes

#### 3.5: Integration (Week 5)

24. Extend veil-parsers types.rs with new Position variants
25. Extend veil-parsers detect.rs with Office format detection
26. Integrate veil-office into parse_bytes() dispatch
27. Add error handling tests (encrypted, legacy, corrupted)
28. Integration tests with real Office files
29. Performance benchmarks (10MB files, 100K rows)

#### 3.6: Polish (Week 6)

30. Documentation comments on all public items
31. Update veil-parsers README with Office support
32. Clippy and fmt enforcement
33. Final performance tuning
34. Update CLAUDE.md with Office parser info

## Module Organization

### veil-office/src/lib.rs

```rust
//! Office Open XML document parser (DOCX, XLSX, PPTX).

pub mod docx;
pub mod xlsx;
pub mod pptx;
pub mod metadata;
pub mod detect;
pub mod error;
mod utils;

pub use error::OfficeError;
pub use metadata::OfficeMetadata;

/// Parse DOCX file bytes.
pub fn parse_docx(bytes: &[u8], options: &ParseOptions) -> Result<ParseResult, OfficeError>;

/// Parse XLSX file bytes.
pub fn parse_xlsx(bytes: &[u8], options: &ParseOptions) -> Result<ParseResult, OfficeError>;

/// Parse PPTX file bytes.
pub fn parse_pptx(bytes: &[u8], options: &ParseOptions) -> Result<ParseResult, OfficeError>;
```

### Key Algorithms

#### XLSX Cell Reference Conversion

```rust
/// Convert column index (0-based) to Excel column letter.
/// 0 -> A, 25 -> Z, 26 -> AA, 27 -> AB, etc.
pub fn column_to_letter(col: usize) -> String {
    let mut col = col;
    let mut result = String::new();
    loop {
        result.insert(0, (b'A' + (col % 26) as u8) as char);
        if col < 26 {
            break;
        }
        col = col / 26 - 1;
    }
    result
}

/// Format cell reference: Sheet1!B5
pub fn format_cell_ref(sheet: &str, row: usize, col: usize) -> String {
    format!("{}!{}{}", sheet, column_to_letter(col), row)
}
```

#### DOCX Paragraph Extraction (quick-xml)

```rust
use quick_xml::Reader;
use quick_xml::events::Event;

fn extract_paragraphs(xml: &[u8]) -> Result<Vec<String>, OfficeError> {
    let mut reader = Reader::from_reader(xml);
    let mut paragraphs = Vec::new();
    let mut current_text = String::new();
    let mut in_paragraph = false;

    loop {
        match reader.read_event()? {
            Event::Start(e) if e.name().as_ref() == b"w:p" => {
                in_paragraph = true;
                current_text.clear();
            }
            Event::Text(t) if in_paragraph => {
                current_text.push_str(&t.unescape()?);
            }
            Event::End(e) if e.name().as_ref() == b"w:p" => {
                paragraphs.push(current_text.clone());
                in_paragraph = false;
            }
            Event::Eof => break,
            _ => {}
        }
    }

    Ok(paragraphs)
}
```

#### Encrypted File Detection

```rust
use zip::ZipArchive;

fn is_encrypted(archive: &mut ZipArchive<impl Read + Seek>) -> bool {
    // Check for EncryptedPackage or EncryptionInfo
    for i in 0..archive.len() {
        if let Ok(file) = archive.by_index(i) {
            let name = file.name();
            if name.contains("EncryptedPackage") || name.contains("EncryptionInfo") {
                return true;
            }
        }
    }
    false
}
```

## Testing Strategy

### Unit Tests

- Cell reference conversion (A1, Z99, AA1, ZZ999)
- Column letter generation (edge cases)
- XML parsing for each Office format
- Metadata extraction from docProps XML
- Error handling (encrypted, corrupted, legacy)

### Integration Tests

- Parse real Office files from fixtures/
- Verify all sheets/slides/sections extracted
- Verify position metadata accuracy
- Verify metadata extraction
- Performance tests (large files)

### Edge Case Tests

- Empty documents (no content)
- Documents with only metadata
- Hidden sheets in Excel
- Comments and track changes in Word
- Text in shapes/text boxes
- Formulas (verify display value extracted, not formula)
- Multi-language text (Unicode)
- Very long cells/paragraphs
- Corrupted ZIP files
- Encrypted files
- Legacy binary formats

### Acceptance Criteria (from spec)

- **SC-001**: DOCX text extraction matches copy-paste from Word with 99% accuracy
- **SC-002**: XLSX cells extracted with 100% correct cell references
- **SC-003**: PPTX text from all slides and notes extracted completely
- **SC-004**: Document metadata extracted when present
- **SC-005**: 10MB Office document parsed in <5 seconds
- **SC-006**: Excel files with 100K rows processed without memory issues

## Performance Targets

| Metric | Target | Implementation |
|--------|--------|----------------|
| 10MB XLSX parsing | <5s | Calamine streaming API |
| 100K row Excel | <500MB memory | Row-by-row iteration (no full sheet load) |
| DOCX text extraction | <1s per MB | Zero-copy XML parsing with quick-xml |
| Position metadata | <10% overhead | Inline calculation (no extra pass) |

## Security Considerations

### ZIP Bomb Protection

```rust
const MAX_UNCOMPRESSED_SIZE: u64 = 50 * 1024 * 1024; // 50MB per spec

fn check_zip_entry_size(entry: &ZipFile) -> Result<(), OfficeError> {
    if entry.size() > MAX_UNCOMPRESSED_SIZE {
        return Err(OfficeError::FileTooLarge {
            size: entry.size() as usize,
            max: MAX_UNCOMPRESSED_SIZE as usize,
        });
    }
    Ok(())
}
```

### Path Traversal Prevention

```rust
fn sanitize_zip_path(path: &str) -> Result<&str, OfficeError> {
    if path.contains("..") || path.starts_with('/') {
        return Err(OfficeError::Corrupted(
            "Invalid ZIP entry path (possible traversal attack)".to_string()
        ));
    }
    Ok(path)
}
```

### XXE Prevention

quick-xml defaults are safe (no external entity resolution), but explicitly disable if needed:

```rust
let mut reader = Reader::from_reader(xml);
reader.config_mut().expand_entities = false; // Explicit (already default false)
```

## Dependencies Justification

### calamine (0.24)

- **Purpose**: XLSX parsing
- **Justification**: Excel format is extremely complex (shared strings, styles, formulas, multiple sheets, cell references). Calamine is the industry-standard Rust library, actively maintained, used in production.
- **Alternatives rejected**: Manual parsing would be 1000+ LOC and error-prone; xlsx_reader less mature.
- **Transitive deps**: Acceptable (zip, quick-xml, encoding_rs - already in workspace)

### zip (0.6)

- **Purpose**: Extract Office Open XML ZIP archives
- **Justification**: OOXML spec mandates ZIP format; no alternative.
- **Alternatives rejected**: None (ZIP is mandatory for OOXML)
- **Transitive deps**: Minimal (flate2, crc32fast)

### quick-xml (0.31)

- **Purpose**: Parse Office XML (document.xml, workbook.xml, etc.)
- **Justification**: Zero-copy event-driven parsing; efficient for large documents; safe defaults.
- **Alternatives rejected**: serde_xml too rigid; roxmltree loads entire tree (memory issue); xml-rs slower.
- **Transitive deps**: Minimal (memchr)

## Post-Design Constitution Re-Check

*Re-evaluated after Phase 1 design completion (2025-12-15)*

| Principle | Status | Post-Design Notes |
|-----------|--------|-------------------|
| I. Security First | ✅ PASS | No unsafe; ZIP path sanitization implemented; encrypted file detection; XXE prevented (quick-xml safe); ZIP bomb protection via max size |
| II. Stability & Error Handling | ✅ PASS | Result<ParseResult, OfficeError> everywhere; specific errors for encryption/legacy/corrupted; graceful degradation for unknown elements |
| III. Performance | ✅ PASS | Calamine streaming for XLSX (meets 100K row requirement); quick-xml zero-copy; lazy sheet evaluation; <5s for 10MB target achievable |
| IV. Simplicity & Minimalism | ✅ PASS | Manual parsing keeps code understandable; one parser per format; no over-abstraction; types mirror Office structure |
| V. Test-First Development | ✅ PASS | Test fixtures for each format; edge cases (encrypted, corrupted, large, empty); acceptance criteria mapped to tests |
| VI. Dependency Discipline | ✅ PASS | 3 new workspace deps justified: calamine (Excel complexity), zip (OOXML requirement), quick-xml (efficiency) - all actively maintained, minimal transitive deps |
| VII. Rust Standards | ✅ PASS | thiserror for OfficeError; serde derives on public types; documented public API; clippy/fmt enforced |

**Post-Design Gate Result**: PASS - Ready for task generation (/speckit.tasks)

## Open Questions for Implementation

1. **Comments extraction**: Should Word comments (word/comments.xml) be extracted?
   - **Decision**: Yes, Priority P2 (may contain PII like reviewer names)
   - **Implementation**: Add CommentExtractor module after core DOCX parser

2. **Track changes**: Should revision history be extracted?
   - **Decision**: No for v1 (low PII risk, high complexity)
   - **Defer to**: Future feature if users request

3. **Page numbers in DOCX**: Spec requests approximate page numbers, but DOCX doesn't store them.
   - **Decision**: Implement heuristic (estimate ~500 words/page) or omit page numbers
   - **Implementation**: Add Optional<usize> page field, populate with heuristic

4. **Formulas vs values in XLSX**: Spec requires display values, not formulas.
   - **Decision**: Calamine returns display values by default (correct behavior)
   - **Implementation**: Verify in tests

5. **Macro-enabled files (.docm, .xlsm, .pptxm)**: Treat same as regular files?
   - **Decision**: Yes, ignore macros (VBA code not extracted)
   - **Implementation**: Detect same as regular OOXML

## Success Metrics

- [ ] All acceptance criteria (SC-001 to SC-006) pass
- [ ] Constitution check passes (all principles ✅)
- [ ] Clippy warnings = 0
- [ ] Test coverage >80% (measured with cargo-tllvm-cov)
- [ ] Real-world Office files parse correctly (100% of common cases)
- [ ] Performance targets met (10MB <5s, 100K rows <500MB)

## Next Steps

1. Run `/speckit.tasks` to generate implementation tasks
2. Create veil-office crate scaffold
3. Implement XLSX parser (highest priority)
4. Implement DOCX parser
5. Implement PPTX parser
6. Integration and polish
