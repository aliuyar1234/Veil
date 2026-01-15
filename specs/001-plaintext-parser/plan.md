# Implementation Plan: Plaintext Parser

**Branch**: `001-plaintext-parser` | **Date**: 2025-12-09 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-plaintext-parser/spec.md`

## Summary

Build the foundational parsing library for Veil that extracts text content from plain text, CSV,
JSON, and HTML files. Each parser returns a unified `ParseResult` containing `TextSegment`s with
position metadata. This is the first crate in the Veil workspace and establishes patterns for
all subsequent parsers.

## Technical Context

**Language/Version**: Rust (stable)
**Primary Dependencies**: serde (serialization), csv (CSV parsing), encoding_rs (character encoding), scraper (HTML parsing)
**Storage**: N/A (pure library, no persistence)
**Testing**: cargo test (unit + integration tests with real files)
**Target Platform**: Cross-platform library (Linux, macOS, Windows, WASM-compatible)
**Project Type**: Single crate (veil-parsers), later part of workspace
**Performance Goals**: <1s per MB text, <3x memory overhead, streaming for files >10MB
**Constraints**: 100MB max file size, UTF-8/UTF-16/ISO-8859-1 encoding support
**Scale/Scope**: Parse single files, return structured text segments

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Security First | ✅ PASS | No unsafe needed; input validation for malformed files |
| II. Stability & Error Handling | ✅ PASS | Result types for all fallible operations; graceful handling of malformed input |
| III. Performance | ✅ PASS | Streaming for large files; zero-copy where possible |
| IV. Simplicity & Minimalism | ✅ PASS | One parser per format; unified output type |
| V. Test-First Development | ✅ PASS | Test files for each format and edge case |
| VI. Dependency Discipline | ⚠️ REVIEW | csv, encoding_rs, scraper needed - all well-maintained |
| VII. Rust Standards | ✅ PASS | Clippy/fmt; documented public API |

**Gate Result**: PASS (dependencies justified for format-specific parsing)

## Project Structure

### Documentation (this feature)

```text
specs/001-plaintext-parser/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (Rust trait definitions)
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
Cargo.toml               # Workspace root
crates/
└── veil-parsers/
    ├── Cargo.toml
    ├── src/
    │   ├── lib.rs           # Public API exports
    │   ├── error.rs         # Error types (thiserror)
    │   ├── types.rs         # ParseResult, TextSegment, Position
    │   ├── detect.rs        # Format auto-detection
    │   ├── text.rs          # Plain text parser
    │   ├── csv.rs           # CSV parser
    │   ├── json.rs          # JSON parser
    │   └── html.rs          # HTML parser
    └── tests/
        ├── fixtures/        # Test files for each format
        │   ├── plain/
        │   ├── csv/
        │   ├── json/
        │   └── html/
        ├── text_tests.rs
        ├── csv_tests.rs
        ├── json_tests.rs
        └── html_tests.rs
```

**Structure Decision**: Single crate for all text-based parsers. This keeps related parsers
together and allows shared types. Later features (PDF, Office) will be separate crates
with heavier dependencies.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| csv crate | RFC 4180 parsing is non-trivial | Hand-rolled parser would be error-prone and slower |
| encoding_rs crate | Character encoding detection is complex | std only supports UTF-8; need UTF-16/ISO-8859-1 |
| scraper crate | HTML parsing with proper DOM handling | Regex-based extraction is fragile and incorrect |

## Post-Design Constitution Re-Check

*Re-evaluated after Phase 1 design completion (2025-12-09)*

| Principle | Status | Post-Design Notes |
|-----------|--------|-------------------|
| I. Security First | ✅ PASS | No unsafe code; all input validated; ParseError for malformed data |
| II. Stability & Error Handling | ✅ PASS | Result<ParseResult, ParseError> everywhere; ParseWarning for non-fatal issues |
| III. Performance | ✅ PASS | Streaming API for large files; BufReader pattern; zero-copy where possible |
| IV. Simplicity & Minimalism | ✅ PASS | 4 parsers, unified output type; single Parser trait |
| V. Test-First Development | ✅ PASS | Test fixtures directory; integration tests per format |
| VI. Dependency Discipline | ✅ PASS | 4 crates justified: csv, encoding_rs, scraper, serde_json (all well-maintained) |
| VII. Rust Standards | ✅ PASS | thiserror for errors; serde derives; documented public API |

**Post-Design Gate Result**: PASS - Ready for task generation
