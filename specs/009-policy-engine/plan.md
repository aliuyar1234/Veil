# Implementation Plan: Policy Engine

**Branch**: `009-policy-engine` | **Date**: 2025-12-15 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/009-policy-engine/spec.md`

## Summary

Extend the existing `veil-policy` crate to add key references, full protection execution via veil-crypto integration, and consistent pseudonymization tracking. The crate already has basic policy loading and detection filtering.

## Technical Context

**Language/Version**: Rust (stable, 1.75+)
**Primary Dependencies**: serde_yaml (existing), veil-detect (existing), veil-redact (existing), veil-crypto (NEW)
**Storage**: N/A (in-memory policy processing)
**Testing**: cargo test
**Target Platform**: Cross-platform library
**Project Type**: Extend existing crate
**Performance Goals**: Policy parsing <100ms, protection <10ms per finding
**Constraints**: No async required, pure library
**Scale/Scope**: Core orchestration layer connecting detection and protection

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Security First | ✅ PASS | Keys never stored in policy; resolved at runtime |
| II. Stability & Error Handling | ✅ PASS | Result<T, PolicyError> for all operations |
| III. Performance | ✅ PASS | Simple HashMap for consistency cache |
| IV. Simplicity & Minimalism | ✅ PASS | Extends existing crate; no new dependencies except veil-crypto |
| V. Test-First Development | ✅ PASS | TDD for new modules |
| VI. Dependency Discipline | ✅ PASS | Only adding veil-crypto (internal crate) |
| VII. Rust Standards | ✅ PASS | Clippy clean; documented public API |

## Project Structure

### Documentation (this feature)

```text
specs/009-policy-engine/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
└── tasks.md             # Phase 2 output (via /speckit.tasks)
```

### Source Code (repository root)

```text
crates/veil-policy/src/
├── lib.rs           # Public exports (EXISTS - EXTEND)
├── schema.rs        # Policy struct (EXISTS)
├── rules.rs         # Detection/Protection rules (EXISTS - EXTEND with key_ref)
├── loader.rs        # YAML loading (EXISTS)
├── validation.rs    # Policy validation (EXISTS - EXTEND)
├── defaults.rs      # Default policy (EXISTS)
├── locale.rs        # Locale handling (EXISTS)
├── error.rs         # PolicyError (EXISTS - EXTEND)
├── apply.rs         # apply_policy_to_findings (EXISTS)
├── keyref.rs        # NEW: KeyRef and resolution
├── executor.rs      # NEW: PolicyExecutor
└── protect.rs       # NEW: Protection dispatch to veil-crypto
```

**Structure Decision**: Extend existing `veil-policy` crate by adding three new modules (keyref, executor, protect) and extending existing types.

## Complexity Tracking

> No violations identified.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| N/A | - | - |
