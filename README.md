# Veil

[![CI](https://github.com/aliuyar1234/Veil/actions/workflows/ci.yml/badge.svg)](https://github.com/aliuyar1234/Veil/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)

A high-performance, privacy-focused toolkit for detecting and redacting Personally Identifiable Information (PII) in documents. Built in Rust for speed, safety, and cross-platform compatibility.

## Features

- **Multi-format Support**: Parse and analyze text, CSV, JSON, HTML, PDF, Office documents (XLSX, DOCX), and email files
- **Smart Detection**: Regex patterns, dictionary matching, and contextual analysis for accurate PII identification
- **Flexible Redaction**: Multiple strategies including masking, replacement, tokenization, and format-preserving encryption
- **Policy Engine**: YAML-based rules with GDPR, HIPAA, and PCI-DSS presets
- **Audit Logging**: Immutable JSONL audit trails with cryptographic verification
- **Streaming Support**: Process large files with bounded memory usage
- **Batch Processing**: Parallel processing of entire directories
- **WASM Support**: Run in browsers with full functionality
- **REST API**: Production-ready HTTP server for integration

## Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/aliuyar1234/Veil.git
cd veil

# Build the CLI
cargo build --release -p veil-cli

# Install locally
cargo install --path crates/veil-cli
```

### Basic Usage

```bash
# Scan a file for PII
veil scan document.txt

# Redact PII from a file
veil redact document.txt -o redacted.txt

# Scan with a specific policy
veil scan document.txt --policy gdpr

# Process a directory
veil batch ./documents --output ./redacted

# Start the API server
veil serve --port 8080
```

### Library Usage

```rust
use veil_detect::{Detector, DetectorConfig};
use veil_redact::{Redactor, RedactionStrategy};

// Create a detector
let detector = Detector::new(DetectorConfig::default());

// Detect PII in text
let text = "Contact John Smith at john@example.com or 555-123-4567";
let findings = detector.detect(text)?;

// Redact the findings
let redactor = Redactor::new(RedactionStrategy::Mask { char: '*' });
let redacted = redactor.redact(text, &findings)?;

println!("{}", redacted);
// Output: "Contact [REDACTED] at ****@*******.*** or ***-***-****"
```

## Architecture

```
veil/
├── veil-parsers    # Document parsing (text, CSV, JSON, HTML, PDF)
├── veil-detect     # PII detection engine (regex, dictionary, ML-ready)
├── veil-redact     # Redaction strategies and execution
├── veil-crypto     # Encryption, tokenization, key management
├── veil-policy     # Policy engine with YAML rule definitions
├── veil-audit      # Immutable audit logging with verification
├── veil-office     # Office document support (XLSX, DOCX)
├── veil-email      # Email parsing (EML, MIME)
├── veil-discovery  # File type detection and routing
├── veil-stream     # Memory-bounded streaming for large files
├── veil-batch      # Parallel batch processing
├── veil-api        # REST API server (Axum-based)
├── veil-cli        # Command-line interface
└── veil-wasm       # WebAssembly bindings for browsers
```

## Supported PII Types

| Category | Types |
|----------|-------|
| Identity | Names, SSN, Passport numbers, Driver's license |
| Contact | Email, Phone, Address |
| Financial | Credit card, Bank account, IBAN |
| Health | Medical record numbers (HIPAA) |
| Technical | IP addresses, MAC addresses |
| Custom | Extensible via regex patterns |

## Policies

Veil supports YAML-based policies for customizable detection and redaction:

```yaml
# gdpr.yaml
name: GDPR Compliance
version: "1.0"

rules:
  - name: personal_email
    pattern: email
    severity: high
    action: redact

  - name: phone_number
    pattern: phone
    severity: medium
    action: mask

settings:
  min_confidence: 0.8
  include_context: true
```

## Benchmarks

| Operation | Throughput | Memory |
|-----------|------------|--------|
| Text detection | ~50 MB/s | O(1) |
| PDF parsing | ~10 MB/s | O(n) |
| Batch (10K files) | <10 min | ~100 MB |
| WASM detection | ~20 MB/s | ~50 MB |

## Development

```bash
# Run all tests
cargo test --workspace

# Run with coverage
cargo tarpaulin --workspace

# Check formatting
cargo fmt --all -- --check

# Run linter
cargo clippy --workspace -- -D warnings

# Build documentation
cargo doc --workspace --no-deps --open

# Build WASM
wasm-pack build crates/veil-wasm --target web
```

## Security

- **No unsafe code** without explicit justification and audit
- **Constant-time comparisons** for cryptographic operations
- **Zeroization** of sensitive data in memory
- **Audit trail integrity** via HMAC verification
- **Dependency auditing** via `cargo-audit`

Report security vulnerabilities via GitHub Issues.

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push to branch (`git push origin feature/amazing`)
5. Open a Pull Request

Please ensure tests pass before submitting PRs.

## License

MIT OR Apache-2.0

## Acknowledgments

Built with these excellent crates:
- [regex](https://crates.io/crates/regex) - Fast regex matching
- [scraper](https://crates.io/crates/scraper) - HTML parsing
- [calamine](https://crates.io/crates/calamine) - Excel parsing
- [mailparse](https://crates.io/crates/mailparse) - Email parsing
- [axum](https://crates.io/crates/axum) - Web framework
- [wasm-bindgen](https://crates.io/crates/wasm-bindgen) - WASM bindings
