# Quickstart: Office Document Parser

**Feature**: 006-office-parser
**Last Updated**: 2025-12-15

## Overview

This guide demonstrates how to use the veil-office parser to extract text from Microsoft Office documents (DOCX, XLSX, PPTX) for PII detection.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
veil-parsers = "0.1"
```

The office parser is integrated into the veil-parsers crate and will be automatically available.

## Basic Usage

### Parse Any Office Document

```rust
use veil_parsers::{parse_file, ParseOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse with automatic format detection
    let result = parse_file("document.docx", &ParseOptions::default())?;

    // Print all extracted text segments
    for segment in &result.segments {
        println!("{}", segment.content);
    }

    // Print metadata
    println!("Format: {:?}", result.metadata.format);
    println!("Total characters: {}", result.total_chars);
    println!("Parsed in {} ms", result.duration_ms);

    Ok(())
}
```

### Parse from Memory

```rust
use veil_parsers::{parse_bytes, ParseOptions, FileFormat};

fn parse_uploaded_file(bytes: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
    let options = ParseOptions {
        format: Some(FileFormat::Xlsx), // Optional: specify format
        ..Default::default()
    };

    let result = parse_bytes(&bytes, &options)?;

    println!("Extracted {} text segments", result.segments.len());

    Ok(())
}
```

## Excel (XLSX) Examples

### Extract All Cells with References

```rust
use veil_parsers::{parse_file, ParseOptions, Position};

fn extract_excel_cells() -> Result<(), Box<dyn std::error::Error>> {
    let result = parse_file("customers.xlsx", &ParseOptions::default())?;

    for segment in &result.segments {
        if let Position::Xlsx { sheet, row, column_letter, cell_ref, .. } = &segment.position {
            println!("{}: {}", cell_ref, segment.content);
            // Example output: "Customers!B5: john.doe@example.com"
        }
    }

    Ok(())
}
```

### Find PII in Specific Cells

```rust
use veil_parsers::{parse_file, ParseOptions, Position};

fn check_email_column() -> Result<(), Box<dyn std::error::Error>> {
    let result = parse_file("employees.xlsx", &ParseOptions::default())?;

    // Find all cells in column B (email column)
    for segment in &result.segments {
        if let Position::Xlsx { column, cell_ref, .. } = &segment.position {
            if *column == 1 {  // Column B (0-indexed)
                if segment.content.contains('@') {
                    println!("Email found at {}: {}", cell_ref, segment.content);
                }
            }
        }
    }

    Ok(())
}
```

### Process Large Excel Files (Streaming)

```rust
use veil_parsers::{parse_file, ParseOptions};

fn process_large_excel() -> Result<(), Box<dyn std::error::Error>> {
    let options = ParseOptions {
        max_size_bytes: Some(100 * 1024 * 1024), // 100MB limit
        enable_streaming: Some(true),  // Enable streaming for large files
        ..Default::default()
    };

    let result = parse_file("large_dataset.xlsx", &options)?;

    println!("Processed {} cells", result.segments.len());
    // Memory usage stays low even with 100K+ rows

    Ok(())
}
```

## Word (DOCX) Examples

### Extract All Paragraphs

```rust
use veil_parsers::{parse_file, ParseOptions, Position};

fn extract_docx_paragraphs() -> Result<(), Box<dyn std::error::Error>> {
    let result = parse_file("report.docx", &ParseOptions::default())?;

    for segment in &result.segments {
        if let Position::Docx { section, paragraph, .. } = &segment.position {
            println!("Section {:?}, Para {}: {}", section, paragraph, segment.content);
        }
    }

    Ok(())
}
```

### Extract Only Body Text (Exclude Headers/Footers)

```rust
use veil_parsers::{parse_file, ParseOptions, Position, DocxSection};

fn extract_body_only() -> Result<(), Box<dyn std::error::Error>> {
    let result = parse_file("contract.docx", &ParseOptions::default())?;

    for segment in &result.segments {
        if let Position::Docx { section: DocxSection::Body, .. } = &segment.position {
            println!("{}", segment.content);
        }
    }

    Ok(())
}
```

### Extract Tables

```rust
use veil_parsers::{parse_file, ParseOptions, Position};

fn extract_tables() -> Result<(), Box<dyn std::error::Error>> {
    let result = parse_file("data.docx", &ParseOptions::default())?;

    for segment in &result.segments {
        if let Position::Docx { section, table_cell, .. } = &segment.position {
            if let Some(cell) = table_cell {
                println!(
                    "Table {}, Row {}, Col {}: {}",
                    cell.table_index, cell.row, cell.column, segment.content
                );
            }
        }
    }

    Ok(())
}
```

## PowerPoint (PPTX) Examples

### Extract Slide Text

```rust
use veil_parsers::{parse_file, ParseOptions, Position};

fn extract_slides() -> Result<(), Box<dyn std::error::Error>> {
    let result = parse_file("presentation.pptx", &ParseOptions::default())?;

    for segment in &result.segments {
        if let Position::Pptx { slide, element, .. } = &segment.position {
            println!("Slide {}, {:?}: {}", slide, element, segment.content);
        }
    }

    Ok(())
}
```

### Extract Speaker Notes Only

```rust
use veil_parsers::{parse_file, ParseOptions, Position, PptxElement};

fn extract_speaker_notes() -> Result<(), Box<dyn std::error::Error>> {
    let result = parse_file("training.pptx", &ParseOptions::default())?;

    for segment in &result.segments {
        if let Position::Pptx { slide, element: PptxElement::Note, .. } = &segment.position {
            println!("Slide {} notes: {}", slide, segment.content);
        }
    }

    Ok(())
}
```

## Metadata Examples

### Extract Document Author and Company

```rust
use veil_parsers::{parse_file, ParseOptions, Position};

fn extract_metadata() -> Result<(), Box<dyn std::error::Error>> {
    let result = parse_file("proposal.docx", &ParseOptions::default())?;

    for segment in &result.segments {
        if let Position::OfficeMetadata { field, format } = &segment.position {
            println!("{} ({}): {}", field, format, segment.content);
            // Example output:
            // creator (docx): John Smith
            // company (docx): Acme Corp
            // last_modified_by (docx): Jane Doe
        }
    }

    Ok(())
}
```

### Check for PII in Metadata

```rust
use veil_parsers::{parse_file, ParseOptions, Position};

fn check_metadata_pii() -> Result<(), Box<dyn std::error::Error>> {
    let result = parse_file("document.xlsx", &ParseOptions::default())?;

    let mut metadata_segments = Vec::new();

    for segment in &result.segments {
        if matches!(segment.position, Position::OfficeMetadata { .. }) {
            metadata_segments.push(segment);
        }
    }

    if !metadata_segments.is_empty() {
        println!("Document contains metadata that may include PII:");
        for seg in metadata_segments {
            println!("  {}", seg.content);
        }
    }

    Ok(())
}
```

## Error Handling

### Handle Encrypted Documents

```rust
use veil_parsers::{parse_file, ParseOptions, ParseError};

fn handle_encrypted() -> Result<(), Box<dyn std::error::Error>> {
    match parse_file("encrypted.docx", &ParseOptions::default()) {
        Ok(result) => {
            println!("Parsed successfully");
        }
        Err(ParseError::Encrypted) => {
            eprintln!("Error: Document is encrypted. Please remove encryption and try again.");
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
        }
    }

    Ok(())
}
```

### Handle Legacy Formats

```rust
use veil_parsers::{parse_file, ParseOptions, ParseError};

fn handle_legacy_format() -> Result<(), Box<dyn std::error::Error>> {
    match parse_file("old_document.doc", &ParseOptions::default()) {
        Ok(result) => {
            println!("Parsed successfully");
        }
        Err(ParseError::UnsupportedFormat { format }) => {
            eprintln!("Error: {} is not supported.", format);
            eprintln!("Please convert to .docx/.xlsx/.pptx format.");
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
        }
    }

    Ok(())
}
```

### Handle Corrupted Files

```rust
use veil_parsers::{parse_file, ParseOptions, ParseError};

fn handle_corrupted() -> Result<(), Box<dyn std::error::Error>> {
    match parse_file("possibly_corrupted.xlsx", &ParseOptions::default()) {
        Ok(result) => {
            // Check for warnings
            if !result.warnings.is_empty() {
                println!("Warnings during parsing:");
                for warning in &result.warnings {
                    println!("  {}", warning.message);
                }
            }
            println!("Extracted {} segments", result.segments.len());
        }
        Err(ParseError::FormatError { message }) => {
            eprintln!("File appears corrupted: {}", message);
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
        }
    }

    Ok(())
}
```

## Advanced Usage

### Combine with PII Detection

```rust
use veil_parsers::{parse_file, ParseOptions};
use veil_detect::{Detector, DetectorOptions};

fn scan_for_pii() -> Result<(), Box<dyn std::error::Error>> {
    // Parse Office document
    let parse_result = parse_file("sensitive.xlsx", &ParseOptions::default())?;

    // Create PII detector
    let detector = Detector::new(&DetectorOptions::default())?;

    // Scan each segment
    for segment in &parse_result.segments {
        let findings = detector.detect(&segment.content)?;

        if !findings.is_empty() {
            // Print location information
            match &segment.position {
                Position::Xlsx { cell_ref, .. } => {
                    println!("PII found at {}", cell_ref);
                }
                Position::Docx { section, paragraph, .. } => {
                    println!("PII found in {:?}, paragraph {}", section, paragraph);
                }
                Position::Pptx { slide, element, .. } => {
                    println!("PII found on slide {}, {:?}", slide, element);
                }
                _ => {}
            }

            // Print findings
            for finding in findings {
                println!("  {}: {}", finding.pii_type, finding.text);
            }
        }
    }

    Ok(())
}
```

### Batch Processing Multiple Files

```rust
use veil_parsers::{parse_file, ParseOptions};
use std::fs;
use std::path::Path;

fn batch_process_directory(dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let options = ParseOptions::default();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        // Check file extension
        if let Some(ext) = path.extension() {
            let ext = ext.to_str().unwrap_or("");
            if matches!(ext, "docx" | "xlsx" | "pptx") {
                println!("Processing: {:?}", path);

                match parse_file(&path, &options) {
                    Ok(result) => {
                        println!("  Extracted {} segments", result.segments.len());
                    }
                    Err(e) => {
                        eprintln!("  Error: {}", e);
                    }
                }
            }
        }
    }

    Ok(())
}
```

### Custom Format Detection

```rust
use veil_parsers::{detect_format, FileFormat};
use std::fs;

fn detect_office_format(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Read first 1KB for detection
    let bytes = fs::read(path)?;
    let sample = &bytes[..bytes.len().min(1024)];

    let format = detect_format(sample, Some(path));

    match format {
        FileFormat::Docx => println!("Detected: Word document"),
        FileFormat::Xlsx => println!("Detected: Excel spreadsheet"),
        FileFormat::Pptx => println!("Detected: PowerPoint presentation"),
        _ => println!("Detected: {:?}", format),
    }

    Ok(())
}
```

## Testing with Sample Data

### Create Test Files

```rust
#[cfg(test)]
mod tests {
    use veil_parsers::{parse_bytes, ParseOptions, FileFormat, Position};
    use std::fs;

    #[test]
    fn test_xlsx_parsing() {
        // Load test file
        let bytes = fs::read("tests/fixtures/sample.xlsx").unwrap();
        let options = ParseOptions {
            format: Some(FileFormat::Xlsx),
            ..Default::default()
        };

        let result = parse_bytes(&bytes, &options).unwrap();

        // Verify segments extracted
        assert!(!result.segments.is_empty());

        // Verify cell references
        let has_xlsx_position = result.segments.iter().any(|seg| {
            matches!(seg.position, Position::Xlsx { .. })
        });
        assert!(has_xlsx_position);
    }

    #[test]
    fn test_docx_table_extraction() {
        let bytes = fs::read("tests/fixtures/table.docx").unwrap();
        let result = parse_bytes(&bytes, &ParseOptions::default()).unwrap();

        // Find table cells
        let table_cells: Vec<_> = result
            .segments
            .iter()
            .filter(|seg| {
                matches!(seg.position, Position::Docx { table_cell: Some(_), .. })
            })
            .collect();

        assert!(!table_cells.is_empty(), "Should extract table cells");
    }
}
```

## Performance Tips

1. **Large Excel Files**: Enable streaming for files >10MB
   ```rust
   let options = ParseOptions {
       enable_streaming: Some(true),
       ..Default::default()
   };
   ```

2. **File Size Limits**: Adjust max size based on available memory
   ```rust
   let options = ParseOptions {
       max_size_bytes: Some(50 * 1024 * 1024), // 50MB
       ..Default::default()
   };
   ```

3. **Batch Processing**: Process files in parallel with rayon
   ```rust
   use rayon::prelude::*;

   files.par_iter().for_each(|path| {
       let _ = parse_file(path, &ParseOptions::default());
   });
   ```

## Common Patterns

### Filter by Position Type

```rust
use veil_parsers::{parse_file, ParseOptions, Position};

fn filter_positions() -> Result<(), Box<dyn std::error::Error>> {
    let result = parse_file("document.docx", &ParseOptions::default())?;

    // Get only body paragraphs
    let body_text: Vec<_> = result
        .segments
        .iter()
        .filter(|seg| {
            matches!(seg.position, Position::Docx { section: DocxSection::Body, .. })
        })
        .map(|seg| &seg.content)
        .collect();

    println!("Body paragraphs: {}", body_text.len());

    Ok(())
}
```

### Export to JSON

```rust
use veil_parsers::{parse_file, ParseOptions};
use serde_json;

fn export_to_json() -> Result<(), Box<dyn std::error::Error>> {
    let result = parse_file("data.xlsx", &ParseOptions::default())?;

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&result.segments)?;
    std::fs::write("output.json", json)?;

    println!("Exported to output.json");

    Ok(())
}
```

## Next Steps

- See [data-model.md](./data-model.md) for detailed type information
- See [plan.md](./plan.md) for implementation architecture
- See [spec.md](./spec.md) for complete requirements

## Troubleshooting

### "Document is encrypted"
Remove password protection in Office before scanning.

### "Legacy Office format not supported"
Convert .doc/.xls/.ppt to .docx/.xlsx/.pptx using Microsoft Office or LibreOffice.

### "File too large"
Increase `max_size_bytes` in ParseOptions or enable streaming for Excel files.

### "XML parsing error"
File may be corrupted. Try opening and re-saving in Microsoft Office.
