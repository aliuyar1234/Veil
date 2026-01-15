# Data Model: Office Document Parser

**Feature**: 006-office-parser
**Version**: 1.0.0
**Last Updated**: 2025-12-15

## Overview

This document defines the data types for parsing Office Open XML documents (DOCX, XLSX, PPTX) and integrating with the veil-parsers interface.

## Design Principles

1. **Parser Interface Compatibility**: Office parsers must output `TextSegment` with `Position` enum compatible with existing parsers
2. **Position Precision**: Every extracted text must include precise location metadata (sheet+cell, slide+element, page+paragraph)
3. **Metadata as Segments**: Document metadata (author, company) is extracted as special TextSegments
4. **Streaming Support**: Data structures must support incremental processing for large Excel files
5. **Error Context**: Errors must include document context (which file part failed)

## Core Types

### FileFormat Extension (in veil-parsers/src/types.rs)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileFormat {
    // ... existing variants ...
    /// Microsoft Word document (.docx)
    Docx,
    /// Microsoft Excel spreadsheet (.xlsx)
    Xlsx,
    /// Microsoft PowerPoint presentation (.pptx)
    Pptx,
}
```

### Position Variants Extension (in veil-parsers/src/types.rs)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Position {
    // ... existing variants (Text, Csv, Json, Html, Pdf) ...

    /// Word document position.
    Docx {
        /// Section type (body, header, footer, footnote, table).
        section: DocxSection,
        /// Paragraph number within section (1-indexed).
        paragraph: usize,
        /// Character offset within paragraph (0-indexed).
        char_offset: usize,
        /// Character length.
        char_length: usize,
        /// Page number (approximate, 1-indexed).
        #[serde(skip_serializing_if = "Option::is_none")]
        page: Option<usize>,
        /// Table row and column if inside a table.
        #[serde(skip_serializing_if = "Option::is_none")]
        table_cell: Option<TableCell>,
    },

    /// Excel spreadsheet position.
    Xlsx {
        /// Sheet name.
        sheet: String,
        /// Row number (1-indexed, as displayed in Excel).
        row: usize,
        /// Column index (0-indexed).
        column: usize,
        /// Column letter (A, B, ..., AA, AB, etc.).
        column_letter: String,
        /// Cell reference (e.g., "Sheet1!B5").
        cell_ref: String,
        /// Whether this cell is from a hidden sheet.
        #[serde(skip_serializing_if = "Option::is_none")]
        hidden_sheet: Option<bool>,
    },

    /// PowerPoint presentation position.
    Pptx {
        /// Slide number (1-indexed).
        slide: usize,
        /// Element type (title, body, note, shape).
        element: PptxElement,
        /// Text index within element (for multiple text runs).
        text_index: usize,
        /// Character offset within text run.
        char_offset: usize,
        /// Character length.
        char_length: usize,
    },

    /// Office document metadata position.
    OfficeMetadata {
        /// Metadata field name (e.g., "author", "company", "title").
        field: String,
        /// Document format this metadata is from.
        format: FileFormat,
    },
}
```

### Supporting Enums

```rust
/// Section type in a Word document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocxSection {
    /// Main document body.
    Body,
    /// Page header.
    Header,
    /// Page footer.
    Footer,
    /// Footnote or endnote.
    Note,
    /// Table cell.
    Table,
    /// Text box or shape.
    TextBox,
    /// Comment.
    Comment,
}

/// Table cell location.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableCell {
    /// Table index in document (0-indexed).
    pub table_index: usize,
    /// Row number in table (1-indexed).
    pub row: usize,
    /// Column number in table (1-indexed).
    pub column: usize,
}

/// Element type in a PowerPoint slide.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PptxElement {
    /// Slide title placeholder.
    Title,
    /// Slide body text.
    Body,
    /// Speaker notes.
    Note,
    /// Text in a shape or text box.
    Shape,
    /// Table cell in slide.
    Table,
}
```

## Office-Specific Types (in veil-office crate)

### Document Containers

```rust
/// A parsed Office Open XML document (internal representation).
#[derive(Debug)]
pub(crate) struct OfficeDocument {
    /// Archive reader for the ZIP container.
    archive: zip::ZipArchive<R>,
    /// Document type detected from content types.
    doc_type: OfficeDocType,
    /// Extracted metadata.
    metadata: OfficeMetadata,
}

/// Office document type detected from ZIP contents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OfficeDocType {
    Docx,
    Xlsx,
    Pptx,
}
```

### Metadata

```rust
/// Office document metadata extracted from docProps/*.xml.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OfficeMetadata {
    /// Document title (dc:title).
    pub title: Option<String>,
    /// Document subject (dc:subject).
    pub subject: Option<String>,
    /// Document creator/author (dc:creator).
    pub creator: Option<String>,
    /// Keywords (cp:keywords).
    pub keywords: Option<String>,
    /// Last modified by (cp:lastModifiedBy).
    pub last_modified_by: Option<String>,
    /// Creation date (dcterms:created).
    pub created: Option<String>,
    /// Modification date (dcterms:modified).
    pub modified: Option<String>,
    /// Company name (extended property).
    pub company: Option<String>,
    /// Manager name (extended property).
    pub manager: Option<String>,
}

impl OfficeMetadata {
    /// Convert metadata to TextSegments for PII detection.
    pub fn to_text_segments(&self, format: FileFormat) -> Vec<TextSegment> {
        let mut segments = Vec::new();

        // Helper to add non-empty fields
        let add = |segs: &mut Vec<TextSegment>, field: &str, value: &Option<String>| {
            if let Some(val) = value {
                if !val.is_empty() {
                    segs.push(TextSegment {
                        content: val.clone(),
                        position: Position::OfficeMetadata {
                            field: field.to_string(),
                            format,
                        },
                    });
                }
            }
        };

        add(&mut segments, "title", &self.title);
        add(&mut segments, "subject", &self.subject);
        add(&mut segments, "creator", &self.creator);
        add(&mut segments, "keywords", &self.keywords);
        add(&mut segments, "last_modified_by", &self.last_modified_by);
        add(&mut segments, "company", &self.company);
        add(&mut segments, "manager", &self.manager);

        segments
    }
}
```

### DOCX-Specific Types

```rust
/// Word document content (internal).
#[derive(Debug)]
pub(crate) struct DocxContent {
    /// Main document body.
    body: DocxBody,
    /// Headers (multiple per section possible).
    headers: Vec<DocxHeader>,
    /// Footers (multiple per section possible).
    footers: Vec<DocxFooter>,
    /// Comments.
    comments: Vec<DocxComment>,
}

/// Document body representation.
#[derive(Debug)]
pub(crate) struct DocxBody {
    /// Paragraphs in document order.
    paragraphs: Vec<DocxParagraph>,
    /// Tables in document order.
    tables: Vec<DocxTable>,
}

/// A paragraph in a Word document.
#[derive(Debug, Clone)]
pub(crate) struct DocxParagraph {
    /// Paragraph index in section (0-indexed).
    pub index: usize,
    /// Full paragraph text (concatenated runs).
    pub text: String,
    /// Character positions of text runs.
    pub runs: Vec<TextRun>,
}

/// A text run (contiguous formatted text).
#[derive(Debug, Clone)]
pub(crate) struct TextRun {
    /// Start character offset in paragraph.
    pub start: usize,
    /// Length in characters.
    pub length: usize,
}

/// A table in a Word document.
#[derive(Debug, Clone)]
pub(crate) struct DocxTable {
    /// Table index in document (0-indexed).
    pub index: usize,
    /// Rows in the table.
    pub rows: Vec<DocxTableRow>,
}

/// A table row.
#[derive(Debug, Clone)]
pub(crate) struct DocxTableRow {
    /// Row index in table (0-indexed).
    pub index: usize,
    /// Cells in the row.
    pub cells: Vec<DocxTableCell>,
}

/// A table cell.
#[derive(Debug, Clone)]
pub(crate) struct DocxTableCell {
    /// Column index in row (0-indexed).
    pub column: usize,
    /// Cell text content.
    pub text: String,
}

/// Header content.
#[derive(Debug, Clone)]
pub(crate) struct DocxHeader {
    /// Header paragraphs.
    pub paragraphs: Vec<DocxParagraph>,
}

/// Footer content.
#[derive(Debug, Clone)]
pub(crate) struct DocxFooter {
    /// Footer paragraphs.
    pub paragraphs: Vec<DocxParagraph>,
}

/// Comment content.
#[derive(Debug, Clone)]
pub(crate) struct DocxComment {
    /// Comment author.
    pub author: String,
    /// Comment text.
    pub text: String,
}
```

### XLSX-Specific Types

```rust
/// Excel spreadsheet content (streaming-friendly).
#[derive(Debug)]
pub(crate) struct XlsxContent {
    /// Iterator over sheets (for streaming).
    sheets: Vec<XlsxSheet>,
}

/// A worksheet in an Excel file.
#[derive(Debug)]
pub(crate) struct XlsxSheet {
    /// Sheet name (as displayed in Excel).
    pub name: String,
    /// Sheet index (0-indexed).
    pub index: usize,
    /// Whether sheet is hidden.
    pub hidden: bool,
    /// Sheet data (can be streamed row-by-row with calamine).
    pub data: SheetData,
}

/// Sheet data representation.
#[derive(Debug)]
pub(crate) enum SheetData {
    /// In-memory (for small sheets).
    InMemory(Vec<Vec<String>>),
    /// Streaming handle (for large sheets).
    Streaming(Box<dyn Iterator<Item = Result<Vec<String>, CalamineError>>>),
}

/// Cell reference utility.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellReference {
    /// Sheet name.
    pub sheet: String,
    /// Row number (1-indexed, as displayed in Excel).
    pub row: usize,
    /// Column index (0-indexed).
    pub column: usize,
}

impl CellReference {
    /// Convert column index to letter (0 -> A, 25 -> Z, 26 -> AA, etc.).
    pub fn column_letter(column: usize) -> String {
        let mut col = column;
        let mut letter = String::new();
        loop {
            letter.insert(0, (b'A' + (col % 26) as u8) as char);
            if col < 26 {
                break;
            }
            col = col / 26 - 1;
        }
        letter
    }

    /// Format as Excel cell reference (e.g., "Sheet1!B5").
    pub fn to_string(&self) -> String {
        format!("{}!{}{}", self.sheet, Self::column_letter(self.column), self.row)
    }
}
```

### PPTX-Specific Types

```rust
/// PowerPoint presentation content.
#[derive(Debug)]
pub(crate) struct PptxContent {
    /// Slides in presentation order.
    slides: Vec<PptxSlide>,
}

/// A slide in a PowerPoint presentation.
#[derive(Debug, Clone)]
pub(crate) struct PptxSlide {
    /// Slide number (1-indexed).
    pub number: usize,
    /// Slide title (if present).
    pub title: Option<String>,
    /// Text elements on the slide.
    pub elements: Vec<PptxTextElement>,
    /// Speaker notes.
    pub notes: Option<String>,
}

/// A text element on a slide.
#[derive(Debug, Clone)]
pub(crate) struct PptxTextElement {
    /// Element type.
    pub element_type: PptxElement,
    /// Element index on slide (0-indexed).
    pub index: usize,
    /// Text content.
    pub text: String,
}
```

## Error Types (in veil-office crate)

```rust
use thiserror::Error;

/// Errors that can occur when parsing Office documents.
#[derive(Debug, Error)]
pub enum OfficeError {
    /// File is not a valid ZIP archive.
    #[error("Not a valid Office document (invalid ZIP archive)")]
    NotZipArchive(#[from] zip::result::ZipError),

    /// File is a valid ZIP but not an Office Open XML document.
    #[error("Not an Office Open XML document (missing [Content_Types].xml)")]
    NotOfficeOpenXml,

    /// Document is encrypted.
    #[error("Document is encrypted and cannot be processed. Please remove encryption and try again.")]
    Encrypted,

    /// Legacy binary format detected.
    #[error("Legacy Office format (.doc/.xls/.ppt) is not supported. Please convert to .docx/.xlsx/.pptx and try again.")]
    LegacyFormat,

    /// Unsupported Office format variant.
    #[error("Unsupported Office format: {0}")]
    UnsupportedFormat(String),

    /// XML parsing error.
    #[error("XML parsing error in {file}: {message}")]
    XmlError {
        file: String,
        message: String,
    },

    /// Corrupted or malformed document.
    #[error("Document appears corrupted: {0}")]
    Corrupted(String),

    /// File too large.
    #[error("File too large ({size} bytes exceeds limit of {max} bytes)")]
    FileTooLarge {
        size: usize,
        max: usize,
    },

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Convert OfficeError to ParseError for veil-parsers interface.
impl From<OfficeError> for ParseError {
    fn from(err: OfficeError) -> Self {
        match err {
            OfficeError::NotZipArchive(_) | OfficeError::NotOfficeOpenXml => {
                ParseError::FormatError {
                    message: err.to_string(),
                }
            }
            OfficeError::Encrypted => {
                ParseError::Encrypted
            }
            OfficeError::LegacyFormat => {
                ParseError::UnsupportedFormat {
                    format: "Legacy Office (binary)".to_string(),
                }
            }
            OfficeError::UnsupportedFormat(fmt) => {
                ParseError::UnsupportedFormat { format: fmt }
            }
            OfficeError::FileTooLarge { size, max } => {
                ParseError::FileTooLarge { size, max }
            }
            OfficeError::Corrupted(msg) | OfficeError::XmlError { message: msg, .. } => {
                ParseError::FormatError { message: msg }
            }
            OfficeError::Io(e) => ParseError::Io(e),
        }
    }
}
```

## ParseError Extensions (in veil-parsers)

```rust
/// Add new error variants to ParseError enum.
#[derive(Debug, Error)]
pub enum ParseError {
    // ... existing variants ...

    /// Document is encrypted.
    #[error("Document is encrypted and cannot be processed")]
    Encrypted,

    /// Format is not supported.
    #[error("Unsupported format: {format}")]
    UnsupportedFormat {
        format: String,
    },
}
```

## Integration with veil-parsers

### Parser Trait (Optional - for consistency)

```rust
/// Internal trait for format-specific parsers.
pub(crate) trait FormatParser {
    /// Parse bytes into TextSegments.
    fn parse_bytes(&self, bytes: &[u8], options: &ParseOptions)
        -> Result<ParseResult, ParseError>;
}

/// Implement for Office parsers.
impl FormatParser for OfficeParser {
    fn parse_bytes(&self, bytes: &[u8], options: &ParseOptions)
        -> Result<ParseResult, ParseError> {
        // Detect specific Office format
        let format = detect_office_format(bytes)?;

        match format {
            FileFormat::Docx => self.parse_docx(bytes, options),
            FileFormat::Xlsx => self.parse_xlsx(bytes, options),
            FileFormat::Pptx => self.parse_pptx(bytes, options),
            _ => Err(ParseError::UnsupportedFormat {
                format: format!("{:?}", format)
            }),
        }
    }
}
```

## Data Flow Example

### XLSX Parsing Flow

```
1. parse_bytes(xlsx_bytes)
   ↓
2. Open ZIP archive
   ↓
3. Extract workbook.xml → Get sheet names
   ↓
4. For each sheet:
   - Extract sheet.xml
   - Stream rows with calamine
   - For each cell:
     * Create CellReference
     * Convert to TextSegment with Position::Xlsx
   ↓
5. Extract docProps/*.xml → OfficeMetadata
   ↓
6. Return ParseResult {
     metadata: DocumentMetadata,
     segments: Vec<TextSegment>,
     warnings: Vec<ParseWarning>,
   }
```

### Memory Footprint (100K row Excel)

- **Streaming mode**: ~10MB (buffer + shared strings table)
- **Non-streaming**: ~100MB+ (entire sheet in memory)
- **Target**: <500MB per constitution (achievable with streaming)

## Example Serialized Output

### XLSX Cell

```json
{
  "content": "john.doe@example.com",
  "position": {
    "type": "xlsx",
    "sheet": "Customers",
    "row": 5,
    "column": 1,
    "column_letter": "B",
    "cell_ref": "Customers!B5"
  }
}
```

### DOCX Paragraph

```json
{
  "content": "This document was prepared by Jane Smith.",
  "position": {
    "type": "docx",
    "section": "body",
    "paragraph": 12,
    "char_offset": 0,
    "char_length": 43,
    "page": 3
  }
}
```

### Office Metadata

```json
{
  "content": "John Q. Manager",
  "position": {
    "type": "office_metadata",
    "field": "last_modified_by",
    "format": "docx"
  }
}
```

## Constitutional Compliance

### Security First
- No `unsafe` blocks required (ZIP and XML libraries are safe)
- All user input validated (ZIP paths, XML content)
- Encrypted files detected and rejected safely

### Stability & Error Handling
- All errors propagate via `Result<T, OfficeError>`
- ZIP extraction errors gracefully handled
- Corrupted XML triggers clear error messages

### Performance
- Streaming for large Excel files (no clone() of 100K rows)
- Zero-copy XML parsing with quick-xml (borrows from buffer)
- Lazy evaluation of sheets (only parse when needed)

### Simplicity
- Types mirror Office structure (slide → elements, sheet → cells)
- No complex state machines or async (straightforward imperative parsing)
- Each format has dedicated parser module

### Dependency Discipline
- `calamine`: 0.24 - justified (Excel is complex, well-maintained crate)
- `zip`: 0.6 - justified (OOXML is ZIP-based)
- `quick-xml`: 0.31 - justified (efficient XML parsing)
- Total: 3 new workspace dependencies (+ their minimal transitive deps)

## Type Checklist

- [x] FileFormat extended (Docx, Xlsx, Pptx)
- [x] Position extended (Docx, Xlsx, Pptx, OfficeMetadata)
- [x] OfficeMetadata struct
- [x] DocxContent structs (Paragraph, Table, etc.)
- [x] XlsxContent structs (Sheet, CellReference)
- [x] PptxContent structs (Slide, TextElement)
- [x] OfficeError enum
- [x] ParseError extensions
- [x] Supporting enums (DocxSection, PptxElement)
- [x] TableCell struct
- [x] CellReference utility

## Next Steps

See `plan.md` for implementation sequence and module organization.
