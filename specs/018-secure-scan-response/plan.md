# Implementation Plan: Secure Scan Response (PII-Safe API)

**Branch**: `018-secure-scan-response` | **Date**: 2025-12-17 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/018-secure-scan-response/spec.md`

## Summary

Remove PII values from all scan response outputs (API, CLI, WASM) by default to prevent data leakage. Add an opt-in mechanism with explicit security acknowledgment for legitimate use cases requiring PII values.

**Technical Approach**: Modify the response DTOs in each interface to omit the `value`/`matched_text` field by default. Add `include_values` parameter with corresponding acknowledgment requirements (header for API, interactive confirmation for CLI, option flag for WASM).

## Technical Context

**Language/Version**: Rust 1.75+ (stable, 2021 edition)
**Primary Dependencies**: axum (API), clap (CLI), wasm-bindgen (WASM), serde (serialization)
**Storage**: N/A (stateless request/response modification)
**Testing**: cargo test (unit + integration tests)
**Target Platform**: Linux, macOS, Windows (native); WASM (browsers)
**Project Type**: Multi-crate workspace
**Performance Goals**: <5% overhead from field omission (should actually improve due to less data)
**Constraints**: Breaking API change requires migration documentation
**Scale/Scope**: 3 crates affected (veil-api, veil-cli, veil-wasm)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Security First | PASS | This feature enhances security by preventing PII exposure |
| II. Stability & Error Handling | PASS | Proper Result<T,E> for acknowledgment validation |
| III. Performance | PASS | Removing fields reduces response size |
| IV. Simplicity & Minimalism | PASS | Minimal code change, no new abstractions |
| V. Test-First Development | PASS | Tests will be written for each acceptance scenario |
| VI. Dependency Discipline | PASS | No new dependencies required |
| VII. Rust Standards | PASS | Will use serde skip_serializing_if for optional fields |

**Gate Result**: PASSED - All principles satisfied

## Project Structure

### Documentation (this feature)

```text
specs/018-secure-scan-response/
├── spec.md              # Feature specification
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   └── api-changes.md   # Breaking change documentation
└── tasks.md             # Phase 2 output
```

### Source Code (repository root)

```text
crates/
├── veil-api/
│   └── src/
│       ├── routes/scan.rs      # Modify Finding response
│       └── models.rs           # Add include_values to ScanOptions
├── veil-cli/
│   └── src/
│       ├── cli.rs              # Add --include-values flag
│       └── commands/scan.rs    # Remove text from FindingOutput
└── veil-wasm/
    └── src/
        ├── types.rs            # Add includeValues to ScanOptions
        └── scan.rs             # Conditionally include value
```

**Structure Decision**: Existing multi-crate workspace structure is used. Changes are localized to scan-related modules in each interface crate.

## Complexity Tracking

> No violations - implementation is minimal and straightforward.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| N/A | N/A | N/A |

## Implementation Phases

### Phase 1: API Changes (P1)
1. Modify `ScanOptions` to include `include_values: bool` (default false)
2. Add header check for `X-Acknowledge-PII-Exposure: accepted`
3. Modify `Finding` response to use `Option<String>` for value
4. Return HTTP 400 if include_values=true without header

### Phase 2: CLI Changes (P1)
1. Add `--include-values` flag to ScanArgs
2. Add interactive confirmation prompt when flag is used
3. Modify `FindingOutput` to omit `text` field by default
4. Update output formatting functions

### Phase 3: WASM Changes (P2)
1. Add `includeValues` and `acknowledgeExposure` to ScanOptions
2. Validate acknowledgment when includeValues is true
3. Conditionally include value in Finding response

### Phase 4: Documentation & Migration (P2)
1. Create migration guide for existing integrations
2. Update API documentation
3. Add CHANGELOG entry
