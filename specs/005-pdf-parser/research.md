# Research: PDF Parser

**Feature**: 005-pdf-parser
**Date**: 2025-12-15

## 1. PDF Parsing Library Selection

**Decision**: Use `pdf-extract` crate built on `pdf` crate for pure-Rust PDF text extraction

**Rationale**:
- Pure Rust implementation - no external C dependencies
- Focuses specifically on text extraction (our use case)
- Handles text positioning and reading order
- Active maintenance, reasonable download count
- Cross-platform including WASM compatibility potential

**Alternatives Considered**:
- `pdfium-render`: Higher quality extraction but requires PDFium binary (~20MB), complicates deployment
- `mupdf-rs`: Excellent quality but requires MuPDF C library, GPL license concerns
- `lopdf`: Lower-level, would require significant work to extract text properly
- `poppler`: Requires system library, not cross-platform

**Configuration**:
```toml
[dependencies]
pdf-extract = "0.7"
pdf = "0.9"
```

## 2. Text Extraction Strategy

**Decision**: Extract text by iterating pages, then text objects, preserving position metadata

**Rationale**:
- Page-by-page extraction enables streaming for large documents
- Position information (bounding boxes) needed for redaction mapping
- Reading order can be inferred from text object positions

**Implementation Approach**:
1. Parse PDF structure using `pdf` crate
2. For each page, extract text objects with positions
3. Sort text objects by reading order (top-to-bottom, left-to-right)
4. Group into logical text blocks based on proximity
5. Output as TextSegment with Position::Pdf metadata

## 3. Position Tracking

**Decision**: Track page number, bounding box (x, y, width, height), and byte offset

**Rationale**:
- Page number essential for multi-page documents
- Bounding box enables visual location and future redaction
- Byte offset maintains compatibility with existing Position enum

**Position Metadata Structure**:
```rust
Position::Pdf {
    page: usize,           // 1-indexed page number
    x: f32,                // Left edge in PDF points
    y: f32,                // Bottom edge in PDF points (PDF coordinate system)
    width: f32,            // Text block width
    height: f32,           // Text block height
    byte_offset: usize,    // Cumulative offset for Finding compatibility
}
```

## 4. Reading Order Algorithm

**Decision**: Use Y-coordinate clustering with X-coordinate sorting within clusters

**Rationale**:
- Simple algorithm handles 90% of standard layouts
- Column detection via X-coordinate gaps
- Avoids complex machine learning approaches

**Algorithm**:
1. Cluster text objects by Y-coordinate (within threshold)
2. Within each Y-cluster, sort by X-coordinate
3. Detect column breaks by large X gaps
4. For multi-column, process columns top-to-bottom before moving right

## 5. Form Field Extraction

**Decision**: Use `pdf` crate's AcroForm API to extract interactive form fields

**Rationale**:
- Standard PDF forms use AcroForm structure
- Field names provide context for values
- Checkbox/radio values indicate selections

**Supported Field Types**:
- Text fields → extract value as string
- Checkboxes → extract checked state as "Yes"/"No"
- Radio buttons → extract selected option value
- Dropdowns/Comboboxes → extract selected value

## 6. Error Handling

**Decision**: Return ParseError variants for different failure modes

**Rationale**:
- Users need to understand why parsing failed
- Different errors require different responses
- Encrypted PDFs should suggest password option (future)

**Error Categories**:
- `EncryptedPdf`: Password required
- `CorruptedPdf`: File structure invalid
- `NoTextContent`: Scanned/image-only PDF
- `UnsupportedVersion`: PDF version not supported

## 7. Memory Management for Large PDFs

**Decision**: Process pages sequentially, not loading entire document into memory

**Rationale**:
- 1000-page PDFs could exceed memory limits if fully loaded
- Sequential processing matches constitution's <500MB target
- Enables progress reporting per page

**Strategy**:
- Stream page content on demand
- Release page resources after extraction
- Use iterators rather than collecting all text upfront

## 8. Scanned PDF Detection

**Decision**: Detect based on text content ratio and image presence

**Rationale**:
- Scanned PDFs have images but minimal/no text objects
- Users need clear feedback that OCR is required
- Mixed documents should process text pages, flag image pages

**Detection Heuristics**:
- Page with <10 characters but has images → likely scanned
- Document with >50% scanned pages → flag as "mostly scanned"
- Zero text across all pages → "No extractable text"

## 9. Unicode and Encoding

**Decision**: Use PDF crate's built-in encoding handling, normalize output to UTF-8

**Rationale**:
- PDFs can contain various encodings (WinAnsi, MacRoman, etc.)
- Unicode normalization ensures consistent detection
- Ligatures (fi, fl) should be expanded where possible

**Handling**:
- Rely on `pdf` crate's text decoding
- Apply Unicode NFC normalization to output
- Log warnings for undecodable characters

## 10. Integration with veil-parsers

**Decision**: Implement as new module in veil-parsers crate

**Rationale**:
- Consistent with existing parser architecture
- Shares Position enum and TextSegment types
- Enables format detection via magic bytes

**Integration Points**:
- Add `parse_pdf()` function to veil-parsers
- Extend `detect_format()` for PDF magic bytes (`%PDF-`)
- Add `Format::Pdf` variant if not present

## Summary of Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| pdf-extract | 0.7 | High-level text extraction |
| pdf | 0.9 | Low-level PDF parsing |

## Performance Targets (from Constitution)

- 100 pages in <5 seconds (constitution) / <10 seconds (spec)
- <500 MB memory for 1000-page document
- Process 100MB file without memory exhaustion

## Open Questions Resolved

1. **Q: How to handle PDFs with embedded fonts that don't map to Unicode?**
   A: Log warning, use replacement character, continue extraction

2. **Q: How to handle rotated pages?**
   A: Transform coordinates to normalized orientation, note rotation in metadata

3. **Q: How to handle PDF/A vs standard PDF?**
   A: Both use same text extraction; PDF/A may have better text mapping

4. **Q: What about PDF 2.0 features?**
   A: Basic text extraction compatible; advanced features out of scope for v1
