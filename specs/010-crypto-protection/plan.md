# Implementation Plan: Cryptographic Protection

**Branch**: `010-crypto-protection` | **Date**: 2025-12-15 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/010-crypto-protection/spec.md`

## Summary

Implement cryptographic protection module providing four protection strategies: AES-256-GCM encryption, SHA-256/SHA-512 hashing, locale-aware pseudonymization, and tokenization with vault storage. Uses RustCrypto ecosystem for pure-Rust, WASM-compatible cryptographic operations.

## Technical Context

**Language/Version**: Rust (stable, 1.75+)
**Primary Dependencies**: aes-gcm, sha2, hmac, rand, fake, uuid, base64, zeroize, subtle
**Storage**: In-memory token vault (trait-based for pluggability)
**Testing**: cargo test
**Target Platform**: Cross-platform library (Linux, macOS, Windows, WASM-compatible)
**Project Type**: Single library crate
**Performance Goals**: 10,000 encryptions in <1 second
**Constraints**: Pure Rust only, no C dependencies for WASM compatibility
**Scale/Scope**: Core library used by other Veil crates

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Security First | ✅ PASS | Using audited RustCrypto libraries (aes-gcm, sha2); no unsafe needed |
| II. Stability & Error Handling | ✅ PASS | Result<T, CryptoError> for all operations; thiserror for errors |
| III. Performance | ✅ PASS | Pure Rust crypto with optional hardware acceleration |
| IV. Simplicity & Minimalism | ✅ PASS | Single-purpose crate; trait-based vault abstraction |
| V. Test-First Development | ✅ PASS | TDD for all crypto operations; test vectors from standards |
| VI. Dependency Discipline | ✅ PASS | RustCrypto ecosystem is widely trusted; all deps justified |
| VII. Rust Standards | ✅ PASS | Clippy clean; public API documented |

## Project Structure

### Documentation (this feature)

```text
specs/010-crypto-protection/
├── plan.md              # This file
├── research.md          # Phase 0 output - library decisions
├── data-model.md        # Phase 1 output - type definitions
├── quickstart.md        # Phase 1 output - usage examples
└── tasks.md             # Phase 2 output (via /speckit.tasks)
```

### Source Code (repository root)

```text
crates/
├── veil-crypto/         # NEW: Cryptographic protection
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs       # Public API exports
│       ├── encrypt.rs   # AES-256-GCM encryption
│       ├── hash.rs      # SHA-256/SHA-512 hashing
│       ├── pseudonym.rs # Fake data generation
│       ├── tokenize.rs  # Token generation
│       ├── vault/
│       │   ├── mod.rs   # TokenVault trait
│       │   └── memory.rs# InMemoryVault
│       ├── types.rs     # Configs and enums
│       ├── error.rs     # CryptoError
│       └── protector.rs # High-level API
├── veil-parsers/        # Existing: Document parsing
├── veil-detect/         # Existing: PII detection
└── veil-wasm/           # Existing: WASM bindings

tests/
└── crypto/              # Integration tests
    └── roundtrip_tests.rs
```

**Structure Decision**: New `veil-crypto` crate following existing workspace pattern. Module organization mirrors the four protection modes (encrypt, hash, pseudonymize, tokenize) with shared types and error handling.

## Complexity Tracking

> No violations identified - design follows constitution principles.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| N/A | - | - |
