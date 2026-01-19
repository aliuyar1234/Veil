# Veil

[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)

Local-first PII detection and redaction toolkit. Runs offline by default and keeps data on your machine.

## At a glance

- Detects and protects PII in text, PDFs, Office docs, and emails.
- Local processing only; no telemetry or runtime network calls.
- Outputs hide raw values by default; explicit opt-in for value exposure.
- Audit logging with integrity checks; optional encryption.
- Batch and streaming modes for large datasets.

## Security and data handling

- Data stays on the host; Veil does not initiate network connections at runtime.
- Output and logs redact PII by default; `scan --include-values` requires confirmation.
- Sensitive strings are zeroized on drop and redacted in Debug, Display, and Serialize.
- Audit logs are chained for tamper detection; on Unix they are created with 0600 perms.
- Plaintext key and token storage is disabled by default. Set `VEIL_ALLOW_PLAINTEXT_STORAGE=1`
  to use `LocalKeyProvider` or `FileVault` for development.
- Encrypted audit logs require the `encryption` feature.

## Quick start (CLI)

```bash
# Install Veil
git clone https://github.com/aliuyar1234/Veil.git
cd Veil
cargo build --release -p veil-cli

# Scan a document for sensitive data
./target/release/veil scan document.txt

# Protect a document (redact sensitive data)
./target/release/veil protect document.txt -o safe_document.txt

# Scan an entire folder
./target/release/veil scan ./documents --recursive
```

## Use in your code

```rust
use veil_detect::DetectorRegistry;
use veil_parsers::{parse_str, ParseOptions};

// Parse your document
let result = parse_str("Contact: john@example.com, SSN: 123-45-6789", &ParseOptions::default())?;

// Find all sensitive data
let registry = DetectorRegistry::default();
let findings = registry.detect_all(&result.segments);

// Each finding tells you what was found and where
for finding in findings {
    println!("Found {} at position {}-{}", finding.category, finding.start, finding.end);
}
```

## Supported data and formats

### PII categories

| Category | What It Finds |
|----------|---------------|
| **Personal Identity** | Names, Social Security Numbers, Passport Numbers, Driver's License Numbers |
| **Contact Information** | Email addresses, Phone numbers (US, UK, EU, international), Physical addresses |
| **Financial Data** | Credit card numbers, Bank accounts, IBANs, EU VAT Numbers |
| **Health Information** | Medical record numbers (HIPAA compliance) |
| **Technical Data** | IP addresses, MAC addresses |
| **EU/DACH Region** | German Tax ID (Steuer-ID), Swiss AHV Number, German National ID (Personalausweis), VAT Numbers (DE, AT, CH, FR, IT, ES, NL, BE, PL, UK, and more) |

### Formats

- Text files (TXT, CSV, JSON, HTML)
- Office documents (DOCX, XLSX, PPTX)
- PDFs
- Emails (EML, MSG)

## How it works

```
Document -> Parse -> Detect -> Protect -> Output
```

1. Parse: extract text and structure from input.
2. Detect: match patterns and validators against extracted segments.
3. Protect: mask, label, replace, tokenize, or encrypt based on policy.
4. Output: return a safe document and optional audit log.

## Configuration and safety switches

- `scan --include-values` exposes matched values and prompts for confirmation; add `-y` to skip the prompt.
- `--policy` selects a policy file; `policy init` generates a starter policy.
- `batch --max-size` caps file size (MB); `--jobs` controls parallelism.
- `batch --zip` and `--zip-password` enable archive processing.
- Plaintext key and token storage is disabled by default; set `VEIL_ALLOW_PLAINTEXT_STORAGE=1` for dev.

## Performance and limits

- Streaming and batch modes avoid loading large files into memory.
- Large PDF and Office parses dominate runtime; use batching and size caps when needed.
- Benchmarks are for comparison, not absolute guarantees. Run on your target hardware:
  - `cargo bench -p veil-crypto`
  - `cargo bench -p veil-parsers`

## Project structure

```
Veil/
  crates/
    veil-core        # Secure data types (zeroization, redaction)
    veil-parsers     # Document parsing (TXT, CSV, JSON, HTML, PDF)
    veil-detect      # PII detection engine (patterns, validators)
    veil-redact      # Masking and replacement
    veil-crypto      # Encryption, hashing, tokenization
    veil-policy      # Policy engine (GDPR, HIPAA, PCI-DSS)
    veil-audit       # Tamper-evident logging
    veil-batch       # Parallel file processing
    veil-stream      # Streaming detection
    veil-discovery   # File discovery and scanning
    veil-office      # Office documents (DOCX, XLSX, PPTX)
    veil-email       # Email parsing (EML, MSG)
    veil-cli         # Command line tool
    veil-wasm        # Browser support (WebAssembly)
```

## For developers

### Requirements

- Rust 1.85 or newer
- Cargo (comes with Rust)

### Build and test

```bash
# Build everything
cargo build --workspace

# Run tests
cargo test --workspace

# Run with all features
cargo test --workspace --all-features

# Check code quality
cargo clippy --workspace -- -D warnings

# Format code
cargo fmt --all
```

### Benchmarks

```bash
cargo bench -p veil-crypto
cargo bench -p veil-parsers
```

## License

MIT OR Apache-2.0

## Questions

- Open an issue on GitHub: https://github.com/aliuyar1234/Veil/issues
- Check the CHANGELOG: CHANGELOG.md
