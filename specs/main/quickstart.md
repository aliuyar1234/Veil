# Quickstart: Veil MVP Development

**Date**: 2025-12-15 | **Plan**: specs/main/plan.md

## Prerequisites

- Rust 1.75+ (stable)
- Git
- Cargo (comes with Rust)

```bash
# Check Rust version
rustc --version  # Should be 1.75.0 or higher

# Update if needed
rustup update stable
```

## Repository Setup

```bash
# Clone repository
git clone https://github.com/your-org/veil.git
cd veil

# Verify workspace structure
ls crates/
# Should show: veil-parsers, veil-detect, veil-redact, veil-policy, veil-audit, veil-cli
```

## Build

```bash
# Build all crates
cargo build

# Build release version
cargo build --release

# Build specific crate
cargo build -p veil-parsers
```

## Test

```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p veil-detect

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_email_detection
```

## Lint & Format

```bash
# Format code
cargo fmt

# Check formatting without changes
cargo fmt --check

# Run clippy (warnings = errors)
cargo clippy -- -D warnings

# Run clippy for specific crate
cargo clippy -p veil-parsers -- -D warnings
```

## Development Workflow

### 1. TDD Cycle

```bash
# 1. Write failing test
cargo test test_new_feature  # Should FAIL

# 2. Implement minimal code to pass
# ... edit src/...

# 3. Test passes
cargo test test_new_feature  # Should PASS

# 4. Refactor if needed
cargo fmt && cargo clippy -- -D warnings

# 5. Commit
git add . && git commit -m "feat: add new feature"
```

### 2. Adding a New Detector (Example)

```bash
# 1. Create test file
cat > crates/veil-detect/src/patterns/passport.rs << 'EOF'
use crate::{Detector, Match, PiiCategory, ValidationStatus};

pub struct PassportDetector;

impl Detector for PassportDetector {
    fn name(&self) -> &str { "passport" }
    fn category(&self) -> PiiCategory { PiiCategory::Custom("passport".into()) }
    fn detect(&self, text: &str) -> Vec<Match> { todo!() }
    fn validate(&self, _matched: &str) -> ValidationStatus { ValidationStatus::Unvalidated }
    fn base_confidence(&self) -> f32 { 0.8 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_austrian_passport() {
        let detector = PassportDetector;
        let text = "Passport: P1234567";
        let matches = detector.detect(text);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].text, "P1234567");
    }
}
EOF

# 2. Add module to mod.rs
echo "pub mod passport;" >> crates/veil-detect/src/patterns/mod.rs

# 3. Run test (should fail)
cargo test -p veil-detect test_detect_austrian_passport

# 4. Implement detector
# ... edit passport.rs

# 5. Run test (should pass)
cargo test -p veil-detect test_detect_austrian_passport
```

### 3. Running the CLI

```bash
# Build CLI
cargo build -p veil-cli

# Run directly
./target/debug/veil --help

# Or via cargo
cargo run -p veil-cli -- scan test.txt
cargo run -p veil-cli -- protect test.txt -o redacted.txt
cargo run -p veil-cli -- policy validate policy.yaml
```

## Project Structure

```text
veil/
├── Cargo.toml              # Workspace manifest
├── crates/
│   ├── veil-parsers/       # 001: Document parsing
│   ├── veil-detect/        # 002: PII detection
│   ├── veil-redact/        # 003: Redaction engine
│   ├── veil-policy/        # 009: Policy engine
│   ├── veil-audit/         # 011: Audit logging
│   └── veil-cli/           # 004: CLI application
├── tests/
│   ├── fixtures/           # Test data files
│   └── integration/        # Integration tests
├── specs/                  # Feature specifications
└── docs/                   # Documentation
```

## Workspace Cargo.toml

```toml
[workspace]
resolver = "2"
members = [
    "crates/veil-parsers",
    "crates/veil-detect",
    "crates/veil-redact",
    "crates/veil-policy",
    "crates/veil-audit",
    "crates/veil-cli",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT OR Apache-2.0"
repository = "https://github.com/your-org/veil"

[workspace.dependencies]
# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"

# Parsing
csv = "1.3"
scraper = "0.18"
encoding_rs = "0.8"

# Detection
regex = "1.10"
once_cell = "1.19"

# CLI
clap = { version = "4.4", features = ["derive"] }
indicatif = "0.17"
console = "0.15"

# Errors
thiserror = "1.0"
miette = { version = "7.0", features = ["fancy"] }

# Audit
chrono = { version = "0.4", features = ["serde"] }
sha2 = "0.10"
uuid = { version = "1.6", features = ["v4", "serde"] }

# Testing
tempfile = "3.10"
```

## Crate Dependencies

### veil-parsers/Cargo.toml

```toml
[package]
name = "veil-parsers"
version.workspace = true
edition.workspace = true

[dependencies]
serde.workspace = true
serde_json.workspace = true
csv.workspace = true
scraper.workspace = true
encoding_rs.workspace = true
thiserror.workspace = true

[dev-dependencies]
tempfile.workspace = true
```

### veil-detect/Cargo.toml

```toml
[package]
name = "veil-detect"
version.workspace = true
edition.workspace = true

[dependencies]
veil-parsers = { path = "../veil-parsers" }
regex.workspace = true
once_cell.workspace = true
serde.workspace = true
thiserror.workspace = true
```

### veil-cli/Cargo.toml

```toml
[package]
name = "veil-cli"
version.workspace = true
edition.workspace = true

[[bin]]
name = "veil"
path = "src/main.rs"

[dependencies]
veil-parsers = { path = "../veil-parsers" }
veil-detect = { path = "../veil-detect" }
veil-redact = { path = "../veil-redact" }
veil-policy = { path = "../veil-policy" }
veil-audit = { path = "../veil-audit" }
clap.workspace = true
indicatif.workspace = true
console.workspace = true
miette.workspace = true
serde_json.workspace = true
```

## Test Fixtures

### tests/fixtures/sample.txt

```text
Contact Information:
Email: john.doe@example.com
Phone: +43 664 1234567
IBAN: AT611904300234573201

Credit Card: 4111 1111 1111 1111
SVNr: 1234 010190
```

### tests/fixtures/sample.csv

```csv
name,email,phone,iban
John Doe,john@example.com,+43 664 1234567,AT611904300234573201
Jane Smith,jane@test.org,+49 89 12345678,DE89370400440532013000
```

### tests/fixtures/policies/gdpr.yaml

```yaml
version: "1.0"
name: "GDPR Standard"
locale: "de-AT"

detection:
  - types: [email, phone, iban, credit_card]
    confidence_threshold: 0.8
    enabled: true

protection:
  - types: [email, phone]
    action: redact
    style: label

  - types: [iban, credit_card]
    action: mask
```

## Common Tasks

### Check Everything Before Commit

```bash
cargo fmt --check && cargo clippy -- -D warnings && cargo test
```

### Generate Documentation

```bash
cargo doc --open
```

### Run Specific Integration Test

```bash
cargo test --test integration_scan
```

### Profile Performance

```bash
cargo build --release
hyperfine './target/release/veil scan large_file.txt'
```

## Troubleshooting

### "unresolved import" errors

```bash
# Make sure all workspace members are listed in root Cargo.toml
# Check that path dependencies point to correct locations
cargo clean && cargo build
```

### Clippy warnings

```bash
# Fix automatically where possible
cargo clippy --fix --allow-dirty
```

### Test failures

```bash
# Run with full output
cargo test -- --nocapture

# Run single test with backtrace
RUST_BACKTRACE=1 cargo test test_name -- --nocapture
```
