# Veil

[![CI](https://github.com/aliuyar1234/Veil/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/aliuyar1234/Veil/actions/workflows/ci.yml)
[![Code Coverage](https://codecov.io/gh/aliuyar1234/Veil/branch/main/graph/badge.svg)](https://codecov.io/gh/aliuyar1234/Veil)
[![CodeQL](https://github.com/aliuyar1234/Veil/actions/workflows/codeql.yml/badge.svg?branch=main)](https://github.com/aliuyar1234/Veil/actions/workflows/codeql.yml)
[![MSRV](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

Local-first PII detection and redaction toolkit for text, PDFs, Office documents, and emails. Runs offline by default and keeps data on your machine.

## Highlights

- **Offline by default**: no telemetry and no runtime network calls.
- **Safe output defaults**: matched values are hidden unless explicitly requested.
- **Policy-driven protection**: redact/mask/label with configurable rules.
- **Audit logging**: tamper-evident hash chain; optional at-rest encryption.
- **Built for scale**: batch + streaming workflows for large datasets.

## Install

From source:

```bash
git clone https://github.com/aliuyar1234/Veil.git
cd Veil
cargo install --path crates/veil-cli --locked
veil --help
```

## Quick start (CLI)

```bash
# Scan a document for sensitive data (values are hidden by default)
veil scan document.txt

# Redact sensitive data
veil protect document.txt -o safe_document.txt

# Scan an entire folder
veil scan ./documents --recursive

# CI-friendly: fail if any findings are detected
veil scan ./documents --recursive --fail-on-findings
```

## Use in your code

```rust
use veil_detect::DetectorRegistry;
use veil_parsers::{parse_str, ParseOptions};

let result = parse_str(
    "Contact: john@example.com, SSN: 123-45-6789",
    &ParseOptions::default(),
)?;

let registry = DetectorRegistry::default();
let findings = registry.detect_all(&result.segments);

for finding in findings {
    println!("Found {} at position {}-{}", finding.category, finding.start, finding.end);
}
```

## Security

- Data stays on the host; Veil does not initiate network connections at runtime.
- `scan --include-values` is an explicit opt-in and is designed to be hard to use accidentally in automation.
- Audit logs are chained for tamper detection; on Unix they are created with `0600` perms and on Windows the logger applies a restrictive DACL (best-effort).
- Supply-chain checks in CI: `cargo audit`, `cargo deny`, `cargo vet`, and secret scanning (`detect-secrets`).

Further docs:

- `SECURITY_GUIDE.md`
- `THREAT_MODEL.md`
- `OPERATIONS.md`
- `ENTERPRISE_BACKLOG.md`

## Repository layout

```
Veil/
  crates/
    veil-core        # Secure data types (zeroization, redaction)
    veil-parsers     # Document parsing (TXT, CSV, JSON, HTML, PDF)
    veil-detect      # PII detection engine (patterns, validators)
    veil-redact      # Masking and replacement
    veil-crypto      # Encryption, hashing, tokenization
    veil-policy      # Policy engine (GDPR/HIPAA/PCI-ish)
    veil-audit       # Tamper-evident audit logging
    veil-discovery   # File discovery and scanning
    veil-office      # Office documents (DOCX/XLSX/PPTX)
    veil-email       # Email parsing (EML/MSG)
    veil-stream      # Streaming detection
    veil-batch       # Parallel file processing
    veil-cli         # Command line tool
    veil-wasm        # WebAssembly (browser) support
```

## Development

```bash
# Full local gates
just check
just deny
just vet

# Other useful commands
just fuzz email_parser
just bench
```

## License

MIT OR Apache-2.0

## Support

- Issues: https://github.com/aliuyar1234/Veil/issues
- Changelog: `CHANGELOG.md`
