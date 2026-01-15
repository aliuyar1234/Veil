# Implementation Plan: CLI Scan & Protect

**Branch**: `004-cli-scan-protect` | **Date**: 2025-12-15 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/004-cli-scan-protect/spec.md`

## Summary

Enhance the existing `veil-cli` crate with complete scan and protect functionality including recursive directory scanning, progress indication, multiple output formats, and policy integration. The CLI already has basic argument parsing and stub implementations; this feature adds the full production-ready implementation with comprehensive error handling and user experience features.

## Technical Context

**Language/Version**: Rust (stable, 1.75+)
**Primary Dependencies**: clap (existing), indicatif (existing), console (existing), veil-parsers, veil-detect, veil-redact, veil-policy, veil-audit
**Storage**: Local filesystem (JSONL audit logs via veil-audit)
**Testing**: cargo test, integration tests with real files
**Target Platform**: Cross-platform CLI (Linux, macOS, Windows)
**Project Type**: Extend existing crate (veil-cli)
**Performance Goals**: <2s for 1MB files, 1000 files in <5 minutes
**Constraints**: Synchronous operation, human-friendly progress and errors
**Scale/Scope**: Primary user interface for Veil, integrates all core crates

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Security First | ✅ PASS | No unsafe; miette for user-facing errors |
| II. Stability & Error Handling | ✅ PASS | Result propagation; graceful handling of file errors |
| III. Performance | ✅ PASS | Sequential processing sufficient; batch scan with progress |
| IV. Simplicity & Minimalism | ✅ PASS | Extends existing CLI; delegates to library crates |
| V. Test-First Development | ✅ PASS | Integration tests with tempfile fixtures |
| VI. Dependency Discipline | ✅ PASS | All dependencies already in workspace |
| VII. Rust Standards | ✅ PASS | Clippy clean; documented commands |

## Project Structure

### Documentation (this feature)

```text
specs/004-cli-scan-protect/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (CLI contract tests)
└── tasks.md             # Phase 2 output (via /speckit.tasks)
```

### Source Code (repository root)

```text
crates/veil-cli/
├── src/
│   ├── main.rs          # Entry point (EXISTS - minimal changes)
│   ├── cli.rs           # Clap definitions (EXISTS - EXTEND)
│   ├── output.rs        # Output formatting (EXISTS - EXTEND)
│   ├── progress.rs      # NEW: Progress indication
│   ├── error.rs         # NEW: CLI-specific error types
│   ├── walker.rs        # NEW: Directory traversal
│   └── commands/
│       ├── mod.rs       # Command dispatch (EXISTS)
│       ├── scan.rs      # Scan implementation (EXISTS - COMPLETE)
│       ├── protect.rs   # Protect implementation (EXISTS - COMPLETE)
│       └── policy.rs    # Policy validation (EXISTS - COMPLETE)
│
tests/
├── integration/
│   ├── scan_tests.rs    # NEW: Scan command tests
│   ├── protect_tests.rs # NEW: Protect command tests
│   ├── policy_tests.rs  # NEW: Policy command tests
│   └── fixtures/        # NEW: Test files with known PII
│       ├── sample.txt
│       ├── nested/
│       └── policies/
└── contract/
    └── cli_contract.rs  # NEW: CLI behavior contract tests
```

**Structure Decision**: Extend existing `veil-cli` crate. Commands module already exists with basic implementations; we add new support modules (progress, error, walker) and complete the command implementations with full functionality.

## Complexity Tracking

> No violations identified.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| N/A | - | - |
