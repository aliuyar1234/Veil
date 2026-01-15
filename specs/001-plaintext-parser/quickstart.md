# Quickstart: veil-parsers

This guide shows how to use the Veil parsing library to extract text from documents.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
veil-parsers = "0.1"
```

## Basic Usage

### Parse a File

```rust
use veil_parsers::{parse_file, ParseOptions};

fn main() -> Result<(), veil_parsers::ParseError> {
    // Parse with auto-detection
    let result = parse_file("document.txt", &ParseOptions::default())?;

    println!("Format: {:?}", result.metadata.format);
    println!("Encoding: {}", result.metadata.encoding);
    println!("Segments: {}", result.segments.len());

    for segment in &result.segments {
        println!("{}", segment.content);
    }

    Ok(())
}
```

### Parse from Bytes

```rust
use veil_parsers::{parse_bytes, ParseOptions, FileFormat};

let content = b"name,email\nJohn,john@example.com";

let options = ParseOptions {
    format: Some(FileFormat::Csv),
    ..Default::default()
};

let result = parse_bytes(content, &options)?;

for segment in result.segments {
    if let veil_parsers::Position::Csv { row, column, header } = segment.position {
        println!("Row {}, Col {} ({}): {}", row, column, header.unwrap_or_default(), segment.content);
    }
}
```

## Format-Specific Examples

### Plain Text

```rust
use veil_parsers::{parse_file, ParseOptions};

let result = parse_file("logfile.txt", &ParseOptions::default())?;

for segment in result.segments {
    if let veil_parsers::Position::Text { line, column, .. } = segment.position {
        println!("Line {}: {}", line, segment.content);
    }
}
```

### CSV with Custom Delimiter

```rust
use veil_parsers::{parse_file, ParseOptions, FileFormat};

let options = ParseOptions {
    format: Some(FileFormat::Csv),
    csv_delimiter: Some(b';'),       // Semicolon-separated
    csv_has_headers: Some(true),
    ..Default::default()
};

let result = parse_file("data.csv", &options)?;

// Access header names
for segment in result.segments {
    if let veil_parsers::Position::Csv { header: Some(header), .. } = &segment.position {
        println!("{}: {}", header, segment.content);
    }
}
```

### JSON with Path Tracking

```rust
use veil_parsers::{parse_file, ParseOptions};

let result = parse_file("config.json", &ParseOptions::default())?;

for segment in result.segments {
    if let veil_parsers::Position::Json { path } = &segment.position {
        println!("{} = {}", path, segment.content);
    }
}

// Output:
// $.database.host = localhost
// $.database.user = admin
// $.users[0].email = alice@example.com
```

### HTML Text Extraction

```rust
use veil_parsers::{parse_file, ParseOptions};

let result = parse_file("page.html", &ParseOptions::default())?;

// Only visible text is extracted (no script/style content)
let full_text: String = result.segments
    .iter()
    .map(|s| s.content.as_str())
    .collect::<Vec<_>>()
    .join(" ");

println!("Extracted text: {}", full_text);
```

## Handling Large Files

For files larger than 10MB, use streaming:

```rust
use veil_parsers::{parse_reader, ParseOptions};
use std::fs::File;
use std::io::BufReader;

let file = File::open("large_file.csv")?;
let reader = BufReader::new(file);

let options = ParseOptions {
    enable_streaming: Some(true),
    max_size_bytes: Some(100 * 1024 * 1024), // 100MB limit
    ..Default::default()
};

let result = parse_reader(reader, &options)?;
```

## Format Detection

```rust
use veil_parsers::{detect_format, FileFormat};

let content = br#"{"name": "test"}"#;
let format = detect_format(content, Some("data.json"));

assert_eq!(format, FileFormat::Json);

// Works without extension too
let format = detect_format(content, None);
assert_eq!(format, FileFormat::Json);
```

## Error Handling

```rust
use veil_parsers::{parse_file, ParseError, ParseOptions};

match parse_file("document.csv", &ParseOptions::default()) {
    Ok(result) => {
        // Check for warnings
        for warning in &result.warnings {
            eprintln!("Warning: {} - {}", warning.code, warning.message);
        }
        // Process segments...
    }
    Err(ParseError::FileTooLarge { size, max }) => {
        eprintln!("File is {} bytes, max allowed is {}", size, max);
    }
    Err(ParseError::CsvError { row, message }) => {
        eprintln!("CSV error at row {}: {}", row, message);
    }
    Err(e) => {
        eprintln!("Parse error: {}", e);
    }
}
```

## Encoding Handling

The library auto-detects encoding from BOM or content:

```rust
use veil_parsers::{parse_file, ParseOptions};

let result = parse_file("utf16_document.txt", &ParseOptions::default())?;

println!("Detected encoding: {}", result.metadata.encoding);

if result.metadata.encoding_lossy {
    eprintln!("Warning: Some characters could not be converted");
}
```

Override encoding detection:

```rust
let options = ParseOptions {
    encoding: Some("ISO-8859-1".to_string()),
    ..Default::default()
};
```

## Integration with PII Detection

The parsing library is designed to feed the detection engine:

```rust
use veil_parsers::{parse_file, ParseOptions};
// use veil_detection::{scan, ScanOptions};  // Future crate

let parse_result = parse_file("document.csv", &ParseOptions::default())?;

// Each segment can be scanned for PII
// The position metadata allows precise location reporting
for segment in parse_result.segments {
    // let findings = scan(&segment.content, &ScanOptions::default())?;
    // for finding in findings {
    //     println!("Found {} at {:?}", finding.category, segment.position);
    // }
}
```

## Performance Tips

1. **Use streaming for large files**: Set `enable_streaming: Some(true)` for files >10MB
2. **Specify format when known**: Avoids detection overhead
3. **Reuse ParseOptions**: Create once, use for multiple files
4. **Process segments lazily**: Don't collect all segments if streaming through

## Supported Formats

| Format | Extensions | Detection |
|--------|-----------|-----------|
| Text | .txt, .log, .md | Default fallback |
| CSV | .csv, .tsv | Delimiter patterns |
| JSON | .json | Starts with `{` or `[` |
| HTML | .html, .htm | Starts with `<` + DOCTYPE/html tags |

## Limitations

- Maximum file size: 100MB (configurable)
- JSON nesting: 100 levels max
- CSV columns: 10,000 max
- Encoding: UTF-8, UTF-16, ISO-8859-1 supported
