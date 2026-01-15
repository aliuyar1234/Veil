# Email Parser Quickstart Guide

**Feature**: 007-email-parser
**Date**: 2025-12-15

## Overview

This guide demonstrates how to use the veil-email crate to parse EML and MSG email files for PII detection. The parser extracts headers, body text, and attachment metadata, outputting TextSegments compatible with the veil-parsers interface.

## Installation

Add to `Cargo.toml`:

```toml
[dependencies]
veil-parsers = { path = "../veil-parsers" }
veil-email = { path = "../veil-email" }
```

## Basic Usage

### Parse an EML File

```rust
use veil_parsers::{parse_file, ParseOptions, FileFormat};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse email file
    let options = ParseOptions {
        format: Some(FileFormat::Eml),
        ..Default::default()
    };

    let result = parse_file("sample.eml", &options)?;

    // Print metadata
    println!("Format: {:?}", result.metadata.format);
    println!("Encoding: {}", result.metadata.encoding);
    println!("Total chars: {}", result.total_chars);

    // Print text segments
    for segment in result.segments {
        println!("Content: {}", segment.content);
        println!("Position: {:?}", segment.position);
        println!("---");
    }

    Ok(())
}
```

**Output**:
```
Format: Eml
Encoding: utf-8
Total chars: 1247
Content: john.doe@example.com
Position: Email { field: "From", field_index: None, part_index: None, byte_offset: 0, byte_length: 20 }
---
Content: Quarterly Report
Position: Email { field: "Subject", field_index: None, part_index: None, byte_offset: 0, byte_length: 16 }
---
Content: Please find the Q4 report attached.
Position: Email { field: "body", field_index: None, part_index: Some(0), byte_offset: 0, byte_length: 38 }
---
```

### Parse a MSG File

```rust
use veil_parsers::{parse_file, ParseOptions, FileFormat};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = ParseOptions {
        format: Some(FileFormat::Msg),
        ..Default::default()
    };

    let result = parse_file("outlook_export.msg", &options)?;

    println!("Parsed {} segments from MSG file", result.segments.len());

    Ok(())
}
```

### Extract Email Headers

```rust
use veil_email::{parse_email, EmailParseOptions};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read email file
    let bytes = fs::read("email.eml")?;

    // Parse with default options
    let options = EmailParseOptions::default();
    let email = parse_email(&bytes, &options)?;

    // Print headers
    for header in &email.headers {
        println!("{}: {:?}", header.name, header.value);
    }

    Ok(())
}
```

**Output**:
```
From: Address(EmailAddress { display_name: Some("John Doe"), address: "john@example.com" })
To: AddressList([EmailAddress { display_name: None, address: "jane@example.com" }])
Subject: Text("Meeting Tomorrow")
Date: DateTime("2025-12-15T10:30:00Z")
```

### Extract Email Addresses

```rust
use veil_email::{parse_email, EmailParseOptions, EmailHeaderValue};

fn extract_all_addresses(email_bytes: &[u8]) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let email = parse_email(email_bytes, &EmailParseOptions::default())?;
    let mut addresses = Vec::new();

    for header in &email.headers {
        match &header.value {
            EmailHeaderValue::Address(addr) => {
                addresses.push(addr.address.clone());
            }
            EmailHeaderValue::AddressList(list) => {
                for addr in list {
                    addresses.push(addr.address.clone());
                }
            }
            _ => {}
        }
    }

    Ok(addresses)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = std::fs::read("email.eml")?;
    let addresses = extract_all_addresses(&bytes)?;

    println!("Found {} email addresses:", addresses.len());
    for addr in addresses {
        println!("  - {}", addr);
    }

    Ok(())
}
```

### Extract Email Body

```rust
use veil_email::{parse_email, EmailParseOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = std::fs::read("email.eml")?;
    let email = parse_email(&bytes, &EmailParseOptions::default())?;

    println!("Email has {} body part(s)", email.body_parts.len());

    for (idx, part) in email.body_parts.iter().enumerate() {
        println!("\nPart {}:", idx);
        println!("  Content-Type: {}", part.content_type);
        println!("  Charset: {}", part.charset);
        println!("  Is Quoted: {}", part.is_quoted);
        println!("  Content length: {} chars", part.content.len());
        println!("  Preview: {}...", part.content.chars().take(50).collect::<String>());
    }

    Ok(())
}
```

**Output**:
```
Email has 2 body part(s)

Part 0:
  Content-Type: text/plain
  Charset: utf-8
  Is Quoted: false
  Content length: 245 chars
  Preview: Hi Jane,\n\nI wanted to follow up on our discus...

Part 1:
  Content-Type: text/html
  Charset: utf-8
  Is Quoted: false
  Content length: 512 chars
  Preview: Hi Jane,\n\nI wanted to follow up on our discus...
```

### List Attachments

```rust
use veil_email::{parse_email, EmailParseOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = std::fs::read("email_with_attachments.eml")?;
    let email = parse_email(&bytes, &EmailParseOptions::default())?;

    println!("Attachments: {}", email.attachments.len());

    for (idx, attachment) in email.attachments.iter().enumerate() {
        println!("\nAttachment {}:", idx + 1);
        println!("  Filename: {}", attachment.filename.as_deref().unwrap_or("<unnamed>"));
        println!("  Type: {}", attachment.content_type);
        println!("  Size: {} bytes", attachment.size_bytes);
        println!("  Inline: {}", attachment.inline);
    }

    Ok(())
}
```

**Output**:
```
Attachments: 3

Attachment 1:
  Filename: Q4_Report.pdf
  Type: application/pdf
  Size: 245678 bytes
  Inline: false

Attachment 2:
  Filename: logo.png
  Type: image/png
  Size: 12345 bytes
  Inline: true

Attachment 3:
  Filename: data.xlsx
  Type: application/vnd.openxmlformats-officedocument.spreadsheetml.sheet
  Size: 89012 bytes
  Inline: false
```

## Advanced Usage

### Detect Quoted Content

```rust
use veil_email::{parse_email, EmailParseOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = EmailParseOptions {
        detect_quotes: true,
        ..Default::default()
    };

    let bytes = std::fs::read("email_thread.eml")?;
    let email = parse_email(&bytes, &options)?;

    for part in &email.body_parts {
        if part.is_quoted {
            println!("QUOTED: {}", part.content.lines().take(2).collect::<Vec<_>>().join("\n"));
        } else {
            println!("ORIGINAL: {}", part.content.lines().take(2).collect::<Vec<_>>().join("\n"));
        }
    }

    Ok(())
}
```

**Output**:
```
ORIGINAL: Thanks for the update!
ORIGINAL:
QUOTED: > On Mon, Dec 15, 2025, John Doe wrote:
QUOTED: > Here is the latest status report.
```

### Custom Parsing Options

```rust
use veil_email::{parse_email, EmailParseOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = EmailParseOptions {
        convert_html: true,           // Convert HTML to plain text
        detect_quotes: true,           // Detect quoted content
        extract_attachments: true,     // List attachments
        max_attachment_size: Some(10 * 1024 * 1024), // Skip attachments >10MB
        prefer_plain_text: true,       // Prefer text/plain over text/html
    };

    let bytes = std::fs::read("email.eml")?;
    let email = parse_email(&bytes, &options)?;

    println!("Parsed successfully with custom options");

    Ok(())
}
```

### Integration with veil-parsers

```rust
use veil_parsers::{parse_file, ParseOptions, FileFormat};

fn scan_email_for_pii(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Use veil-parsers API (automatically uses veil-email for .eml/.msg)
    let options = ParseOptions {
        format: Some(FileFormat::Eml),
        ..Default::default()
    };

    let result = parse_file(path, &options)?;

    // Each segment represents a PII detection target
    for segment in result.segments {
        // Run PII detection on segment.content
        // Position metadata tells you where PII was found (header/body/attachment)
        println!("Scanning: {}", segment.content);
    }

    Ok(())
}
```

### Error Handling

```rust
use veil_email::{parse_email, EmailParseOptions, EmailParseError};

fn main() {
    let bytes = std::fs::read("email.eml").unwrap();

    match parse_email(&bytes, &EmailParseOptions::default()) {
        Ok(email) => {
            println!("Parsed successfully: {} headers", email.headers.len());
        }
        Err(EmailParseError::InvalidFormat(msg)) => {
            eprintln!("Invalid email format: {}", msg);
        }
        Err(EmailParseError::MimeError(msg)) => {
            eprintln!("MIME parsing failed: {}", msg);
        }
        Err(EmailParseError::EncodingError(msg)) => {
            eprintln!("Encoding error: {}", msg);
        }
        Err(e) => {
            eprintln!("Other error: {}", e);
        }
    }
}
```

### Parse from Bytes (No File I/O)

```rust
use veil_email::{parse_email, EmailParseOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Email content as bytes (e.g., from network, database, etc.)
    let email_bytes = b"From: sender@example.com\r
To: recipient@example.com\r
Subject: Test\r
\r
Hello, world!\r
";

    let email = parse_email(email_bytes, &EmailParseOptions::default())?;

    println!("Parsed email with {} headers", email.headers.len());

    Ok(())
}
```

## CLI Usage

Once integrated into veil-cli, you can use:

```bash
# Scan EML file
veil scan email.eml

# Scan MSG file
veil scan outlook_export.msg

# Scan with specific format
veil scan --format eml suspicious.dat

# Scan directory of emails
veil scan emails/
```

## Testing Examples

### Unit Test: Parse Simple Email

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_email() {
        let email_bytes = b"From: john@example.com\r
To: jane@example.com\r
Subject: Test\r
\r
Hello, Jane!\r
";

        let email = parse_email(email_bytes, &EmailParseOptions::default()).unwrap();

        assert_eq!(email.headers.len(), 3);
        assert_eq!(email.body_parts.len(), 1);
        assert_eq!(email.body_parts[0].content.trim(), "Hello, Jane!");
    }

    #[test]
    fn test_extract_from_address() {
        let email_bytes = b"From: John Doe <john@example.com>\r
Subject: Test\r
\r
Body\r
";

        let email = parse_email(email_bytes, &EmailParseOptions::default()).unwrap();

        let from_header = email.headers.iter().find(|h| h.name == "From").unwrap();

        if let EmailHeaderValue::Address(addr) = &from_header.value {
            assert_eq!(addr.display_name, Some("John Doe".to_string()));
            assert_eq!(addr.address, "john@example.com");
        } else {
            panic!("Expected Address value");
        }
    }
}
```

### Integration Test: Real Email File

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_parse_real_gmail_export() {
        let bytes = fs::read("tests/fixtures/gmail_export.eml").unwrap();
        let email = parse_email(&bytes, &EmailParseOptions::default()).unwrap();

        // Verify headers extracted
        assert!(email.headers.iter().any(|h| h.name == "From"));
        assert!(email.headers.iter().any(|h| h.name == "To"));
        assert!(email.headers.iter().any(|h| h.name == "Subject"));

        // Verify body extracted
        assert!(!email.body_parts.is_empty());

        // Verify attachments listed
        println!("Found {} attachments", email.attachments.len());
    }
}
```

## Common Patterns

### Extract All Text for PII Scanning

```rust
use veil_email::{parse_email, EmailParseOptions};

fn extract_all_text(email_bytes: &[u8]) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let email = parse_email(email_bytes, &EmailParseOptions::default())?;
    let mut texts = Vec::new();

    // Headers
    for header in &email.headers {
        texts.push(format!("{}: {}", header.name, header.raw_value));
    }

    // Body parts
    for part in &email.body_parts {
        texts.push(part.content.clone());
    }

    // Attachment filenames
    for attachment in &email.attachments {
        if let Some(filename) = &attachment.filename {
            texts.push(filename.clone());
        }
    }

    Ok(texts)
}
```

### Filter Out Quoted Content

```rust
use veil_email::{parse_email, EmailParseOptions};

fn extract_original_content_only(email_bytes: &[u8]) -> Result<String, Box<dyn std::error::Error>> {
    let options = EmailParseOptions {
        detect_quotes: true,
        ..Default::default()
    };

    let email = parse_email(email_bytes, &options)?;

    let original: Vec<String> = email.body_parts
        .iter()
        .filter(|part| !part.is_quoted)
        .map(|part| part.content.clone())
        .collect();

    Ok(original.join("\n\n"))
}
```

### Count Recipients

```rust
use veil_email::{parse_email, EmailParseOptions, EmailHeaderValue};

fn count_recipients(email_bytes: &[u8]) -> Result<usize, Box<dyn std::error::Error>> {
    let email = parse_email(email_bytes, &EmailParseOptions::default())?;
    let mut count = 0;

    for header in &email.headers {
        if header.name == "To" || header.name == "Cc" || header.name == "Bcc" {
            if let EmailHeaderValue::AddressList(list) = &header.value {
                count += list.len();
            }
        }
    }

    Ok(count)
}
```

## Performance Tips

1. **Reuse Options**: Create `EmailParseOptions` once and reuse for batch processing
2. **Skip HTML Conversion**: Set `convert_html: false` if you only need plain text parts
3. **Disable Quote Detection**: Set `detect_quotes: false` if not needed (saves regex overhead)
4. **Limit Attachment Size**: Set `max_attachment_size` to skip listing enormous attachments
5. **Stream Processing**: For large batches, use `rayon` to parse emails in parallel

```rust
use rayon::prelude::*;
use std::fs;

fn batch_parse_emails(paths: &[String]) -> Vec<Result<EmailMessage, EmailParseError>> {
    paths.par_iter().map(|path| {
        let bytes = fs::read(path).unwrap();
        parse_email(&bytes, &EmailParseOptions::default())
    }).collect()
}
```

## Troubleshooting

### "Invalid email format" Error

- Verify file is actually EML or MSG format
- Check file isn't truncated or corrupted
- Try specifying format explicitly: `format: Some(FileFormat::Eml)`

### "Encoding error" Warning

- Email uses uncommon character encoding
- Parser will do best-effort conversion to UTF-8
- Check `result.warnings` for details

### Missing Headers

- Some emails omit standard headers (e.g., no Subject)
- Use `headers.iter().find(|h| h.name == "Subject")` and handle `None`

### Empty Body Parts

- Email may be multipart with no text/plain or text/html parts
- Check `body_parts.is_empty()` before accessing

### Attachment Content Not Available

- By design: attachment content is not loaded into memory
- Use separate API (future) to extract specific attachment content

## Next Steps

- See `data-model.md` for detailed type definitions
- See `plan.md` for implementation details
- See `spec.md` for full feature specification
- Run `cargo doc --open` for API documentation

## Example Output Schema

When converted to TextSegments, email produces:

```json
{
  "metadata": {
    "format": "eml",
    "encoding": "utf-8",
    "size_bytes": 5432
  },
  "segments": [
    {
      "content": "john.doe@example.com",
      "position": {
        "type": "email",
        "field": "From",
        "byte_offset": 0,
        "byte_length": 20
      }
    },
    {
      "content": "Please review the attached report.",
      "position": {
        "type": "email",
        "field": "body",
        "part_index": 0,
        "byte_offset": 0,
        "byte_length": 34
      }
    }
  ],
  "total_chars": 1247,
  "duration_ms": 12
}
```

This matches the veil-parsers interface, enabling seamless integration with PII detection.
