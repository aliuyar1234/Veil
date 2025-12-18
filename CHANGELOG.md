# Changelog

All notable changes to Veil will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Encrypted vault storage**: New `EncryptedVault` for secure token-to-original mappings
  - AES-256-GCM with envelope encryption pattern
  - `KeyProvider` trait for key management abstraction
  - `LocalKeyProvider` for file-based key storage
  - `EnvKeyProvider` for environment variable keys (12-factor app support)
  - Key rotation support without re-encrypting all data
- **Enhanced CI pipeline**: Comprehensive quality gates
  - Code coverage with `cargo-llvm-cov` (90% threshold)
  - Mutation testing with `cargo-mutants` (80% threshold)
  - SBOM generation using `cargo-sbom` in release workflow
- **Benchmark infrastructure**: Criterion benchmarks for performance tracking
  - Detection benchmarks (`cargo bench -p veil-detect`)
  - Parsing benchmarks (`cargo bench -p veil-parsers`)
  - Encryption benchmarks (`cargo bench -p veil-crypto`)
- **Fuzz testing**: Coverage-guided fuzzing targets
  - PDF parser fuzzing
  - Email parser fuzzing
  - Office document fuzzing
  - YAML policy fuzzing
- **Property-based testing**: `proptest` tests for validators
  - Email format validation
  - Phone number validation
  - SSN validation
- **Documentation**: Architecture and examples
  - `docs/ARCHITECTURE.md` with Mermaid diagrams
  - `examples/basic_scan.rs` - Simple PII detection
  - `examples/batch_processing.rs` - Directory scanning
  - `examples/policy_usage.rs` - YAML policy application
  - `examples/streaming.rs` - Chunk-based processing
- **PII memory zeroization**: Secure memory cleanup for enterprise security compliance (SOC2/HIPAA/PCI-DSS)
  - New `veil-core` crate with `SensitiveString` type that securely zeroes memory on drop
  - `Finding.matched_text` now uses `SensitiveString` - PII values automatically zeroed when findings are dropped
  - `TextSegment.content` now uses `SensitiveString` - parsed document content automatically zeroed
  - Uses battle-tested `zeroize` crate (same pattern as encryption key handling in veil-crypto)
  - Cross-platform support: Linux, macOS, Windows, WASM (best-effort)
  - Debug output intentionally redacts content to prevent accidental PII leakage in logs
- **Identity document detection**: New detectors for critical identity documents (HIPAA/KYC/AML compliance)
  - US Social Security Numbers (SSN): Hyphenated (123-45-6789) and space-separated formats
  - SSN validation: Rejects invalid area numbers (000, 666, 9XX), group 00, and serial 0000
  - Passport numbers: US (9 digits, alphanumeric), UK (9 digits), EU (German, French alphanumeric)
  - Driver's license numbers: CA (1 letter + 7 digits), TX (8 digits), FL (1 letter + 12 digits), IL (1 letter + 11 digits), NY (9 digits)
- Three new PII categories: `Ssn`, `Passport`, `DriversLicense`
- 41 new tests for identity document detection
- **Global phone number detection**: Extended phone pattern recognition beyond DACH region
  - US/Canada (NANP): E.164 (+1), parentheses (555) 123-4567, 10-digit, toll-free
  - UK: E.164 (+44) landline and mobile, local mobile (07xxx)
  - France: E.164 (+33) format
  - Generic E.164: Catch-all for any international format (+[country][number])
- 20 new phone number test cases covering all supported formats

### Changed
- **Security**: `SensitiveString::eq` now uses constant-time comparison via `subtle` crate to prevent timing attacks
- **BREAKING**: Scan API responses no longer include PII values by default (security fix)
  - Add `include_values=true` query parameter + `X-Acknowledge-PII-Exposure: accepted` header to include values
- **BREAKING**: CLI scan output no longer includes PII values by default
  - Use `--include-values --yes` flags to include values (requires confirmation)
- **BREAKING**: WASM scan results no longer include PII values by default
  - Set `{includeValues: true, acknowledgeExposure: true}` in options to include values

### Security
- Fixed critical data exposure vulnerability: PII values were previously returned in all scan responses
- All interfaces (API, CLI, WASM) now require explicit opt-in with security acknowledgment to view PII values
- This change enforces GDPR data minimization principles

### Improved
- Phone number detection now covers global enterprise use cases
- Pattern priority ensures specific country patterns match before generic E.164
- Full backward compatibility maintained for existing DACH region detection

### Added
- Initial public release of Veil PII detection toolkit
- Multi-format document parsing (text, CSV, JSON, HTML, PDF)
- PII detection with regex patterns and dictionary matching
- Context-aware detection for improved accuracy
- Multiple redaction strategies (mask, replace, tokenize, encrypt)
- Policy engine with YAML-based rules
- GDPR, HIPAA, and PCI-DSS compliance presets
- Cryptographic protection (AES-256-GCM encryption, SHA-256/512 hashing)
- Tokenization with vault storage
- Format-preserving pseudonymization
- Audit logging with cryptographic chain verification
- Office document support (XLSX, DOCX, PPTX)
- Email parsing (EML, MIME)
- Streaming support for large files
- Parallel batch processing
- REST API server with JWT authentication
- WebAssembly bindings for browser use
- Command-line interface

### Security
- No unsafe code in codebase
- Constant-time comparisons for cryptographic operations
- Zeroization of sensitive data in memory
- Audit trail integrity via HMAC verification
- cargo-audit integrated in CI pipeline

## [0.1.0] - 2025-12-17

### Added
- Initial workspace structure with 14 crates
- Core detection patterns (email, phone, IBAN, credit card)
- Luhn and IBAN checksum validation
- Dictionary-based name detection with fuzzy matching
- Basic redaction engine
- File format detection
- Test coverage for all major components

[Unreleased]: https://github.com/aliuyar1234/Veil/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/aliuyar1234/Veil/releases/tag/v0.1.0
