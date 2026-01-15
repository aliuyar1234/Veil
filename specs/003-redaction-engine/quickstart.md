# Quickstart: Redaction Engine

**Feature**: 003-redaction-engine
**Audience**: Developers integrating veil-redact into applications
**Prerequisites**: Rust 1.75+, familiarity with veil-detect

---

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
veil-redact = { path = "../crates/veil-redact" }
veil-detect = { path = "../crates/veil-detect" }
```

---

## Basic Usage

### Example 1: Default Label Redaction

```rust
use veil_redact::redact;
use veil_detect::{Finding, PiiCategory, ValidationStatus};

fn main() {
    let text = "Contact me at john@example.com for details.";

    let findings = vec![
        Finding::new(
            "john@example.com",
            PiiCategory::Email,
            14,  // start position
            31,  // end position
            1.0, // confidence
            ValidationStatus::Valid,
            0,   // segment index
        ),
    ];

    let result = redact(text, &findings);

    println!("Redacted: {}", result.text);
    // Output: "Contact me at [EMAIL] for details."

    println!("Redactions applied: {}", result.redaction_count());
    // Output: 1
}
```

---

## Redaction Styles

### Label Replacement (Default)

Replace PII with category labels like `[EMAIL]`, `[PHONE]`.

```rust
use veil_redact::{redact_with_style, RedactionStyle};

let result = redact_with_style(text, &findings, RedactionStyle::label());
// "Contact me at [EMAIL] for details."
```

**Use case**: Document sanitization, clear indication of redacted content.

---

### Black Bar Redaction

Replace PII with solid characters (█) matching original length.

```rust
let result = redact_with_style(
    "IBAN: DE89370400440532013000",
    &findings,
    RedactionStyle::black_bar(),
);
// "IBAN: ██████████████████████"
```

**Custom character**:

```rust
let style = RedactionStyle::black_bar_with_char('X');
let result = redact_with_style(text, &findings, style);
// "IBAN: XXXXXXXXXXXXXXXXXXXXXX"
```

**Use case**: Legal documents, preserving layout.

---

### Partial Masking

Show first/last N characters, mask the middle.

```rust
use veil_redact::MaskingRule;

let rule = MaskingRule::new(1, 4);  // Show 1 first, 4 last
let style = RedactionStyle::mask(rule);

let result = redact_with_style(
    "Email: john.doe@example.com",
    &findings,
    style,
);
// "Email: j***********.com"
```

**Preserve structural characters**:

```rust
let rule = MaskingRule::new(1, 4)
    .with_preserve(vec!['@', '.']);

let result = redact_with_style(
    "Email: john.doe@example.com",
    &findings,
    RedactionStyle::mask(rule),
);
// "Email: j***.***@*******.com"
```

**Use case**: Customer service (show partial info for verification).

---

### Custom Replacement

Replace all PII with a custom string.

```rust
let style = RedactionStyle::custom("***REDACTED***");
let result = redact_with_style(text, &findings, style);
// "Contact me at ***REDACTED*** for details."
```

**Use case**: Legacy system compatibility, custom formats.

---

## Advanced Configuration

### Per-Category Styles

Apply different styles to different PII types.

```rust
use veil_redact::{RedactionConfig, RedactionEngine};
use veil_detect::PiiCategory;

let mut config = RedactionConfig::default();

// Emails: partial mask
config.set_category_style(
    PiiCategory::Email,
    RedactionStyle::mask(MaskingRule::new(1, 4)),
);

// IBANs: black bars
config.set_category_style(
    PiiCategory::Iban,
    RedactionStyle::black_bar(),
);

// Everything else: labels (default)

let engine = RedactionEngine::new(config);
let result = engine.redact(text, &findings);
```

---

## Position Mapping

Use position maps to locate redactions in original coordinates.

```rust
let result = redact(text, &findings);

// Map original position to redacted position
let original_pos = 20;  // Inside "john@example.com"
if let Some(redacted_pos) = result.position_map.map_position(original_pos) {
    println!("Original pos {} → Redacted pos {}", original_pos, redacted_pos);
}
// Output: Original pos 20 → Redacted pos 14
// (Maps to start of [EMAIL])
```

**Use case**: PDF annotation, Excel cell updates.

---

## Working with Detection Results

Integrate with veil-detect for end-to-end workflow.

```rust
use veil_parsers::parse_text;
use veil_detect::Detector;
use veil_redact::redact;

fn protect_document(content: &str) -> String {
    // 1. Parse content
    let parse_result = parse_text(content, "sample.txt").unwrap();

    // 2. Detect PII
    let detector = Detector::default();
    let findings = detector.scan_all(&parse_result);

    // 3. Redact
    let result = redact(content, &findings);

    result.text
}

fn main() {
    let document = "Employee: John Doe, Email: john@example.com, IBAN: DE89370400440532013000";
    let protected = protect_document(document);

    println!("{}", protected);
    // Output: "Employee: John Doe, Email: [EMAIL], IBAN: [IBAN]"
}
```

---

## Inspecting Redaction Results

### List Applied Redactions

```rust
let result = redact(text, &findings);

for redaction in &result.redactions {
    println!(
        "Redacted {} ({}) at positions {:?} → {:?}",
        redaction.category,
        redaction.original,
        redaction.original_position,
        redaction.new_position
    );
}
// Output:
// Redacted EMAIL (john@example.com) at positions (14, 31) → (14, 21)
```

---

### Export to JSON

All result types are serializable.

```rust
use serde_json;

let result = redact(text, &findings);
let json = serde_json::to_string_pretty(&result).unwrap();
println!("{}", json);
```

**Output**:

```json
{
  "text": "Contact me at [EMAIL] for details.",
  "redactions": [
    {
      "original": "john@example.com",
      "replacement": "[EMAIL]",
      "original_position": [14, 31],
      "new_position": [14, 21],
      "category": "email"
    }
  ],
  "position_map": {
    "entries": [
      {
        "original_start": 14,
        "original_end": 31,
        "redacted_start": 14,
        "redacted_end": 21
      }
    ]
  }
}
```

---

## Edge Cases

### No Findings

```rust
let result = redact("No PII here", &[]);
assert_eq!(result.text, "No PII here");
assert_eq!(result.redaction_count(), 0);
```

---

### Overlapping Findings

Longer matches are preferred; shorter overlaps are ignored.

```rust
let findings = vec![
    Finding::new("example.com", PiiCategory::Custom("Domain".into()), 20, 31, 1.0, ValidationStatus::Valid, 0),
    Finding::new("john@example.com", PiiCategory::Email, 14, 31, 1.0, ValidationStatus::Valid, 0),
];

let result = redact("Contact: john@example.com", &findings);
// Only the longer email finding is used
assert_eq!(result.text, "Contact: [EMAIL]");
```

---

### Multiple Redactions

```rust
let text = "Email: a@b.com, Phone: +43 664 1234567";
let findings = vec![
    Finding::new("a@b.com", PiiCategory::Email, 7, 14, 1.0, ValidationStatus::Valid, 0),
    Finding::new("+43 664 1234567", PiiCategory::Phone, 23, 38, 1.0, ValidationStatus::Valid, 0),
];

let result = redact(text, &findings);
assert_eq!(result.text, "Email: [EMAIL], Phone: [PHONE]");
assert_eq!(result.redaction_count(), 2);
```

---

## Testing Your Integration

### Unit Test Example

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use veil_redact::*;
    use veil_detect::*;

    #[test]
    fn test_email_redaction() {
        let text = "Send to: admin@company.com";
        let findings = vec![
            Finding::new(
                "admin@company.com",
                PiiCategory::Email,
                9,
                26,
                1.0,
                ValidationStatus::Valid,
                0,
            ),
        ];

        let result = redact(text, &findings);

        assert_eq!(result.text, "Send to: [EMAIL]");
        assert!(result.has_redactions());
    }
}
```

---

## Performance Considerations

### Batch Processing

Process multiple findings efficiently:

```rust
let config = RedactionConfig::default();
let engine = RedactionEngine::new(config);

for document in documents {
    let findings = detect_pii(&document);
    let result = engine.redact(&document.text, &findings);
    save_redacted(&result);
}
```

**Note**: `RedactionEngine` is lightweight; create once, reuse for multiple documents.

---

### Large Documents

For documents with 1000+ findings:

1. **Profile first**: Use `cargo bench` to confirm performance
2. **Consider chunking**: Process document in segments if memory is constrained
3. **Monitor**: Log redaction counts for audit purposes

Expected performance: <1 second for 10,000 findings (see research.md).

---

## Common Patterns

### Pattern 1: CLI Application

```rust
use clap::Parser;
use std::fs;

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    input: String,

    #[arg(short, long, default_value = "label")]
    style: String,
}

fn main() {
    let args = Args::parse();
    let content = fs::read_to_string(&args.input).unwrap();

    let style = match args.style.as_str() {
        "label" => RedactionStyle::label(),
        "blackbar" => RedactionStyle::black_bar(),
        _ => panic!("Unknown style"),
    };

    let findings = detect_pii(&content);
    let result = redact_with_style(&content, &findings, style);

    println!("{}", result.text);
}
```

---

### Pattern 2: Web API

```rust
use actix_web::{post, web, App, HttpResponse, HttpServer};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct RedactRequest {
    text: String,
}

#[derive(Serialize)]
struct RedactResponse {
    redacted: String,
    count: usize,
}

#[post("/redact")]
async fn redact_endpoint(req: web::Json<RedactRequest>) -> HttpResponse {
    let findings = detect_pii(&req.text);
    let result = redact(&req.text, &findings);

    HttpResponse::Ok().json(RedactResponse {
        redacted: result.text,
        count: result.redaction_count(),
    })
}
```

---

### Pattern 3: Library Integration

```rust
pub struct DataProcessor {
    engine: RedactionEngine,
}

impl DataProcessor {
    pub fn new(config: RedactionConfig) -> Self {
        Self {
            engine: RedactionEngine::new(config),
        }
    }

    pub fn process(&self, data: &str) -> ProcessedData {
        let findings = self.detect(data);
        let result = self.engine.redact(data, &findings);

        ProcessedData {
            original_hash: hash(data),
            redacted: result.text,
            metadata: result.redactions,
        }
    }
}
```

---

## Troubleshooting

### Issue: Redaction not applied

**Cause**: Finding positions don't match input text.

**Solution**: Verify `finding.start` and `finding.end` are valid byte offsets:

```rust
assert_eq!(&text[finding.start..finding.end], finding.matched_text);
```

---

### Issue: Position map returns unexpected values

**Cause**: Querying position outside original text bounds.

**Solution**: Check position is within `0..text.len()`:

```rust
if pos < text.len() {
    if let Some(mapped) = result.position_map.map_position(pos) {
        // Use mapped position
    }
}
```

---

### Issue: Unicode characters broken in output

**Cause**: Byte offset splits multi-byte character.

**Solution**: Ensure findings use character boundaries (veil-detect handles this automatically).

---

## Next Steps

- **Production use**: Combine with veil-audit for compliance logging (Spec 004)
- **Format-specific**: Integrate with PDF/Excel parsers for format-aware redaction
- **Custom detectors**: Extend veil-detect with domain-specific patterns (Spec 002)

---

## Resources

- API docs: Run `cargo doc --open` in `crates/veil-redact`
- Examples: `D:\Projekte\Veil\examples\redaction\`
- Tests: `D:\Projekte\Veil\crates\veil-redact\src\*.rs` (see `#[cfg(test)]` modules)
- Spec: `D:\Projekte\Veil\specs\003-redaction-engine\spec.md`

---

## Support

For issues or questions:
1. Check existing tests in `crates/veil-redact/src/`
2. Review data model: `specs/003-redaction-engine/data-model.md`
3. File issues with minimal reproducible example
