# Quickstart: PDF Parser

**Feature**: 005-pdf-parser

## Overview

The PDF parser extracts text content from PDF documents for PII detection. It preserves
reading order, page structure, and position information for accurate finding locations.

## Basic Usage

### Rust API

```rust
use veil_parsers::{parse_bytes, ParseOptions, Format, Position};

// Parse a PDF file
let pdf_bytes = std::fs::read("document.pdf")?;
let options = ParseOptions {
    format: Some(Format::Pdf),
    ..Default::default()
};

let segments = parse_bytes(&pdf_bytes, options)?;

for segment in segments {
    println!("Text: {}", segment.content);

    if let Position::Pdf { page, x, y, .. } = segment.position {
        println!("  Page {}, position ({}, {})", page, x, y);
    }
}
```

### With PDF-Specific Options

```rust
use veil_parsers::pdf::{PdfDocument, PdfParseOptions};

// Parse with custom options
let options = PdfParseOptions {
    extract_form_fields: true,
    include_positions: true,
    max_pages: Some(100),  // Limit to first 100 pages
    ..Default::default()
};

let doc = PdfDocument::from_bytes_with_options(&pdf_bytes, options)?;

println!("Pages: {}", doc.page_count);
println!("Title: {:?}", doc.title);

// Access individual pages
for page in doc.pages() {
    println!("Page {}: {} text blocks", page.page_num, page.text_blocks.len());

    // Check form fields
    for field in &page.form_fields {
        println!("  Form field '{}': {:?}", field.name, field.value);
    }
}
```

### CLI Usage

```bash
# Scan a PDF for PII
veil scan document.pdf

# Scan with specific output format
veil scan --format json document.pdf

# Scan only first 10 pages
veil scan --max-pages 10 large_document.pdf

# Protect (redact) a PDF
veil protect document.pdf --output redacted.pdf
```

## Handling Edge Cases

### Encrypted PDFs

```rust
use veil_parsers::pdf::{PdfDocument, PdfParseOptions, PdfError};

let options = PdfParseOptions {
    password: Some("secret123".to_string()),
    ..Default::default()
};

match PdfDocument::from_bytes_with_options(&pdf_bytes, options) {
    Ok(doc) => println!("Parsed {} pages", doc.page_count),
    Err(PdfError::Encrypted) => println!("Wrong password or no password provided"),
    Err(e) => println!("Error: {}", e),
}
```

### Scanned PDFs

```rust
use veil_parsers::pdf::PdfDocument;

let doc = PdfDocument::from_bytes(&pdf_bytes)?;

if doc.is_scanned() {
    println!("Warning: Document appears to be scanned.");
    println!("Text extraction may be incomplete.");
    println!("Consider using OCR for better results.");
}

// Check individual pages
for page in doc.pages() {
    if page.is_scanned {
        println!("Page {} appears to be scanned", page.page_num);
    }
}
```

### Large Documents

```rust
use veil_parsers::pdf::{PdfDocument, PdfParseOptions};

// Process large PDFs efficiently
let options = PdfParseOptions {
    max_pages: Some(1000),
    skip_scanned_pages: true,  // Skip pages without text
    ..Default::default()
};

let doc = PdfDocument::from_bytes_with_options(&pdf_bytes, options)?;

// Process pages one at a time to manage memory
for page in doc.pages() {
    let segments = page.to_segments(&mut 0);
    // Process segments...
    // Memory is released after each page
}
```

## Position Information

PDF positions use PDF coordinate system (origin at bottom-left):

```rust
if let Position::Pdf { page, x, y, width, height, byte_offset, byte_length } = segment.position {
    println!("Page: {}", page);
    println!("Position: ({}, {}) - {}x{} points", x, y, width, height);
    println!("Byte range: {}..{}", byte_offset, byte_offset + byte_length);
}
```

### Converting to Visual Coordinates

```rust
// PDF uses bottom-left origin, convert to top-left for display
fn to_visual_y(pdf_y: f32, pdf_height: f32, page_height: f32) -> f32 {
    page_height - pdf_y - pdf_height
}
```

## Form Fields

```rust
use veil_parsers::pdf::{PdfDocument, PdfFieldType};

let doc = PdfDocument::from_bytes(&pdf_bytes)?;

for page in doc.pages() {
    for field in &page.form_fields {
        match field.field_type {
            PdfFieldType::Text => {
                println!("Text field '{}': {:?}", field.name, field.value);
            }
            PdfFieldType::Checkbox => {
                let checked = field.value.as_deref() == Some("Yes");
                println!("Checkbox '{}': {}", field.name, checked);
            }
            PdfFieldType::Dropdown => {
                println!("Dropdown '{}': {:?}", field.name, field.value);
            }
            _ => {}
        }
    }
}
```

## Integration with Detection

```rust
use veil_parsers::{parse_bytes, ParseOptions, Format};
use veil_detect::{detect_pii, DetectorRegistry};

// Parse PDF
let segments = parse_bytes(&pdf_bytes, ParseOptions {
    format: Some(Format::Pdf),
    ..Default::default()
})?;

// Detect PII
let registry = DetectorRegistry::default();
let findings = detect_pii(&segments, &registry);

for finding in findings {
    println!("Found {}: '{}' (confidence: {:.2})",
        finding.category,
        finding.matched_text,
        finding.confidence
    );

    // Position includes page number for PDFs
    if let Position::Pdf { page, .. } = &finding.position {
        println!("  on page {}", page);
    }
}
```

## Error Handling

```rust
use veil_parsers::pdf::{PdfDocument, PdfError};

match PdfDocument::from_bytes(&pdf_bytes) {
    Ok(doc) => {
        // Success
    }
    Err(PdfError::Encrypted) => {
        eprintln!("Error: PDF is password-protected");
        eprintln!("Use --password option to provide password");
    }
    Err(PdfError::Corrupted(msg)) => {
        eprintln!("Error: PDF file is corrupted - {}", msg);
    }
    Err(PdfError::NoTextContent) => {
        eprintln!("Warning: No extractable text found");
        eprintln!("Document may be scanned - OCR required");
    }
    Err(e) => {
        eprintln!("Error parsing PDF: {}", e);
    }
}
```

## Performance Tips

1. **Limit pages**: Use `max_pages` for large documents during scanning
2. **Skip scanned pages**: Use `skip_scanned_pages` to avoid processing image-only pages
3. **Stream processing**: Process pages sequentially rather than loading all at once
4. **Disable positions**: Set `include_positions: false` if only text content needed

```rust
// Optimized for scanning large documents
let options = PdfParseOptions {
    extract_form_fields: true,
    include_positions: false,  // Faster if positions not needed
    max_pages: Some(500),
    skip_scanned_pages: true,
    ..Default::default()
};
```
