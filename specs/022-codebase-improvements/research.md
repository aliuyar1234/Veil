# Research: Codebase Excellence Initiative

**Branch**: `022-codebase-improvements` | **Date**: 2025-12-18

## Research Topics

This document consolidates research findings for technical decisions required by the implementation plan.

---

## 1. Encrypted Vault Implementation

**Decision**: Use AES-256-GCM with envelope encryption pattern

**Rationale**:
- AES-256-GCM already in use (aes-gcm crate) - maintains consistency
- Envelope encryption allows key rotation without re-encrypting all data
- GCM provides authenticated encryption (integrity + confidentiality)

**Alternatives Considered**:
- ChaCha20-Poly1305: Faster on systems without AES-NI, but less universal
- XChaCha20: Larger nonce, but unnecessary for file-at-rest encryption

---

## 2. Key Provider Architecture

**Decision**: Trait-based abstraction with LocalKeyProvider, EnvKeyProvider implementations

**Rationale**:
- Trait allows future KMS integration without code changes
- Env provider supports 12-factor app configuration
- Local file provider supports air-gapped deployments

**Alternatives Considered**:
- Direct KMS integration: Too opinionated, limits deployment flexibility
- Config file only: Does not support secure key injection in containers

---

## 3. Fuzz Testing Setup

**Decision**: Use cargo-fuzz with libFuzzer backend

**Rationale**:
- cargo-fuzz is the standard Rust fuzzing tool
- libFuzzer provides coverage-guided fuzzing
- Integrates with OSS-Fuzz for continuous fuzzing

**Alternatives Considered**:
- AFL++: More complex setup, better for long-running campaigns
- Honggfuzz: Good alternative but less Rust ecosystem support

---

## 4. Mutation Testing Tool

**Decision**: Use cargo-mutants

**Rationale**:
- Pure Rust, no external dependencies
- Fast incremental mutation testing
- Good CI integration

**Alternatives Considered**:
- mutagen: Requires nightly Rust
- cargo-mutagen: Less maintained

---

## 5. Coverage Tool

**Decision**: Use cargo-llvm-cov

**Rationale**:
- Uses LLVM native coverage instrumentation
- Accurate line and branch coverage
- Supports multiple output formats (HTML, lcov, JSON)

**Alternatives Considered**:
- cargo-tarpaulin: Slower, less accurate
- grcov: More complex setup

---

## 6. OpenAPI Generation

**Decision**: Use utoipa crate

**Rationale**:
- Derives OpenAPI from Rust types
- Integrates with axum
- Generates accurate schemas from serde types

**Alternatives Considered**:
- paperclip: Less maintained
- Manual YAML: Error-prone, drift risk

---

## 7. Streaming JSON Parser

**Decision**: Use serde_json with custom deserializer for incremental parsing

**Rationale**:
- serde_json already a dependency
- StreamDeserializer supports incremental parsing
- No new dependencies required

**Alternatives Considered**:
- simd-json: Faster but does not support streaming
- json-stream: Less maintained

---

## 8. Byte Scanning Optimization

**Decision**: Use memchr crate for hot path scanning

**Rationale**:
- SIMD-optimized byte searching
- 10-100x faster than naive iteration
- Well-maintained, trusted crate

**Alternatives Considered**:
- Hand-rolled SIMD: Maintenance burden
- regex only: Slower for simple patterns

---

## 9. TypeScript Definition Generation

**Decision**: Use wasm-bindgen with ts-rs crate

**Rationale**:
- wasm-bindgen already generates basic TS types
- ts-rs provides more accurate type mappings
- Enables IDE autocompletion for WASM consumers

**Alternatives Considered**:
- Manual type definitions: Drift risk
- typeshare: More complex setup

---

## 10. Pre-commit Hook Framework

**Decision**: Use pre-commit framework with .pre-commit-config.yaml

**Rationale**:
- Language-agnostic, widely adopted
- Easy to configure multiple hooks
- Runs only on changed files

**Alternatives Considered**:
- Git hooks directly: No parallelism, harder to maintain
- lefthook: Less ecosystem support

---

## 11. WASM PDF Support

**Decision**: Use pdf-extract compiled to WASM with size optimization

**Rationale**:
- pdf-extract already used in native crates
- Compiles to WASM with some limitations
- Shared code path reduces maintenance

**Alternatives Considered**:
- pdfium: Larger binary, C++ complexity
- pdf.js via wasm-bindgen: JavaScript interop overhead
- Server-side only: Defeats offline use case

---

## Summary

All research topics resolved. No NEEDS CLARIFICATION items remaining.

| Topic | Decision | Confidence |
|-------|----------|------------|
| Encrypted Vault | AES-256-GCM envelope | High |
| Key Provider | Trait abstraction | High |
| Fuzzing | cargo-fuzz | High |
| Mutation Testing | cargo-mutants | High |
| Coverage | cargo-llvm-cov | High |
| OpenAPI | utoipa | High |
| Streaming JSON | serde_json StreamDeserializer | High |
| Byte Scanning | memchr | High |
| TypeScript | wasm-bindgen + ts-rs | Medium |
| Pre-commit | pre-commit framework | High |
| WASM PDF | pdf-extract compiled | Medium |
