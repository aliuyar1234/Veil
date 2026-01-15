# Research: MVP Dependencies and Patterns

**Date**: 2025-12-15 | **Plan**: specs/main/plan.md

## Dependency Decisions

### 1. CSV Parsing

**Decision**: Use `csv` crate (v1.3+)

**Rationale**:
- De-facto standard for CSV in Rust ecosystem
- RFC 4180 compliant with configurable delimiters
- Streaming API for memory efficiency
- Maintained by BurntSushi (ripgrep author)

**Alternatives Considered**:
- `polars` - Overkill for simple parsing, large dependency tree
- Manual parsing - Error-prone, RFC 4180 has edge cases

**Usage Pattern**:
```rust
use csv::ReaderBuilder;

let mut reader = ReaderBuilder::new()
    .delimiter(b';')
    .has_headers(true)
    .from_path(path)?;

for result in reader.records() {
    let record = result?;
    // Process each row
}
```

---

### 2. JSON Parsing with Path Tracking

**Decision**: Use `serde_json` with custom visitor for path extraction

**Rationale**:
- serde_json is the standard JSON library
- Path tracking requires traversing the JSON tree manually
- No need for additional dependency

**Alternatives Considered**:
- `jsonpath` crate - Overkill, we need extraction not querying
- `simd-json` - Faster but doesn't support path tracking out of box

**Usage Pattern**:
```rust
fn extract_strings(value: &Value, path: &str, segments: &mut Vec<TextSegment>) {
    match value {
        Value::String(s) => segments.push(TextSegment::new(s, path)),
        Value::Object(map) => {
            for (k, v) in map {
                extract_strings(v, &format!("{}.{}", path, k), segments);
            }
        }
        Value::Array(arr) => {
            for (i, v) in arr.iter().enumerate() {
                extract_strings(v, &format!("{}[{}]", path, i), segments);
            }
        }
        _ => {} // Skip numbers, booleans, null
    }
}
```

---

### 3. HTML Parsing

**Decision**: Use `scraper` crate

**Rationale**:
- Built on `html5ever` (Mozilla's HTML parser)
- CSS selector support for targeting visible elements
- Handles malformed HTML gracefully
- Lighter than `kuchiki` while still feature-complete

**Alternatives Considered**:
- `html5ever` directly - Lower level, more code needed
- `select.rs` - Less maintained
- `kuchiki` - Larger dependency, more features than needed

**Usage Pattern**:
```rust
use scraper::{Html, Selector};

let document = Html::parse_document(html);
let selector = Selector::parse("body *:not(script):not(style)").unwrap();

for element in document.select(&selector) {
    let text = element.text().collect::<String>();
    // Process visible text
}
```

---

### 4. Character Encoding Detection

**Decision**: Use `encoding_rs` crate

**Rationale**:
- Used by Firefox, battle-tested
- Supports UTF-8, UTF-16, ISO-8859-1 and more
- Streaming API for large files
- WHATWG Encoding Standard compliant

**Alternatives Considered**:
- `chardet` - Python port, less maintained
- `charset` - Wraps encoding_rs, unnecessary layer

**Usage Pattern**:
```rust
use encoding_rs::{Encoding, UTF_8};

fn decode_to_utf8(bytes: &[u8]) -> (String, bool) {
    // Try UTF-8 first
    let (decoded, _, had_errors) = UTF_8.decode(bytes);
    if !had_errors {
        return (decoded.into_owned(), false);
    }

    // Fallback to encoding detection
    let encoding = Encoding::for_label(b"windows-1252").unwrap_or(UTF_8);
    let (decoded, _, _) = encoding.decode(bytes);
    (decoded.into_owned(), true)
}
```

---

### 5. Regex Engine

**Decision**: Use `regex` crate with lazy compilation

**Rationale**:
- Guaranteed linear time matching (no catastrophic backtracking)
- Unicode support by default
- Lazy compilation for startup performance
- Well-maintained, standard in Rust ecosystem

**Alternatives Considered**:
- `fancy-regex` - Supports backreferences, but we don't need them
- `pcre2` - C binding, portability concerns for WASM

**Usage Pattern**:
```rust
use regex::Regex;
use once_cell::sync::Lazy;

static EMAIL_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap()
});

fn detect_emails(text: &str) -> Vec<Finding> {
    EMAIL_RE.find_iter(text)
        .map(|m| Finding::new("email", m.as_str(), m.start(), m.end()))
        .collect()
}
```

---

### 6. IBAN Validation (MOD-97)

**Decision**: Implement MOD-97 algorithm directly (no external crate)

**Rationale**:
- Algorithm is simple (ISO 7064)
- No need for external dependency
- Full control over validation logic
- Only ~20 lines of code

**Alternatives Considered**:
- `iban` crate - Pulls in unnecessary features
- `iban_validate` - Unmaintained

**Implementation**:
```rust
fn validate_iban(iban: &str) -> bool {
    let clean: String = iban.chars()
        .filter(|c| c.is_alphanumeric())
        .collect();

    if clean.len() < 15 || clean.len() > 34 {
        return false;
    }

    // Move first 4 chars to end
    let rearranged = format!("{}{}", &clean[4..], &clean[..4]);

    // Convert letters to numbers (A=10, B=11, etc.)
    let numeric: String = rearranged.chars()
        .map(|c| {
            if c.is_alphabetic() {
                format!("{}", c.to_ascii_uppercase() as u32 - 55)
            } else {
                c.to_string()
            }
        })
        .collect();

    // MOD 97 check
    mod97(&numeric) == 1
}

fn mod97(digits: &str) -> u32 {
    digits.chars()
        .fold(0u32, |acc, c| {
            (acc * 10 + c.to_digit(10).unwrap()) % 97
        })
}
```

---

### 7. Luhn Algorithm (Credit Card)

**Decision**: Implement directly (no external crate)

**Rationale**:
- Algorithm is 10 lines of code
- Standard validation used everywhere
- No dependency needed

**Implementation**:
```rust
fn validate_luhn(number: &str) -> bool {
    let digits: Vec<u32> = number
        .chars()
        .filter(|c| c.is_digit(10))
        .filter_map(|c| c.to_digit(10))
        .collect();

    if digits.len() < 13 || digits.len() > 19 {
        return false;
    }

    let sum: u32 = digits.iter().rev().enumerate()
        .map(|(i, &d)| {
            if i % 2 == 1 {
                let doubled = d * 2;
                if doubled > 9 { doubled - 9 } else { doubled }
            } else {
                d
            }
        })
        .sum();

    sum % 10 == 0
}
```

---

### 8. YAML Policy Parsing

**Decision**: Use `serde_yaml` crate

**Rationale**:
- Standard YAML library for Rust
- Direct integration with serde
- Supports all YAML 1.1 features

**Alternatives Considered**:
- `yaml-rust` - Lower level, more manual work
- TOML instead of YAML - Less human-friendly for policies

**Schema Validation Pattern**:
```rust
#[derive(Debug, Deserialize)]
pub struct Policy {
    pub version: String,
    pub name: String,
    #[serde(default)]
    pub locale: Option<String>,
    #[serde(default)]
    pub detection: Vec<DetectionRule>,
    #[serde(default)]
    pub protection: Vec<ProtectionRule>,
}

impl Policy {
    pub fn load(path: &Path) -> Result<Self, PolicyError> {
        let content = fs::read_to_string(path)?;
        let policy: Policy = serde_yaml::from_str(&content)?;
        policy.validate()?;
        Ok(policy)
    }

    fn validate(&self) -> Result<(), PolicyError> {
        if !self.version.starts_with("1.") {
            return Err(PolicyError::UnsupportedVersion(self.version.clone()));
        }
        // More validation...
        Ok(())
    }
}
```

---

### 9. Audit Log Format

**Decision**: JSON Lines (JSONL) for append-only logs

**Rationale**:
- One JSON object per line
- Easy to append without rewriting file
- Easy to parse line by line
- Standard format for log aggregation (Splunk, ELK)

**Alternatives Considered**:
- SQLite - Overkill for simple audit
- CSV - Doesn't handle nested data well
- Structured binary - Not human-readable

**Format**:
```jsonl
{"timestamp":"2025-12-15T10:30:00Z","operation":"scan","file":"doc.txt","findings":{"email":3,"iban":1},"checksum":"abc123"}
{"timestamp":"2025-12-15T10:30:05Z","operation":"protect","input":"doc.txt","output":"doc_redacted.txt","redactions":4,"checksum":"def456"}
```

---

### 10. CLI Framework

**Decision**: Use `clap` v4 with derive macros

**Rationale**:
- Standard CLI library for Rust
- Derive macros reduce boilerplate
- Automatic help generation
- Shell completion support

**Alternatives Considered**:
- `argh` - Google's library, less features
- `structopt` - Merged into clap v3+

**Pattern**:
```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "veil")]
#[command(about = "PII detection and redaction tool")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(short, long)]
    pub quiet: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Scan files for PII
    Scan(ScanArgs),
    /// Protect files by redacting PII
    Protect(ProtectArgs),
    /// Manage policies
    Policy(PolicyArgs),
}
```

---

## Open Questions Resolved

| Question | Resolution |
|----------|------------|
| How to track positions in CSV? | Use (row, col) tuple in TextSegment.location |
| How to track positions in JSON? | Use JSON path string (e.g., "$.users[0].email") |
| How to handle encoding detection? | Use encoding_rs with UTF-8 fallback |
| IBAN validation crate? | Implement MOD-97 directly (~20 LOC) |
| Credit card validation crate? | Implement Luhn directly (~15 LOC) |
| Audit log format? | JSON Lines (.jsonl) for append-only |
| How to handle large files? | Streaming for text, chunked for others |

## Best Practices Applied

1. **Streaming where possible**: CSV, text files
2. **Lazy regex compilation**: Compile patterns once at startup
3. **Position tracking**: Every text extraction includes source position
4. **Error propagation**: Use `?` operator, never unwrap user input
5. **Memory efficiency**: Process in chunks, avoid loading entire file
6. **Unicode correctness**: Use char indices, not byte indices
