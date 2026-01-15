# Implementation Plan: Identity Document Detection

**Branch**: `020-identity-document-detection` | **Date**: 2025-12-17 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/020-identity-document-detection/spec.md`

## Summary

Add detection patterns for critical identity documents: US Social Security Numbers (SSN), passport numbers (US, UK, EU), and driver's license numbers. These are essential PII types required for HIPAA compliance, financial services (KYC/AML), and HR systems. Implementation follows existing pattern detector architecture with new PII categories and validation rules.

## Technical Context

**Language/Version**: Rust 1.75+ (stable, 2021 edition)
**Primary Dependencies**: regex, once_cell (already in use)
**Storage**: N/A (stateless pattern matching)
**Testing**: cargo test -p veil-detect
**Target Platform**: Cross-platform library (same as veil-detect)
**Project Type**: Single crate modification
**Performance Goals**: <1ms per document scan for identity patterns
**Constraints**: No external API calls for validation; pattern-only detection
**Scale/Scope**: Adding 3 new detector modules, 3 new PII categories

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Security First | ✅ PASS | Identity documents are highly sensitive PII; detection helps prevent exposure |
| II. Stability & Error Handling | ✅ PASS | Using Result types, no unwrap on user input |
| III. Performance | ✅ PASS | Lazy regex compilation, pattern matching is O(n) |
| IV. Simplicity & Minimalism | ✅ PASS | Following existing detector pattern, no new abstractions |
| V. Test-First Development | ✅ PASS | Tests for each format before implementation |
| VI. Dependency Discipline | ✅ PASS | Using existing regex/once_cell dependencies |
| VII. Rust Standards | ✅ PASS | Clippy clean, proper documentation |

## Project Structure

### Documentation (this feature)

```text
specs/020-identity-document-detection/
├── plan.md              # This file
├── research.md          # SSN/passport/DL format research
├── data-model.md        # Entity definitions
├── quickstart.md        # Implementation guide
├── contracts/           # Test cases by document type
└── tasks.md             # Implementation tasks
```

### Source Code (repository root)

```text
crates/veil-detect/src/
├── category.rs          # Add Ssn, Passport, DriversLicense categories
├── patterns/
│   ├── mod.rs           # Export new detectors
│   ├── ssn.rs           # NEW: SSN detection patterns
│   ├── passport.rs      # NEW: Passport detection patterns
│   └── drivers_license.rs # NEW: Driver's license patterns
└── validators/
    └── ssn.rs           # NEW: SSN area number validation
```

**Structure Decision**: Following existing veil-detect pattern architecture with new detector modules.

## Complexity Tracking

> No violations - implementation follows existing patterns.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| N/A | N/A | N/A |
