# Veil

[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)

**Protect sensitive data automatically.** Veil finds and hides personal information in your documents before it falls into the wrong hands.

---

## What is Veil?

Veil is a privacy protection toolkit that automatically detects and removes sensitive personal information from documents. Whether you're handling customer data, employee records, or business documents, Veil helps you stay compliant with privacy regulations like GDPR, HIPAA, and PCI-DSS.

### The Problem

Every day, organizations handle thousands of documents containing sensitive data:
- Customer emails with phone numbers and addresses
- Spreadsheets with social security numbers
- PDFs with credit card information
- Employee records with personal details

Manually reviewing these documents is slow, expensive, and error-prone. A single data leak can result in massive fines and reputation damage.

### The Solution

Veil automatically scans your documents and:
- **Finds** personal information (emails, phone numbers, SSNs, credit cards, etc.)
- **Protects** it by masking, replacing, or encrypting the data
- **Logs** everything for audit compliance

All of this happens in seconds, not hours.

---

## What Can Veil Detect?

| Category | What It Finds |
|----------|---------------|
| **Personal Identity** | Names, Social Security Numbers, Passport Numbers, Driver's License Numbers |
| **Contact Information** | Email addresses, Phone numbers (US, UK, EU, international), Physical addresses |
| **Financial Data** | Credit card numbers, Bank accounts, IBANs, EU VAT Numbers |
| **Health Information** | Medical record numbers (HIPAA compliance) |
| **Technical Data** | IP addresses, MAC addresses |
| **EU/DACH Region** | German Tax ID (Steuer-ID), Swiss AHV Number, German National ID (Personalausweis), VAT Numbers (DE, AT, CH, FR, IT, ES, NL, BE, PL, UK, and more) |

Veil supports documents in many formats:
- **Text files** (TXT, CSV, JSON, XML, HTML)
- **Office documents** (Excel, Word, PowerPoint)
- **PDFs**
- **Emails** (EML files)

---

## Key Features

### Enterprise-Grade Security

- **Memory Protection**: Sensitive data is automatically wiped from memory after processing
- **Encrypted Storage**: Tokenized data is stored securely
- **Audit Trails**: Every action is logged with tamper-proof verification

### Compliance Ready

Pre-built policies for major regulations:
- **GDPR** - European data protection
- **HIPAA** - Healthcare privacy (US)
- **PCI-DSS** - Payment card security

### Flexible Deployment

- **Command Line** - Process files from your terminal
- **Web Browser** - Run directly in the browser (WebAssembly)
- **Library** - Embed in your own Rust applications

### High Performance

- Process large files without running out of memory
- Batch process thousands of files in parallel
- Handle streaming data in real-time

---

## Getting Started

### Option 1: Command Line

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

### Option 2: Use in Your Code

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

---

## How It Works

```
Your Document → Parse → Detect → Protect → Safe Document
     ↓            ↓        ↓         ↓          ↓
   PDF/Excel   Extract   Find     Mask or    Output
   Email/etc   text      PII      encrypt    clean file
```

1. **Parse**: Veil reads your document and extracts the text content
2. **Detect**: Smart algorithms scan for patterns matching sensitive data
3. **Protect**: Found data is masked (`***`), replaced (`[EMAIL]`), or encrypted
4. **Output**: You get a clean document safe for sharing

---

## Project Structure

```
Veil/
├── veil-core        # Secure data types (memory protection, constants)
├── veil-parsers     # Document reading (TXT, CSV, JSON, HTML, PDF)
├── veil-detect      # PII detection engine (patterns, validators)
├── veil-redact      # Data masking and replacement
├── veil-crypto      # Encryption, hashing, and tokenization
├── veil-policy      # Compliance rules (GDPR, HIPAA, PCI-DSS)
├── veil-audit       # Tamper-proof logging (encrypted, chained)
├── veil-batch       # Parallel file processing
├── veil-stream      # Streaming detection for large files
├── veil-discovery   # File discovery and scanning
├── veil-office      # Office documents (DOCX, XLSX, PPTX)
├── veil-email       # Email parsing (EML, MSG)
├── veil-cli         # Command line tool
└── veil-wasm        # Browser support (WebAssembly)
```

---

## Security Features

Veil was built with security as a priority:

| Feature | What It Does |
|---------|--------------|
| **Memory Zeroization** | Sensitive data is wiped from memory immediately after use |
| **Constant-Time Comparison** | Prevents timing attacks when comparing sensitive strings |
| **Secure Display** | PII is hidden in logs and error messages by default |
| **No Panics on User Input** | Graceful error handling - no crashes from malformed data |
| **Path Traversal Protection** | ZIP archives are validated against directory traversal attacks |
| **ZIP Bomb Protection** | Limits compression ratios and total size to prevent DoS |
| **Audit Logging** | Complete trail of all operations with integrity verification |
| **No Network Access** | Runs entirely locally - no data leaves your machine |
| **No Unsafe Code** | Zero `unsafe` blocks in production code |

---

## For Developers

### Requirements

- Rust 1.75 or newer
- Cargo (comes with Rust)

### Building

```bash
# Build everything
cargo build --workspace

# Run tests (560+ tests including property-based tests)
cargo test --workspace

# Check code quality
cargo clippy --workspace -- -D warnings

# Format code
cargo fmt --all
```

### Testing

```bash
# Run all tests
cargo test --workspace

# Run specific crate tests
cargo test -p veil-detect
cargo test -p veil-parsers

# Run with all features
cargo test --workspace --all-features
```

### Test Coverage

The codebase includes:
- **Unit tests** for all core functionality
- **Property-based tests** (proptest) for validators and parsers
- **Integration tests** for end-to-end workflows
- **Fuzz targets** for security-critical parsers

---

## License

MIT OR Apache-2.0

---

## Questions?

- Open an issue on [GitHub](https://github.com/aliuyar1234/Veil/issues)
- Check the [CHANGELOG](CHANGELOG.md) for recent updates
